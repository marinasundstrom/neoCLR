#include "native-gc.h"
#include "gate.h"
#include <stdio.h>
#include <pthread.h>
#include <stdlib.h>
#include <string.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_live_wait(void);
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
extern void experiment_gc_store_lookup_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while(0)
static void *object(neoclr_aot_context *c,uint64_t bytes) {
    void *p; CHECK(!neoclr_gc_allocate_v1(&c->text,bytes,NEOCLR_GC_OBJECT,&p));
    *(uint64_t *)p=17; return p;
}
static void ranges(void) {
    unsigned widths[]={1,2,4,8};
    for(unsigned w=0;w<4;w++) for(unsigned offset=8;offset+widths[w]<=35;offset++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        unsigned char *root=object(&c,35);
        memset(root+8,0xcc,27);
        neoclr_probe_storage slot={&root,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        experiment_gc_live_wait();
        uint64_t bits=UINT64_C(0x8877665544332211);
        CHECK(!experiment_gc_store(&c,root+offset,bits,widths[w]));
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        for(unsigned i=8;i<35;i++) if(i<offset || i>=offset+widths[w]) CHECK(root[i]==0xcc);
        switch(widths[w]) {
            case 1: { uint8_t v; memcpy(&v,root+offset,1); CHECK(v==0x11); break; }
            case 2: { uint16_t v; memcpy(&v,root+offset,2); CHECK(v==0x2211); break; }
            case 4: { uint32_t v; memcpy(&v,root+offset,4); CHECK(v==UINT32_C(0x44332211)); break; }
            case 8: { uint64_t v; memcpy(&v,root+offset,8); CHECK(v==bits); break; }
        }
        root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
}
/* Exercise the non-marker paths at every unaligned offset, preserving adjacent
 * bytes and rejecting unsupported widths before changing the destination. */
static void direct_ranges(void) {
    unsigned widths[]={1,2,4,8};
    for(unsigned mode=0;mode<3;mode++) for(unsigned w=0;w<4;w++)
    for(unsigned offset=8;offset+widths[w]<=35;offset++) {
        uint64_t heap[256]={0}; unsigned char stack[35];
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        unsigned char *root=object(&c,35);
        neoclr_probe_storage slot={&root,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        if(mode==2) {
            CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
            experiment_gc_live_wait();
            root=object(&c,35);
        }
        unsigned char *destination=mode==0?stack:root;
        memset(destination+8,0xcc,27);
        uint64_t bits=UINT64_C(0x8877665544332211);
        CHECK(!experiment_gc_store(&c,destination+offset,bits,widths[w]));
        unsigned char expected[8];
        switch(widths[w]) {
            case 1: { uint8_t v=(uint8_t)bits; memcpy(expected,&v,1); break; }
            case 2: { uint16_t v=(uint16_t)bits; memcpy(expected,&v,2); break; }
            case 4: { uint32_t v=(uint32_t)bits; memcpy(expected,&v,4); break; }
            default: memcpy(expected,&bits,8); break;
        }
        for(unsigned j=8;j<35;j++)
            CHECK(destination[j]==(j>=offset && j<offset+widths[w]?expected[j-offset]:0xcc));
        unsigned char saved[27]; memcpy(saved,destination+8,27);
        unsigned invalid[]={0,3,5,16};
        for(unsigned j=0;j<4;j++) {
            CHECK(experiment_gc_store(&c,destination+8,0,invalid[j])==3);
            CHECK(!memcmp(saved,destination+8,27));
        }
        if(mode==2) CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 291 outside/inactive/new unaligned scalar ranges and invalid-width rejection");
}
static void cleaned(void *state) { (*(unsigned *)state)++; }
static void cancellation(void) {
    uint64_t heap[256]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}}, foreign={0};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24),*unpublished=object(&c,24);
    unsigned cleanups=0; neoclr_gc_resource resource={0};
    CHECK(!neoclr_gc_resource_attach_v1(&c,unpublished,&resource,&cleanups,cleaned));
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    experiment_gc_live_wait();
    uint64_t collections=neoclr_gc_statistics_v1().collections;
    uint64_t stack=0;
    CHECK(!experiment_gc_store(&c,&stack,123,8) && stack==123);
    for(uint64_t i=0;i<GATE_WORDS/2+1;i++) CHECK(!experiment_gc_store(&c,root+1,i,8));
    CHECK(root[1]==GATE_WORDS/2 && *unpublished==17 && !cleanups);
    CHECK(neoclr_gc_statistics_v1().collections==collections);
    CHECK(experiment_gc_finish(&c,&frame,NULL,0)==3);
    CHECK(!experiment_gc_store(&c,root+2,(uintptr_t)unpublished,8));
    CHECK(!experiment_gc_store(&c,&stack,123,8) && stack==123);
    CHECK(experiment_gc_store(&foreign,&stack,55,8)==3 && stack==123);
    CHECK(experiment_gc_store(&c,root-1,55,8)==3);
    CHECK(experiment_gc_store(&c,(unsigned char *)root+23,55,2)==3);
    CHECK(experiment_gc_store(&c,c.text.data+c.text.used,55,8)==3);
    CHECK(experiment_gc_store(&c,&stack,55,3)==3 && stack==123);
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && *unpublished==17 && !cleanups);
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(cleanups==1);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static void construction(void) {
    uint64_t heap[256]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24),*child=object(&c,24),*array;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    experiment_gc_live_wait();
    CHECK(!neoclr_gc_allocate_v1(&c.text,25,NEOCLR_GC_STRINGS,(void **)&array));
    CHECK(!experiment_gc_store(&c,array,UINT64_C(0x80000004),8));
    CHECK(!experiment_gc_store(&c,array+1,1,8));
    CHECK(!experiment_gc_store(&c,array+2,(uintptr_t)child,8));
    CHECK(!experiment_gc_store(&c,(unsigned char *)array+24,1,1));
    CHECK(!experiment_gc_store(&c,root+1,(uintptr_t)array,8));
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0) && *child==17);
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    CHECK(experiment_gc_store(&c,array+1,0,8)==3 && array[1]==1);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static void detached_identity(void) {
    uint64_t heap[128]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24);
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0)); experiment_gc_live_wait();
    /* Deliberately violate immutable old identity while the worker is idle.
     * Restore it before remark; a mismatching descriptor cannot authorize writes. */
    uint64_t *header=root-4, saved_bytes=header[1], saved_kind=header[2];
    header[1]=23;
    CHECK(experiment_gc_store(&c,root+1,42,8)==3 && root[1]==0);
    header[1]=saved_bytes; header[2]=NEOCLR_GC_BYTES;
    CHECK(experiment_gc_store(&c,root+1,42,8)==3 && root[1]==0);
    header[2]=saved_kind;
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
/* A remembered payload may become interior storage after coalescing and reuse.
 * Header-looking user bytes must not authorize that stale allocation identity. */
