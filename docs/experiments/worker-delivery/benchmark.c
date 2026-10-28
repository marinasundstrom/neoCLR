#define _POSIX_C_SOURCE 200809L
#include "native-runtime.h"
#ifdef DELIVERY_TRACE
#include "native-async-trace.h"
#endif
#include "native-worker-delivery.h"
#include "socket-listener.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
#define JOBS 8
typedef struct {
    neoclr_worker_handle ticket, result;
    uint64_t operation, body, finished, resumed;
} task;
static task tasks[JOBS];
static neoclr_workers *pool;
static unsigned delivered, resumes, io_done, pending;
static uint64_t io_signal, io_transfer, io_body;
static int peer, accepted_fd;
static unsigned rounds;
static uint64_t now(void) {
    struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC,&t));
    return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;
}
static void *sender(void *state) {
    (void)state;
    for (unsigned i=0;i<rounds;i++) {
        char byte; CHECK(recv(peer,&byte,1,0)==1 && byte=='R');
        struct timespec pause={0,200000}; CHECK(!nanosleep(&pause,NULL));
        CHECK(send(peer,"x",1,0)==1);
    }
    return NULL;
}
static int work(neoclr_worker_job *j,void *state) {
    task *t=state;
    struct timespec pause={0,400000}; CHECK(!nanosleep(&pause,NULL));
    neoclr_worker_enter(j);
    uint64_t *shared=neoclr_worker_capture(j); ++*shared;
    neoclr_worker_result(j,(neoclr_worker_value){3,0,shared});
    neoclr_worker_leave(j);
    t->finished=now(); return 0;
}
static void *callback(neoclr_aot_context *c,uint64_t tag) {
    uint64_t *v=NULL; CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&v));
    v[0]=UINT32_MAX; v[1]=tag; return v;
}
static int32_t invoke(uint64_t h,neoclr_aot_context *c) {
    uint64_t *v=NULL; CHECK(!neoclr_gc_callback_read_v1(c,h,(void **)&v));
    uint64_t tag=v[1]; int32_t ready;
    if (tag==100) {
        uint64_t value[2]; CHECK(!neoclr_socket_transfer_result_v1(io_transfer,c,value) && value[0]==1 && value[1]==1);
        CHECK(!neoclr_runtime_async_completeunit_v1(io_signal,c,&ready) && ready); return 0;
    }
    uint64_t *local=callback(c,42);
    neoclr_probe_storage slot={&local,8,0}; neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,c,1,&slot,1,"{}",2);
    CHECK(!neoclr_runtime_async_waitcompletion_v1(tag==99?io_signal:tasks[tag].operation,c,&ready) && ready);
    CHECK(local[1]==42);
    if (tag==99) io_done++;
    else { tasks[tag].resumed=now(); resumes++; }
    neoclr_probe_leave_v1(&frame); return 0;
}
static void completion(neoclr_workers *p,neoclr_worker_handle ticket,void *state) {
    neoclr_aot_context *c=state; task *t=NULL;
    for (unsigned i=0;i<JOBS;i++)
        if (tasks[i].ticket.slot==ticket.slot && tasks[i].ticket.generation==ticket.generation) t=&tasks[i];
    CHECK(t && !t->result.pool); int outcome; neoclr_worker_value value;
    CHECK(neoclr_workers_take(p,ticket,&outcome,&value,&t->result)==1 && outcome==0 && !value.reference);
    CHECK(t->result.pool); int32_t ready;
    CHECK(!neoclr_runtime_async_completeunit_v1(t->operation,c,&ready) && ready);
    delivered++; CHECK(pending); pending--;
}
static int source_poll(neoclr_aot_context *c,void *state) {
    CHECK(state==pool);
    CHECK(neoclr_workers_dispatch(pool,2,completion,c)>=0);
    return pending!=0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
static int compare(const void *a,const void *b) {
    uint64_t x=*(const uint64_t *)a,y=*(const uint64_t *)b; return (x>y)-(x<y);
}
int main(int argc,char **argv) {
    CHECK(argc==3); int wake=atoi(argv[1]); rounds=(unsigned)atoi(argv[2]);
    CHECK((wake==0 || wake==1) && rounds && rounds<=4096);
#ifdef DELIVERY_TRACE
    CHECK(neoclr_async_trace_begin(65536));
#endif
    pool=neoclr_workers_create(2,JOBS,8192); CHECK(pool);
    int fd=neoclr_workers_completion_descriptor(pool); CHECK(fd>=0);
    neoclr_async_external_source source={wake?fd:-1,source_poll,pool};
    uint64_t *storage=calloc(131072,8); CHECK(storage);
    neoclr_aot_context c={.text={(unsigned char *)storage,131072*8,0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    neoclr_socket_scope scope; CHECK(!neoclr_socket_scope_enter_v1(&scope,&c));
    const struct { uint64_t length; char bytes[16]; } address={9,"127.0.0.1"};
    uint64_t value[2]; CHECK(!neoclr_socket_listen_v1((const void *)&address,0,1,&c,value) && value[0]==5);
    uint64_t listener=value[1]; CHECK(!neoclr_socket_local_port_v1(listener,&c,value) && value[0]==1);
    struct sockaddr_in endpoint={0}; endpoint.sin_family=AF_INET; endpoint.sin_port=htons((uint16_t)value[1]);
    endpoint.sin_addr.s_addr=htonl(UINT32_C(0x7f000001));
    CHECK(!neoclr_socket_accept_v1(listener,callback(&c,100),&c,value) && value[0]==5);
    uint64_t accept_id=value[1],root=0;
    peer=socket(AF_INET,SOCK_STREAM,0); CHECK(peer>=0 && !connect(peer,(void *)&endpoint,sizeof(endpoint)));
    int ready=0;
    for (unsigned i=0;i<5000 && !ready;i++) {
        ready=neoclr_socket_poll_v1(&c,&root); if (!ready) neoclr_scheduler_wait_v1(&c);
    }
    CHECK(ready==1 && !neoclr_socket_connect_result_v1(accept_id,&c,value) && value[0]==5);
    uint64_t accepted=value[1]; accepted_fd=-1;
    for (unsigned i=0;i<64;i++) if (scope.slots[i].id==accepted) accepted_fd=scope.slots[i].descriptor;
    CHECK(accepted_fd>=0 && !neoclr_socket_close_v1(listener,&c,value));
    void *array=NULL; CHECK(!neoclr_reserve_bytes_v1(&c.text,1,&array));
    uint64_t array_root; CHECK(!neoclr_gc_host_root_create_v1(&c,array,&array_root));
    pthread_t thread; CHECK(!pthread_create(&thread,NULL,sender,NULL));
    uint64_t *latencies=calloc(rounds*JOBS,8); CHECK(latencies);
    uint64_t begin=now(); clock_t cpu=clock();
    for (unsigned round=0;round<rounds;round++) {
        neoclr_worker_handle shared; CHECK(neoclr_workers_root_new(pool,0,&shared));
        pending=JOBS;
        for (unsigned i=0;i<JOBS;i++) {
            tasks[i]=(task){0};
            CHECK(!neoclr_runtime_async_create_v1(&c,&tasks[i].operation));
            CHECK(!neoclr_runtime_async_start_v1(callback(&c,i),&c,&tasks[i].body,invoke));
            CHECK(neoclr_workers_submit(pool,shared,work,&tasks[i],&tasks[i].ticket));
        }
        CHECK(neoclr_workers_root_release(pool,shared)); /* No serialized copies: tickets own shared identity. */
        uint64_t extent; CHECK(neoclr_workers_collect(pool,&extent) && extent);
        CHECK(!neoclr_runtime_async_create_v1(&c,&io_signal));
        CHECK(!neoclr_socket_receive_v1(accepted,array,0,1,callback(&c,100),&c,value) && value[0]==5);
        io_transfer=value[1];
        CHECK(!neoclr_runtime_async_start_v1(callback(&c,99),&c,&io_body,invoke));
        CHECK(send(accepted_fd,"R",1,0)==1);
        CHECK(!neoclr_runtime_async_pump_external_v1(&c,invoke,drain,&source));
        CHECK(!pending && delivered==(round+1)*JOBS && resumes==delivered && io_done==round+1);
        CHECK(((unsigned char *)array)[16]=='x');
        CHECK(neoclr_workers_collect(pool,&extent) && extent);
        for (unsigned i=0;i<JOBS;i++) {
            uint64_t shared_value; CHECK(neoclr_workers_root_read(pool,tasks[i].result,&shared_value) && shared_value==JOBS);
            CHECK(tasks[i].resumed>=tasks[i].finished);
            latencies[round*JOBS+i]=tasks[i].resumed-tasks[i].finished;
            CHECK(neoclr_workers_root_release(pool,tasks[i].result));
            int32_t done;
            CHECK(!neoclr_runtime_async_release_v1(tasks[i].body,&c,&done) && done);
            CHECK(!neoclr_runtime_async_release_v1(tasks[i].operation,&c,&done) && done);
        }
        int32_t done;
        CHECK(!neoclr_runtime_async_release_v1(io_body,&c,&done) && done);
        CHECK(!neoclr_runtime_async_release_v1(io_signal,&c,&done) && done);
        CHECK(!neoclr_gc_idle_v1(&c,NULL));
        CHECK(neoclr_workers_collect(pool,&extent) && !extent);
    }
    double seconds=(now()-begin)/1e9,cpu_seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
    CHECK(!pthread_join(thread,NULL) && !close(peer));
    CHECK(!neoclr_runtime_async_shutdown_v1(&c));
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_host_root_release_v1(&c,array_root));
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    uint64_t epochs[6]={0};
#ifdef TRACE_LIVE
    extern void experiment_gc_candidate_counts(uint64_t *);
    experiment_gc_candidate_counts(epochs); CHECK(epochs[0] && epochs[1] && !epochs[5]);
    CHECK(epochs[0]==epochs[1]+epochs[2]);
#endif
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(neoclr_workers_close(pool) && neoclr_workers_destroy(pool));
    size_t count=rounds*JOBS; qsort(latencies,count,8,compare);
    printf("{\"wake\":%d,\"rounds\":%u,\"jobs\":%zu,\"seconds\":%.9f,\"cpuSeconds\":%.9f,\"p50ns\":%llu,\"p99ns\":%llu,\"epochsStarted\":%llu,\"epochsFinished\":%llu,\"epochsCancelled\":%llu}\n",wake,rounds,count,seconds,cpu_seconds,(unsigned long long)latencies[count/2],(unsigned long long)latencies[(count*99+99)/100-1],(unsigned long long)epochs[0],(unsigned long long)epochs[1],(unsigned long long)epochs[2]);
#ifdef DELIVERY_TRACE
    neoclr_async_trace_view trace; CHECK(neoclr_async_trace_finish(&trace) && !trace.dropped);
    FILE *file=fopen("trace.json","w"); CHECK(file);
    fprintf(file,"{\"schema\":\"async-1\",\"origin_ns\":%llu,\"attempted\":%llu,\"dropped\":%llu,\"capacity\":%zu,\"events\":[",(unsigned long long)trace.origin_ns,(unsigned long long)trace.attempted,(unsigned long long)trace.dropped,trace.capacity);
    for (size_t i=0;i<trace.count;i++) {
        const neoclr_async_trace_event *e=&trace.events[i];
        fprintf(file,"%s[%llu,%llu,%llu,%llu,%u,%u]",i?",":"",(unsigned long long)e->ns,(unsigned long long)e->sequence,(unsigned long long)e->context,(unsigned long long)e->related,e->kind,e->detail);
    }
    CHECK(fputs("]}\n",file)>=0 && !fclose(file)); CHECK(neoclr_async_trace_dispose());
#endif
    free(latencies); free(storage); return 0;
}
