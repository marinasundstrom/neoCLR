#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern void experiment_gc_store_walk_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) abort(); } while(0)
int main(void) {
    uint64_t heap[128]={0};
    neoclr_aot_context context={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
    uint64_t *objects[3];
    for(unsigned i=0;i<3;i++)
        CHECK(!neoclr_gc_allocate_v1(&context.text,24,NEOCLR_GC_OBJECT,(void **)&objects[i]));
    CHECK(!experiment_gc_store(&context,objects[2]+1,42,8));
    uint64_t counts[3];experiment_gc_store_walk_counts(counts);
    CHECK(counts[0]==3 && counts[1]==3 && counts[2]==0);
    CHECK(!experiment_gc_store(&context,objects[2]+2,43,8));
    experiment_gc_store_walk_counts(counts);
    CHECK(counts[0]==3 && counts[1]==3 && counts[2]==0);
    CHECK(!experiment_gc_store(&context,objects[1]+1,44,8));
    experiment_gc_store_walk_counts(counts);
    CHECK(counts[0]==5 && counts[1]==3 && counts[2]==0);
    uint64_t *fourth;
    CHECK(!neoclr_gc_allocate_v1(&context.text,24,NEOCLR_GC_OBJECT,(void **)&fourth));
    CHECK(!experiment_gc_store(&context,fourth+1,45,8));
    experiment_gc_store_walk_counts(counts);
#if defined(NEOCLR_LIVE_FULL_STORE_WALK)
    CHECK(counts[0]==9 && counts[1]==4 && counts[2]==0);
#elif NEOCLR_LIVE_STORE_RANGES == 1
    CHECK(counts[0]==8 && counts[1]==3 && counts[2]==0);
#else
    CHECK(counts[0]==7 && counts[1]==3 && counts[2]==0);
#endif
    neoclr_probe_frame frame={.context=&context};
    CHECK(!neoclr_gc_collect_v1(&context,&frame) && !context.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&context.text));
    puts("PASS exact fallback header counts and cache-hit exclusion");
}
