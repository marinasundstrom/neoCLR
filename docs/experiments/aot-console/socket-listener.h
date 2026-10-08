#ifndef NEOCLR_AOT_SOCKET_LISTENER_H
#define NEOCLR_AOT_SOCKET_LISTENER_H
#include "text-arena.h"
/* Private synchronous POSIX listener experiment. Host enters one stack-owned scope
 * per entry and leaves it on EVERY success/fault path before resetting the context.
 * No same-context reentry or cross-thread scope use. LIFO nested distinct contexts
 * are supported. Closing the scope releases sockets even when guest code faults.
 * Handles are opaque, process-unique positive Int64s, never file descriptors.
 * OS resources are separate from the managed heap's byte budget. */
typedef struct neoclr_socket_scope {
    neoclr_aot_context *context;
    struct neoclr_socket_scope *previous;
    struct { uint64_t id; int descriptor; } slots[64];
} neoclr_socket_scope;
int32_t neoclr_socket_scope_enter_v1(neoclr_socket_scope *scope, neoclr_aot_context *context);
int32_t neoclr_socket_scope_leave_v1(neoclr_socket_scope *scope);
/* Status 0 publishes erased Int64 handle / Int32 port / Void, or Byte SocketError.
 * Invalid native arguments or missing scope fault with status 3, no publication.
 * Services neither allocate managed storage nor collect. No accept/read/write yet. */
int32_t neoclr_socket_listen_v1(const neoclr_aot_text *address, int32_t port, int32_t backlog,
                              neoclr_aot_context *context, void *output);
int32_t neoclr_socket_local_port_v1(uint64_t handle, neoclr_aot_context *context, void *output);
int32_t neoclr_socket_close_v1(uint64_t handle, neoclr_aot_context *context, void *output);
#endif
