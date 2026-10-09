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
typedef struct neoclr_socket_scope {
    neoclr_aot_context *context;
    struct neoclr_socket_scope *previous;
    struct { uint64_t id; neoclr_socket_descriptor descriptor; int listener; } slots[64];
    /* Private operations: state 1 pending, 2 ready, 3 delivered; kind 1 accept,
     * 2 receive, 3 send, 4 connect, 5 DNS. Transfer snapshots have a separate bounded native budget. */
    struct { uint64_t id, listener, callback, tag, value, buffer_root, deadline;
        uint32_t state, kind, offset, count; unsigned char *buffer;
        neoclr_dns_work *dns;
        neoclr_socket_descriptor connecting;
        uint32_t addresses[16], address_count, next_address;
        uint16_t port;
        uint64_t attempt_deadline;
    } operations[64];
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
/* Quiescent owner-thread poll: 0 no ready work, 1 publishes one retained callback
 * handle, -3 host misuse. Never invokes guest code. Consume result during dispatch;
 * scope teardown releases abandoned operations, roots and accepted sockets. */
int32_t neoclr_socket_poll_v1(neoclr_aot_context *context, uint64_t *callback);
/* Trusted host entry pump; validates the exact published suspension boundary. */
int32_t neoclr_socket_poll_suspended_v1(neoclr_aot_context *, uint64_t *, const neoclr_probe_frame *);
#endif
#endif
