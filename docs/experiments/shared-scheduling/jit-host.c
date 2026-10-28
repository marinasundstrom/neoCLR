#include "../../../runtime/native/trace/clock.h"
#include <sys/resource.h>
#include <time.h>
#ifdef NEOCLR_EXECUTION_TRACE
#include "../../../runtime/native/trace/session.h"
#include "native-async-trace.h"
#endif
static void report_profile(struct rusage *, struct rusage *, struct timespec *, struct timespec *);

#ifdef NEOCLR_MANAGED_TLS
#include "tls-managed.h"
#endif
/* Private JIT entry host. Managed awaits drain within this call; no state escapes. */
#include "native-gc.h"
#include "console.h"
#ifdef NEOCLR_RUNTIME_ASYNC
#include "native-runtime.h"
#endif
#include "task-queue.h"
#include "socket-listener.h"
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <fcntl.h>
#include <io.h>
typedef wchar_t entry_char;
#else
typedef char entry_char;
#endif

#ifndef NEOCLR_HTTP_HEAP_BYTES
#define NEOCLR_HTTP_HEAP_BYTES (1024 * 1024)
#endif
_Static_assert(NEOCLR_HTTP_HEAP_BYTES > 0, "positive JIT heap budget");

typedef int32_t (*entry_v4)(int32_t, int32_t *, neoclr_aot_context *);
typedef int32_t (*entry_args)(int32_t, int32_t *, neoclr_aot_context *, const entry_char **);

/* Timer operations currently share the native I/O table. The generated entry
   consumes each delivered callback before polling again, retaining its root. */
static int32_t poll_entry(neoclr_aot_context *context, uint64_t *callback,
    const neoclr_probe_frame *boundary, void *state) {
    (void)state;
    int32_t progress = neoclr_scheduler_acquire_v1(context, callback, boundary);
    if (progress == NEOCLR_SCHEDULER_PENDING) neoclr_scheduler_wait_v1(context);
    return progress;
}

static int32_t run_entry(const void *entry, int32_t with_args, int32_t argc, const entry_char **argv) {
    if (!entry || argc < 1 || !argv || (with_args != 0 && with_args != 1)) return 2;
    const size_t heap_bytes = NEOCLR_HTTP_HEAP_BYTES;
    unsigned char *heap = calloc(1, heap_bytes);
    if (!heap) return 2;
    neoclr_aot_context context = {.text = {heap, heap_bytes, 0}};
    if (neoclr_gc_heap_acquire_v1(&context.text)) { free(heap); return 2; }
    neoclr_socket_scope operations;
    if (neoclr_socket_scope_enter_v1(&operations, &context)) { if (neoclr_gc_heap_release_v1(&context.text)) abort(); free(heap); return 2; }
    neoclr_task_scope tasks;
    if (neoclr_task_scope_enter_v1(&tasks, &context)) {
        if (neoclr_socket_scope_leave_v1(&operations)) abort();
        if (neoclr_gc_heap_release_v1(&context.text)) abort(); free(heap);
        return 2;
    }
    tasks.poll = poll_entry;
    tasks.complete = neoclr_scheduler_complete_v1;
    neoclr_terminal_scope_v1 terminal = {0};
    if (neoclr_terminal_scope_enter_v1(&terminal, &context)) {
        if (neoclr_socket_scope_leave_v1(&operations) || neoclr_task_scope_leave_v1(&tasks)) abort();
        if (neoclr_gc_heap_release_v1(&context.text)) abort(); free(heap);
        return 2;
    }
    int32_t result = 0;
    entry_args args_function;
    entry_v4 plain_function;
    _Static_assert(sizeof(args_function) == sizeof(entry), "native entry pointer size");
    _Static_assert(sizeof(plain_function) == sizeof(entry), "native entry pointer size");
    memcpy(&args_function, &entry, sizeof(entry));
    memcpy(&plain_function, &entry, sizeof(entry));
    int32_t status = with_args
        ? args_function(argc, &result, &context, argv)
        : plain_function(0, &result, &context);
#ifdef NEOCLR_RUNTIME_ASYNC
    if (neoclr_runtime_async_shutdown_v1(&context)) abort();
#endif
    int terminal_error = neoclr_terminal_scope_leave_v1(&terminal) != 0;
    int render_error = status && neoclr_aot_render_fault(stderr, &context.fault);
    /* Cleanup must not return to a caller about to unload JIT code while any
       frame/handle still owns its descriptors. Treat a broken invariant as fatal. */
    if (neoclr_root_probe_head_v1() != NULL ||
        neoclr_socket_scope_leave_v1(&operations) ||
        neoclr_task_scope_leave_v1(&tasks)) abort();
#ifdef NEOCLR_MANAGED_TLS
    neoclr_tls_managed_dispose_v1(&context);
#endif
    if (neoclr_gc_owned_roots_check_v1(&context, NULL, 0) ||
        neoclr_gc_collect_v1(&context, NULL)) abort();
#ifdef NEOCLR_RUNTIME_ASYNC
    memset(&context.fault, 0, sizeof(context.fault));
    if (neoclr_gc_collect_v1(&context, NULL) || context.text.used) abort();
#endif
    if (neoclr_gc_heap_release_v1(&context.text)) abort(); free(heap);
    return render_error || terminal_error ? 2 : status ? 1 : result;
}

