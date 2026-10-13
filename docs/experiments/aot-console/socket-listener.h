#ifndef NEOCLR_AOT_SOCKET_LISTENER_H
#define NEOCLR_AOT_SOCKET_LISTENER_H
#include "socket-os.h"
#include "text-arena.h"
/* Private macOS/Windows socket experiment; GC builds additionally support polled accept/transfers. Host enters one stack-owned scope
 * per invocation and leaves it on EVERY terminal success/fault path before resetting
 * the context. Async work may outlive entry return inside that scope.
 * No same-context reentry or cross-thread scope use. LIFO nested distinct contexts
 * are supported. Closing the scope releases sockets even when guest code faults.
 * Handles are opaque, process-unique positive Int64s, never file descriptors.
 * OS resources are separate from the managed heap's byte budget. */
typedef struct neoclr_dns_work neoclr_dns_work;
typedef struct neoclr_runnable neoclr_runnable;
typedef struct neoclr_socket_scope {
    neoclr_aot_context *context;
    struct neoclr_socket_scope *previous;
    struct { uint64_t id; neoclr_socket_descriptor descriptor; int listener;
#ifdef NEOCLR_HTTPS_SERVER
        void *tls;
#endif
    } slots[64];
#ifdef NEOCLR_HTTPS_SERVER
    void *tls_config;
#endif
    /* Private operations: state 1 pending, 2 ready, 3 delivered; kind 1 accept,
     * 2 receive, 3 send, 4 connect, 5 DNS, 6 one-shot timer, 7 submitted runnable, 8 dormant registration. Transfer snapshots have a separate bounded native budget. */
    struct { uint64_t id, listener, callback, tag, value, buffer_root, deadline;
        uint32_t state, kind, offset, count; unsigned char *buffer;
        neoclr_dns_work *dns;
        neoclr_socket_descriptor connecting;
        uint32_t addresses[16], address_count, next_address;
        uint16_t port;
        uint64_t attempt_deadline;
    } operations[64];
    neoclr_runnable *runnable_head, *runnable_tail;
    uint64_t runnable_delivery; /* Detached submission root until acquire/next poll. */
    unsigned prefer_runnable;
    uint64_t timer_delivery; /* One-shot delivery root, retained until next poll/leave. */
    uint64_t scheduler_destination; /* Lazy, never-reused invocation identity. */
    uint64_t scheduler_delivery; /* Independent dispatch root, released by completion/leave. */
    const struct neoclr_probe_frame *scheduler_boundary;
    uint64_t transfer_bytes;
    unsigned poll_cursor;
} neoclr_socket_scope;
int32_t neoclr_socket_scope_enter_v1(neoclr_socket_scope *scope, neoclr_aot_context *context);
int32_t neoclr_socket_scope_leave_v1(neoclr_socket_scope *scope);
/* Status 0 publishes erased Int64 handle / Int32 port / Void, or Byte SocketError.
 * Invalid native arguments or missing scope fault with status 3, no publication.
 * Services neither allocate managed storage nor collect. Async submission adds a
 * strong root in GC builds. Transfer scratch uses bounded native allocations;
 * DNS/connect use bounded host work and deferred completion. */
int32_t neoclr_socket_listen_v1(const neoclr_aot_text *address, int32_t port, int32_t backlog,
                              neoclr_aot_context *context, void *output);
