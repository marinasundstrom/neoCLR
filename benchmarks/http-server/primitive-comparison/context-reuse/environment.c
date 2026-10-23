#define _DARWIN_C_SOURCE 1
#include "native-runtime.h"
#include "socket-listener.h"
#include <assert.h>
#include <fenv.h>
#include <signal.h>
#include <stdint.h>
static int expected_round, calls, fail, park;
static uint64_t signal_id;
static uint64_t expected_value;
void neoclr_scheduler_wait_v1(neoclr_aot_context *c) { (void)c; }
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c,uint64_t *out,const neoclr_probe_frame *f) {
    (void)c;(void)out;(void)f;return NEOCLR_SCHEDULER_IDLE;
}
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *c,uint64_t id,const neoclr_probe_frame *f) {
    (void)c;(void)id;(void)f;return 3;
}
static int32_t invoke(uint64_t handle,neoclr_aot_context *c) {
    void *value=NULL;
    assert(!neoclr_gc_callback_read_v1(c,handle,&value));
    assert(((uint64_t *)value)[1]==expected_value);
    assert(fegetround()==expected_round);
    sigset_t mask;
    assert(!sigprocmask(0,NULL,&mask) && !sigismember(&mask,SIGUSR1));
    uint64_t saved_value=((uint64_t *)value)[1];
    if (park) {
        int32_t completed=0;
        assert(!neoclr_runtime_async_waitcompletion_v1(signal_id,c,&completed) && completed);
        assert(((uint64_t *)value)[1]==saved_value);
    }
    assert(!fesetround(FE_DOWNWARD));
    sigemptyset(&mask);sigaddset(&mask,SIGUSR1);
    assert(!sigprocmask(SIG_BLOCK,&mask,NULL));
    ++calls;
    if (fail) { c->fault.code=3; return 3; }
    return 0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
int main(void) {
    sigset_t initial, clean;
    assert(!sigprocmask(0,NULL,&initial));clean=initial;sigdelset(&clean,SIGUSR1);
    int original_round=fegetround();
    for (int invocation=0;invocation<2;invocation++) {
        uint64_t storage[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
        for (int i=0;i<300;i++) {
            expected_round=(i&1)?FE_UPWARD:FE_TONEAREST;
            assert(!fesetround(expected_round));assert(!sigprocmask(SIG_SETMASK,&clean,NULL));
            uint64_t *callback=NULL,operation=0;
            assert(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&callback));
            callback[0]=UINT32_MAX;callback[1]=expected_value=(uint64_t)i+invocation*300;
            assert(!neoclr_runtime_async_start_v1(callback,&c,&operation,invoke));
            int32_t result=0;
            assert(!neoclr_runtime_async_iscompleted_v1(operation,&c,&result) && result);
            assert(!neoclr_runtime_async_release_v1(operation,&c,&result) && result);
            assert(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        }
        /* More live contexts than cache slots: park all, collect, complete all,
         * then verify that cached contexts are neither active work nor leaked. */
        uint64_t bodies[12];
        assert(!neoclr_runtime_async_create_v1(&c,&signal_id));
        park=1;
        for (unsigned i=0;i<12;i++) {
            expected_round=FE_TONEAREST;
            assert(!fesetround(expected_round));assert(!sigprocmask(SIG_SETMASK,&clean,NULL));
            uint64_t *callback=NULL;
            assert(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&callback));
            callback[0]=UINT32_MAX;callback[1]=expected_value=i;
            assert(!neoclr_runtime_async_start_v1(callback,&c,&bodies[i],invoke));
        }
        assert(!neoclr_gc_collect_v1(&c,NULL));
        int32_t completed=0;
        assert(!neoclr_runtime_async_completeunit_v1(signal_id,&c,&completed) && completed);
        assert(!neoclr_runtime_async_pump_v1(&c,invoke,drain));
        park=0;
        for (unsigned i=0;i<12;i++) {
            assert(!neoclr_runtime_async_iscompleted_v1(bodies[i],&c,&completed) && completed);
            assert(!neoclr_runtime_async_release_v1(bodies[i],&c,&completed) && completed);
        }
        assert(!neoclr_runtime_async_release_v1(signal_id,&c,&completed) && completed);
        assert(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        /* Faulting completion must not leave a reusable body or roots behind. */
        expected_round=FE_TONEAREST;
        assert(!fesetround(expected_round));assert(!sigprocmask(SIG_SETMASK,&clean,NULL));
        uint64_t *callback=NULL,failed_operation=0;
        assert(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&callback));
        callback[0]=UINT32_MAX;callback[1]=expected_value=999;
        fail=1;
        assert(neoclr_runtime_async_start_v1(callback,&c,&failed_operation,invoke)==3);
        assert(c.fault.code==3);
        assert(!neoclr_runtime_async_shutdown_v1(&c));
        fail=0;c.fault.code=0;
        assert(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        neoclr_async_statistics stats=neoclr_runtime_async_statistics_v1();
        assert(!stats.reserved_bytes && stats.mappings==stats.unmappings);
    }
    assert(calls==626);assert(!fesetround(original_round));assert(!sigprocmask(SIG_SETMASK,&initial,NULL));
}
