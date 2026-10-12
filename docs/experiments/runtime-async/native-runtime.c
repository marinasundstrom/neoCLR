/* Private macOS retained-stack experiment. ucontext is deprecated on macOS;
 * this carrier is qualification scaffolding, not the selected production ABI. */
#define _DARWIN_C_SOURCE 1
#define _XOPEN_SOURCE 700
#include "native-runtime.h"
#include "native-stack.h"
#include "socket-listener.h"
#include <ucontext.h>
#include <sys/mman.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>
#if !defined(__APPLE__) || !defined(__aarch64__)
#error "runtime-async stack carrier is currently a macOS ARM64 experiment"
#endif
#define LIMIT 128
#define STACK_BYTES (1024u * 1024u)
#ifndef NEOCLR_ASYNC_STACK_CACHE_LIMIT
#define NEOCLR_ASYNC_STACK_CACHE_LIMIT 8
#endif
#if NEOCLR_ASYNC_STACK_CACHE_LIMIT < 0 || NEOCLR_ASYNC_STACK_CACHE_LIMIT > LIMIT
#error "invalid experimental stack cache bound"
#endif
typedef struct { void *mapping; size_t bytes; uintptr_t lower; } cached_stack;
static _Thread_local neoclr_async_statistics statistics;
neoclr_async_statistics neoclr_runtime_async_statistics_v1(void) { return statistics; }
static void unmap_stack(void *mapping, size_t bytes) {
    if (munmap(mapping, bytes)) abort();
    ++statistics.unmappings;
    statistics.reserved_bytes -= bytes;
}
typedef struct { uint64_t id; int complete, body; uint64_t owner; } operation;
typedef struct fiber {
    ucontext_t machine;
    neoclr_parked_roots roots;
    void *mapping;
    size_t mapping_bytes;
    uintptr_t lower, upper;
    uint64_t callback, operation, waiting;
    void *owner;
    uint64_t active_owner, active_callback, ready_owner;
    int done, status, drain, parked, managed;
    ucontext_t *caller;
} fiber;
static _Thread_local struct {
    neoclr_aot_context *context;
    neoclr_async_callback invoke;
    neoclr_async_drain drain;
    operation operations[LIMIT];
    fiber fibers[LIMIT];
    fiber *current;
    cached_stack cache[NEOCLR_ASYNC_STACK_CACHE_LIMIT + 1];
    unsigned cached, owned;
    int pumping;
} state;
static _Thread_local uint64_t next_id;
static operation *find(uint64_t id) {
    for (unsigned i = 0; i < LIMIT; ++i)
        if (id && state.operations[i].id == id) return &state.operations[i];
    return NULL;
}
static int admit(neoclr_aot_context *context) {
    if (!context || context->fault.code) return 3;
    if (!state.context) state.context = context;
    return state.context == context ? 0 : 3;
}
static void bounds(fiber *f) {
    neoclr_native_stack_bounds_v1(f ? f->lower : 0, f ? f->upper : 0);
}
static void yield(void) {
    fiber *f = state.current;
    if (f->active_callback && neoclr_gc_host_root_release_v1(state.context, f->active_callback)) abort();
    if (f->active_owner && neoclr_gc_host_root_release_v1(state.context, f->active_owner)) abort();
    f->active_callback = f->active_owner = 0;
    if (f->owner && !f->done) {
        if (neoclr_gc_roots_park_owned_v1(state.context, f->owner, &f->roots)) abort();
    } else neoclr_roots_park_v1(&f->roots);
    f->parked = 1;
    if (swapcontext(&f->machine, f->caller)) abort();
}
static void body(void) {
    fiber *f = state.current;
    f->status = f->drain ? state.drain(state.context)
        : state.invoke(f->owner ? f->active_callback : f->callback, state.context);
    if (neoclr_root_probe_head_v1()) abort();
    if (f->callback && neoclr_gc_host_root_release_v1(state.context, f->callback)) abort();
    f->callback = 0;
    operation *op = find(f->operation);
    if (op && !f->status) op->complete = 1;
    f->done = 1;
    yield();
    abort();
}
static fiber *allocate_fiber(uint64_t callback, uint64_t op, int drain) {
    fiber *f = NULL;
    for (unsigned i = 0; i < LIMIT; ++i)
        if (!state.fibers[i].mapping) { f = &state.fibers[i]; break; }
    if (!f) return NULL;
    cached_stack stack;
    if (state.cached) {
        stack = state.cache[--state.cached];
        memset(&state.cache[state.cached], 0, sizeof(stack));
    } else {
        long page_size = sysconf(_SC_PAGESIZE);
        if (page_size <= 0) return NULL;
        size_t bytes = STACK_BYTES + 2 * (size_t)page_size;
        void *mapping = mmap(NULL, bytes, PROT_NONE, MAP_PRIVATE | MAP_ANON, -1, 0);
        if (mapping == MAP_FAILED) return NULL;
        ++statistics.mappings;
        statistics.reserved_bytes += bytes;
        if (statistics.reserved_bytes > statistics.peak_reserved_bytes)
            statistics.peak_reserved_bytes = statistics.reserved_bytes;
        void *lower = (char *)mapping + page_size;
        if (mprotect(lower, STACK_BYTES, PROT_READ | PROT_WRITE)) {
            unmap_stack(mapping, bytes);
            return NULL;
        }
        stack = (cached_stack){mapping, bytes, (uintptr_t)lower};
    }
    memset(f, 0, sizeof(*f));
    f->mapping = stack.mapping; f->mapping_bytes = stack.bytes;
    f->lower = stack.lower; f->upper = f->lower + STACK_BYTES;
    ++statistics.fibers;
    f->callback = callback; f->operation = op; f->drain = drain;
    if (getcontext(&f->machine)) abort();
    f->machine.uc_stack.ss_sp = (void *)f->lower;
    f->machine.uc_stack.ss_size = STACK_BYTES;
    f->machine.uc_link = NULL;
    makecontext(&f->machine, body, 0);
    return f;
}
/* Called only on the owner thread outside collection, never from a GC callback.
 * Only explicitly owned, expired parked bodies are eligible. Their externally
 * owned completion signals are neither completed nor released here. */
