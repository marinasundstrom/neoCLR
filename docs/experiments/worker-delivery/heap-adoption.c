#include "heap-adoption.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d: %s\n",__LINE__,#x); abort(); } } while (0)
static void *transfer(void *state) {
    neoclr_aot_context *c=state;
    CHECK(!neoclr_gc_heap_adopt_exclusive_v1(&c->text));
    uint64_t root;
    void *value=NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text,8,NEOCLR_GC_BYTES,&value));
    *(uint64_t *)value=99;
    CHECK(!neoclr_gc_host_root_create_v1(c,value,&root));
#ifdef DELIVERY_CONCURRENT
    extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
    extern void experiment_gc_candidate_counts(uint64_t *);
    CHECK(!experiment_gc_begin(c,NULL,NULL,0));
    uint64_t epochs[6]; experiment_gc_candidate_counts(epochs);
    CHECK(epochs[0] && epochs[5]);
#endif
    /* Release must join before the arena/context or roots can migrate. */
    CHECK(!neoclr_gc_heap_release_v1(&c->text));
#ifdef DELIVERY_CONCURRENT
    experiment_gc_candidate_counts(epochs);
    CHECK(!epochs[5] && epochs[0]==epochs[1]+epochs[2]);
#endif
    CHECK(*(uint64_t *)value==99);
    CHECK(!neoclr_gc_host_root_release_v1(c,root));
    return NULL;
}
int main(void) {
    uint64_t storage[1024]={0}, saved[1024];
    neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
    void *first, *gap, *last;
    CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&first));
    CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&gap));
    CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&last));
    *(uint64_t *)first=41; *(uint64_t *)last=43;
    uint64_t a,b;
    CHECK(!neoclr_gc_host_root_create_v1(&c,first,&a));
    CHECK(!neoclr_gc_host_root_create_v1(&c,last,&b));
    CHECK(!neoclr_gc_collect_v1(&c,NULL));
    memcpy(saved,storage,sizeof(storage));
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3); /* TLS roots not transferable. */
    CHECK(!memcmp(saved,storage,sizeof(storage)));
    CHECK(!neoclr_gc_host_root_release_v1(&c,a));
    CHECK(!neoclr_gc_host_root_release_v1(&c,b));
    uint64_t used=c.text.used;
    storage[0]=1; /* Bad leading span. Full validation must fail without repair. */
    memcpy(saved,storage,sizeof(storage));
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    CHECK(!memcmp(saved,storage,sizeof(storage)) && c.text.used==used);
    storage[0]=40;
    /* Reject a malformed trailing block even after valid leading allocations. */
    uint64_t *last_header=(uint64_t *)last-4;
    uint64_t span=last_header[0]; last_header[0]=1;
    memcpy(saved,storage,sizeof(storage));
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    CHECK(!memcmp(saved,storage,sizeof(storage)));
    last_header[0]=span;
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(NULL)==3);
#ifdef DELIVERY_CONCURRENT
    extern void experiment_gc_index_fail_next(void);
    experiment_gc_index_fail_next();
    memcpy(saved,storage,sizeof(storage));
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    CHECK(!memcmp(saved,storage,sizeof(storage)) && c.text.used==used);
#endif
    neoclr_probe_frame frame;
    neoclr_parked_roots parked={0};
    neoclr_probe_storage slot={&first,8,0};
    neoclr_gc_enter_v1(&frame,&c,0,&slot,1,"[]",2);
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    neoclr_roots_park_v1(&parked);
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    neoclr_roots_resume_v1(&parked);
    neoclr_gc_leave_v1(&frame);
    uint64_t other_storage[16]={0};
    neoclr_aot_text_arena other={(unsigned char *)other_storage,sizeof(other_storage),0};
    CHECK(!neoclr_gc_heap_acquire_v1(&other));
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    CHECK(!neoclr_gc_heap_release_v1(&other));
    CHECK(!neoclr_gc_heap_adopt_exclusive_v1(&c.text));
    CHECK(neoclr_gc_heap_adopt_exclusive_v1(&c.text)==3);
    CHECK(!neoclr_gc_host_root_create_v1(&c,first,&a));
    CHECK(!neoclr_gc_host_root_create_v1(&c,last,&b));
    void *reuse=NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&reuse) && reuse==gap);
    CHECK(*(uint64_t *)first==41 && *(uint64_t *)last==43);
    CHECK(!neoclr_gc_host_root_release_v1(&c,a));
    CHECK(!neoclr_gc_host_root_release_v1(&c,b));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    /* pthread join supplies exclusive handoff; no concurrent mutators. */
    for (unsigned i=0;i<8;i++) {
        pthread_t worker;
        CHECK(!pthread_create(&worker,NULL,transfer,&c));
        CHECK(!pthread_join(worker,NULL));
        CHECK(*(uint64_t *)first==41 && *(uint64_t *)last==43);
    }
    /* Release leaves an unowned live arena; the next lease rebuilds its indexes. */
    CHECK(!neoclr_gc_heap_adopt_exclusive_v1(&c.text));
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    puts("PASS exclusive adoption: rejection atomicity, malformed spans, TLS roots, free reuse, live bases and reacquisition");
}
