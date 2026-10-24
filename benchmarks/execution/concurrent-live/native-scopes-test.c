#include "native-gc.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"native scopes line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_native_enter_v1(neoclr_aot_context *,uint64_t *);
extern int32_t experiment_gc_native_leave_v1(neoclr_aot_context *,uint64_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_native_scope_counts(uint64_t *);
extern uint64_t experiment_gc_live_capabilities_v1(uint32_t);
static void *foreign_thread(void *context) {
    uint64_t token=123;
    CHECK(experiment_gc_native_enter_v1(context,&token)==3 && token==123);
    return NULL;
}
static uint64_t *object(neoclr_aot_context *c) {
    void *p;CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,&p));
    ((uint64_t *)p)[0]=17;return p;
}
int main(void) {
    CHECK(experiment_gc_live_capabilities_v1(1)&16);
    for(unsigned round=0;round<16;round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}}, foreign={0};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t unframed=123;
        CHECK(experiment_gc_native_enter_v1(&c,&unframed)==3 && unframed==123);
        uint64_t *root=object(&c),*temporary=object(&c);
        neoclr_probe_storage slot={&root,8,0};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,1};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,&slot,1,&map,sizeof(map));
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        uint64_t collections=neoclr_gc_statistics_v1().collections, token=0;
        CHECK(!experiment_gc_native_enter_v1(&c,&token));
        CHECK(token && !experiment_gc_live_active(&c));
        CHECK(neoclr_gc_statistics_v1().collections==collections);
        uint64_t extent;uint32_t kind;
        CHECK(!neoclr_gc_allocation_info_v1(&c,temporary,&extent,&kind) && temporary[0]==17);
        /* Uninstrumented native payload write is safe only after cancellation. */
        root[1]=(uintptr_t)temporary;
        uint64_t nested[63],bad=123;
        for(unsigned i=0;i<63;i++) CHECK(!experiment_gc_native_enter_v1(&c,nested+i));
        CHECK(experiment_gc_native_enter_v1(&c,&bad)==3 && bad==123);
        CHECK(experiment_gc_native_enter_v1(&foreign,&bad)==3 && bad==123);
        CHECK(experiment_gc_native_leave_v1(&c,token)==3);
        CHECK(neoclr_gc_heap_release_v1(&c.text)==3);
        CHECK(experiment_gc_begin(&c,&frame,NULL,0)==3);
        CHECK(neoclr_gc_collect_v1(&foreign,&frame)==3);
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && temporary[0]==17);
        CHECK(!neoclr_gc_idle_v1(&c,&frame) && !experiment_gc_live_active(&c));
        for(unsigned i=0;i<4100;i++) CHECK(!neoclr_gc_boundary_v1(&c,&frame));
        CHECK(!experiment_gc_live_active(&c));
        /* Rooted native allocators may collect while nested; all such work is synchronous. */
        for(unsigned i=0;i<300;i++) {
            void *garbage;
            CHECK(!neoclr_allocate_object_rooted_v1(&c.text,17,16,&garbage));
        }
        CHECK(!experiment_gc_live_active(&c) && temporary[0]==17);
        for(unsigned i=63;i>0;i--) CHECK(!experiment_gc_native_leave_v1(&c,nested[i-1]));
        pthread_t worker;CHECK(!pthread_create(&worker,NULL,foreign_thread,&c));CHECK(!pthread_join(worker,NULL));
        uint64_t *result;
        CHECK(!neoclr_allocate_object_rooted_v1(&c.text,17,24,(void **)&result));
        collections=neoclr_gc_statistics_v1().collections;
        CHECK(!experiment_gc_native_leave_v1(&c,token));
        CHECK(neoclr_gc_statistics_v1().collections==collections && !experiment_gc_live_active(&c));
        CHECK(!neoclr_gc_allocation_info_v1(&c,result,&extent,&kind));
        CHECK(experiment_gc_native_leave_v1(&c,token)==3);
        uint64_t next;
        CHECK(!experiment_gc_native_enter_v1(&c,&next) && next!=token);
        CHECK(experiment_gc_native_leave_v1(&c,token)==3);
        CHECK(!experiment_gc_native_leave_v1(&c,next));
        root[2]=(uintptr_t)result;
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
#ifdef NEOCLR_GC_PACED
        CHECK(experiment_gc_live_active(&c));
#endif
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(result[0]==17 && temporary[0]==17);
        root=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    uint64_t counts[5];experiment_gc_native_scope_counts(counts);
    CHECK(counts[0]==16*65 && counts[1]==16 && counts[2]>0 && counts[3]==64 && !counts[4]);
    puts("PASS native scopes: cancellation, unpublished results, nested sync collection, pacing suppression, token/owner rejection and resumption");
}