int32_t neoclr_socket_local_port_v1(uint64_t handle, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_close_v1(uint64_t handle, neoclr_aot_context *context, void *output);
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
/* Private one-shot timers share the bounded host operation table. Delay is 0 to
 * INT32_MAX milliseconds; cancellation returns false for stale/delivered handles.
 * Timer poll roots survive until the NEXT poll (or scope leave). Install/invoke
 * the returned callback before polling again. No public Timer API is implied. */
int32_t neoclr_timer_start_v1(int32_t, void *, neoclr_aot_context *, int64_t *);
int32_t neoclr_timer_cancel_v1(uint64_t, neoclr_aot_context *, int32_t *);
/* Callback must be a verified fn<Void> descriptor from this image/context. */
int32_t neoclr_socket_accept_v1(uint64_t listener, void *callback, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_connect_result_v1(uint64_t operation, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_cancel_v1(uint64_t operation, neoclr_aot_context *context, int32_t *output);
int32_t neoclr_socket_receive_v1(uint64_t socket, void *buffer, int32_t offset, int32_t count,
    void *callback, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_send_v1(uint64_t socket, void *buffer, int32_t offset, int32_t count,
    void *callback, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_transfer_result_v1(uint64_t operation, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_deadline_after_v1(int32_t milliseconds, neoclr_aot_context *context, int64_t *output);
int32_t neoclr_socket_deadline_expired_v1(int64_t stamp, neoclr_aot_context *context, int32_t *output);
int32_t neoclr_socket_receive_until_v1(uint64_t socket, void *array, int32_t offset, int32_t count,
    int64_t stamp, void *callback, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_send_until_v1(uint64_t socket, void *array, int32_t offset, int32_t count,
    int64_t stamp, void *callback, neoclr_aot_context *context, void *output);
/* Client services share the owner-thread operation table. DNS workers never own
 * managed pointers; result snapshots use private erased tag 7 (traced String[]). */
int32_t neoclr_dns_lookup_v1(const neoclr_aot_text *, void *, neoclr_aot_context *, void *);
int32_t neoclr_dns_lookup_until_v1(const neoclr_aot_text *, int64_t, void *, neoclr_aot_context *, void *);
int32_t neoclr_dns_cancel_v1(uint64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_dns_result_v1(uint64_t, neoclr_aot_context *, void *);
int32_t neoclr_dns_addresses_v1(uint32_t, uint64_t, neoclr_aot_context *, void **);
int32_t neoclr_socket_connect_v1(const neoclr_aot_text *, int32_t, void *, neoclr_aot_context *, void *);
int32_t neoclr_socket_connect_addresses_v1(const void *, int32_t, void *, neoclr_aot_context *, void *);
int32_t neoclr_socket_connect_addresses_until_v1(const void *, int32_t, int64_t, void *, neoclr_aot_context *, void *);
/* Private native scheduler step shared by JIT and AOT hosts. A NULL boundary
 * requires quiescence; otherwise it must be the exact published top frame.
 * Does not wait or invoke guest code. READY transfers the existing callback root
 * under the poll lifetime below; the host must dispatch it before the next step.
 * IDLE/PENDING/error leave the callback output unchanged. Delivered operations
 * still awaiting result consumption keep the scheduler pending. */
enum { NEOCLR_SCHEDULER_IDLE = 0, NEOCLR_SCHEDULER_READY = 1, NEOCLR_SCHEDULER_PENDING = 2 };
/* Destination capture identifies this owner-thread invocation. SubmitTo rejects
 * foreign/stale identities before retaining work; it does not route across scopes.
 * The token is not a GC root, scheduler customization or transferable capability. */
int32_t neoclr_scheduler_capture_v1(neoclr_aot_context *, int64_t *, const neoclr_probe_frame *);
int32_t neoclr_scheduler_submit_to_v1(neoclr_aot_context *, int64_t, uint64_t, const neoclr_probe_frame *);
/* Dormant registrations retain a callback and reserve capacity but do not keep
 * the invocation pending. Activate or cancel succeeds once, before dispatch owns
 * the callback. These managed bridges require the exact current guest boundary. */
int32_t neoclr_scheduler_register_managed_v1(int64_t, void *, neoclr_aot_context *, int64_t *);
int32_t neoclr_scheduler_activate_managed_v1(int64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_scheduler_cancel_managed_v1(int64_t, neoclr_aot_context *, int32_t *);
int32_t neoclr_scheduler_capture_managed_v1(neoclr_aot_context *, int64_t *);
int32_t neoclr_scheduler_submit_to_managed_v1(int64_t, void *, neoclr_aot_context *, int32_t *);
/* Private owner-thread host wait after PENDING. Windows observes socket readiness
 * with a requested <=1 ms timeout; other platforms retain their existing pause.
 * No callbacks, collection or scope mutation. Timers/DNS retain bounded polling. */
void neoclr_scheduler_wait_v1(neoclr_aot_context *);
int32_t neoclr_scheduler_step_v1(neoclr_aot_context *, uint64_t *, const neoclr_probe_frame *);
/* Private runtime submission prototype. Clone a rooted image-local fn<Void>
 * into this owner-thread scope; success means admitted, never inline execution.
 * Caller retains its input handle. No Task/Promise or managed queue is required.
 * Exact published boundary or quiescence required. Runnable nodes use independent
 * native storage plus the shared host-root budget; rejection preserves caller ownership. No FIFO/parallelism or
 * cancellation ticket contract. Teardown abandons pending submissions. */
int32_t neoclr_scheduler_submit_v1(neoclr_aot_context *, uint64_t, const neoclr_probe_frame *);
/* Private compiled managed-call adapter; output is the private admission acknowledgment (Int32 zero).
 * Called on a published frame, converts a managed callable to an admission root. */
int32_t neoclr_schedule_task_managed_v1(void *, neoclr_aot_context *, void *);
int32_t neoclr_scheduler_submit_managed_v1(void *, neoclr_aot_context *, int32_t *);
/* Opt-in owned delivery. Acquire returns the same states as step, but READY
 * publishes an owned root/ticket. Runnable roots transfer directly; I/O results
 * retain separate source ownership. No poll may advance until complete.
 * Complete at the acquiring boundary after callback return, including guest fault;
 * it releases dispatch ownership, not the operation result. Stale/wrong tickets
 * return 3; scope leave abandons an outstanding delivery. Acquisition failure is
 * terminal: tear down the scope. These are private host contracts, not stable ABI. */
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *, uint64_t *, const neoclr_probe_frame *);
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *, uint64_t, const neoclr_probe_frame *);
/* Quiescent owner-thread poll: 0 no ready work, 1 publishes one retained callback
 * handle, -3 host misuse. Never invokes guest code. Consume result during dispatch;
 * scope teardown releases abandoned operations, roots and accepted sockets. */
int32_t neoclr_socket_poll_v1(neoclr_aot_context *context, uint64_t *callback);
/* Trusted host entry pump; validates the exact published suspension boundary. */
int32_t neoclr_socket_poll_suspended_v1(neoclr_aot_context *, uint64_t *, const neoclr_probe_frame *);
#endif
#endif
