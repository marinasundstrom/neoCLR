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
typedef struct { uint64_t id; int complete, body; } operation;
typedef struct fiber {
    ucontext_t machine;
    neoclr_parked_roots roots;
    void *mapping;
    size_t mapping_bytes;
    uintptr_t lower, upper;
    uint64_t callback, operation, waiting;
    int done, status, drain, parked;
    ucontext_t *caller;
} fiber;
static _Thread_local struct {
    neoclr_aot_context *context;
    neoclr_async_callback invoke;
    neoclr_async_drain drain;
    operation operations[LIMIT];
    fiber fibers[LIMIT];
    fiber *current;
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
    neoclr_roots_park_v1(&f->roots);
    f->parked = 1;
    if (swapcontext(&f->machine, f->caller)) abort();
}
static void body(void) {
    fiber *f = state.current;
    f->status = f->drain ? state.drain(state.context)
        : state.invoke(f->callback, state.context);
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
    size_t page = (size_t)sysconf(_SC_PAGESIZE);
    size_t bytes = STACK_BYTES + 2 * page;
    void *mapping = mmap(NULL, bytes, PROT_NONE, MAP_PRIVATE | MAP_ANON, -1, 0);
    if (mapping == MAP_FAILED) return NULL;
    void *stack = (char *)mapping + page;
    if (mprotect(stack, STACK_BYTES, PROT_READ | PROT_WRITE)) { munmap(mapping, bytes); return NULL; }
    memset(f, 0, sizeof(*f));
    f->mapping = mapping; f->mapping_bytes = bytes;
    f->lower = (uintptr_t)stack; f->upper = f->lower + STACK_BYTES;
    f->callback = callback; f->operation = op; f->drain = drain;
    if (getcontext(&f->machine)) abort();
    f->machine.uc_stack.ss_sp = stack;
    f->machine.uc_stack.ss_size = STACK_BYTES;
    f->machine.uc_link = NULL;
    makecontext(&f->machine, body, 0);
    return f;
}
static int resume(fiber *f) {
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
        neoclr_roots_discard_v1(&f->roots);
        if (munmap(f->mapping, f->mapping_bytes)) abort();
        memset(f, 0, sizeof(*f));
    }
    return status;
}
int32_t neoclr_runtime_async_create_v1(neoclr_aot_context *c, uint64_t *out) {
    if (admit(c) || !out || next_id == INT64_MAX) return 3;
    for (unsigned i = 0; i < LIMIT; ++i) if (!state.operations[i].id) {
        state.operations[i] = (operation){++next_id, 0, 0};
        *out = next_id;
        return 0;
    }
    return 5;
}
int32_t neoclr_runtime_async_start_v1(void *callback, neoclr_aot_context *c,
    uint64_t *out, neoclr_async_callback invoke) {
    if (admit(c) || !out || !invoke || (state.invoke && state.invoke != invoke)) return 3;
    state.invoke = invoke;
    uint64_t root = 0, op = 0;
    int status = neoclr_runtime_async_create_v1(c, &op);
    if (status) return status;
    if (neoclr_gc_host_root_create_v1(c, callback, &root)) { find(op)->id = 0; return 3; }
    fiber *f = allocate_fiber(root, op, 0);
    if (!f) { neoclr_gc_host_root_release_v1(c, root); find(op)->id = 0; return 5; }
    find(op)->body = 1;
    *out = op;
    return resume(f);
}
int32_t neoclr_runtime_async_completeunit_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    if (admit(c) || !out) return 3;
    operation *op = find(id);
    *out = op && !op->complete && !op->body;
    if (*out) op->complete = 1;
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
    find(id)->id = 0;
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
        int draining = 0;
        for (unsigned i = 0; i < LIMIT; ++i) {
            fiber *f = &state.fibers[i];
            if (!f->mapping) continue;
            operation *op = find(f->waiting);
            if (op && op->complete) status = resume(f);
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
    int leaked = 0;
    for (unsigned i = 0; i < LIMIT; ++i) {
        fiber *f = &state.fibers[i];
        if (f->mapping) {
            leaked = 1;
            if (f->parked) neoclr_roots_discard_v1(&f->roots);
            if (f->callback && neoclr_gc_host_root_release_v1(c, f->callback)) abort();
            if (munmap(f->mapping, f->mapping_bytes)) abort();
        }
        leaked |= state.operations[i].id != 0;
    }
    memset(&state, 0, sizeof(state));
    bounds(NULL);
    return leaked && !c->fault.code ? 3 : 0;
}

int32_t neoclr_runtime_async_gccollect_v1(neoclr_aot_context *c, void *out) {
    (void)out;
    return neoclr_gc_collect_v1(c, neoclr_root_probe_head_v1());
}
