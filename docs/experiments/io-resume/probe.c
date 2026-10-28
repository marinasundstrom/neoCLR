#define _POSIX_C_SOURCE 200809L
#include "native-runtime.h"
#ifdef IO_TRACE
#include "native-async-trace.h"
#endif
#include "socket-listener.h"
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static uint64_t signal_id, transfer_id, completion_ns, resume_ns, load_root;
static unsigned load_left, shape, resumed;
static _Atomic unsigned armed, stop_peer;
static _Atomic uint64_t sent_ns;
static int peer;
static uint64_t checksum;
static uint64_t now(void) {
    struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC,&t));
    return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;
}
static void *sender(void *unused) {
    (void)unused;
    while (!atomic_load(&stop_peer)) {
        if (!atomic_exchange(&armed,0)) {
            struct timespec t={0,50000}; nanosleep(&t,NULL); continue;
        }
        struct timespec t={0,200000}; CHECK(!nanosleep(&t,NULL));
        atomic_store(&sent_ns,now()); CHECK(send(peer,"x",1,0)==1);
    }
    return NULL;
}
static void *callback(neoclr_aot_context *c,uint64_t tag) {
    uint64_t *v=NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&v));
    v[0]=UINT32_MAX; v[1]=tag; return v;
}
static int32_t invoke(uint64_t h,neoclr_aot_context *c) {
    uint64_t *v=NULL; CHECK(!neoclr_gc_callback_read_v1(c,h,(void **)&v));
    uint64_t tag=v[1]; int32_t result;
    if (tag==1) {
        uint64_t *local=callback(c,42);
        neoclr_probe_storage slot={&local,8,0}; neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame,c,1,&slot,1,"{}",2);
        CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id,c,&result) && result);
        resume_ns=now(); CHECK(local[1]==42); resumed++;
        neoclr_probe_leave_v1(&frame);
    } else if (tag==2) {
        uint64_t result_value[2];
        CHECK(!neoclr_socket_transfer_result_v1(transfer_id,c,result_value));
        CHECK(result_value[0]==1 && result_value[1]==1);
        CHECK(!neoclr_runtime_async_completeunit_v1(signal_id,c,&result) && result);
        completion_ns=now();
    } else if (tag==3) {
        uint64_t x=checksum+1;
        for (unsigned n=0;n<(shape==3?320000u:20000u);n++) { x^=x<<13; x^=x>>7; x^=x<<17; }
        checksum=x;
        if (shape==2) for (unsigned n=0;n<32;n++) (void)callback(c,n);
        CHECK(load_left); load_left--;
        if (load_left) CHECK(!neoclr_scheduler_submit_v1(c,load_root,NULL));
    } else CHECK(0);
    return 0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
