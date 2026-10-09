#include "socket-listener.h"
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
#include <stdatomic.h>
#include <string.h>
#include <stdlib.h>

static _Thread_local neoclr_socket_scope *head;
static _Atomic uint64_t next_id = 1;
#ifdef NEOCLR_NATIVE_GC
static void release_transfer(neoclr_socket_scope *scope, unsigned i) {
    if (scope->operations[i].buffer_root) {
        neoclr_gc_host_root_release_v1(scope->context, scope->operations[i].buffer_root);
        scope->operations[i].buffer_root = 0;
    }
    if (scope->operations[i].buffer) {
        free(scope->operations[i].buffer);
        scope->operations[i].buffer = NULL;
        scope->transfer_bytes -= scope->operations[i].count;
    }
}
static uint64_t monotonic_ns(void) {
    return neoclr_os_monotonic_ns();
}
static _Atomic uint64_t network_origin;
static uint64_t get_network_origin(uint64_t now) {
    uint64_t origin = atomic_load_explicit(&network_origin, memory_order_relaxed);
    if (!origin) {
        atomic_compare_exchange_strong_explicit(&network_origin, &origin, now,
            memory_order_relaxed, memory_order_relaxed);
        if (!origin) origin = now;
    }
    return origin;
}
static int32_t deadline_ns(int64_t stamp, uint64_t *output) {
    uint64_t now = monotonic_ns();
    if (stamp < 0 || !now) return 3;
    uint64_t origin = get_network_origin(now);
    if ((uint64_t)stamp > (UINT64_MAX - origin) / UINT64_C(1000000)) return 3;
    *output = origin + (uint64_t)stamp * UINT64_C(1000000);
    return 0;
}
int32_t neoclr_socket_deadline_after_v1(int32_t milliseconds, neoclr_aot_context *context, int64_t *output) {
    if (!context || !output || milliseconds < 1 || milliseconds > 60000) return 3;
    uint64_t now = monotonic_ns();
    if (!now) return 3;
    uint64_t origin = get_network_origin(now);
    now = monotonic_ns(); /* Origin may have been published by a later-sampling thread. */
    if (!now || now < origin) return 3;
    uint64_t elapsed = now - origin;
    *output = (int64_t)(elapsed / UINT64_C(1000000) + (elapsed % UINT64_C(1000000) != 0) + (uint64_t)milliseconds);
    return 0;
}
int32_t neoclr_socket_deadline_expired_v1(int64_t stamp, neoclr_aot_context *context, int32_t *output) {
    uint64_t deadline;
    if (!context || !output || deadline_ns(stamp, &deadline)) return 3;
    uint64_t now = monotonic_ns();
    if (!now) return 3;
    *output = now >= deadline;
    return 0;
}
#endif
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
    return neoclr_os_error_code(error);
}
int32_t neoclr_socket_scope_enter_v1(neoclr_socket_scope *scope, neoclr_aot_context *context) {
    if (!scope || !context || find_scope(context)) return 3;
    for (neoclr_socket_scope *s = head; s; s = s->previous) if (s == scope) return 3;
    if (neoclr_os_start()) return 3;
    memset(scope, 0, sizeof(*scope));
    scope->context = context; scope->previous = head; head = scope;
    return 0;
}
int32_t neoclr_socket_scope_leave_v1(neoclr_socket_scope *scope) {
    if (!scope || scope != head) return 3;
    int32_t status = 0;
    for (unsigned i = 0; i < 64; i++) {
        if (scope->slots[i].id && neoclr_os_close(scope->slots[i].descriptor)) status = 3;
        scope->slots[i].id = 0;
    }
#ifdef NEOCLR_NATIVE_GC
    for (unsigned i = 0; i < 64; i++) {
        if (scope->operations[i].callback && neoclr_gc_host_root_release_v1(scope->context, scope->operations[i].callback)) status = 3;
        release_transfer(scope, i);
        memset(&scope->operations[i], 0, sizeof(scope->operations[i]));
    }
#endif
    if (neoclr_os_stop()) status = 3;
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
    neoclr_socket_descriptor descriptor = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (descriptor == NEOCLR_INVALID_SOCKET) return publish(output, 2, error_code(neoclr_os_error()));
    struct sockaddr_in endpoint = {0};
    endpoint.sin_family = AF_INET; endpoint.sin_port = htons((uint16_t)port);
    endpoint.sin_addr.s_addr = htonl((octets[0] << 24) | (octets[1] << 16) | (octets[2] << 8) | octets[3]);
    if (neoclr_os_configure(descriptor) ||
        bind(descriptor, (const struct sockaddr *)&endpoint, sizeof(endpoint)) || listen(descriptor, backlog)) {
        int error = neoclr_os_error();
        neoclr_os_close(descriptor);
        return publish(output, 2, error_code(error));
    }
    scope->slots[slot].id = id; scope->slots[slot].descriptor = descriptor; scope->slots[slot].listener = 1;
    return publish(output, 5, id);
}
int32_t neoclr_socket_local_port_v1(uint64_t handle, neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !output) return 3;
    for (unsigned i = 0; i < 64; i++) if (handle && scope->slots[i].id == handle) {
        struct sockaddr_in endpoint;
        neoclr_socklen length = sizeof(endpoint);
        if (getsockname(scope->slots[i].descriptor, (struct sockaddr *)&endpoint, &length)) return publish(output, 2, error_code(neoclr_os_error()));
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
#ifdef NEOCLR_NATIVE_GC
        for (unsigned op = 0; op < 64; op++) {
            if (scope->operations[op].state == 1 && scope->operations[op].listener == handle) {
                scope->operations[op].state = 2;
                scope->operations[op].tag = 2; scope->operations[op].value = 1;
                release_transfer(scope, op);
            }
        }
#endif
        neoclr_os_close(scope->slots[i].descriptor);
        break;
    }
    return publish(output, 0, 0);
}

