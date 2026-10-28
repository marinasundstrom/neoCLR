#ifndef NEOCLR_NATIVE_ASYNC_TRACE_H
#define NEOCLR_NATIVE_ASYNC_TRACE_H
#include <stdint.h>
#include <stddef.h>
/* Private macOS ARM64 diagnostic, requires NEOCLR_ASYNC_TRACE. All calls and
 * records belong to the current owner thread. Begin before async admission;
 * finish only after shutdown. View borrows storage until dispose. No live read,
 * cross-thread access, signal reentry or managed references. Zero capacity is
 * rejected; simply do not begin to disable recording in an instrumented build.
 * Normal builds omit the recorder, hooks and extra fiber identity field. */
enum {
    NEOCLR_ASYNC_TRACE_CREATE = 1, NEOCLR_ASYNC_TRACE_COMPLETE,
    NEOCLR_ASYNC_TRACE_ENTER, NEOCLR_ASYNC_TRACE_LEAVE,
    NEOCLR_ASYNC_TRACE_PARK, NEOCLR_ASYNC_TRACE_FINISH,
    NEOCLR_ASYNC_TRACE_ABANDON, NEOCLR_ASYNC_TRACE_WAIT_BEGIN,
    NEOCLR_ASYNC_TRACE_WAIT_END
};
typedef struct {
    uint64_t ns, sequence, context, related;
    uint32_t kind, detail;
} neoclr_async_trace_event;
typedef struct {
    const neoclr_async_trace_event *events;
    size_t count, capacity;
    uint64_t origin_ns, attempted, dropped;
} neoclr_async_trace_view;
int neoclr_async_trace_begin(size_t capacity);
int neoclr_async_trace_finish(neoclr_async_trace_view *);
int neoclr_async_trace_dispose(void);
#endif
