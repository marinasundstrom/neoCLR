#define _POSIX_C_SOURCE 200809L
#include "native-workers.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); abort(); } } while (0)
typedef struct {
    pthread_mutex_t gate;
    pthread_cond_t changed;
    unsigned started, prepared;
    int release, publish, mode;
} control;
static void init(control *c, int mode) {
    *c = (control){.mode = mode};
    CHECK(!pthread_mutex_init(&c->gate, NULL));
    CHECK(!pthread_cond_init(&c->changed, NULL));
}
static void dispose(control *c) {
    CHECK(!pthread_cond_destroy(&c->changed)); CHECK(!pthread_mutex_destroy(&c->gate));
}
static void wait_count(control *c, unsigned *value, unsigned expected) {
    struct timespec until;
    CHECK(!clock_gettime(CLOCK_REALTIME, &until)); until.tv_sec += 10;
    CHECK(!pthread_mutex_lock(&c->gate));
    while (*value < expected) CHECK(!pthread_cond_timedwait(&c->changed, &c->gate, &until));
    CHECK(!pthread_mutex_unlock(&c->gate));
}
static void release(control *c, int publish) {
    CHECK(!pthread_mutex_lock(&c->gate));
    if (publish) c->publish = 1; else c->release = 1;
    CHECK(!pthread_cond_broadcast(&c->changed)); CHECK(!pthread_mutex_unlock(&c->gate));
}
static int work(neoclr_worker_job *j, void *state) {
    control *c = state;
    CHECK(!pthread_mutex_lock(&c->gate));
    ++c->started; CHECK(!pthread_cond_broadcast(&c->changed));
    while (!c->release) CHECK(!pthread_cond_wait(&c->changed, &c->gate));
    CHECK(!pthread_mutex_unlock(&c->gate));
    neoclr_worker_enter(j);
    uint64_t *capture = neoclr_worker_capture(j);
    ++*capture;
    uint64_t value = *capture;
    if (c->mode == 1) {
        uint64_t *created = neoclr_worker_allocate(j, 8, NEOCLR_GC_BYTES);
        CHECK(created); *created = value + 100;
        neoclr_worker_result(j, (neoclr_worker_value){7, (int64_t)value, created});
    } else if (c->mode != 2) {
        neoclr_worker_result(j, (neoclr_worker_value){3, (int64_t)value, capture});
    }
    neoclr_worker_leave(j);
    CHECK(!pthread_mutex_lock(&c->gate));
    ++c->prepared; CHECK(!pthread_cond_broadcast(&c->changed));
    /* Result is rooted before settlement, even during this parked interval. */
    while (!c->publish) CHECK(!pthread_cond_wait(&c->changed, &c->gate));
    CHECK(!pthread_mutex_unlock(&c->gate));
    return c->mode == 2;
}
static int cooperative(neoclr_worker_job *j, void *state) {
    control *c = state;
    CHECK(!pthread_mutex_lock(&c->gate)); ++c->started;
    CHECK(!pthread_cond_broadcast(&c->changed)); CHECK(!pthread_mutex_unlock(&c->gate));
    while (!neoclr_worker_cancelled(j)) {
        struct timespec delay = {0, 1000000}; nanosleep(&delay, NULL);
    }
    return 0;
}
static int quick(neoclr_worker_job *j, void *unused) {
    (void)unused;
    neoclr_worker_enter(j);
    neoclr_worker_result(j, (neoclr_worker_value){1, 42, NULL});
    neoclr_worker_leave(j); return 0;
}
static int take(neoclr_workers *p, neoclr_worker_handle t, neoclr_worker_value *v, neoclr_worker_handle *r) {
    CHECK(neoclr_workers_wait(p, t, 10000) == 1);
    int outcome = -1;
    CHECK(neoclr_workers_take(p, t, &outcome, v, r) == 1);
    return outcome;
}
static void lifetimes(void) {
    neoclr_workers *p = neoclr_workers_create(2, 4, 8192);
    CHECK(p);
    neoclr_worker_handle root, t[4], adopted[2], ignored;
    CHECK(neoclr_workers_root_new(p, 10, &root));
    control c; init(&c, 0);
    CHECK(neoclr_workers_submit(p, root, work, &c, &t[0]));
    CHECK(neoclr_workers_submit(p, root, work, &c, &t[1]));
    wait_count(&c, &c.started, 2);
    CHECK(neoclr_workers_wait(p, t[0], 0) == 0);
    CHECK(neoclr_workers_submit(p, root, quick, NULL, &t[2]));
    CHECK(neoclr_workers_submit(p, root, quick, NULL, &t[3]));
    ignored = (neoclr_worker_handle){99, 99, 99, 99};
    CHECK(!neoclr_workers_submit(p, root, quick, NULL, &ignored) && ignored.pool == 99);
    CHECK(!neoclr_workers_root_release(p, t[0]));
    CHECK(!neoclr_workers_cancel(p, root));
    CHECK(neoclr_workers_root_release(p, root));
    uint64_t extent, value;
    CHECK(neoclr_workers_collect(p, &extent) && extent > 0);
    CHECK(neoclr_workers_cancel(p, t[2]));
    CHECK(!neoclr_workers_cancel(p, t[2]));
    release(&c, 0); wait_count(&c, &c.prepared, 2);
    CHECK(neoclr_workers_collect(p, &extent) && extent > 0);
    release(&c, 1);
    neoclr_worker_value v;
    for (unsigned i = 0; i < 2; i++) {
        CHECK(take(p, t[i], &v, &adopted[i]) == NEOCLR_WORKER_OK && v.tag == 3 && !v.reference);
        CHECK(!neoclr_workers_cancel(p, t[i]));
        CHECK(neoclr_workers_root_read(p, adopted[i], &value) && value == 12);
    }
    CHECK(take(p, t[2], &v, &ignored) == NEOCLR_WORKER_CANCELLED && !ignored.pool);
    CHECK(take(p, t[3], &v, &ignored) == NEOCLR_WORKER_OK && v.scalar == 42 && !ignored.pool);
    CHECK(neoclr_workers_collect(p, &extent) && extent > 0);
    neoclr_worker_handle fresh;
    CHECK(neoclr_workers_submit(p, adopted[0], quick, NULL, &fresh));
    CHECK(!neoclr_workers_cancel(p, t[0]));
    CHECK(take(p, fresh, &v, &ignored) == NEOCLR_WORKER_OK);
    CHECK(neoclr_workers_close(p));
    CHECK(!neoclr_workers_submit(p, adopted[0], quick, NULL, &fresh));
    CHECK(!neoclr_workers_destroy(p));
    for (unsigned i = 0; i < 2; i++) CHECK(neoclr_workers_root_release(p, adopted[i]));
    CHECK(neoclr_workers_collect(p, &extent) && !extent);
    CHECK(neoclr_workers_destroy(p)); dispose(&c);
    puts("PASS bounded admission, two active participants, shared identity, roots, stale tickets");
}
static void result_and_cancel(void) {
    for (int mode = 0; mode < 3; mode++) {
        neoclr_workers *p = neoclr_workers_create(1, 1, 8192);
        neoclr_worker_handle root, ticket, result;
        CHECK(p && neoclr_workers_root_new(p, 20, &root));
        control c; init(&c, mode == 2 ? 2 : 1);
        CHECK(neoclr_workers_submit(p, root, work, &c, &ticket));
        wait_count(&c, &c.started, 1); CHECK(neoclr_workers_root_release(p, root));
        release(&c, 0); wait_count(&c, &c.prepared, 1);
        uint64_t extent, value;
        CHECK(neoclr_workers_collect(p, &extent) && extent > 0);
        if (mode == 1) CHECK(neoclr_workers_cancel(p, ticket));
        release(&c, 1);
        neoclr_worker_value v;
        int outcome = take(p, ticket, &v, &result);
        CHECK(outcome == (mode == 0 ? NEOCLR_WORKER_OK : mode == 1 ? NEOCLR_WORKER_CANCELLED : NEOCLR_WORKER_FAILED));
        CHECK(!neoclr_workers_cancel(p, ticket));
        if (!mode) {
            CHECK(v.tag == 7 && v.scalar == 21);
            CHECK(neoclr_workers_collect(p, &extent));
            CHECK(neoclr_workers_root_read(p, result, &value) && value == 121);
            CHECK(neoclr_workers_root_release(p, result));
        } else CHECK(!result.pool && !v.reference);
        CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p)); dispose(&c);
    }
    puts("PASS unpublished result retention, tagged payload, cancellation winner, callback failure");
}
static void shutdown(void) {
    neoclr_workers *p = neoclr_workers_create(2, 4, 8192);
    neoclr_worker_handle root, t[4], r; neoclr_worker_value v;
    CHECK(p && neoclr_workers_root_new(p, 0, &root));
    control c; init(&c, 0);
    for (unsigned i = 0; i < 2; i++) CHECK(neoclr_workers_submit(p, root, cooperative, &c, &t[i]));
    wait_count(&c, &c.started, 2);
    for (unsigned i = 2; i < 4; i++) CHECK(neoclr_workers_submit(p, root, quick, NULL, &t[i]));
    CHECK(neoclr_workers_root_release(p, root));
    CHECK(neoclr_workers_close(p) && neoclr_workers_close(p));
    CHECK(!neoclr_workers_destroy(p));
    for (unsigned i = 0; i < 4; i++) CHECK(take(p, t[i], &v, &r) == NEOCLR_WORKER_CANCELLED);
    CHECK(neoclr_workers_destroy(p)); dispose(&c);
    p = neoclr_workers_create(1, 1, 1024); CHECK(p);
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    puts("PASS close cancels queued and active work, reserved completions drain, joined teardown");
}
static void pressure_and_foreign(void) {
    neoclr_workers *p = neoclr_workers_create(1, 1, 8192);
    neoclr_workers *other = neoclr_workers_create(1, 1, 8192);
    CHECK(p && other);
    neoclr_worker_handle roots[32], ticket, adopted;
    for (unsigned i = 0; i < 32; i++) CHECK(neoclr_workers_root_new(p, i, &roots[i]));
    CHECK(!neoclr_workers_root_new(p, 99, &adopted));
    CHECK(!neoclr_workers_submit(other, roots[0], quick, NULL, &adopted));
    control c; init(&c, 1); release(&c, 0); release(&c, 1);
    CHECK(neoclr_workers_submit(p, roots[0], work, &c, &ticket));
    CHECK(neoclr_workers_wait(p, ticket, 10000) == 1);
    CHECK(neoclr_workers_wait(other, ticket, 0) == -1);
    CHECK(!neoclr_workers_cancel(p, ticket));
    int outcome = 99; neoclr_worker_value value = {.tag = 99};
    adopted = (neoclr_worker_handle){99, 99, 99, 99};
    CHECK(neoclr_workers_take(p, ticket, &outcome, &value, &adopted) == 0);
    CHECK(outcome == 99 && value.tag == 99 && adopted.pool == 99);
    uint64_t extent, scalar;
    CHECK(neoclr_workers_collect(p, &extent));
    CHECK(neoclr_workers_root_release(p, roots[31]));
    CHECK(take(p, ticket, &value, &adopted) == NEOCLR_WORKER_OK);
    CHECK(neoclr_workers_root_read(p, adopted, &scalar) && scalar == 101);
    CHECK(!neoclr_workers_root_release(p, roots[31])); /* reused root generation */
    for (unsigned i = 0; i < 31; i++) CHECK(neoclr_workers_root_release(p, roots[i]));
    CHECK(neoclr_workers_root_release(p, adopted));
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    CHECK(neoclr_workers_close(other) && neoclr_workers_destroy(other));
    dispose(&c);
    puts("PASS result backpressure, foreign pool rejection, root generations and output preservation");
}
static void settlement_race(void) {
    neoclr_workers *p = neoclr_workers_create(2, 2, 8192);
    neoclr_worker_handle root; CHECK(p && neoclr_workers_root_new(p, 0, &root));
    for (unsigned i = 0; i < 100; i++) {
        control c; init(&c, 0);
        neoclr_worker_handle ticket, adopted;
        CHECK(neoclr_workers_submit(p, root, work, &c, &ticket));
        release(&c, 0); wait_count(&c, &c.prepared, 1);
        release(&c, 1);
        int won = neoclr_workers_cancel(p, ticket);
        neoclr_worker_value value;
        CHECK(take(p, ticket, &value, &adopted) == (won ? NEOCLR_WORKER_CANCELLED : NEOCLR_WORKER_OK));
        if (adopted.pool) CHECK(neoclr_workers_root_release(p, adopted));
        dispose(&c);
    }
    CHECK(neoclr_workers_root_release(p, root));
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    puts("PASS 100 completion/cancellation contests with exactly one outcome");
}
static int exhaust(neoclr_worker_job *j, void *unused) {
    (void)unused;
    neoclr_worker_enter(j);
    void *too_large = neoclr_worker_allocate(j, 1024 * 1024, NEOCLR_GC_BYTES);
    neoclr_worker_leave(j);
    CHECK(!too_large);
    return 1;
}
static void recovery(void) {
    neoclr_workers *p = neoclr_workers_create(1, 1, 256);
    neoclr_worker_handle root, t, r; neoclr_worker_value v;
    CHECK(p && neoclr_workers_root_new(p, 0, &root));
    CHECK(neoclr_workers_submit(p, root, exhaust, NULL, &t));
    CHECK(take(p, t, &v, &r) == NEOCLR_WORKER_FAILED);
    CHECK(neoclr_workers_submit(p, root, quick, NULL, &t));
    CHECK(take(p, t, &v, &r) == NEOCLR_WORKER_OK && v.scalar == 42);
    CHECK(neoclr_workers_root_release(p, root));
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    puts("PASS allocation failure preserves pool and subsequent work completes");
}
static int participant(neoclr_worker_job *j, void *state) {
    control *c = state;
    neoclr_worker_enter(j);
    for (unsigned i = 0; i < NEOCLR_WORKER_LOCAL_ROOTS; i++) {
        CHECK(!neoclr_worker_local_load(j, i)); /* job slot reuse must be clean */
        uint64_t *value = neoclr_worker_allocate(j, 8, NEOCLR_GC_BYTES);
        CHECK(value); *value = 100 + i;
        CHECK(neoclr_worker_local_store(j, i, value));
    }
    uint64_t foreign = 7;
    CHECK(!neoclr_worker_local_store(j, 0, &foreign));
    CHECK(!neoclr_worker_local_store(j, 0, (char *)neoclr_worker_local_load(j, 0) + 1));
    CHECK(!neoclr_worker_local_store(j, NEOCLR_WORKER_LOCAL_ROOTS, NULL));
    CHECK(!neoclr_worker_local_load(j, NEOCLR_WORKER_LOCAL_ROOTS));
    neoclr_worker_leave(j);
    CHECK(!pthread_mutex_lock(&c->gate));
    ++c->prepared; CHECK(!pthread_cond_broadcast(&c->changed));
    while (!c->release) CHECK(!pthread_cond_wait(&c->changed, &c->gate));
    CHECK(!pthread_mutex_unlock(&c->gate));
    neoclr_worker_enter(j);
    for (unsigned i = 0; i < NEOCLR_WORKER_LOCAL_ROOTS; i++) {
        uint64_t *value = neoclr_worker_local_load(j, i);
        CHECK(value && *value == 100 + i);
        CHECK(neoclr_worker_local_store(j, i, value)); /* still an allocated base */
    }
    /* Transfer one local into completion ownership; leave others registered to
     * prove automatic cleanup on success, failure and cancellation. */
    void *result = neoclr_worker_local_load(j, 7);
    neoclr_worker_result(j, (neoclr_worker_value){1, 107, result});
    CHECK(neoclr_worker_local_store(j, 7, NULL));
    CHECK(!neoclr_worker_local_load(j, 7));
    neoclr_worker_leave(j);
    return c->mode == 1;
}
static void participant_roots(void) {
    neoclr_workers *p = neoclr_workers_create(4, 4, 16384);
    CHECK(p);
    for (int mode = 0; mode < 3; mode++) {
        control c; init(&c, mode);
        neoclr_worker_handle capture, tickets[4];
        CHECK(neoclr_workers_root_new(p, 42, &capture));
        for (unsigned i = 0; i < 4; i++)
            CHECK(neoclr_workers_submit(p, capture, participant, &c, &tickets[i]));
        CHECK(neoclr_workers_root_release(p, capture));
        wait_count(&c, &c.prepared, 4);
        uint64_t extent;
        /* No capture or result refers to the 32 new temporaries. Only the
         * participants' published local slots can keep these objects alive. */
        for (unsigned i = 0; i < 3; i++) CHECK(neoclr_workers_collect(p, &extent) && extent);
        if (mode == 2) for (unsigned i = 0; i < 4; i++)
            CHECK(neoclr_workers_cancel(p, tickets[i]));
        release(&c, 0);
        for (unsigned i = 0; i < 4; i++) {
            neoclr_worker_value value; neoclr_worker_handle adopted;
            int expected = mode == 0 ? NEOCLR_WORKER_OK : mode == 1 ? NEOCLR_WORKER_FAILED : NEOCLR_WORKER_CANCELLED;
            CHECK(take(p, tickets[i], &value, &adopted) == expected);
            if (mode == 0) {
                uint64_t scalar;
                CHECK(neoclr_workers_collect(p, &extent));
                CHECK(neoclr_workers_root_read(p, adopted, &scalar) && scalar == 107);
                CHECK(neoclr_workers_root_release(p, adopted));
            } else CHECK(!adopted.pool);
        }
        CHECK(neoclr_workers_collect(p, &extent) && !extent);
        dispose(&c);
    }
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    puts("PASS four participant root sets across collection, result handoff, failure, cancellation and reuse");
}
typedef struct {
    pthread_t owner;
    neoclr_worker_handle root, expected, followup;
    unsigned calls;
    int closing;
} delivery;
static void delivered(neoclr_workers *p, neoclr_worker_handle ticket, void *state) {
    delivery *d = state;
    CHECK(pthread_equal(pthread_self(), d->owner));
    CHECK(ticket.pool == d->expected.pool && ticket.slot == d->expected.slot && ticket.generation == d->expected.generation);
    CHECK(neoclr_workers_dispatch(p, 1, delivered, state) == -1);
    uint64_t extent;
    CHECK(neoclr_workers_collect(p, &extent)); /* callback is outside the gate */
    neoclr_worker_value value; neoclr_worker_handle root;
    int outcome;
    CHECK(neoclr_workers_take(p, ticket, &outcome, &value, &root) == 1);
    if (!d->calls) {
        CHECK(outcome == NEOCLR_WORKER_CANCELLED && !root.pool);
        CHECK(neoclr_workers_submit(p, d->root, quick, NULL, &d->followup));
        CHECK(neoclr_workers_wait(p, d->followup, 10000) == 1);
    } else {
        CHECK(outcome == NEOCLR_WORKER_OK);
        if (root.pool) {
            uint64_t scalar;
            CHECK(neoclr_workers_root_read(p, root, &scalar) && scalar == 1);
            CHECK(neoclr_workers_root_release(p, root));
        } else CHECK(value.scalar == 42);
    }
    ++d->calls;
    if (d->closing) {
        CHECK(neoclr_workers_root_release(p, d->root));
        CHECK(neoclr_workers_close(p));
        CHECK(!neoclr_workers_destroy(p)); /* even though all roots/tickets drained */
    }
}
static void notified(neoclr_workers *p, neoclr_worker_handle ticket, void *state) {
    (void)p; (void)ticket;
    ++*(unsigned *)state;
}
static int wrong_owner(neoclr_worker_job *j, void *state) {
    (void)j;
    unsigned unused = 0;
    CHECK(neoclr_workers_dispatch(state, 1, notified, &unused) == -1 && !unused);
    return 0;
}
static void completion_dispatch(void) {
    neoclr_workers *p = neoclr_workers_create(1, 2, 8192);
    delivery d = {.owner = pthread_self()};
    CHECK(p && neoclr_workers_root_new(p, 0, &d.root));
    neoclr_worker_handle t, r; neoclr_worker_value v;
    CHECK(neoclr_workers_submit(p, d.root, wrong_owner, p, &t));
    CHECK(take(p, t, &v, &r) == NEOCLR_WORKER_OK);
    unsigned calls = 0;
    CHECK(neoclr_workers_dispatch(p, 2, notified, &calls) == 0 && !calls); /* polled take suppresses */
    CHECK(neoclr_workers_submit(p, d.root, quick, NULL, &t));
    CHECK(neoclr_workers_wait(p, t, 10000) == 1);
    CHECK(neoclr_workers_dispatch(p, 0, notified, &calls) == 0 && !calls);
    CHECK(neoclr_workers_dispatch(p, 17, notified, &calls) == -1 && !calls);
    CHECK(neoclr_workers_dispatch(p, 1, notified, &calls) == 1 && calls == 1);
    CHECK(neoclr_workers_dispatch(p, 2, notified, &calls) == 0 && calls == 1);
    CHECK(take(p, t, &v, &r) == NEOCLR_WORKER_OK); /* notification retained the ticket */
    control c; init(&c, 0);
    neoclr_worker_handle first, second;
    CHECK(neoclr_workers_submit(p, d.root, work, &c, &first));
    wait_count(&c, &c.started, 1);
    CHECK(neoclr_workers_submit(p, d.root, quick, NULL, &second));
    CHECK(neoclr_workers_cancel(p, second));
    release(&c, 0); release(&c, 1);
    CHECK(neoclr_workers_wait(p, first, 10000) == 1);
    d.expected = second;
    CHECK(neoclr_workers_dispatch(p, 1, delivered, &d) == 1 && d.calls == 1);
    d.expected = first;
    CHECK(neoclr_workers_dispatch(p, 1, delivered, &d) == 1 && d.calls == 2);
    d.expected = d.followup; d.closing = 1;
    CHECK(neoclr_workers_dispatch(p, 1, delivered, &d) == 1 && d.calls == 3);
    CHECK(neoclr_workers_destroy(p));
    dispose(&c);
    puts("PASS owner completion delivery, settlement order, bounded dispatch, reentrant submission, notification retention and close");
}
int main(void) {
    lifetimes(); result_and_cancel(); shutdown(); pressure_and_foreign(); settlement_race(); recovery(); participant_roots(); completion_dispatch();
    return 0;
}