#ifdef NEOCLR_NATIVE_GC
static uint64_t new_operation_id(void) {
    uint64_t id = atomic_load_explicit(&next_id, memory_order_relaxed);
    do { if (id >= INT64_MAX) return 0; }
    while (!atomic_compare_exchange_weak_explicit(&next_id, &id, id + 1,
                memory_order_relaxed, memory_order_relaxed));
    return id;
}
int32_t neoclr_socket_accept_v1(uint64_t listener, void *callback,
                               neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !output || !callback) return 3;
    unsigned socket = 0, operation = 0, resources = 0;
    while (socket < 64 && scope->slots[socket].id != listener) socket++;
    if (!listener || socket == 64) return publish(output, 2, 1);
    if (!scope->slots[socket].listener) return publish(output, 2, 12);
    for (unsigned i = 0; i < 64; i++) {
        resources += scope->slots[i].id != 0;
        resources += scope->operations[i].state == 1 && scope->operations[i].kind == 1;
        if (scope->operations[i].state == 1 && scope->operations[i].kind == 1 && scope->operations[i].listener == listener)
            return publish(output, 2, 2);
    }
    while (operation < 64 && scope->operations[operation].state) operation++;
    if (operation == 64 || resources >= 64) return publish(output, 2, 4);
    uint64_t id = new_operation_id(), root;
    if (!id) return publish(output, 2, 4);
    int32_t status = neoclr_gc_host_root_create_v1(context, callback, &root);
    if (status) return status;
    scope->operations[operation].id = id;
    scope->operations[operation].listener = listener;
    scope->operations[operation].callback = root;
    scope->operations[operation].state = 1;
    scope->operations[operation].kind = 1;
    return publish(output, 5, id);
}
int32_t neoclr_socket_cancel_v1(uint64_t operation, neoclr_aot_context *context, int32_t *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !output) return 3;
    for (unsigned i = 0; i < 64; i++) if (operation && scope->operations[i].id == operation) {
        *output = scope->operations[i].state == 1;
        if (*output) {
            scope->operations[i].state = 2;
            scope->operations[i].tag = 2; scope->operations[i].value = 5;
            release_transfer(scope, i);
        }
        return 0;
    }
    return 3;
}
static int32_t take_result(uint64_t operation, neoclr_aot_context *context, void *output, int accept_result) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !output) return 3;
    for (unsigned i = 0; i < 64; i++) if (operation && scope->operations[i].id == operation) {
        if (scope->operations[i].state != 3 || ((scope->operations[i].kind == 1) != accept_result)) return 3;
        if (neoclr_gc_host_root_release_v1(context, scope->operations[i].callback)) return 3;
        publish(output, scope->operations[i].tag, scope->operations[i].value);
        release_transfer(scope, i);
        memset(&scope->operations[i], 0, sizeof(scope->operations[i]));
        return 0;
    }
    return 3;
}
int32_t neoclr_socket_connect_result_v1(uint64_t operation, neoclr_aot_context *context, void *output) {
    return take_result(operation, context, output, 1);
}
int32_t neoclr_socket_transfer_result_v1(uint64_t operation, neoclr_aot_context *context, void *output) {
    return take_result(operation, context, output, 0);
}
static int32_t transfer(int sending, uint64_t until, uint64_t socket, void *array, int32_t offset, int32_t count,
    void *callback, neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !array || !callback || !output) return 3;
    unsigned slot = 0, op = 0;
    while (slot < 64 && scope->slots[slot].id != socket) slot++;
    if (!socket || slot == 64) return publish(output, 2, 1);
    if (scope->slots[slot].listener) return publish(output, 2, 12);
    unsigned kind = sending ? 3 : 2;
    for (unsigned i = 0; i < 64; i++)
        if (scope->operations[i].state == 1 && scope->operations[i].kind == kind && scope->operations[i].listener == socket)
            return publish(output, 2, 2);
    /* Verified CIL supplies the array; range/header checks precede payload access. */
    uint64_t header, length;
    memcpy(&header, array, 8); memcpy(&length, (unsigned char *)array + 8, 8);
    if ((header != UINT64_C(0x80000001) && header != UINT64_C(0x80000002)) || length > 65536) return 3;
    if (offset < 0 || count < 0 || (uint64_t)offset > length || (uint64_t)count > length - (uint64_t)offset)
        return publish(output, 2, 3);
    while (op < 64 && scope->operations[op].state) op++;
    if (op == 64 || (uint64_t)count > UINT64_C(262144) - scope->transfer_bytes) return publish(output, 2, 4);
    uint64_t now = monotonic_ns(), id = new_operation_id(), root = 0, destination = 0;
    if (!now) return 3;
    if (until && now >= until) return publish(output, 2, 10);
    if (sending && neoclr_check_bytes_initialized_v1(array, offset, count)) return 3;
    if (!id) return publish(output, 2, 4);
    unsigned char *copy = count ? malloc((size_t)count) : NULL;
    if (count && !copy) return publish(output, 2, 4);
    int32_t status = neoclr_gc_host_root_create_v1(context, callback, &root);
    if (!status && !sending) status = neoclr_gc_host_root_create_v1(context, array, &destination);
    if (status) {
        if (root) neoclr_gc_host_root_release_v1(context, root);
        free(copy); return status;
    }
    if (sending && count) memcpy(copy, (unsigned char *)array + 16 + offset, (size_t)count);
    scope->operations[op].id = id; scope->operations[op].listener = socket;
    scope->operations[op].callback = root; scope->operations[op].buffer_root = destination;
    scope->operations[op].state = 1; scope->operations[op].kind = kind;
    scope->operations[op].offset = (uint32_t)offset; scope->operations[op].count = (uint32_t)count;
    scope->operations[op].buffer = copy; scope->operations[op].deadline = now + UINT64_C(5000000000);
    if (until && until < scope->operations[op].deadline) scope->operations[op].deadline = until;
    scope->transfer_bytes += (uint64_t)count;
    return publish(output, 5, id);
}
int32_t neoclr_socket_receive_v1(uint64_t socket, void *array, int32_t offset, int32_t count,
    void *callback, neoclr_aot_context *context, void *output) {
    return transfer(0, 0, socket, array, offset, count, callback, context, output);
}
int32_t neoclr_socket_send_v1(uint64_t socket, void *array, int32_t offset, int32_t count,
    void *callback, neoclr_aot_context *context, void *output) {
    return transfer(1, 0, socket, array, offset, count, callback, context, output);
}
int32_t neoclr_socket_receive_until_v1(uint64_t socket, void *array, int32_t offset, int32_t count,
    int64_t stamp, void *callback, neoclr_aot_context *context, void *output) {
    uint64_t deadline;
    if (deadline_ns(stamp, &deadline)) return 3;
    return transfer(0, deadline, socket, array, offset, count, callback, context, output);
}
int32_t neoclr_socket_send_until_v1(uint64_t socket, void *array, int32_t offset, int32_t count,
    int64_t stamp, void *callback, neoclr_aot_context *context, void *output) {
    uint64_t deadline;
    if (deadline_ns(stamp, &deadline)) return 3;
    return transfer(1, deadline, socket, array, offset, count, callback, context, output);
}
int32_t neoclr_socket_poll_v1(neoclr_aot_context *context, uint64_t *callback) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !callback || context->fault.code) return -3;
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context == context) return -3;
    for (unsigned n = 0; n < 64; n++) {
        unsigned i = (scope->poll_cursor + n) % 64;
        if (scope->operations[i].state == 1 && scope->operations[i].kind != 1) {
            unsigned slot = 0;
            while (slot < 64 && scope->slots[slot].id != scope->operations[i].listener) slot++;
            if (slot == 64) return -3;
            uint64_t now = monotonic_ns();
            if (!now) return -3;
            uint32_t count = scope->operations[i].count;
            int sending = scope->operations[i].kind == 3;
            uint64_t tag = 1, value = 0;
            if (count && now >= scope->operations[i].deadline) { tag = 2; value = 10; }
            else if (count) {
                neoclr_socket_descriptor fd = scope->slots[slot].descriptor;
                int64_t transferred = neoclr_os_transfer(fd, scope->operations[i].buffer, count, sending);
                if (transferred == -2) return -3; /* Host signal-suppression setup failed. */
                if (transferred < 0 && neoclr_os_retry(neoclr_os_error())) continue;
                if (transferred < 0) { tag = 2; value = error_code(neoclr_os_error()); }
                else if (!transferred && sending) { tag = 2; value = 11; }
                else value = (uint64_t)transferred;
            }
            if (tag == 1 && !sending && value) {
                void *array = NULL;
                if (neoclr_gc_host_root_read_v1(context, scope->operations[i].buffer_root, &array)) return -3;
                uint64_t header, length;
                memcpy(&header, array, 8); memcpy(&length, (unsigned char *)array + 8, 8);
                memcpy((unsigned char *)array + 16 + scope->operations[i].offset, scope->operations[i].buffer, (size_t)value);
                if (header == UINT64_C(0x80000002))
                    memset((unsigned char *)array + 16 + length + scope->operations[i].offset, 1, (size_t)value);
            }
            scope->operations[i].tag = tag; scope->operations[i].value = value;
            scope->operations[i].state = 2; release_transfer(scope, i);
        }
        if (scope->operations[i].state == 1 && scope->operations[i].kind == 1) {
            unsigned listener = 0, slot = 0;
            while (listener < 64 && scope->slots[listener].id != scope->operations[i].listener) listener++;
            if (listener == 64) return -3;
            neoclr_socket_descriptor fd = accept(scope->slots[listener].descriptor, NULL, NULL);
            if (fd == NEOCLR_INVALID_SOCKET && neoclr_os_retry(neoclr_os_error())) continue;
            uint64_t tag = 2, value = fd == NEOCLR_INVALID_SOCKET ? error_code(neoclr_os_error()) : 0;
            if (fd != NEOCLR_INVALID_SOCKET) {
                while (slot < 64 && scope->slots[slot].id) slot++;
                uint64_t id = new_operation_id();
                if (slot == 64 || !id) value = 4;
                else if (neoclr_os_configure(fd)) value = error_code(neoclr_os_error());
                else {
                    scope->slots[slot].id = id; scope->slots[slot].descriptor = fd;
                    scope->slots[slot].listener = 0; tag = 5; value = id;
                }
                if (tag == 2) neoclr_os_close(fd);
            }
            scope->operations[i].tag = tag; scope->operations[i].value = value;
            scope->operations[i].state = 2;
        }
        if (scope->operations[i].state == 2) {
            scope->operations[i].state = 3;
            scope->poll_cursor = (i + 1) % 64;
            *callback = scope->operations[i].callback;
            return 1;
        }
    }
    return 0;
}
#endif
