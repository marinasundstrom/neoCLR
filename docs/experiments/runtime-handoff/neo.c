#define _POSIX_C_SOURCE 200809L
#include "native-runtime.h"
#include "native-workers.h"
#include "socket-listener.h"
#include <netinet/tcp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static uint64_t signal_id, transfer_id, checksum;
static int accepted_fd;
static unsigned resumed, iterations;
static uint64_t compute(void) {
    uint64_t x=1;
    for (unsigned n=0;n<iterations;n++) { x^=x<<13; x^=x>>7; x^=x<<17; }
    return x;
}
static int work(neoclr_worker_job *job,void *state) {
    (void)job; *(uint64_t *)state=compute(); return 0;
}
static void *callback(neoclr_aot_context *c,uint64_t tag) {
    uint64_t *v=NULL; CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&v));
    v[0]=UINT32_MAX; v[1]=tag; return v;
}
static int32_t invoke(uint64_t handle,neoclr_aot_context *c) {
    uint64_t *v=NULL; CHECK(!neoclr_gc_callback_read_v1(c,handle,(void **)&v));
    uint64_t tag=v[1]; int32_t result;
    if (tag==1) {
        uint64_t *local=callback(c,42);
        neoclr_probe_storage slot={&local,8,0}; neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame,c,1,&slot,1,"{}",2);
        CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id,c,&result) && result);
        CHECK(local[1]==42); resumed++;
        CHECK(send(accepted_fd,"A",1,0)==1);
        neoclr_probe_leave_v1(&frame);
    } else if (tag==2) {
        uint64_t value[2]; CHECK(!neoclr_socket_transfer_result_v1(transfer_id,c,value));
        CHECK(value[0]==1 && value[1]==1);
        CHECK(!neoclr_runtime_async_completeunit_v1(signal_id,c,&result) && result);
    } else CHECK(0);
    return 0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
static uint64_t now(void) {
    struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC,&t));
    return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;
}
int main(int argc,char **argv) {
    CHECK(argc==4); const char *mode=argv[1]; unsigned rounds=(unsigned)atoi(argv[2]);
    iterations=(unsigned)atoi(argv[3]); CHECK(rounds && rounds<=4096 && iterations && iterations<=100000000);
    int offload=!strcmp(mode,"offload"),inline_work=!strcmp(mode,"inline");
    CHECK(offload || inline_work || !strcmp(mode,"idle"));
    neoclr_workers *pool=NULL; neoclr_worker_handle capture={0};
    if (offload) {
        pool=neoclr_workers_create(1,1,8192); CHECK(pool);
        CHECK(neoclr_workers_root_new(pool,42,&capture));
    }
    uint64_t *storage=calloc(131072,8); CHECK(storage);
    neoclr_aot_context c={.text={(unsigned char *)storage,131072*8,0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    neoclr_socket_scope scope; CHECK(!neoclr_socket_scope_enter_v1(&scope,&c));
    const struct { uint64_t length; char bytes[16]; } address={9,"127.0.0.1"};
    uint64_t value[2]; CHECK(!neoclr_socket_listen_v1((const void *)&address,0,1,&c,value) && value[0]==5);
    uint64_t listener=value[1]; CHECK(!neoclr_socket_local_port_v1(listener,&c,value) && value[0]==1);
    printf("%llu\n",(unsigned long long)value[1]); CHECK(!fflush(stdout));
    CHECK(!neoclr_socket_accept_v1(listener,callback(&c,2),&c,value) && value[0]==5);
    uint64_t accept_id=value[1],root=0;
    int ready=0;
    for (unsigned n=0;n<10000 && !ready;n++) {
        ready=neoclr_socket_poll_v1(&c,&root);
        if (!ready) neoclr_scheduler_wait_v1(&c);
    }
    CHECK(ready==1);
    CHECK(!neoclr_socket_connect_result_v1(accept_id,&c,value) && value[0]==5);
    uint64_t accepted=value[1]; accepted_fd=-1;
    for (unsigned n=0;n<64;n++) if (scope.slots[n].id==accepted) accepted_fd=scope.slots[n].descriptor;
    CHECK(accepted_fd>=0); int one=1; CHECK(!setsockopt(accepted_fd,IPPROTO_TCP,TCP_NODELAY,&one,sizeof(one)));
    CHECK(!neoclr_socket_close_v1(listener,&c,value));
    void *array=NULL; CHECK(!neoclr_reserve_bytes_v1(&c.text,1,&array));
    uint64_t array_root; CHECK(!neoclr_gc_host_root_create_v1(&c,array,&array_root));
    uint64_t begin=now(); clock_t cpu=clock();
    for (unsigned round=0;round<rounds;round++) {
        CHECK(!neoclr_runtime_async_create_v1(&c,&signal_id));
        CHECK(!neoclr_socket_receive_v1(accepted,array,0,1,callback(&c,2),&c,value) && value[0]==5);
        transfer_id=value[1]; uint64_t body; int32_t done;
        CHECK(!neoclr_runtime_async_start_v1(callback(&c,1),&c,&body,invoke));
        CHECK(resumed==round); CHECK(send(accepted_fd,"R",1,0)==1);
        neoclr_worker_handle ticket={0};
        if (offload) CHECK(neoclr_workers_submit(pool,capture,work,&checksum,&ticket));
        if (inline_work) checksum=compute();
        CHECK(!neoclr_runtime_async_pump_v1(&c,invoke,drain)); CHECK(resumed==round+1);
        CHECK(((unsigned char *)array)[16]=='x');
        if (offload) {
            int outcome; neoclr_worker_value result; neoclr_worker_handle reference;
            CHECK(neoclr_workers_wait(pool,ticket,10000)==1);
            CHECK(neoclr_workers_take(pool,ticket,&outcome,&result,&reference) && outcome==NEOCLR_WORKER_OK);
        }
        CHECK(!neoclr_runtime_async_release_v1(body,&c,&done) && done);
        CHECK(!neoclr_runtime_async_release_v1(signal_id,&c,&done) && done);
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
    }
    double seconds=(now()-begin)/1e9,cpu_seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
    CHECK(!neoclr_runtime_async_shutdown_v1(&c)); CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_host_root_release_v1(&c,array_root)); CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    uint64_t epochs[6]={0};
#ifdef TRACE_LIVE
    extern void experiment_gc_candidate_counts(uint64_t *);
    experiment_gc_candidate_counts(epochs); CHECK(epochs[0] && epochs[1]);
#endif
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    if (pool) {
        CHECK(neoclr_workers_root_release(pool,capture)); CHECK(neoclr_workers_close(pool));
        CHECK(neoclr_workers_destroy(pool));
    }
    printf("{\"mode\":\"%s\",\"rounds\":%u,\"iterations\":%u,\"checksum\":\"%llu\",\"seconds\":%.9f,\"cpuSeconds\":%.9f,\"epochsStarted\":%llu,\"epochsFinished\":%llu}\n",mode,rounds,iterations,(unsigned long long)checksum,seconds,cpu_seconds,(unsigned long long)epochs[0],(unsigned long long)epochs[1]);
    free(storage); return 0;
}