static void cached_interior_header(int recent) {
    uint64_t heap[128]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    void *first=object(&c,32);
    uint64_t *root=object(&c,24),*tail=object(&c,24);
    uint64_t saved_header[4];
    memcpy(saved_header,(unsigned char *)root-sizeof(saved_header),sizeof(saved_header));
    neoclr_probe_storage slots[]={{&root,8,0},{&tail,8,0}};
    neoclr_probe_frame frame={.context=&c,.storage=slots,.storage_count=2};
    CHECK(!experiment_gc_store(&c,root+1,42,8));
    if (recent) CHECK(!experiment_gc_store(&c,tail+1,43,8));
    root=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    root=object(&c,88);CHECK((void *)root==first);
    /* Private descriptor fixture: allocated 32-byte payload, 64-byte span.
     * It appears at the old header address inside the new 88-byte payload. */
    saved_header[0]=64;saved_header[1]=32;
    memcpy((unsigned char *)root+32,saved_header,sizeof(saved_header));
    unsigned char *last=(unsigned char *)root+87;
    unsigned char before[2];memcpy(before,last,2);
    CHECK(experiment_gc_store(&c,last,0x1234,2)==3);
    CHECK(!memcmp(last,before,2));
    root=tail=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    puts("PASS stale cached interior header cannot extend the actual allocation");
}
/* Reuse is a proof of unchanged constraints, not permission to trust stale
 * bytes. Invalid changes reject; a valid runtime kind transition refreshes. */
