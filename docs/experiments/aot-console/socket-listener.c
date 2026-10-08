#include "socket-listener.h"
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <stdatomic.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

static _Thread_local neoclr_socket_scope *head;
static _Atomic uint64_t next_id = 1;
static neoclr_socket_scope *find_scope(neoclr_aot_context *context) {
    for (neoclr_socket_scope *s = head; s; s = s->previous) if (s->context == context) return s;
    return NULL;
}
static int32_t publish(void *output, uint64_t tag, uint64_t value) {
    uint64_t lanes[2] = {tag, value};
    memcpy(output, lanes, sizeof(lanes));
    return 0;
}
static uint64_t error_code(int error) {
    switch (error) {
        case ECONNREFUSED: return 7;
        case ECONNRESET: case EPIPE: return 8;
        case EACCES: case EPERM: return 9;
        case ETIMEDOUT: return 10;
        case EADDRINUSE: return 13;
        default: return 11;
    }
}
int32_t neoclr_socket_scope_enter_v1(neoclr_socket_scope *scope, neoclr_aot_context *context) {
    if (!scope || !context || find_scope(context)) return 3;
    for (neoclr_socket_scope *s = head; s; s = s->previous) if (s == scope) return 3;
    memset(scope, 0, sizeof(*scope));
    scope->context = context; scope->previous = head; head = scope;
    return 0;
}
int32_t neoclr_socket_scope_leave_v1(neoclr_socket_scope *scope) {
    if (!scope || scope != head) return 3;
    int32_t status = 0;
    for (unsigned i = 0; i < 64; i++) {
        if (scope->slots[i].id && close(scope->slots[i].descriptor)) status = 3;
        scope->slots[i].id = 0;
    }
    head = scope->previous;
    scope->context = NULL; scope->previous = NULL;
    return status;
}
int32_t neoclr_socket_listen_v1(const neoclr_aot_text *address, int32_t port, int32_t backlog,
                              neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !address || !output) return 3;
    /* Strict dotted decimal, including rejection of leading zeros: match Rust
     * Ipv4Addr parsing rather than accepting inet_aton's shorthand/octal forms. */
    if (!address->length || address->length > 15) return publish(output, 2, 6);
    const unsigned char *text = (const void *)(address + 1);
    unsigned octets[4], part = 0, value = 0, digits = 0;
    for (uint64_t i = 0; i <= address->length; i++) {
        unsigned c = i == address->length ? '.' : text[i];
        if (c == '.') {
            if (!digits || part == 4) return publish(output, 2, 6);
            octets[part++] = value; value = 0; digits = 0;
        } else {
            if (c < '0' || c > '9' || (digits && !value) || digits == 3) return publish(output, 2, 6);
            value = value * 10 + c - '0'; digits++;
            if (value > 255) return publish(output, 2, 6);
        }
    }
    if (part != 4) return publish(output, 2, 6);
    if (port < 0 || port > 65535 || backlog < 1 || backlog > 128) return publish(output, 2, 3);
    unsigned slot = 0;
    while (slot < 64 && scope->slots[slot].id) slot++;
    if (slot == 64) return publish(output, 2, 4);
    uint64_t id = atomic_load_explicit(&next_id, memory_order_relaxed);
    do {
        if (id >= INT64_MAX) return publish(output, 2, 4);
    } while (!atomic_compare_exchange_weak_explicit(&next_id, &id, id + 1, memory_order_relaxed, memory_order_relaxed));
    int descriptor = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (descriptor < 0) return publish(output, 2, error_code(errno));
    struct sockaddr_in endpoint = {0};
    endpoint.sin_family = AF_INET; endpoint.sin_port = htons((uint16_t)port);
    endpoint.sin_addr.s_addr = htonl((octets[0] << 24) | (octets[1] << 16) | (octets[2] << 8) | octets[3]);
    int flags = fcntl(descriptor, F_GETFL);
    int fdflags = fcntl(descriptor, F_GETFD);
    if (flags < 0 || fdflags < 0 || fcntl(descriptor, F_SETFL, flags | O_NONBLOCK) < 0 ||
        fcntl(descriptor, F_SETFD, fdflags | FD_CLOEXEC) < 0 ||
        bind(descriptor, (const struct sockaddr *)&endpoint, sizeof(endpoint)) || listen(descriptor, backlog)) {
        int error = errno;
        close(descriptor);
        return publish(output, 2, error_code(error));
    }
    scope->slots[slot].id = id; scope->slots[slot].descriptor = descriptor;
    return publish(output, 5, id);
}
int32_t neoclr_socket_local_port_v1(uint64_t handle, neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !output) return 3;
    for (unsigned i = 0; i < 64; i++) if (handle && scope->slots[i].id == handle) {
        struct sockaddr_in endpoint;
        socklen_t length = sizeof(endpoint);
        if (getsockname(scope->slots[i].descriptor, (struct sockaddr *)&endpoint, &length)) return publish(output, 2, error_code(errno));
        return publish(output, 1, ntohs(endpoint.sin_port));
    }
    return publish(output, 2, 1);
}
int32_t neoclr_socket_close_v1(uint64_t handle, neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !output) return 3;
    for (unsigned i = 0; i < 64; i++) if (handle && scope->slots[i].id == handle) {
        /* Match interpreter idempotence; release identity before closing, and never
         * retry close on EINTR where descriptor reuse could close another resource. */
        scope->slots[i].id = 0;
        close(scope->slots[i].descriptor);
        break;
    }
    return publish(output, 0, 0);
}
