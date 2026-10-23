#define _POSIX_C_SOURCE 200809L
#include "native-workers.h"
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <time.h>
#define JOBS 16
#define ROOTS 32
#define THREADS 4
enum { FREE, QUEUED, RUNNING, DONE };
struct neoclr_worker_job {
    struct neoclr_workers *pool;
    uint64_t generation, order;
    unsigned state;
    int cancelled, outcome;
    void *capture;
    neoclr_worker_value result;
    neoclr_worker_callback callback;
    void *native_state;
};
struct root { uint64_t generation; void *value; };
struct neoclr_workers {
    uint64_t identity, submission;
    pthread_t owner, threads[THREADS];
    pthread_mutex_t gate;
    pthread_cond_t ready, completion;
    unsigned thread_count, capacity;
    int closing, joined;
    neoclr_aot_context context;
    struct neoclr_worker_job jobs[JOBS];
    struct root roots[ROOTS];
};
static _Atomic uint64_t next_identity = 1;
static _Thread_local neoclr_worker_job *entered, *executing;
static void lock(neoclr_workers *p) { if (entered || pthread_mutex_lock(&p->gate)) abort(); }
static void unlock(neoclr_workers *p) { if (pthread_mutex_unlock(&p->gate)) abort(); }
static neoclr_worker_job *job_for(neoclr_workers *p, neoclr_worker_handle h) {
    if (h.kind != 2 || h.pool != p->identity || h.slot >= p->capacity) return NULL;
    neoclr_worker_job *j = &p->jobs[h.slot];
    return j->state != FREE && j->generation == h.generation ? j : NULL;
}
static struct root *root_for(neoclr_workers *p, neoclr_worker_handle h) {
    if (h.kind != 1 || h.pool != p->identity || h.slot >= ROOTS) return NULL;
    struct root *r = &p->roots[h.slot];
    return r->value && r->generation == h.generation ? r : NULL;
}
static int adopt(neoclr_workers *p, void *value, neoclr_worker_handle *out) {
    if (!value) { *out = (neoclr_worker_handle){0}; return 1; }
    for (unsigned i = 0; i < ROOTS; i++) {
        struct root *r = &p->roots[i];
        if (!r->value && r->generation != UINT64_MAX) {
            ++r->generation; r->value = value;
            *out = (neoclr_worker_handle){p->identity, r->generation, i, 1};
            return 1;
        }
    }
    return 0;
}
static void *run(void *argument) {
    neoclr_workers *p = argument;
    for (;;) {
        lock(p);
        neoclr_worker_job *j = NULL;
        while (!j && !p->closing) {
            for (unsigned i = 0; i < p->capacity; i++)
                if (p->jobs[i].state == QUEUED && (!j || p->jobs[i].order < j->order)) j = &p->jobs[i];
            if (!j && pthread_cond_wait(&p->ready, &p->gate)) abort();
        }
        if (!j) { unlock(p); return NULL; }
        j->state = RUNNING;
        unlock(p);
        executing = j;
        int outcome = j->callback(j, j->native_state);
        executing = NULL;
        if (entered) abort();
        lock(p);
        j->outcome = j->cancelled ? NEOCLR_WORKER_CANCELLED :
            outcome ? NEOCLR_WORKER_FAILED : NEOCLR_WORKER_OK;
        if (j->outcome != NEOCLR_WORKER_OK) j->result = (neoclr_worker_value){0};
        j->capture = NULL;
        j->state = DONE;
        pthread_cond_broadcast(&p->completion);
        unlock(p);
    }
}
neoclr_workers *neoclr_workers_create(unsigned workers, unsigned capacity, size_t bytes) {
    if (!workers || workers > THREADS || !capacity || capacity > JOBS || bytes < 256 || (bytes & 7)) return NULL;
    neoclr_workers *p = calloc(1, sizeof(*p));
    if (!p) return NULL;
    p->context.text.data = calloc(1, bytes);
    if (!p->context.text.data) { free(p); return NULL; }
    p->context.text.capacity = bytes;
    p->identity = atomic_fetch_add(&next_identity, 1);
    if (!p->identity || p->identity == UINT64_MAX) abort();
    p->owner = pthread_self(); p->capacity = capacity;
    if (pthread_mutex_init(&p->gate, NULL)) { free(p->context.text.data); free(p); return NULL; }
    if (pthread_cond_init(&p->ready, NULL)) {
        pthread_mutex_destroy(&p->gate); free(p->context.text.data); free(p); return NULL;
    }
    if (pthread_cond_init(&p->completion, NULL)) {
        pthread_cond_destroy(&p->ready); pthread_mutex_destroy(&p->gate);
        free(p->context.text.data); free(p); return NULL;
    }
    for (unsigned i = 0; i < capacity; i++) p->jobs[i].pool = p;
    for (; p->thread_count < workers; p->thread_count++) {
        if (pthread_create(&p->threads[p->thread_count], NULL, run, p)) {
            neoclr_workers_close(p); neoclr_workers_destroy(p); return NULL;
        }
    }
    return p;
}
int neoclr_workers_root_new(neoclr_workers *p, uint64_t initial, neoclr_worker_handle *out) {
    lock(p);
    int available = 0;
    for (unsigned i = 0; i < ROOTS; i++) available |= !p->roots[i].value && p->roots[i].generation != UINT64_MAX;
    void *value = NULL;
    int ok = !p->closing && available && !neoclr_gc_allocate_v1(&p->context.text, 8, NEOCLR_GC_BYTES, &value);
    if (ok) { memcpy(value, &initial, 8); ok = adopt(p, value, out); }
    unlock(p); return ok;
}
int neoclr_workers_root_read(neoclr_workers *p, neoclr_worker_handle h, uint64_t *out) {
    lock(p); struct root *r = root_for(p, h);
    uint64_t bytes = 0; uint32_t kind = 0;
    int ok = r && !neoclr_gc_allocation_info_v1(&p->context, r->value, &bytes, &kind) && bytes >= 8;
    if (ok) memcpy(out, r->value, 8);
    unlock(p); return ok;
}
int neoclr_workers_root_release(neoclr_workers *p, neoclr_worker_handle h) {
    lock(p); struct root *r = root_for(p, h);
    if (r) r->value = NULL;
    unlock(p); return r != NULL;
}
int neoclr_workers_submit(neoclr_workers *p, neoclr_worker_handle capture,
    neoclr_worker_callback callback, void *native_state, neoclr_worker_handle *out) {
    lock(p); struct root *r = root_for(p, capture); int ok = 0;
    if (!p->closing && r && callback && p->submission != UINT64_MAX) for (unsigned i = 0; i < p->capacity; i++) {
        neoclr_worker_job *j = &p->jobs[i];
        if (j->state != FREE || j->generation == UINT64_MAX) continue;
        j->order = ++p->submission; ++j->generation; j->capture = r->value; j->callback = callback;
        j->native_state = native_state; j->cancelled = 0; j->result = (neoclr_worker_value){0};
        j->state = QUEUED; *out = (neoclr_worker_handle){p->identity, j->generation, i, 2};
        pthread_cond_signal(&p->ready); ok = 1; break;
    }
    unlock(p); return ok;
}
int neoclr_workers_cancel(neoclr_workers *p, neoclr_worker_handle h) {
    lock(p); neoclr_worker_job *j = job_for(p, h); int ok = j && j->state != DONE && !j->cancelled;
    if (ok) {
        j->cancelled = 1;
        if (j->state == QUEUED) { j->capture = NULL; j->outcome = NEOCLR_WORKER_CANCELLED; j->state = DONE; }
    }
    if (ok) pthread_cond_broadcast(&p->completion);
    unlock(p); return ok;
}
int neoclr_worker_cancelled(neoclr_worker_job *j) {
    lock(j->pool); int result = j->cancelled; unlock(j->pool); return result;
}
void neoclr_worker_enter(neoclr_worker_job *j) { if (executing != j) abort(); lock(j->pool); entered = j; }
void *neoclr_worker_capture(neoclr_worker_job *j) { if (entered != j) abort(); return j->capture; }
void *neoclr_worker_allocate(neoclr_worker_job *j, uint64_t bytes, uint32_t kind) {
    if (entered != j) abort();
    void *result = NULL;
    if (neoclr_gc_allocate_v1(&j->pool->context.text, bytes, kind, &result)) return NULL;
    return result;
}
void neoclr_worker_result(neoclr_worker_job *j, neoclr_worker_value value) {
    if (entered != j) abort();
    uint64_t bytes; uint32_t kind;
    if (value.reference && neoclr_gc_allocation_info_v1(&j->pool->context, value.reference, &bytes, &kind)) abort();
    j->result = value;
}
void neoclr_worker_leave(neoclr_worker_job *j) { if (entered != j) abort(); entered = NULL; unlock(j->pool); }
int neoclr_workers_wait(neoclr_workers *p, neoclr_worker_handle h, unsigned milliseconds) {
    if ((executing && executing->pool == p) || milliseconds > 60000) return -1;
    struct timespec until;
    if (clock_gettime(CLOCK_REALTIME, &until)) return -1;
    until.tv_sec += milliseconds / 1000;
    until.tv_nsec += (long)(milliseconds % 1000) * 1000000;
    if (until.tv_nsec >= 1000000000) { ++until.tv_sec; until.tv_nsec -= 1000000000; }
    lock(p);
    int status = -1;
    for (;;) {
        neoclr_worker_job *j = job_for(p, h);
        if (!j) break;
        if (j->state == DONE) { status = 1; break; }
        int error = pthread_cond_timedwait(&p->completion, &p->gate, &until);
        if (error == ETIMEDOUT) {
            j = job_for(p, h);
            status = !j ? -1 : j->state == DONE ? 1 : 0;
            break;
        }
        if (error) break;
    }
    unlock(p); return status;
}
int neoclr_workers_take(neoclr_workers *p, neoclr_worker_handle h, int *outcome,
    neoclr_worker_value *value, neoclr_worker_handle *root) {
    lock(p); neoclr_worker_job *j = job_for(p, h); int status = -1;
    if (j) {
        status = 0;
        if (j->state == DONE && adopt(p, j->result.reference, root)) {
            *outcome = j->outcome; *value = j->result; value->reference = NULL;
            j->result = (neoclr_worker_value){0}; j->callback = NULL; j->native_state = NULL; j->state = FREE; status = 1;
        }
    }
    unlock(p); return status;
}
int neoclr_workers_collect(neoclr_workers *p, uint64_t *extent) {
    if (!pthread_equal(p->owner, pthread_self())) return 0;
    lock(p);
    neoclr_probe_storage slots[ROOTS + 2 * JOBS]; unsigned count = 0;
    for (unsigned i = 0; i < ROOTS; i++) slots[count++] = (neoclr_probe_storage){&p->roots[i].value, 8, 0};
    for (unsigned i = 0; i < p->capacity; i++) {
        slots[count++] = (neoclr_probe_storage){&p->jobs[i].capture, 8, 0};
        slots[count++] = (neoclr_probe_storage){&p->jobs[i].result.reference, 8, 0};
    }
    neoclr_probe_frame frame = {.context = &p->context, .storage = slots, .storage_count = count};
    int ok = !neoclr_gc_collect_v1(&p->context, &frame);
    if (ok) *extent = p->context.text.used;
    unlock(p); return ok;
}
int neoclr_workers_close(neoclr_workers *p) {
    if (!pthread_equal(p->owner, pthread_self()) || entered) return 0;
    lock(p);
    if (p->joined) { unlock(p); return 1; }
    p->closing = 1;
    for (unsigned i = 0; i < p->capacity; i++) {
        neoclr_worker_job *j = &p->jobs[i];
        if (j->state == RUNNING) j->cancelled = 1;
        if (j->state == QUEUED) {
            j->cancelled = 1; j->capture = NULL; j->outcome = NEOCLR_WORKER_CANCELLED; j->state = DONE;
        }
    }
    pthread_cond_broadcast(&p->ready); pthread_cond_broadcast(&p->completion); unlock(p);
    for (unsigned i = 0; i < p->thread_count; i++) if (pthread_join(p->threads[i], NULL)) abort();
    p->joined = 1; return 1;
}
int neoclr_workers_destroy(neoclr_workers *p) {
    if (!pthread_equal(p->owner, pthread_self()) || !p->joined || entered) return 0;
    for (unsigned i = 0; i < p->capacity; i++) if (p->jobs[i].state != FREE) return 0;
    for (unsigned i = 0; i < ROOTS; i++) if (p->roots[i].value) return 0;
    uint64_t extent;
    if (!neoclr_workers_collect(p, &extent) || extent) return 0;
    pthread_cond_destroy(&p->completion); pthread_cond_destroy(&p->ready); pthread_mutex_destroy(&p->gate);
    free(p->context.text.data); free(p); return 1;
}
