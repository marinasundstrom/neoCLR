#include "native-runtime.h"
#include "socket-listener.h"
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"async wrappers line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern int32_t experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
void neoclr_scheduler_wait_v1(neoclr_aot_context *c) { (void)c; }
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c,uint64_t *out,const neoclr_probe_frame *f) {
    (void)c;(void)out;(void)f;return NEOCLR_SCHEDULER_IDLE;
}
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *c,uint64_t id,const neoclr_probe_frame *f) {
    (void)c;(void)id;(void)f;return 3;
}
static int32_t drain(neoclr_aot_context *c) { (void)c;return 0; }
extern void experiment_gc_store_counts(uint64_t *);
typedef int32_t (*create_fn)(void *,neoclr_aot_context *);
typedef int32_t (*operation_fn)(uint64_t,void *,neoclr_aot_context *);
int managed_async_consumer(void *const *functions) {
    create_fn create=(create_fn)functions[0];
    operation_fn complete=(operation_fn)functions[1],await=(operation_fn)functions[2],
        release=(operation_fn)functions[3],iscompleted=(operation_fn)functions[4],wait=(operation_fn)functions[5];
    neoclr_async_callback invoke=(neoclr_async_callback)functions[6];
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *owner,*descriptor;
        CHECK(!neoclr_gc_allocate_v1(&c.text,32,NEOCLR_GC_OBJECT,(void **)&owner));
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&descriptor));
        owner[0]=17;descriptor[0]=UINT32_MAX;descriptor[1]=1;descriptor[2]=(uintptr_t)owner;
        uint64_t keep,body;CHECK(!neoclr_gc_host_root_create_v1(&c,owner,&keep));
        CHECK(!create(owner+3,&c) && owner[3]);
        uint64_t operation=owner[3];
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!neoclr_runtime_async_startmanaged_v1(owner,descriptor,&c,&body,invoke));
        CHECK(!owner[1] && !owner[2] && !neoclr_root_probe_head_v1());
        CHECK(!neoclr_gc_collect_v1(&c,NULL));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        CHECK(!iscompleted(operation,owner+1,&c) && !owner[1]);
        experiment_gc_store_counts(after);
        CHECK(after[2]==before[2]+1 && experiment_gc_live_active(&c));
        CHECK(!experiment_gc_store(&c,owner+1,123,8));
        CHECK(wait(operation,owner+1,&c)==3 && owner[1]==123);
        c.fault.code=0;
        CHECK(await(operation,owner+1,&c)==3 && owner[1]==123);
        c.fault.code=0;
        CHECK(!complete(operation,owner+1,&c) && (uint32_t)owner[1]==1);
        uint64_t collections=neoclr_gc_statistics_v1().collections;
        if(!(round&1)) {
            void *garbage;
            while(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,&garbage)) {}
        }
        CHECK(!neoclr_runtime_async_pump_v1(&c,invoke,drain));
        CHECK((uint32_t)owner[1]==42 && owner[2] && !neoclr_root_probe_head_v1());
        if(!(round&1)) CHECK(neoclr_gc_statistics_v1().collections>collections);
        CHECK(!neoclr_gc_collect_v1(&c,NULL));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!wait(operation,owner+1,&c) && (uint32_t)owner[1]==1);
        experiment_gc_store_counts(before);
        CHECK(!await(operation,owner+1,&c) && (uint32_t)owner[1]==1);
        experiment_gc_store_counts(after);CHECK(after[2]==before[2]);
        CHECK(!release(operation,owner+1,&c) && (uint32_t)owner[1]==1);
        CHECK(iscompleted(operation,owner+1,&c)==3 && (uint32_t)owner[1]==1);
        c.fault.code=0;
        CHECK(!neoclr_gc_host_root_release_v1(&c,keep));
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_runtime_async_shutdown_v1(&c));
        CHECK(!neoclr_gc_entry_check_v1(&c));
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
        CHECK(!neoclr_runtime_async_statistics_v1().reserved_bytes);
    }
    puts("PASS 100 generated async cycles: suspend/resume, parked frames, heap results, faults, pressure and shutdown");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_async_create(void *,neoclr_aot_context *);
extern int32_t live_async_complete(uint64_t,void *,neoclr_aot_context *);
extern int32_t live_async_await(uint64_t,void *,neoclr_aot_context *);
extern int32_t live_async_release(uint64_t,void *,neoclr_aot_context *);
extern int32_t live_async_iscompleted(uint64_t,void *,neoclr_aot_context *);
extern int32_t live_async_wait(uint64_t,void *,neoclr_aot_context *);
extern int32_t neoclr_invoke_void_callback_v1(uint64_t,neoclr_aot_context *);
int main(void) {
    void *functions[]={live_async_create,live_async_complete,live_async_await,live_async_release,
        live_async_iscompleted,live_async_wait,neoclr_invoke_void_callback_v1};
    return managed_async_consumer(functions);
}
#endif