/* Windows CRT translation must not change the platform's UTF-8 byte contract.
   Restore the caller's modes after this invocation, including admission failures. */
int32_t neoclr_jit_run_v1(const void *entry, int32_t with_args, int32_t argc, const entry_char **argv) {
#ifdef _WIN32
    FILE *streams[] = {stdin, stdout, stderr};
    int modes[3];
    for (int i = 0; i < 3; ++i) {
        modes[i] = _setmode(_fileno(streams[i]), _O_BINARY);
        if (modes[i] == -1) {
            while (i > 0) { --i; (void)_setmode(_fileno(streams[i]), modes[i]); }
            return 2;
        }
    }
#endif
    struct rusage before, after;
    struct timespec start, end;
#ifdef NEOCLR_EXECUTION_TRACE
    neoclr_trace_session *trace = NULL;
    if (getenv("NEOCLR_HTTP_TRACE")) {
        trace = neoclr_trace_create(1, 65536);
        if (!trace || !neoclr_trace_attach(trace, 0, 1) || !neoclr_async_trace_begin(65536)) abort();
    }
#endif
    getrusage(RUSAGE_SELF, &before);
    clock_gettime(CLOCK_MONOTONIC, &start);
    int32_t status = run_entry(entry, with_args, argc, argv);
    clock_gettime(CLOCK_MONOTONIC, &end);
    getrusage(RUSAGE_SELF, &after);
    report_profile(&before, &after, &start, &end);
#ifdef NEOCLR_EXECUTION_TRACE
    if (trace) {
        neoclr_async_trace_view async;
        if (!neoclr_async_trace_finish(&async) || !neoclr_async_trace_dispose() ||
            !neoclr_trace_detach() || !neoclr_trace_close(trace)) abort();
        neoclr_trace_view view;
        if (!neoclr_trace_view_lane(trace, 0, &view)) abort();
        fprintf(stderr, "TRACE {\"schema\":\"session-1\",\"clock\":\"" NEOCLR_TRACE_CLOCK_NAME "\","
            "\"origin_ns\":%llu,\"attempted\":%llu,\"dropped\":%llu,\"capacity\":%zu,"
            "\"asyncDropped\":%llu,\"recordBytes\":%zu,\"asyncRecordBytes\":%zu,\"events\":[",
            (unsigned long long)view.origin_ns, (unsigned long long)view.attempted,
            (unsigned long long)view.dropped, view.capacity, (unsigned long long)async.dropped,
            sizeof(neoclr_trace_event), sizeof(neoclr_async_trace_event));
        for (size_t i=0; i<view.count; i++) {
            const neoclr_trace_event *v = &view.events[i];
            fprintf(stderr, "%s[%llu,%llu,%u,%u,%llu,%llu,%llu,%u]", i?",":"",
                (unsigned long long)v->ns, (unsigned long long)v->sequence, v->domain, v->kind,
                (unsigned long long)v->source, (unsigned long long)v->subject,
                (unsigned long long)v->related, v->detail);
        }
        fputs("]}\n", stderr);
        if (!neoclr_trace_destroy(trace)) abort();
    }
#endif


#ifdef _WIN32
    for (int i = 0; i < 3; ++i) {
        if (i && fflush(streams[i])) status = 2;
        if (_setmode(_fileno(streams[i]), modes[i]) == -1) status = 2;
    }
#endif
    return status;
}

