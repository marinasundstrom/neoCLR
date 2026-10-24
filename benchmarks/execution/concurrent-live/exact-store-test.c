#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern void experiment_gc_store_lookup_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while(0)
int main(void) {
    uint64_t heap[8192]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    unsigned char *tiny;
    CHECK(!neoclr_gc_allocate_v1(&c.text,1,NEOCLR_GC_BYTES,(void **)&tiny));
    CHECK(experiment_gc_store(&c,tiny,0xabcd,2)==3 && *tiny==0);
    CHECK(!experiment_gc_store(&c,tiny,42,1) && *tiny==42);
    uint64_t *values[300];
    for(unsigned i=0;i<300;i++) {
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&values[i]));
        uint64_t before[2],after[2];experiment_gc_store_lookup_counts(before);
        CHECK(!experiment_gc_store(&c,values[i],17,8));
        CHECK(!experiment_gc_store(&c,values[i]+1,i,8));
        experiment_gc_store_lookup_counts(after);
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_NO_EXACT_STORE)
        CHECK(after[1]==before[1]);
        CHECK(after[0]==before[0]+2);
#endif
    }
    /* Earlier proof entries may have been evicted. Checked lookup remains valid. */
    CHECK(!experiment_gc_store(&c,values[0],19,8) && values[0][0]==19);
    neoclr_probe_frame frame={.context=&c};
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(experiment_gc_store(&c,values[0],21,8)==3);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    puts("PASS exact-base stores: no walks, short bounds, proof eviction and reclamation");
    return 0;
}