static void cached_header_changes(void) {
    uint64_t heap[256]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24),*header=root-4;
    CHECK(!experiment_gc_store(&c,root+1,42,8));
    uint64_t original[4];memcpy(original,header,sizeof(original));
    struct { unsigned field; uint64_t value; } corruptions[]={
        {0,41},{0,UINT64_MAX},{1,0},{1,UINT64_MAX},{2,0},{2,7},{3,8},{3,2}
    };
    for(unsigned i=0;i<sizeof(corruptions)/sizeof(corruptions[0]);i++) {
        header[corruptions[i].field]=corruptions[i].value;
        CHECK(experiment_gc_store(&c,root+1,99,8)==3 && root[1]==42);
        memcpy(header,original,sizeof(original));
        CHECK(!experiment_gc_store(&c,root+1,42,8));
    }
    /* A valid shorter descriptor must not inherit its former writable extent. */
    header[1]=8;
    CHECK(experiment_gc_store(&c,root+2,99,8)==3 && root[2]==0);
    memcpy(header,original,sizeof(original));
    void *cell; CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_INTERN,&cell));
    CHECK(!experiment_gc_store(&c,cell,0,8));
    CHECK(!neoclr_gc_complete_invocation_v1(&c));
    CHECK(((uint64_t *)cell)[-2]==NEOCLR_GC_BYTES);
    CHECK(!experiment_gc_store(&c,cell,17,8) && *(uint64_t *)cell==17);
    CHECK(!experiment_gc_store(&c,cell,18,8) && *(uint64_t *)cell==18);
    neoclr_probe_frame frame={.context=&c};
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    puts("PASS cached header constraints: mutation rejection, extent shrink and intern conversion");
}
static void cached_range_lifetime(void) {
    uint64_t heap[128]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    for(unsigned round=0;round<100;round++) {
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *root=object(&c,24),*old=root;
        neoclr_probe_storage slot={&root,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        uint64_t before[2],after[2];experiment_gc_store_lookup_counts(before);
        CHECK(!experiment_gc_store(&c,root+1,42,8));
        CHECK(!experiment_gc_store(&c,root+2,43,8));
        experiment_gc_store_lookup_counts(after);
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
        CHECK(after[0]>before[0]);
#endif
        CHECK(experiment_gc_store(&c,(unsigned char *)root+23,44,2)==3);
        CHECK(root[1]==42 && root[2]==43);
        /* Sweep removes the exact-base proof before poisoning/coalescing. */
        root=NULL;CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(experiment_gc_store(&c,old+1,99,8)==3);
        /* Reuse the same base with smaller bounds; old tail is not writable. */
        root=object(&c,16);CHECK(root==old);
        CHECK(experiment_gc_store(&c,old+2,99,8)==3);
        CHECK(!experiment_gc_store(&c,root+1,77,8) && root[1]==77);
        root=NULL;CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
        /* Reacquiring the same native arena address must not revive old proofs. */
    }
    puts("PASS 100 cached store ranges: bounds, reclamation, smaller reuse and owner reacquisition");
}
typedef struct isolated_owners {
    pthread_mutex_t mutex;
    pthread_cond_t changed;
    unsigned arrived, phase;
    neoclr_aot_context *contexts[2];
} isolated_owners;
typedef struct isolated_argument { isolated_owners *owners; unsigned index; } isolated_argument;
static void rendezvous(isolated_owners *owners) {
    CHECK(!pthread_mutex_lock(&owners->mutex));
    unsigned phase=owners->phase;
    if(++owners->arrived==2) {
        owners->arrived=0; owners->phase++;
        CHECK(!pthread_cond_broadcast(&owners->changed));
    } else while(owners->phase==phase)
        CHECK(!pthread_cond_wait(&owners->changed,&owners->mutex));
    CHECK(!pthread_mutex_unlock(&owners->mutex));
}
static void *isolated_thread(void *argument) {
    isolated_argument *arg=argument;
    uint64_t counts[5]; experiment_gc_store_counts(counts);
    for(unsigned i=0;i<5;i++) CHECK(!counts[i]);
    uint64_t heap[256]={0}, outside=0;
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24);
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    experiment_gc_live_wait();
    arg->owners->contexts[arg->index]=&c;
    rendezvous(arg->owners);
    CHECK(experiment_gc_store(arg->owners->contexts[1-arg->index],&outside,42,8)==3);
    CHECK(!outside);
    CHECK(!experiment_gc_store(&c,&outside,10+arg->index,8));
    CHECK(!experiment_gc_store(&c,root+1,20+arg->index,8));
    CHECK(outside==10+arg->index && root[1]==20+arg->index);
    experiment_gc_store_counts(counts);
    CHECK(counts[0]==1 && counts[1]==0 && counts[2]==1 && counts[3]==0 && counts[4]==1);
    rendezvous(arg->owners); /* Keep both foreign context addresses live. */
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    return NULL;
}
static void isolated_threads(void) {
    uint64_t before[5],after[5]; experiment_gc_store_counts(before);
    isolated_owners owners={.mutex=PTHREAD_MUTEX_INITIALIZER,.changed=PTHREAD_COND_INITIALIZER};
    isolated_argument args[2]={{&owners,0},{&owners,1}};
    pthread_t threads[2];
    for(unsigned i=0;i<2;i++) CHECK(!pthread_create(&threads[i],NULL,isolated_thread,&args[i]));
    for(unsigned i=0;i<2;i++) CHECK(!pthread_join(threads[i],NULL));
    CHECK(!pthread_cond_destroy(&owners.changed)); CHECK(!pthread_mutex_destroy(&owners.mutex));
    experiment_gc_store_counts(after); CHECK(!memcmp(before,after,sizeof(before)));
    puts("PASS isolated owner threads: concurrent epochs, foreign-context rejection and private counters");
}
int main(void) {
    ranges(); cancellation(); construction(); detached_identity();
    uint64_t counts[5]; experiment_gc_store_counts(counts);
    CHECK(counts[0]==2 && counts[1]>=5 && counts[2]>=97 && counts[3]==1 && counts[4]==8);
    direct_ranges();
    cached_header_changes();
    cached_range_lifetime();
    cached_interior_header(0);
    cached_interior_header(1);
    isolated_threads();
    puts("PASS 97 scalar ranges, stack/byref classification, new array construction and noncollecting overflow recovery");
}
