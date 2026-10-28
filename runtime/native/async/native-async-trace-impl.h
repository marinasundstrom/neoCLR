#ifdef NEOCLR_ASYNC_TRACE
/* Included after owner state. No callbacks, allocation or I/O during emission. */
#ifdef _WIN32
#error Async tracing is currently qualified only on the macOS ARM64 carrier
#endif
#include "native-async-trace.h"
#include "../trace/session.h"
#include <time.h>
#ifdef NEOCLR_EXECUTION_TRACE
static _Thread_local uint64_t async_trace_session_id;
#endif
static _Thread_local struct {
    neoclr_async_trace_event *events;
    size_t count, capacity;
    uint64_t origin, attempted, dropped, contexts;
    int active;
} async_trace;
static uint64_t async_trace_now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) return 0;
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
int neoclr_async_trace_begin(size_t capacity) {
    if (state.context || async_trace.events || !capacity || capacity > 65536) return 0;
#ifdef NEOCLR_EXECUTION_TRACE
    if (async_trace_session_id == UINT64_MAX) return 0;
#endif
    uint64_t origin = async_trace_now();
    if (!origin) return 0;
    neoclr_async_trace_event *events = calloc(capacity, sizeof(*events));
    if (!events) return 0;
    async_trace.events = events; async_trace.capacity = capacity;
    async_trace.origin = origin; async_trace.active = 1;
#ifdef NEOCLR_EXECUTION_TRACE
    ++async_trace_session_id;
#endif
    return 1;
}
int neoclr_async_trace_finish(neoclr_async_trace_view *out) {
    if (!out || state.context || !async_trace.active) return 0;
    async_trace.active = 0;
    *out = (neoclr_async_trace_view){async_trace.events, async_trace.count,
        async_trace.capacity, async_trace.origin, async_trace.attempted, async_trace.dropped};
    return 1;
}
int neoclr_async_trace_dispose(void) {
    if (state.context || async_trace.active) return 0;
    free(async_trace.events);
    memset(&async_trace, 0, sizeof(async_trace));
    return 1;
}
static void async_trace_emit(unsigned kind, uint64_t context, uint64_t related, unsigned detail) {
    if (!async_trace.active) return;
    NEOCLR_TRACE(NEOCLR_TRACE_ASYNC, kind, async_trace_session_id, context, related, detail);
    uint64_t sequence = ++async_trace.attempted;
    if (async_trace.count == async_trace.capacity) { ++async_trace.dropped; return; }
    uint64_t ns = async_trace_now();
    if (!ns) { ++async_trace.dropped; return; }
    async_trace.events[async_trace.count++] = (neoclr_async_trace_event){ns,sequence,context,related,kind,detail};
}
static void async_trace_enter(fiber *f, fiber *parent) {
    if (!async_trace.active) return;
    if (!f->trace_id) f->trace_id = ++async_trace.contexts;
    async_trace_emit(NEOCLR_ASYNC_TRACE_ENTER, f->trace_id, parent ? parent->trace_id : 0, 0);
}
#define ASYNC_TRACE(kind, f, related, detail) async_trace_emit(kind, (f) ? (f)->trace_id : 0, related, detail)
#else
#define async_trace_enter(f, parent) ((void)0)
#define ASYNC_TRACE(kind, f, related, detail) ((void)0)
#endif
