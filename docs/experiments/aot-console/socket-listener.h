#ifndef NEOCLR_AOT_SOCKET_LISTENER_H
#define NEOCLR_AOT_SOCKET_LISTENER_H
#include "text-arena.h"
/* Private POSIX socket experiment; GC builds additionally support polled accept/transfers. Host enters one stack-owned scope
 * per invocation and leaves it on EVERY terminal success/fault path before resetting
 * the context. Async work may outlive entry return inside that scope.
 * No same-context reentry or cross-thread scope use. LIFO nested distinct contexts
 * are supported. Closing the scope releases sockets even when guest code faults.
 * Handles are opaque, process-unique positive Int64s, never file descriptors.
 * OS resources are separate from the managed heap's byte budget. */
typedef struct neoclr_socket_scope {
    neoclr_aot_context *context;
    struct neoclr_socket_scope *previous;
    struct { uint64_t id; int descriptor; int listener; } slots[64];
    /* Private operations: state 1 pending, 2 ready, 3 delivered; kind 1 accept,
     * 2 receive, 3 send. Transfer snapshots have a separate bounded native budget. */
    struct { uint64_t id, listener, callback, tag, value, buffer_root, deadline;
        uint32_t state, kind, offset, count; unsigned char *buffer;
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
 * outbound connect remains unsupported. */
int32_t neoclr_socket_listen_v1(const neoclr_aot_text *address, int32_t port, int32_t backlog,
                              neoclr_aot_context *context, void *output);
int32_t neoclr_socket_local_port_v1(uint64_t handle, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_close_v1(uint64_t handle, neoclr_aot_context *context, void *output);
#ifdef NEOCLR_NATIVE_GC
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
/* Quiescent owner-thread poll: 0 no ready work, 1 publishes one retained callback
 * handle, -3 host misuse. Never invokes guest code. Consume result during dispatch;
 * scope teardown releases abandoned operations, roots and accepted sockets. */
int32_t neoclr_socket_poll_v1(neoclr_aot_context *context, uint64_t *callback);
#endif
#endif