static void reap_expired(void) {
    if (!state.owned) return;
    for (unsigned i = 0; i < LIMIT; ++i) {
        fiber *f = &state.fibers[i];
        if (!f->mapping || !f->parked || !f->roots.expired) continue;
        if (!f->owner || f == state.current || f->active_owner || f->active_callback) abort();
        neoclr_roots_discard_v1(&f->roots);
        if (neoclr_gc_host_root_release_v1(state.context, f->callback)) abort();
        operation *op = find(f->operation);
        if (op) memset(op, 0, sizeof(*op));
        unmap_stack(f->mapping, f->mapping_bytes);
        --state.owned;
        memset(f, 0, sizeof(*f));
    }
    for (unsigned i = 0; i < LIMIT; ++i) {
        operation *op = &state.operations[i];
        if (!op->id || !op->owner) continue;
        void *owner = NULL;
        if (neoclr_gc_host_root_read_v1(state.context, op->owner, &owner)) {
            if (neoclr_gc_host_root_release_v1(state.context, op->owner)) abort();
            memset(op, 0, sizeof(*op));
            --state.owned;
        }
    }
}
int32_t neoclr_runtime_async_reap_v1(neoclr_aot_context *c) {
    if (!c || state.context != c || state.current || state.pumping || neoclr_root_probe_head_v1()) return 3;
    reap_expired();
    return 0;
}
static int resume(fiber *f) {
    if (f->roots.expired) return 3;
    if (f->owner) {
        void *callback = NULL;
        if (neoclr_gc_host_root_read_v1(state.context, f->callback, &callback)) return 3;
        int status = neoclr_gc_host_root_create_v1(state.context, f->owner, &f->active_owner);
        if (status) return status;
        status = neoclr_gc_host_root_create_v1(state.context, callback, &f->active_callback);
        if (status) {
            if (neoclr_gc_host_root_release_v1(state.context, f->active_owner)) abort();
            f->active_owner = 0;
            return status;
        }
    }
    if (f->ready_owner) {
        if (neoclr_gc_host_root_release_v1(state.context, f->ready_owner)) abort();
        f->ready_owner = 0;
    }
    ++statistics.resumes;
    ucontext_t caller;
    neoclr_parked_roots caller_roots = {0};
    fiber *parent = state.current;
    neoclr_roots_park_v1(&caller_roots);
    if (f->parked) { neoclr_roots_resume_v1(&f->roots); f->parked = 0; }
    state.current = f;
    f->caller = &caller;
    bounds(f);
    if (swapcontext(&caller, &f->machine)) abort();
    state.current = parent;
    bounds(parent);
    neoclr_roots_resume_v1(&caller_roots);
    int status = f->status;
    if (f->done) {
        if (f->owner) --state.owned;
        if (f->managed) memset(find(f->operation), 0, sizeof(operation));
        neoclr_roots_discard_v1(&f->roots);
        /* The callback root and parked root chain have ended. Cache only the
         * raw guarded mapping, never a machine context or managed ownership. */
#if NEOCLR_ASYNC_STACK_CACHE_LIMIT > 0
        if (state.cached < NEOCLR_ASYNC_STACK_CACHE_LIMIT)
            state.cache[state.cached++] = (cached_stack){f->mapping, f->mapping_bytes, f->lower};
        else
#endif
            unmap_stack(f->mapping, f->mapping_bytes);
        memset(f, 0, sizeof(*f));
    }
    return status;
}
int32_t neoclr_runtime_async_create_v1(neoclr_aot_context *c, uint64_t *out) {
    if (admit(c) || !out || next_id == INT64_MAX) return 3;
    for (unsigned i = 0; i < LIMIT; ++i) if (!state.operations[i].id) {
        state.operations[i] = (operation){.id = ++next_id};
        *out = next_id;
        return 0;
    }
    return 5;
}
static int32_t start(void *owner, void *callback, neoclr_aot_context *c,
    uint64_t *out, neoclr_async_callback invoke, int managed) {
    if (admit(c) || !out || !invoke || (state.invoke && state.invoke != invoke)) return 3;
    state.invoke = invoke;
    uint64_t root = 0, op = 0;
    int status = neoclr_runtime_async_create_v1(c, &op);
    if (status) return status;
    status = owner ? neoclr_gc_host_root_create_owned_v1(c, owner, callback, &root)
        : neoclr_gc_host_root_create_v1(c, callback, &root);
    if (status) { find(op)->id = 0; return status; }
    fiber *f = allocate_fiber(root, op, 0);
    if (!f) { neoclr_gc_host_root_release_v1(c, root); find(op)->id = 0; return 5; }
    f->owner = owner;
    f->managed = managed;
    if (owner) ++state.owned;
    find(op)->body = 1;
    if (!owner) *out = op; /* Preserve the original strong-entry publication point. */
    status = resume(f);
    if (status && f->mapping && !f->parked) {
        /* Admission failed before switching stacks (for example, handle quota).
         * No callback ran, so roll back instead of leaving an unrooted owner. */
        if (neoclr_gc_host_root_release_v1(c, f->callback)) abort();
        unmap_stack(f->mapping, f->mapping_bytes);
        if (f->owner) --state.owned;
        memset(f, 0, sizeof(*f));
        find(op)->id = 0;
        return status;
    }
    *out = op;
    return status;
}
int32_t neoclr_runtime_async_start_v1(void *callback, neoclr_aot_context *c,
    uint64_t *out, neoclr_async_callback invoke) {
    return start(NULL, callback, c, out, invoke, 0);
}
int32_t neoclr_runtime_async_start_owned_v1(void *owner, void *callback, neoclr_aot_context *c,
    uint64_t *out, neoclr_async_callback invoke) {
    if (!owner) return 3;
    return start(owner, callback, c, out, invoke, 0);
}
int32_t neoclr_runtime_async_startmanaged_v1(void *owner, void *callback, neoclr_aot_context *c,
    uint64_t *out, neoclr_async_callback invoke) {
    if (!owner) return 3;
    return start(owner, callback, c, out, invoke, 1);
}
int32_t neoclr_runtime_async_currentowner_v1(neoclr_aot_context *c, void **out) {
    if (admit(c) || !out) return 3;
    *out = state.current ? state.current->owner : NULL;
    return 0;
}
int32_t neoclr_runtime_async_createowned_v1(void *owner, neoclr_aot_context *c, uint64_t *out) {
    if (admit(c) || !owner || !out) return 3;
    uint64_t id, root;
    int status = neoclr_runtime_async_create_v1(c, &id);
    if (status) return status;
    status = neoclr_gc_host_root_create_owned_v1(c, owner, owner, &root);
    if (status) { memset(find(id), 0, sizeof(operation)); return status; }
    find(id)->owner = root;
    ++state.owned;
    *out = id;
    return 0;
}
int32_t neoclr_runtime_async_completeunit_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    if (admit(c) || !out) return 3;
    operation *op = find(id);
    *out = op && !op->complete && !op->body;
    if (*out) {
        // The notification may release its last managed owner before dispatch.
        // Runnable bodies must retain their state until active roots take over.
        for (unsigned i = 0; i < LIMIT; ++i) {
            fiber *f = &state.fibers[i];
            if (!f->mapping || f->waiting != id || !f->owner || f->roots.expired) continue;
            int status = neoclr_gc_host_root_create_v1(c, f->owner, &f->ready_owner);
            if (status) {
                for (unsigned j = 0; j < i; ++j) {
                    fiber *prior = &state.fibers[j];
                    if (prior->waiting == id && prior->ready_owner) {
                        if (neoclr_gc_host_root_release_v1(c, prior->ready_owner)) abort();
                        prior->ready_owner = 0;
                    }
                }
                *out = 0;
                return status;
            }
        }
        op->complete = 1;
    }
    return 0;
}
int32_t neoclr_runtime_async_iscompleted_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    if (admit(c) || !out || !find(id)) return 3;
    *out = find(id)->complete;
    return 0;
}
int32_t neoclr_runtime_async_release_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    if (admit(c) || !out) return 3;
    *out = 0;
    if (!find(id) || !find(id)->complete) return 0;
    for (unsigned i = 0; i < LIMIT; ++i)
        if (state.fibers[i].mapping && (state.fibers[i].waiting == id || state.fibers[i].operation == id)) return 0;
    if (find(id)->owner) {
        if (neoclr_gc_host_root_release_v1(c, find(id)->owner)) abort();
        --state.owned;
    }
    memset(find(id), 0, sizeof(operation));
    *out = 1;
    return 0;
}
int32_t neoclr_runtime_async_waitcompletion_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    if (admit(c) || !out || !find(id)) return 3;
    if (!find(id)->complete) {
        fiber *f = state.current;
        if (!f || f->operation == id) return 3;
        f->waiting = id;
        yield();
        f->waiting = 0;
        if (c->fault.code || !find(id) || !find(id)->complete) return 3;
    }
    *out = 1;
    return 0;
}
int32_t neoclr_runtime_async_awaitunit_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    (void)out;
    int32_t completed;
    return neoclr_runtime_async_waitcompletion_v1(id, c, &completed);
}
int32_t neoclr_runtime_async_pump_v1(neoclr_aot_context *c, neoclr_async_callback invoke, neoclr_async_drain drain) {
    if (admit(c) || !invoke || !drain || state.current || state.pumping) return 3;
    state.invoke = invoke; state.drain = drain; state.pumping = 1;
    neoclr_parked_roots entry = {0};
    neoclr_roots_park_v1(&entry);
    int status = 0;
    while (!status) {
        reap_expired();
        int draining = 0;
        for (unsigned i = 0; i < LIMIT; ++i) {
            fiber *f = &state.fibers[i];
            if (!f->mapping) continue;
            operation *op = find(f->waiting);
            if (op && op->complete) {
                status = resume(f);
                reap_expired();
            }
            if (status) break;
            draining |= f->mapping && f->drain;
        }
        if (status) break;
        if (!draining) {
            fiber *f = allocate_fiber(0, 0, 1);
            if (!f) { status = 5; break; }
            status = resume(f);
            if (status) break;
        }
        reap_expired();
        int waiting = 0, ready = 0;
        for (unsigned i = 0; i < LIMIT; ++i) if (state.fibers[i].mapping) {
            waiting = 1;
            operation *op = find(state.fibers[i].waiting);
            ready |= op && op->complete;
        }
        /* Ready continuations must not starve scheduler deliveries: those may
         * release completed body operations and keep bounded ownership reusable. */
        uint64_t delivery = 0;
        int progress = neoclr_scheduler_acquire_v1(c, &delivery, NULL);
        if (progress == NEOCLR_SCHEDULER_READY) {
            /* Transfer dispatch ownership into an independently parked fiber. */
            void *callback = NULL;
            uint64_t root = 0;
            if (neoclr_gc_host_root_read_v1(c, delivery, &callback) ||
                neoclr_gc_host_root_create_v1(c, callback, &root)) { status = 3; break; }
            if (neoclr_scheduler_complete_v1(c, delivery, NULL)) abort();
            fiber *f = allocate_fiber(root, 0, 0);
            if (!f) { neoclr_gc_host_root_release_v1(c, root); status = 5; break; }
            status = resume(f);
        } else if (progress == NEOCLR_SCHEDULER_PENDING) {
            if (!ready) neoclr_os_pause();
        }
        else if (progress == NEOCLR_SCHEDULER_IDLE) {
            if (ready) continue;
            status = waiting ? 3 : 0;
            break;
        } else status = 3;
    }
    neoclr_roots_resume_v1(&entry);
    state.pumping = 0;
    return status;
}
int32_t neoclr_runtime_async_shutdown_v1(neoclr_aot_context *c) {
    if (!state.context) return 0;
    if (state.context != c || state.current || state.pumping || neoclr_root_probe_head_v1()) return 3;
    reap_expired();
    int leaked = 0;
    for (unsigned i = 0; i < LIMIT; ++i) {
        fiber *f = &state.fibers[i];
        if (f->mapping) {
            leaked = 1;
            if (f->parked) neoclr_roots_discard_v1(&f->roots);
            if (f->callback && neoclr_gc_host_root_release_v1(c, f->callback)) abort();
            if (f->ready_owner && neoclr_gc_host_root_release_v1(c, f->ready_owner)) abort();
            unmap_stack(f->mapping, f->mapping_bytes);
        }
        leaked |= state.operations[i].id != 0;
        if (state.operations[i].owner && neoclr_gc_host_root_release_v1(c, state.operations[i].owner)) abort();
    }
    for (unsigned i = 0; i < state.cached; ++i)
        unmap_stack(state.cache[i].mapping, state.cache[i].bytes);
    if (statistics.reserved_bytes) abort();
    memset(&state, 0, sizeof(state));
    bounds(NULL);
    return leaked && !c->fault.code ? 3 : 0;
}
