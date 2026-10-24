#include "native-gc.h"
#include "task-queue.h"
#include "socket-listener.h"
#include <stdio.h>
#include <stdlib.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"queue timers line %d: %s\n",__LINE__,#x); abort(); } } while(0)
typedef int32_t (*one_fn)(void *,void *,neoclr_aot_context *);
typedef int32_t (*get_fn)(void *,neoclr_aot_context *);
typedef int32_t (*start_fn)(int32_t,void *,void *,neoclr_aot_context *);
typedef int32_t (*cancel_fn)(uint64_t,void *,neoclr_aot_context *);
static uint64_t *object(neoclr_aot_context *c,uint64_t kind) {
    uint64_t *p;CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&p));
    p[0]=kind;return p;
}
int managed_queue_timers_consumer(void *const *f) {
    one_fn body=(one_fn)f[0],reg=(one_fn)f[1];
    get_fn def=(get_fn)f[2],current=(get_fn)f[3];
    start_fn start=(start_fn)f[4];cancel_fn cancel=(cancel_fn)f[5];
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *carrier=object(&c,17),keep;
        CHECK(!neoclr_gc_host_root_create_v1(&c,carrier,&keep));
        carrier[1]=777;
        CHECK(def(carrier+1,&c)==3 && carrier[1]==777);c.fault.code=0;
        neoclr_task_scope tasks;neoclr_socket_scope sockets;
        CHECK(!neoclr_task_scope_enter_v1(&tasks,&c));
        CHECK(!neoclr_socket_scope_enter_v1(&sockets,&c));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!def(carrier+1,&c) && !carrier[1]);
        CHECK(!experiment_gc_store(&c,carrier+1,777,8));
        CHECK(reg(NULL,carrier+1,&c)==3 && carrier[1]==777);c.fault.code=0;
        uint64_t *queue=object(&c,1);
        CHECK(!reg(queue,carrier+1,&c) && !carrier[1]);
        CHECK(!neoclr_gc_collect_v1(&c,NULL));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!def(carrier+1,&c) && carrier[1]==(uintptr_t)queue);
        CHECK(!current(carrier+1,&c) && carrier[1]==(uintptr_t)queue);
        CHECK(!experiment_gc_store(&c,carrier+1,777,8));
        CHECK(reg(queue,carrier+1,&c)==3 && carrier[1]==777);c.fault.code=0;
        uint64_t *other=object(&c,1),other_root;
        CHECK(!neoclr_gc_host_root_create_v1(&c,other,&other_root));
        /* Current inside Drain must use its receiver, not the default queue. */
        CHECK(!body(other,carrier+1,&c) && !carrier[1] && other[1]==1 && other[2]);
        CHECK(!neoclr_gc_host_root_release_v1(&c,other_root));
        uint64_t *callback=object(&c,UINT32_MAX);callback[1]=1;
        CHECK(!experiment_gc_store(&c,carrier+1,777,8));
        CHECK(start(-1,callback,carrier+1,&c)==3 && carrier[1]==777);c.fault.code=0;
        CHECK(!start(60000,callback,carrier+1,&c) && carrier[1]);
        uint64_t timer=carrier[1];
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && callback[0]==UINT32_MAX);
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!cancel(timer,carrier+1,&c) && carrier[1]==1);
        CHECK(!cancel(timer,carrier+1,&c) && !carrier[1]);
        callback=object(&c,UINT32_MAX);callback[1]=1;
        CHECK(!start(0,callback,carrier+1,&c));timer=carrier[1];
        uint64_t ticket=0;void *read=NULL;
        CHECK(neoclr_scheduler_acquire_v1(&c,&ticket,NULL)==NEOCLR_SCHEDULER_READY);
        CHECK(!neoclr_gc_host_root_read_v1(&c,ticket,&read) && read==callback);
        CHECK(!neoclr_scheduler_complete_v1(&c,ticket,NULL));
        CHECK(neoclr_scheduler_acquire_v1(&c,&ticket,NULL)==NEOCLR_SCHEDULER_IDLE);
        CHECK(!cancel(timer,carrier+1,&c) && !carrier[1]);
        CHECK(!neoclr_root_probe_head_v1());
        CHECK(!neoclr_socket_scope_leave_v1(&sockets));
        CHECK(!neoclr_task_scope_leave_v1(&tasks));
        CHECK(!neoclr_gc_host_root_release_v1(&c,keep));
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 100 queue/timer service cycles: ownership, current receiver, heap results, cancellation and expiry");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_queue_body(void *,void *,neoclr_aot_context *);
extern int32_t live_queue_register(void *,void *,neoclr_aot_context *);
extern int32_t live_queue_default(void *,neoclr_aot_context *);
extern int32_t live_queue_current(void *,neoclr_aot_context *);
extern int32_t live_timer_start(int32_t,void *,void *,neoclr_aot_context *);
extern int32_t live_timer_cancel(uint64_t,void *,neoclr_aot_context *);
int main(void) {void *f[]={live_queue_body,live_queue_register,live_queue_default,live_queue_current,live_timer_start,live_timer_cancel};return managed_queue_timers_consumer(f);}
#endif
