/* Proof-of-concept POSIX worker; not a runtime API or collector. */
#ifndef GC_EXPERIMENT_WORKER_H
#define GC_EXPERIMENT_WORKER_H
#include <pthread.h>
#include <stdatomic.h>
#include <stdbool.h>

/* Input must stay valid until take/destroy completes. Only immutable data may
 * be shared; callbacks may mutate private scratch/output. Cancellation is
 * cooperative: callback must check it and return in bounded work intervals. */
typedef int (*gc_job)(void *input, const atomic_bool *cancel);
typedef struct {
    pthread_t thread;
    pthread_mutex_t mutex;
    pthread_cond_t changed;
    atomic_bool cancel;
    unsigned state; /* 0 empty, 1 pending/running, 2 completed */
    bool stopping;
    gc_job execute;
    void *input;
    int result;
} gc_worker;

/* All APIs have one owner thread. No callback may call these APIs. Storage must
 * not move until destroy returns. A failed init needs no destroy. */
int gc_worker_init(gc_worker *worker);
/* Refuse a second job until the first result is consumed. */
bool gc_worker_submit(gc_worker *worker, gc_job execute, void *input);
/* Return false without blocking until a result is available. Taking a result
 * transfers scratch ownership back to the owner, including on callback failure. */
bool gc_worker_take(gc_worker *worker, bool wait, bool cancel, int *result);
/* Cancel/join outstanding work before releasing synchronization state. Caller
 * still owns input/scratch and may free it only after this returns. */
void gc_worker_destroy(gc_worker *worker);
#endif