static int compare(const void *a,const void *b) {
    uint64_t x=*(const uint64_t *)a,y=*(const uint64_t *)b; return (x>y)-(x<y);
}
int main(int argc,char **argv) {
    CHECK(argc==3); shape=(unsigned)atoi(argv[1]); unsigned rounds=(unsigned)atoi(argv[2]);
    CHECK(shape<4 && rounds && rounds<=4096);
#ifdef IO_TRACE
    CHECK(neoclr_async_trace_begin(65536));
#endif
    uint64_t *storage=calloc(131072,sizeof(uint64_t)); CHECK(storage);
    neoclr_aot_context c={.text={(unsigned char *)storage,131072*8,0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    neoclr_socket_scope scope; CHECK(!neoclr_socket_scope_enter_v1(&scope,&c));
    const struct { uint64_t length; char bytes[16]; } address={9,"127.0.0.1"};
    uint64_t result[2]; CHECK(!neoclr_socket_listen_v1((const void *)&address,0,4,&c,result) && result[0]==5);
    uint64_t listener=result[1];
    CHECK(!neoclr_socket_local_port_v1(listener,&c,result) && result[0]==1);
    struct sockaddr_in endpoint={0}; endpoint.sin_family=AF_INET;
    endpoint.sin_port=htons((uint16_t)result[1]); endpoint.sin_addr.s_addr=htonl(UINT32_C(0x7f000001));
    CHECK(!neoclr_socket_accept_v1(listener,callback(&c,2),&c,result) && result[0]==5);
    uint64_t accept_id=result[1], delivery=0;
    peer=socket(AF_INET,SOCK_STREAM,0); CHECK(peer>=0 && !connect(peer,(void *)&endpoint,sizeof(endpoint)));
    int ready=0;
    for (unsigned n=0;n<5000 && !ready;n++) {
        ready=neoclr_socket_poll_v1(&c,&delivery);
        if (!ready) neoclr_scheduler_wait_v1(&c);
    }
    CHECK(ready==1);
    CHECK(!neoclr_socket_connect_result_v1(accept_id,&c,result) && result[0]==5);
    uint64_t accepted=result[1];
    CHECK(!neoclr_socket_close_v1(listener,&c,result));
    void *array=NULL; CHECK(!neoclr_reserve_bytes_v1(&c.text,1,&array));
    uint64_t array_root; CHECK(!neoclr_gc_host_root_create_v1(&c,array,&array_root));
    CHECK(!neoclr_gc_host_root_create_v1(&c,callback(&c,3),&load_root));
    pthread_t thread; CHECK(!pthread_create(&thread,NULL,sender,NULL));
    uint64_t *handoff=calloc(rounds,8),*end_to_end=calloc(rounds,8); CHECK(handoff && end_to_end);
    uint64_t begin=now(); clock_t cpu=clock();
    for (unsigned round=0;round<rounds;round++) {
        completion_ns=resume_ns=0; load_left=shape==3?1:shape?16:0;
        CHECK(!neoclr_runtime_async_create_v1(&c,&signal_id));
        CHECK(!neoclr_socket_receive_v1(accepted,array,0,1,callback(&c,2),&c,result) && result[0]==5);
        transfer_id=result[1]; uint64_t body; int32_t done;
        CHECK(!neoclr_runtime_async_start_v1(callback(&c,1),&c,&body,invoke));
        CHECK(resumed==round);
        if (load_left) CHECK(!neoclr_scheduler_submit_v1(&c,load_root,NULL));
        atomic_store(&armed,1);
        CHECK(!neoclr_runtime_async_pump_v1(&c,invoke,drain));
        CHECK(resumed==round+1 && !load_left && completion_ns && resume_ns>=completion_ns);
        uint64_t sent=atomic_load(&sent_ns); CHECK(sent && resume_ns>=sent);
        handoff[round]=resume_ns-completion_ns; end_to_end[round]=resume_ns-sent;
        CHECK(!neoclr_runtime_async_release_v1(body,&c,&done) && done);
        CHECK(!neoclr_runtime_async_release_v1(signal_id,&c,&done) && done);
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
    }
    double wall=(now()-begin)/1e9, cpu_seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
    atomic_store(&stop_peer,1); CHECK(!pthread_join(thread,NULL)); CHECK(!close(peer));
    CHECK(!neoclr_runtime_async_shutdown_v1(&c));
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_host_root_release_v1(&c,array_root));
    CHECK(!neoclr_gc_host_root_release_v1(&c,load_root));
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    uint64_t epochs[6]={0};
#ifdef TRACE_LIVE
    extern void experiment_gc_candidate_counts(uint64_t *);
    experiment_gc_candidate_counts(epochs); CHECK(epochs[0] && epochs[1]);
#endif
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    qsort(handoff,rounds,8,compare); qsort(end_to_end,rounds,8,compare);
    printf("{\"shape\":%u,\"rounds\":%u,\"seconds\":%.9f,\"cpuSeconds\":%.9f,\"handoffP50ns\":%llu,\"handoffP99ns\":%llu,\"sendResumeP50ns\":%llu,\"sendResumeP99ns\":%llu,\"epochsStarted\":%llu,\"epochsFinished\":%llu,\"checksum\":%llu}\n",shape,rounds,wall,cpu_seconds,(unsigned long long)handoff[rounds/2],(unsigned long long)handoff[(rounds*99+99)/100-1],(unsigned long long)end_to_end[rounds/2],(unsigned long long)end_to_end[(rounds*99+99)/100-1],(unsigned long long)epochs[0],(unsigned long long)epochs[1],(unsigned long long)checksum);
#ifdef IO_TRACE
    neoclr_async_trace_view trace; CHECK(neoclr_async_trace_finish(&trace) && !trace.dropped);
    FILE *file=fopen("trace.json","w"); CHECK(file);
    fprintf(file,"{\"schema\":\"async-1\",\"origin_ns\":%llu,\"attempted\":%llu,\"dropped\":%llu,\"capacity\":%zu,\"events\":[",(unsigned long long)trace.origin_ns,(unsigned long long)trace.attempted,(unsigned long long)trace.dropped,trace.capacity);
    for (size_t i=0;i<trace.count;i++) {
        const neoclr_async_trace_event *e=&trace.events[i];
        fprintf(file,"%s[%llu,%llu,%llu,%llu,%u,%u]",i?",":"",(unsigned long long)e->ns,(unsigned long long)e->sequence,(unsigned long long)e->context,(unsigned long long)e->related,e->kind,e->detail);
    }
    CHECK(fputs("]}\n",file)>=0 && !fclose(file)); CHECK(neoclr_async_trace_dispose());
#endif
    free(storage); free(handoff); free(end_to_end); return 0;
}