static double cpu(struct rusage *r) {
 return r->ru_utime.tv_sec + r->ru_stime.tv_sec +
        (r->ru_utime.tv_usec + r->ru_stime.tv_usec) / 1000000.0;
}
static void report_profile(struct rusage *a, struct rusage *b, struct timespec *s, struct timespec *e) {
 neoclr_async_statistics stacks = neoclr_runtime_async_statistics_v1();
 fprintf(stderr, "ASYNC %llu %llu %llu %llu %llu %llu\n",
  (unsigned long long)stacks.fibers, (unsigned long long)stacks.resumes,
  (unsigned long long)stacks.mappings, (unsigned long long)stacks.unmappings,
  (unsigned long long)stacks.peak_reserved_bytes, (unsigned long long)stacks.reserved_bytes);
 /* Process-level OS switches include JIT and marker helpers. They are not
    cooperative carrier switches and cannot be divided into a per-resume cost. */
 fprintf(stderr, "OS_SWITCHES %ld %ld\n", b->ru_nvcsw-a->ru_nvcsw, b->ru_nivcsw-a->ru_nivcsw);
#ifdef OWNER_PAUSE_PROFILE
 extern void experiment_owner_pauses_report(void);
 experiment_owner_pauses_report();
#endif
#ifdef BEGIN_PHASE_PROFILE
 extern void experiment_begin_phases_report(void);
 experiment_begin_phases_report();
#endif
#ifdef FINISH_PHASE_PROFILE
 extern void experiment_finish_phases_report(void);
 experiment_finish_phases_report();
#endif
#ifdef COLLECTION_PART_PROFILE
 extern void experiment_collection_parts_report(void);
 experiment_collection_parts_report();
#endif
 neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
 neoclr_gc_profile profile = neoclr_gc_profile_v1();
 neoclr_gc_lookup_profile lookup = neoclr_gc_lookup_profile_v1();
 fprintf(stderr, "LOOKUPS %llu %llu %llu %llu\n",
  (unsigned long long)lookup.base_calls, (unsigned long long)lookup.base_cache_hits,
  (unsigned long long)lookup.base_walks, (unsigned long long)lookup.base_spans);
 fprintf(stderr, "PROFILE %.9f %.9f %llu %llu %llu %llu %llu\n",
  cpu(b)-cpu(a), (e->tv_sec-s->tv_sec)+(e->tv_nsec-s->tv_nsec)/1000000000.0,
  (unsigned long long)stats.allocations, (unsigned long long)profile.allocated_bytes,
  (unsigned long long)stats.collections, (unsigned long long)profile.peak_live_bytes,
  (unsigned long long)profile.peak_arena_used);
#ifdef WORKLOAD_LIVE
#ifdef NEOCLR_STORE_SITES
 extern void experiment_store_sites_report(void);
 experiment_store_sites_report();
#endif
 extern void experiment_gc_candidate_counts(uint64_t *);
 extern void experiment_gc_store_counts(uint64_t *);
 extern void experiment_gc_store_lookup_counts(uint64_t *);
 uint64_t counts[6]; experiment_gc_candidate_counts(counts);
 fprintf(stderr, "CANDIDATE");
 for(unsigned i=0;i<6;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\n");
 experiment_gc_store_counts(counts);
 fprintf(stderr,"STORES");
 for(unsigned i=0;i<5;i++) fprintf(stderr," %llu",(unsigned long long)counts[i]);
 fprintf(stderr,"\n");
 experiment_gc_store_lookup_counts(counts);
 fprintf(stderr,"STORE_LOOKUPS %llu %llu\n",(unsigned long long)counts[0],(unsigned long long)counts[1]);
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
 extern void experiment_gc_index_counts(uint64_t *);
 experiment_gc_index_counts(counts);
 fprintf(stderr,"INDEX_METADATA %llu %llu %llu\n",(unsigned long long)counts[0],(unsigned long long)counts[1],(unsigned long long)counts[2]);
#endif
#ifdef NEOCLR_LIVE_WALK_PROFILE
 extern void experiment_gc_store_walk_counts(uint64_t *);
 experiment_gc_store_walk_counts(counts);
 fprintf(stderr,"STORE_WALKS %llu %llu %llu\n",(unsigned long long)counts[0],(unsigned long long)counts[1],(unsigned long long)counts[2]);
#endif
#endif
}
