#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern void experiment_gc_index_fail_next(void);
extern void experiment_gc_index_counts(uint64_t *);
extern void *experiment_gc_index_before(const neoclr_aot_context *,uint64_t);
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern int32_t experiment_gc_prepare_entry_v1(neoclr_aot_context *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"index line %d\n",__LINE__); abort(); } } while(0)
static uint64_t random_state=1;
static uint64_t random_word(void) { random_state=random_state*UINT64_C(6364136223846793005)+1; return random_state; }
static void check(neoclr_aot_context *c,void **objects,uint64_t *sizes,unsigned count) {
    for(unsigned i=0;i<count;i++) if(objects[i]) {
        uint64_t bytes=0;uint32_t kind=0;
        CHECK(!neoclr_gc_allocation_info_v1(c,objects[i],&bytes,&kind));
        CHECK(bytes==sizes[i] && kind==NEOCLR_GC_OBJECT);
    }
    for(unsigned n=0;n<12000;n++) {
        uint64_t offset=random_word()%c->text.used;
        uintptr_t address=(uintptr_t)c->text.data+offset,expected=0;
        uint64_t bytes=0;
        for(unsigned i=0;i<count;i++) {
            uintptr_t p=(uintptr_t)objects[i];
            if (p && p<=address && p>expected) { expected=p; bytes=sizes[i]; }
        }
        CHECK((uintptr_t)experiment_gc_index_before(c,offset)==expected);
        uint64_t reported_bytes=0;uint32_t kind=0;
        int exact=expected && address==expected;
        CHECK(neoclr_gc_allocation_info_v1(c,(void *)address,&reported_bytes,&kind)==(exact?0:3));
        if(exact) CHECK(reported_bytes==bytes && kind==NEOCLR_GC_OBJECT);
        int writable=expected && address-expected<bytes;
        CHECK(experiment_gc_store(c,(void *)address,0,1)==(writable?0:3));
    }
}
int main(void) {
    unsigned char *heap=calloc(1,1024*1024);CHECK(heap);
    neoclr_aot_context c={.text={heap,1024*1024,0}};
    void *roots[1200]={0};uint64_t sizes[1200]={0};neoclr_probe_storage slots[1200];
    for(unsigned i=0;i<1200;i++)slots[i]=(neoclr_probe_storage){&roots[i],8,0};
    neoclr_probe_frame frame={.context=&c,.storage=slots,.storage_count=1200};
    experiment_gc_index_fail_next();
    CHECK(neoclr_gc_heap_acquire_v1(&c.text)==3 && !c.text.used);
    uint64_t counts[3];experiment_gc_index_counts(counts);
    CHECK(!counts[0] && !counts[1] && !counts[2]);
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    for(unsigned i=0;i<1000;i++) {
        sizes[i]=8+random_word()%280;
        CHECK(!neoclr_gc_allocate_v1(&c.text,sizes[i],NEOCLR_GC_OBJECT,&roots[i]));
    }
    check(&c,roots,sizes,1000);
    for(unsigned i=0;i<1000;i++)if(i%3)roots[i]=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    check(&c,roots,sizes,1000);
    for(unsigned i=1000;i<1200;i++) {
        sizes[i]=8+random_word()%400;
        CHECK(!neoclr_gc_allocate_v1(&c.text,sizes[i],NEOCLR_GC_OBJECT,&roots[i]));
    }
    check(&c,roots,sizes,1200);
    /* Reset while old payloads still contain header-looking bytes. */
    CHECK(!experiment_gc_prepare_entry_v1(&c) && !c.text.used);
    for(unsigned i=0;i<1200;i++)roots[i]=NULL;
    sizes[0]=4096;CHECK(!neoclr_gc_allocate_v1(&c.text,sizes[0],NEOCLR_GC_OBJECT,&roots[0]));
    check(&c,roots,sizes,1);
    roots[0]=NULL;CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    sizes[0]=4096;CHECK(!neoclr_gc_allocate_v1(&c.text,sizes[0],NEOCLR_GC_OBJECT,&roots[0]));
    check(&c,roots,sizes,1);
    roots[0]=NULL;CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    sizes[0]=sizes[2]=8;sizes[1]=128*1024;
    for(unsigned i=0;i<3;i++)CHECK(!neoclr_gc_allocate_v1(&c.text,sizes[i],NEOCLR_GC_OBJECT,&roots[i]));
    roots[1]=NULL;CHECK(!neoclr_gc_collect_v1(&c,&frame));
    check(&c,roots,sizes,3); /* Empty summary groups inside a large reclaimed hole. */
    roots[0]=roots[2]=NULL;CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    experiment_gc_index_counts(counts);CHECK(counts[0]==3 && !counts[1] && counts[2]==16640);
    neoclr_aot_text_arena empty={0};
    CHECK(!neoclr_gc_heap_acquire_v1(&empty));CHECK(!neoclr_gc_heap_release_v1(&empty));
    experiment_gc_index_counts(counts);CHECK(counts[0]==3 && !counts[1]);
    free(heap);
    puts("PASS 72000 allocation-index/model and checked-store comparisons across reclaim, reuse, reset and reacquisition");
}
