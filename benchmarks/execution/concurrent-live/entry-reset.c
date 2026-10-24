#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <pthread.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern int32_t experiment_gc_prepare_entry_v1(neoclr_aot_context *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"entry reset line %d: %s\n",__LINE__,#x); abort(); } } while (0)
static uint64_t *allocate(neoclr_aot_context *c) {
    uint64_t *p;
    CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&p));
    p[0]=17; p[1]=42;
    return p;
}
static void *wrong_thread(void *context) {
    CHECK(experiment_gc_prepare_entry_v1(context)==3);
    return NULL;
}
int main(void) {
    for (unsigned round=0; round<100; round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(experiment_gc_prepare_entry_v1(NULL)==3);
        CHECK(experiment_gc_prepare_entry_v1(&c)==3); /* Not owned. */
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *dead=allocate(&c), *retained=allocate(&c);
        uint64_t handle;
        CHECK(!neoclr_gc_host_root_create_v1(&c,retained,&handle));
        CHECK(!neoclr_gc_collect_v1(&c,NULL)); /* Leave a free-list hole. */
        CHECK(*dead==UINT64_C(0xdddddddddddddddd));
        c.static_storage=retained;
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        uint64_t used=c.text.used;
        pthread_t other;
        CHECK(!pthread_create(&other,NULL,wrong_thread,&c));
        CHECK(!pthread_join(other,NULL));
        CHECK(experiment_gc_prepare_entry_v1(&c)==3);
        CHECK(experiment_gc_live_active(&c) && c.text.used==used);
        CHECK(c.static_storage==retained && retained[1]==42);
        CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
        static const neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,0};
        static const neoclr_probe_storage empty={0};
        neoclr_probe_frame frame={0};
        void *state=neoclr_gc_state_enter_v1(&frame,&c,0,&empty,0,&map,sizeof(map));
        CHECK(state && neoclr_root_probe_head_v1()==&frame);
        CHECK(experiment_gc_prepare_entry_v1(&c)==3 && c.text.used==used);
        neoclr_gc_state_leave_v1(state,&frame);
        CHECK(!neoclr_root_probe_head_v1());
        /* Enter may complete an epoch; start a new one for the rejection test. */
        if (!experiment_gc_live_active(&c)) CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        /* Corrupt host cursor must fail without cancelling or repairing it. */
        c.text.used=used-8;
        CHECK(experiment_gc_prepare_entry_v1(&c)==3);
        CHECK(c.text.used==used-8 && experiment_gc_live_active(&c));
        c.text.used=used;
        CHECK(!experiment_gc_prepare_entry_v1(&c));
        CHECK(!experiment_gc_live_active(&c) && !c.text.used && !c.static_storage);
        /* This hits the old positive base cache unless reset clears it. */
        uint64_t rejected=777;
        CHECK(neoclr_gc_host_root_create_v1(&c,retained,&rejected)==3 && rejected==777);
        uint64_t *fresh=allocate(&c);
        CHECK(fresh==dead && fresh[1]==42);
        CHECK(!neoclr_gc_host_root_create_v1(&c,fresh,&handle));
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && fresh[1]==42);
        CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
        CHECK(!experiment_gc_prepare_entry_v1(&c) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 100 owned entry resets: rejection, holes, cache invalidation and reuse");
}
