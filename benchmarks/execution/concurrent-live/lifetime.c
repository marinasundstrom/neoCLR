#include "native-gc.h"
#include "gate.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int experiment_gc_write(neoclr_aot_context *,void *,const mutation_patch *,size_t);
extern void experiment_gc_shadow_gate(int);
extern void experiment_gc_live_wait(void);
extern void experiment_gc_batch_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while(0)
static uint64_t *object(neoclr_aot_context *c) {
    uint64_t *p;
    CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&p));
    p[0]=17;
    return p;
}
static void set(neoclr_aot_context *c,uint64_t *p,unsigned slot,uint64_t v) {
    mutation_patch patch={slot*8,8,&v};
    CHECK(!experiment_gc_write(c,p,&patch,1));
}
static void deleted_edge(void) {
    uint64_t heap[512]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c),*parent=object(&c),*child=object(&c),*dead=object(&c);
    root[1]=(uintptr_t)parent; parent[1]=(uintptr_t)child;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    experiment_gc_shadow_gate(1);
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    set(&c,parent,1,0);
    uint64_t *fresh=object(&c);
    set(&c,parent,2,(uintptr_t)fresh);
    experiment_gc_shadow_gate(0);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    CHECK(*child==17 && *parent==17 && *fresh==17);
    CHECK(*dead==UINT64_C(0xdddddddddddddddd));
    root=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static void late_insertion(void) {
    uint64_t heap[512]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c),*parent=object(&c),*white=object(&c),*child=object(&c);
    root[1]=(uintptr_t)parent; white[1]=(uintptr_t)child;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    experiment_gc_live_wait();
    set(&c,parent,1,(uintptr_t)white);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    CHECK(*white==17 && *child==17);
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static void future_address(void) {
    for (unsigned hole=0;hole<2;hole++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *root=object(&c),*parent=object(&c),*free_slot=NULL;
        root[1]=(uintptr_t)parent;
        if (hole) { free_slot=object(&c); parent[2]=(uintptr_t)object(&c); }
        neoclr_probe_storage slot={&root,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        uintptr_t future=hole?(uintptr_t)free_slot:(uintptr_t)c.text.data+c.text.used+32;
        parent[1]=future+7;
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        experiment_gc_live_wait();
        uint64_t *fresh=object(&c); CHECK((uintptr_t)fresh==future);
        fresh[1]=(uintptr_t)object(&c);
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        CHECK(*fresh==17 && *(uint64_t *)(uintptr_t)fresh[1]==17);
        root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
}
static void array_publication(void) {
    uint64_t heap[512]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c),*child=object(&c),*array;
    CHECK(!neoclr_gc_allocate_v1(&c.text,25,NEOCLR_GC_STRINGS,(void **)&array));
    array[0]=UINT64_C(0x80000004); array[1]=1; root[1]=(uintptr_t)array;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    experiment_gc_live_wait();
    uint64_t value=(uintptr_t)child; unsigned char marker=1;
    mutation_patch patches[]={{16,8,&value},{24,1,&marker}};
    CHECK(!experiment_gc_write(&c,array,patches,2));
    mutation_patch invalid={8,8,&value};
    CHECK(experiment_gc_write(&c,array,&invalid,1)==GATE_INVALID);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0) && *child==17);
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static void overflow(void) {
    uint64_t heap[512]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c),*dead=object(&c);
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    experiment_gc_shadow_gate(1);
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    for(unsigned i=0;i<GATE_WORDS/2;i++) set(&c,root,1,i);
    uint64_t value=999; mutation_patch patch={8,8,&value};
    CHECK(experiment_gc_write(&c,root,&patch,1)==GATE_CANCELLED && root[1]==GATE_WORDS/2-1);
    experiment_gc_shadow_gate(0);
    CHECK(experiment_gc_finish(&c,&frame,NULL,0)==3);
    CHECK(*dead==17); /* failed epoch must not sweep */
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(*dead==UINT64_C(0xdddddddddddddddd));
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static unsigned random_state=9817;
static unsigned random_word(void) {
    random_state ^= random_state << 13;
    random_state ^= random_state >> 17;
    random_state ^= random_state << 5;
    return random_state;
}
static void closure(unsigned char *live,int edges[120][2],unsigned count) {
    int changed;
    do {
        changed=0;
        for(unsigned i=0;i<count;i++) if(live[i])
            for(unsigned j=0;j<2;j++) if(edges[i][j]>=0 && !live[edges[i][j]]) {
                live[edges[i][j]]=1; changed=1;
            }
    } while(changed);
}
static void randomized(void) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[2048]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *nodes[120]; int edges[120][2];
        for(unsigned i=0;i<80;i++) nodes[i]=object(&c);
        for(unsigned i=0;i<80;i++) for(unsigned j=0;j<2;j++) {
            unsigned value=random_word()%120;
            edges[i][j]=value<80?(int)value:-1;
            nodes[i][j+1]=value<80?(uintptr_t)nodes[value]:0;
        }
        unsigned char expected[120]={0}; expected[0]=1;
        closure(expected,edges,80);
        uint64_t root=(uintptr_t)nodes[0];
        neoclr_probe_frame frame={.context=&c,.lanes=&root,.lane_count=1};
        experiment_gc_shadow_gate(1);
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        for(unsigned i=80;i<120;i++) nodes[i]=object(&c);
        for(unsigned i=80;i<120;i++) {
            expected[i]=1; /* first protocol retains every new allocation */
            for(unsigned j=0;j<2;j++) {
                unsigned value=random_word()%180;
                edges[i][j]=value<120?(int)value:-1;
                nodes[i][j+1]=value<120?(uintptr_t)nodes[value]:0;
            }
        }
        for(unsigned mutation=0;mutation<16;mutation++) {
            unsigned i=random_word()%80,j=random_word()%2,value=random_word()%180;
            if(edges[i][j]>=0) expected[edges[i][j]]=1;
            edges[i][j]=value<120?(int)value:-1;
            if(value<120) expected[value]=1;
            set(&c,nodes[i],j+1,value<120?(uintptr_t)nodes[value]:0);
        }
        unsigned next=random_word()%120;
        root=(uintptr_t)nodes[next]; expected[next]=1;
        closure(expected,edges,120);
        experiment_gc_shadow_gate(0);
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        for(unsigned i=0;i<120;i++) {
            uint64_t bytes; uint32_t kind;
            CHECK((neoclr_gc_allocation_info_v1(&c,nodes[i],&bytes,&kind)==0)==expected[i]);
        }
        root=0; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
}
static void large_payloads(void) {
    uint64_t heap[8192]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c),*child=object(&c),*large,*array;
    CHECK(!neoclr_gc_allocate_v1(&c.text,20000,NEOCLR_GC_OBJECT,(void **)&large));
    large[0]=17; large[2499]=(uintptr_t)child; root[1]=(uintptr_t)large;
    CHECK(!neoclr_gc_allocate_v1(&c.text,16+1024*9,NEOCLR_GC_STRINGS,(void **)&array));
    array[0]=UINT64_C(0x80000004); array[1]=1024; root[2]=(uintptr_t)array;
    array[2+1023]=(uintptr_t)child;
    ((unsigned char *)array)[16+1024*8+1023]=1;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0) && *child==17);
    root=NULL; CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
#ifdef NEOCLR_LIVE_BATCH
    uint64_t counts[5]; experiment_gc_batch_counts(counts);
    CHECK(counts[0]>0 && counts[1]==4096 && counts[2]==64);
#ifdef NEOCLR_LIVE_GATE_PROFILE
    CHECK(counts[3]>0 && counts[4]>=counts[3]);
#endif
    printf("BATCH %llu %llu %llu %llu %llu\n",(unsigned long long)counts[0],
        (unsigned long long)counts[1],(unsigned long long)counts[2],
        (unsigned long long)counts[3],(unsigned long long)counts[4]);
#endif
}
int main(void) {
    for(unsigned i=0;i<100;i++) deleted_edge();
    array_publication();
    future_address();
    late_insertion();
    overflow();
    randomized();
    large_payloads();
    puts("PASS live deletion/insertion/future-address/overflow cases and 12000 differential blocks");
}
