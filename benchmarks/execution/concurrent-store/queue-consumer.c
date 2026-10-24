#include "native-gc.h"
#include "task-queue.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
typedef int32_t (*drain_fn)(neoclr_aot_context *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"queue dispatch line %d: %s\n",__LINE__,#x); abort(); } } while (0)
int managed_queue_consumer(drain_fn drain) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        CHECK(drain(&c)==3); /* No task scope. */
        neoclr_task_scope scope;
        CHECK(!neoclr_task_scope_enter_v1(&scope,&c));
        CHECK(!drain(&c)); /* Empty queue. */
        uint64_t *queue;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&queue));
        queue[0]=1;queue[1]=0;queue[2]=0;
        int32_t unit=-1;
        CHECK(!neoclr_task_queue_register_v1(queue,&c,&unit) && unit==0);
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!drain(&c) && queue[1]==1 && queue[2]);
        CHECK(!neoclr_root_probe_head_v1());
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && queue[1]==1 && queue[2]);
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        c.fault.code=9;
        CHECK(drain(&c)==3 && queue[1]==1 && c.fault.code==9);
        c.fault.code=0;
        static const neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,0};
        static const neoclr_probe_storage empty={0};
        neoclr_probe_frame frame={0};
        void *state=neoclr_gc_state_enter_v1(&frame,&c,0,&empty,0,&map,sizeof(map));
        CHECK(state && drain(&c)==3 && queue[1]==1);
        neoclr_gc_state_leave_v1(state,&frame);
        CHECK(!experiment_gc_store(&c,queue+1,UINT64_MAX,8));
        CHECK(drain(&c)!=0 && c.fault.code && queue[1]==0 && queue[2]);
        CHECK(!neoclr_root_probe_head_v1());
        c.fault.code=0;
        CHECK(!drain(&c) && queue[1]==1);
        CHECK(!neoclr_task_scope_leave_v1(&scope));
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 100 generated queue cycles: empty, ownership, allocation, rejection and faults");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t neoclr_drain_default_queue_v1(neoclr_aot_context *);
int main(void) { return managed_queue_consumer(neoclr_drain_default_queue_v1); }
#endif
