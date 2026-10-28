#include "native-runtime.h"
#include "native-worker-delivery.h"
#include "socket-listener.h"
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static neoclr_workers *pool;
static neoclr_worker_handle capture;
static uint64_t signal_id;
static int pending, finished;
static int work(neoclr_worker_job *j,void *state) {
    (void)state;
    neoclr_worker_enter(j);
    uint64_t *value=neoclr_worker_capture(j); ++*value;
    neoclr_worker_result(j,(neoclr_worker_value){3,0,value});
    neoclr_worker_leave(j); return 0;
}
static void completion(neoclr_workers *p,neoclr_worker_handle ticket,void *state) {
    neoclr_aot_context *c=state;
    neoclr_worker_handle root; neoclr_worker_value value; int outcome;
    CHECK(neoclr_workers_take(p,ticket,&outcome,&value,&root)==1 && outcome==0);
    CHECK(neoclr_workers_root_release(p,root));
    int32_t done; CHECK(!neoclr_runtime_async_completeunit_v1(signal_id,c,&done) && done);
    pending=0;
}
static int poll_source(neoclr_aot_context *c,void *state) {
    CHECK(state==pool && neoclr_workers_dispatch(pool,1,completion,c)>=0);
    return pending;
}
static int32_t invoke(uint64_t h,neoclr_aot_context *c) {
    void *callback; CHECK(!neoclr_gc_callback_read_v1(c,h,&callback));
    for (unsigned i=0;i<32;i++) {
        neoclr_worker_handle ticket;
        CHECK(!neoclr_runtime_async_create_v1(c,&signal_id));
        pending=1; CHECK(neoclr_workers_submit(pool,capture,work,NULL,&ticket));
        int32_t done;
        CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id,c,&done) && done);
        CHECK(!neoclr_runtime_async_release_v1(signal_id,c,&done) && done);
        finished++;
    }
    return 0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
int main(void) {
    pool=neoclr_workers_create(1,1,8192); CHECK(pool && neoclr_workers_root_new(pool,0,&capture));
    neoclr_async_external_source source={neoclr_workers_completion_descriptor(pool),poll_source,pool};
    CHECK(source.descriptor>=0);
    uint64_t storage[8192]={0}; neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    neoclr_socket_scope scope; CHECK(!neoclr_socket_scope_enter_v1(&scope,&c));
    uint64_t *callback=NULL; CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&callback));
    callback[0]=UINT32_MAX; callback[1]=1; uint64_t body;
    CHECK(!neoclr_runtime_async_start_v1(callback,&c,&body,invoke));
    CHECK(!neoclr_runtime_async_pump_external_v1(&c,invoke,drain,&source));
    CHECK(finished==32 && !pending);
    uint64_t extent,value; CHECK(neoclr_workers_collect(pool,&extent));
    CHECK(neoclr_workers_root_read(pool,capture,&value) && value==32);
    int32_t done; CHECK(!neoclr_runtime_async_release_v1(body,&c,&done) && done);
    CHECK(!neoclr_runtime_async_shutdown_v1(&c));
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(neoclr_workers_root_release(pool,capture));
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
    puts("PASS resumed contexts submit subsequent work without stale-idle exit");
}
