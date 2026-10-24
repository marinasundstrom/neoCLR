#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
extern void experiment_gc_candidate_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while(0)
static void cleanup(void *p) { (*(int *)p)++; }
int main(void) {
    uint64_t heap[2048] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(!neoclr_gc_idle_v1(&c,NULL));
    void *owner;
    CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&owner));
    neoclr_gc_resource resource={0}; int cleaned=0;
    CHECK(!neoclr_gc_resource_attach_v1(&c,owner,&resource,&cleaned,cleanup));
    CHECK(!neoclr_gc_idle_v1(&c,NULL) && cleaned==1 && !c.text.used);
    CHECK(!neoclr_gc_idle_v1(&c,NULL) && cleaned==1 && !c.text.used);
    /* Unreachable new objects are reclaimed in the completing epoch. */
    uint64_t peak=0;
    for(unsigned round=0;round<1000;round++) {
        for(unsigned i=0;i<64;i++) {
            void *p;
            CHECK(!neoclr_gc_allocate_rooted_v1(&c,NULL,16,NEOCLR_GC_OBJECT,&p));
        }
        if(c.text.used>peak) peak=c.text.used;
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
    }
    CHECK(peak<=64*48);
    uint64_t counts[6]; experiment_gc_candidate_counts(counts);
    CHECK(counts[1]==1002 && counts[3]==0);
    /* Explicit requests still release floating garbage synchronously. */
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    experiment_gc_candidate_counts(counts);
    CHECK(counts[3]==1 && !counts[5]);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    printf("PASS immediate unreachable cleanup and 1000 churn cycles; peak=%llu\n",(unsigned long long)peak);
    return 0;
}
