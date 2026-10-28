#ifndef NEOCLR_TRACE_SESSION_H
#define NEOCLR_TRACE_SESSION_H
#include <stddef.h>
#include <stdint.h>
/* Private POSIX diagnostic. Link session.c only in instrumented builds.
 * Create/close/view/destroy belong to the creating thread. Each producer explicitly
 * attaches once to a distinct lane, then detaches before close. Lane reuse, live
 * readers, signal reentry and concurrent teardown are forbidden. Session must
 * outlive every prospective attach, including threads not yet started. */
typedef struct neoclr_trace_session neoclr_trace_session;
typedef struct {
    uint64_t ns, sequence, source, subject, related;
    uint32_t domain, kind, detail;
} neoclr_trace_event;
typedef struct {
    const neoclr_trace_event *events;
    size_t count, capacity;
    uint64_t session, origin_ns, attempted, dropped;
    unsigned role;
} neoclr_trace_view;
enum { NEOCLR_TRACE_ASYNC=2, NEOCLR_TRACE_GC=3 };
enum { NEOCLR_TRACE_GC_REQUEST_BEGIN=100, NEOCLR_TRACE_GC_REQUEST_END=101,
       NEOCLR_TRACE_GC_EPOCH_BEGIN=110, NEOCLR_TRACE_GC_EPOCH_END=111,
       NEOCLR_TRACE_GC_EPOCH_CANCEL=112 };
neoclr_trace_session *neoclr_trace_create(unsigned lanes, size_t capacity);
int neoclr_trace_attach(neoclr_trace_session *, unsigned lane, unsigned role);
int neoclr_trace_detach(void);
int neoclr_trace_close(neoclr_trace_session *);
int neoclr_trace_view_lane(neoclr_trace_session *, unsigned lane, neoclr_trace_view *);
int neoclr_trace_destroy(neoclr_trace_session *);
/* Single-writer TLS hot path; fixed scalar payload, no allocation/locks/I/O.
 * Source/subject/related are diagnostic IDs, never managed/native pointers. */
void neoclr_trace_record(unsigned domain, unsigned kind, uint64_t source,
    uint64_t subject, uint64_t related, unsigned detail);
#ifdef NEOCLR_EXECUTION_TRACE
#define NEOCLR_TRACE(domain, kind, source, subject, related, detail) \
    neoclr_trace_record(domain, kind, source, subject, related, detail)
#else
#define NEOCLR_TRACE(domain, kind, source, subject, related, detail) ((void)0)
#endif
#endif
