#include "native-workers.h"
#ifdef NEOCLR_WORKER_QUEUES
#include "native-worker-queues.h"
#endif
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if(!(x)) { fprintf(stderr,"line %d\n",__LINE__);abort(); } } while(0)
static neoclr_workers *pool;
static neoclr_worker_handle root,children[12];
static pthread_mutex_t gate=PTHREAD_MUTEX_INITIALIZER;
static pthread_cond_t ready=PTHREAD_COND_INITIALIZER;
static int queued,released,order[13],count;
static int child(neoclr_worker_job *job,void *state) {
    (void)job;order[count++]=(int)(intptr_t)state;return 0;
}
static int parent(neoclr_worker_job *job,void *state) {
    (void)job;(void)state;
    for(int n=0;n<12;n++) CHECK(neoclr_workers_submit(pool,root,child,(void *)(intptr_t)(n+1),&children[n]));
    CHECK(!pthread_mutex_lock(&gate));queued=1;CHECK(!pthread_cond_signal(&ready));
    while(!released) CHECK(!pthread_cond_wait(&ready,&gate));
    CHECK(!pthread_mutex_unlock(&gate));return 0;
}
static void take(neoclr_worker_handle h,int expected) {
    int outcome;neoclr_worker_value value;neoclr_worker_handle out;
    CHECK(neoclr_workers_wait(pool,h,10000)==1);
    CHECK(neoclr_workers_take(pool,h,&outcome,&value,&out)==1 && outcome==expected);
}
int main(void) {
    pool=neoclr_workers_create(1,16,8192);CHECK(pool && neoclr_workers_root_new(pool,42,&root));
    neoclr_worker_handle first,external,replacement;
    CHECK(neoclr_workers_submit(pool,root,parent,NULL,&first));
    CHECK(!pthread_mutex_lock(&gate));while(!queued) CHECK(!pthread_cond_wait(&ready,&gate));CHECK(!pthread_mutex_unlock(&gate));
    CHECK(neoclr_workers_cancel(pool,children[4]));take(children[4],NEOCLR_WORKER_CANCELLED);
    CHECK(!neoclr_workers_cancel(pool,children[4]));
    CHECK(neoclr_workers_submit(pool,root,child,(void *)(intptr_t)100,&external));
    CHECK(external.slot==children[4].slot && external.generation!=children[4].generation);
    CHECK(neoclr_workers_submit(pool,root,child,(void *)(intptr_t)101,&replacement));
    CHECK(!pthread_mutex_lock(&gate));released=1;CHECK(!pthread_cond_signal(&ready));CHECK(!pthread_mutex_unlock(&gate));
    take(first,0);for(int n=0;n<12;n++) if(n!=4) take(children[n],0);
    take(external,0);take(replacement,0);CHECK(count==13);
#ifdef NEOCLR_WORKER_QUEUES
    int expected[]={12,11,10,9,8,7,6,4,100,3,2,1,101};
#else
    int expected[]={1,2,3,4,6,7,8,9,10,11,12,100,101};
#endif
    for(int n=0;n<13;n++) CHECK(order[n]==expected[n]);
    CHECK(neoclr_workers_root_release(pool,root));CHECK(neoclr_workers_close(pool));
#ifdef NEOCLR_WORKER_QUEUES
    neoclr_worker_queue_counts counts;CHECK(neoclr_workers_queue_counts(pool,&counts));
    CHECK(counts.local==11 && counts.global==3 && !counts.stolen);
#endif
    CHECK(neoclr_workers_destroy(pool));
    puts("PASS cancellation unlink, stale ticket, slot reuse, exact execution order and global dispatch bound");
}
