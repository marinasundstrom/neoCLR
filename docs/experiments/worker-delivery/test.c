#define _POSIX_C_SOURCE 200809L
#include "native-worker-delivery.h"
#include <pthread.h>
#include <stdatomic.h>
#include <poll.h>
#include <sys/ioctl.h>
#include <fcntl.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
typedef struct { _Atomic unsigned started, release; int cancel_wait; } control;
static int work(neoclr_worker_job *j,void *state) {
    control *c=state; atomic_fetch_add(&c->started,1);
    while (!atomic_load(&c->release)) {
        if (c->cancel_wait && neoclr_worker_cancelled(j)) return 0;
        struct timespec t={0,100000}; nanosleep(&t,NULL);
    }
    neoclr_worker_enter(j);
    uint64_t *value=neoclr_worker_capture(j); ++*value;
    neoclr_worker_result(j,(neoclr_worker_value){3,0,value});
    neoclr_worker_leave(j); return 0;
}
static int readable(int fd,int timeout) {
    struct pollfd p={fd,POLLIN,0}; int n=poll(&p,1,timeout);
    CHECK(n>=0); return n && (p.revents&POLLIN);
}
typedef struct { unsigned count,cancelled; neoclr_worker_handle roots[16]; } receiver;
static void delivered(neoclr_workers *p,neoclr_worker_handle ticket,void *state) {
    receiver *r=state; int outcome; neoclr_worker_value value; neoclr_worker_handle root;
    CHECK(neoclr_workers_dispatch(p,1,delivered,state)==-1);
    uint64_t extent; CHECK(neoclr_workers_collect(p,&extent));
    CHECK(neoclr_workers_take(p,ticket,&outcome,&value,&root)==1 && !value.reference);
    if (outcome==NEOCLR_WORKER_CANCELLED) { r->cancelled++; CHECK(!root.pool); }
    else { CHECK(outcome==NEOCLR_WORKER_OK && root.pool); r->roots[r->count-r->cancelled]=root; }
    r->count++;
    CHECK(neoclr_workers_take(p,ticket,&outcome,&value,&root)==-1);
}
static void *wrong_owner(void *state) {
    CHECK(neoclr_workers_completion_descriptor(state)==-1); return NULL;
}
int main(void) {
    neoclr_workers *p=neoclr_workers_create(1,3,8192); CHECK(p);
    neoclr_worker_handle capture,t[3]; CHECK(neoclr_workers_root_new(p,0,&capture));
    control c={0}; receiver r={0};
    int fd=neoclr_workers_completion_descriptor(p); CHECK(fd>=0);
    CHECK(neoclr_workers_completion_descriptor(p)==fd && !readable(fd,0));
    CHECK(fcntl(fd,F_GETFL)&O_NONBLOCK); CHECK(fcntl(fd,F_GETFD)&FD_CLOEXEC);
    pthread_t thread; CHECK(!pthread_create(&thread,NULL,wrong_owner,p)); CHECK(!pthread_join(thread,NULL));
    for (unsigned i=0;i<3;i++) CHECK(neoclr_workers_submit(p,capture,work,&c,&t[i]));
    neoclr_worker_handle unused; CHECK(!neoclr_workers_submit(p,capture,work,&c,&unused));
    CHECK(neoclr_workers_root_release(p,capture));
    CHECK(neoclr_workers_cancel(p,t[2]) && readable(fd,1000));
    uint64_t extent; CHECK(neoclr_workers_collect(p,&extent) && extent);
    CHECK(neoclr_workers_dispatch(p,0,delivered,&r)==0 && readable(fd,0));
    CHECK(neoclr_workers_dispatch(p,1,delivered,&r)==1 && r.cancelled==1 && !readable(fd,0));
    atomic_store(&c.release,1);
    CHECK(neoclr_workers_wait(p,t[0],1000)==1 && neoclr_workers_wait(p,t[1],1000)==1);
    int bytes=0; CHECK(!ioctl(fd,FIONREAD,&bytes) && bytes==1);
    CHECK(neoclr_workers_collect(p,&extent) && extent); /* Only ticket roots remain. */
    CHECK(neoclr_workers_dispatch(p,1,delivered,&r)==1 && readable(fd,0));
    CHECK(neoclr_workers_dispatch(p,1,delivered,&r)==1 && !readable(fd,0));
    CHECK(neoclr_workers_dispatch(p,3,delivered,&r)==0 && r.count==3);
    CHECK(neoclr_workers_collect(p,&extent) && extent); /* Adopted roots preserve identity. */
    for (unsigned i=0;i<2;i++) {
        uint64_t value; CHECK(neoclr_workers_root_read(p,r.roots[i],&value) && value==2);
    }
    CHECK(neoclr_workers_submit(p,r.roots[0],work,&c,&t[0]));
    CHECK(neoclr_workers_wait(p,t[0],1000)==1 && readable(fd,0));
    int outcome; neoclr_worker_value value; neoclr_worker_handle root;
    CHECK(neoclr_workers_take(p,t[0],&outcome,&value,&root)==1 && !readable(fd,0));
    uint64_t updated; CHECK(neoclr_workers_root_read(p,r.roots[1],&updated) && updated==3);
    CHECK(neoclr_workers_root_release(p,root));
    for (unsigned i=0;i<2;i++) CHECK(neoclr_workers_root_release(p,r.roots[i]));
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    CHECK(fcntl(fd,F_GETFL)==-1 && errno==EBADF);

    /* Register after completion; close cancels queued/running work and remains drainable. */
    p=neoclr_workers_create(1,3,8192); CHECK(p && neoclr_workers_root_new(p,0,&capture));
    CHECK(neoclr_workers_submit(p,capture,work,&c,&t[0]));
    CHECK(neoclr_workers_wait(p,t[0],1000)==1);
    fd=neoclr_workers_completion_descriptor(p); CHECK(readable(fd,0));
    CHECK(neoclr_workers_take(p,t[0],&outcome,&value,&root)==1 && !readable(fd,0));
    CHECK(neoclr_workers_root_release(p,root));
    control closing={.cancel_wait=1}; r=(receiver){0};
    for (unsigned i=0;i<3;i++) CHECK(neoclr_workers_submit(p,capture,work,&closing,&t[i]));
    CHECK(neoclr_workers_root_release(p,capture));
    CHECK(neoclr_workers_close(p) && readable(fd,0));
    CHECK(!neoclr_workers_destroy(p));
    CHECK(neoclr_workers_dispatch(p,3,delivered,&r)==3 && r.cancelled==3 && !readable(fd,0));
    CHECK(neoclr_workers_destroy(p));
    puts("PASS coalescing, budgets, cancellation, late registration, roots/identity, take suppression and close");
}
