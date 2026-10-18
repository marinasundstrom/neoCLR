#include "worker.h"
#include <stdlib.h>

static void check(int error) {
    if (error) abort();
}
static void *run(void *argument) {
    gc_worker *worker = argument;
    check(pthread_mutex_lock(&worker->mutex));
    for (;;) {
        while (!worker->stopping && worker->state != 1)
            check(pthread_cond_wait(&worker->changed, &worker->mutex));
        if (worker->stopping) break;
        gc_job execute = worker->execute;
        void *input = worker->input;
        check(pthread_mutex_unlock(&worker->mutex));
        int result = execute(input, &worker->cancel);
        /* Publishing state under the mutex makes ordinary scratch writes visible
         * to take(). Cancellation alone never transfers scratch ownership. */
        check(pthread_mutex_lock(&worker->mutex));
        worker->result = result;
        worker->state = 2;
        check(pthread_cond_broadcast(&worker->changed));
    }
    check(pthread_mutex_unlock(&worker->mutex));
    return NULL;
}
int gc_worker_init(gc_worker *worker) {
    *worker = (gc_worker){0};
    atomic_init(&worker->cancel, false);
    int error = pthread_mutex_init(&worker->mutex, NULL);
    if (error) return error;
    error = pthread_cond_init(&worker->changed, NULL);
    if (error) {
        check(pthread_mutex_destroy(&worker->mutex));
        return error;
    }
    error = pthread_create(&worker->thread, NULL, run, worker);
    if (error) {
        check(pthread_cond_destroy(&worker->changed));
        check(pthread_mutex_destroy(&worker->mutex));
    }
    return error;
}
bool gc_worker_submit(gc_worker *worker, gc_job execute, void *input) {
    if (!execute) return false;
    check(pthread_mutex_lock(&worker->mutex));
    bool available = worker->state == 0;
    if (available) {
        worker->execute = execute;
        worker->input = input;
        atomic_store(&worker->cancel, false);
        worker->state = 1;
        check(pthread_cond_signal(&worker->changed));
    }
    check(pthread_mutex_unlock(&worker->mutex));
    return available;
}
bool gc_worker_take(gc_worker *worker, bool wait, bool cancel, int *result) {
    check(pthread_mutex_lock(&worker->mutex));
    if (cancel) atomic_store(&worker->cancel, true);
    while (wait && worker->state == 1)
        check(pthread_cond_wait(&worker->changed, &worker->mutex));
    bool ready = worker->state == 2;
    if (ready) {
        *result = worker->result;
        worker->input = NULL;
        worker->execute = NULL;
        worker->state = 0;
    }
    check(pthread_mutex_unlock(&worker->mutex));
    return ready;
}
void gc_worker_destroy(gc_worker *worker) {
    int ignored;
    gc_worker_take(worker, true, true, &ignored);
    check(pthread_mutex_lock(&worker->mutex));
    worker->stopping = true;
    check(pthread_cond_signal(&worker->changed));
    check(pthread_mutex_unlock(&worker->mutex));
    check(pthread_join(worker->thread, NULL));
    /* The callback and its final publication are both finished before teardown. */
    check(pthread_cond_destroy(&worker->changed));
    check(pthread_mutex_destroy(&worker->mutex));
}
