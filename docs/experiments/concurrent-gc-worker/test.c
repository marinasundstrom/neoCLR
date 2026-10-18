#include "worker.h"
#include <assert.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

typedef struct {
    atomic_bool started, progressed, finished;
    pthread_t owner;
    unsigned input, output;
} payload;
static payload *make_payload(unsigned input) {
    payload *p = calloc(1, sizeof(*p));
    assert(p);
    atomic_init(&p->started, false);
    atomic_init(&p->progressed, false);
    atomic_init(&p->finished, false);
    p->owner = pthread_self();
    p->input = input;
    return p;
}
static int handshake(void *argument, const atomic_bool *cancel) {
    payload *p = argument;
    assert(!pthread_equal(p->owner, pthread_self()));
    atomic_store(&p->started, true);
    while (!atomic_load(&p->progressed) && !atomic_load(cancel)) sched_yield();
    p->output = p->input + 1;
    atomic_store(&p->finished, true);
    return atomic_load(cancel) ? 7 : 0;
}
static void wait_started(payload *p) {
    struct timespec start, now;
    assert(!clock_gettime(CLOCK_MONOTONIC, &start));
    while (!atomic_load(&p->started)) {
        sched_yield();
        assert(!clock_gettime(CLOCK_MONOTONIC, &now));
        assert(now.tv_sec - start.tv_sec < 10);
    }
}
static int fail_job(void *argument, const atomic_bool *cancel) {
    (void)cancel;
    payload *p = argument;
    p->output = 99;
    atomic_store(&p->finished, true);
    return -23;
}
static void wait_ready(gc_worker *worker) {
    for (;;) {
        assert(!pthread_mutex_lock(&worker->mutex));
        bool ready = worker->state == 2;
        assert(!pthread_mutex_unlock(&worker->mutex));
        if (ready) return;
        sched_yield();
    }
}
int main(void) {
    gc_worker worker;
    assert(!gc_worker_init(&worker));
    int result = -1;
    assert(!gc_worker_take(&worker, false, false, &result));
    assert(!gc_worker_submit(&worker, NULL, NULL));
    for (unsigned i = 0; i < 1000; i++) {
        payload *p = make_payload(i);
        assert(gc_worker_submit(&worker, handshake, p));
        wait_started(p);
        assert(!gc_worker_take(&worker, false, false, &result));
        assert(!gc_worker_submit(&worker, handshake, p));
        /* This owner action occurs after worker entry and before worker exit. */
        if (i % 2 == 0) atomic_store(&p->progressed, true);
        assert(gc_worker_take(&worker, true, i % 2 != 0, &result));
        assert(result == (i % 2 ? 7 : 0));
        assert(p->output == i + 1 && atomic_load(&p->finished));
        assert(!gc_worker_take(&worker, false, false, &result));
        free(p); /* Successful take establishes exclusive ownership again. */
    }
    payload *failed = make_payload(0);
    assert(gc_worker_submit(&worker, fail_job, failed));
    wait_ready(&worker);
    assert(!gc_worker_submit(&worker, fail_job, failed));
    assert(gc_worker_take(&worker, false, false, &result));
    assert(result == -23 && failed->output == 99);
    free(failed);
    gc_worker_destroy(&worker);
    for (unsigned i = 0; i < 100; i++) {
        assert(!gc_worker_init(&worker));
        payload *p = make_payload(i);
        assert(gc_worker_submit(&worker, handshake, p));
        if (i % 3) wait_started(p);
        if (i % 3 == 2) atomic_store(&p->progressed, true);
        /* Covers submission followed immediately by shutdown, running work and
         * completion racing with shutdown. Neither callback nor scratch escapes. */
        gc_worker_destroy(&worker);
        assert(atomic_load(&p->finished) && p->output == i + 1);
        free(p);
    }
    assert(!gc_worker_init(&worker));
    payload *completed = make_payload(0);
    assert(gc_worker_submit(&worker, fail_job, completed));
    wait_ready(&worker);
    gc_worker_destroy(&worker);
    assert(completed->output == 99 && atomic_load(&completed->finished));
    free(completed);
    assert(!gc_worker_init(&worker));
    gc_worker_destroy(&worker);
    puts("1000 reusable handoffs, 100 shutdown races, completed/error/idle paths passed");
}
