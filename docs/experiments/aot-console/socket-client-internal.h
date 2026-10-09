/* Included once by socket-listener.c: private client policy uses the same scope,
 * operation identities, monotonic origin and callback ownership as the server. */
#ifdef _WIN32
#include <ws2tcpip.h>
#include <process.h>
#else
#include <netdb.h>
#include <pthread.h>
#endif
#include <stdio.h>
struct neoclr_dns_work {
    _Atomic unsigned refs, done;
    char name[254];
    unsigned count, error;
    uint32_t addresses[16];
};
static _Atomic unsigned resolver_workers;
#ifdef NEOCLR_CLIENT_TEST
static _Atomic unsigned resolver_test_hold;
void neoclr_dns_test_hold(unsigned hold) { atomic_store(&resolver_test_hold, hold); }
unsigned neoclr_dns_test_workers(void) { return atomic_load(&resolver_workers); }
#endif
static void dns_release(neoclr_dns_work *work) {
    if (work && atomic_fetch_sub_explicit(&work->refs, 1, memory_order_acq_rel) == 1) free(work);
}
static void release_client(neoclr_socket_scope *scope, unsigned i) {
    dns_release(scope->operations[i].dns);
    scope->operations[i].dns = NULL;
    if (scope->operations[i].kind == 4 && scope->operations[i].connecting != NEOCLR_INVALID_SOCKET) {
        neoclr_os_close(scope->operations[i].connecting);
        scope->operations[i].connecting = NEOCLR_INVALID_SOCKET;
    }
}
#ifdef _WIN32
static unsigned __stdcall dns_worker(void *arg) {
#else
static void *dns_worker(void *arg) {
#endif
    neoclr_dns_work *work = arg;
#ifdef NEOCLR_CLIENT_TEST
    while (atomic_load(&resolver_test_hold)) neoclr_os_pause();
#endif
    struct addrinfo hints = {0}, *results = NULL;
    hints.ai_family = AF_UNSPEC; hints.ai_socktype = SOCK_STREAM;
    if (neoclr_os_start()) work->error = 6;
    else {
        if (getaddrinfo(work->name, NULL, &hints, &results)) work->error = 6;
        else {
            unsigned scanned = 0;
            for (struct addrinfo *item = results; item; item = item->ai_next) {
                if (++scanned > 256) { work->error = 2; break; }
                if (item->ai_family != AF_INET || item->ai_addrlen < sizeof(struct sockaddr_in)) continue;
                uint32_t address = ((struct sockaddr_in *)item->ai_addr)->sin_addr.s_addr;
                unsigned i = 0;
                while (i < work->count && work->addresses[i] != address) i++;
                if (i != work->count) continue;
                if (work->count == 16) { work->error = 2; break; }
                work->addresses[work->count++] = address;
            }
            if (!work->count && !work->error) work->error = 3;
            freeaddrinfo(results);
        }
        neoclr_os_stop();
    }
    atomic_fetch_sub_explicit(&resolver_workers, 1, memory_order_acq_rel);
    atomic_store_explicit(&work->done, 1, memory_order_release);
    dns_release(work);
#ifdef _WIN32
    return 0;
#else
    return NULL;
#endif
}
static int32_t client_deadline(uint64_t until, uint64_t *end) {
    uint64_t now = monotonic_ns();
    if (!now || now > UINT64_MAX - UINT64_C(5000000000)) return 3;
    *end = now + UINT64_C(5000000000);
    if (until && until < *end) *end = until;
    return 0;
}
static int32_t dns_lookup(const neoclr_aot_text *name, uint64_t until, void *callback,
                          neoclr_aot_context *context, void *output) {
    neoclr_socket_scope *scope = find_scope(context);
    if (!scope || !name || !callback || !output) return 3;
    if (!name->length || name->length > 253) return publish(output, 2, 1);
    const unsigned char *bytes = (const void *)(name + 1);
    for (unsigned i = 0; i < name->length; i++) {
        unsigned char c = bytes[i];
        if (!((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
              (c >= '0' && c <= '9') || c == '.' || c == '-')) return publish(output, 2, 1);
    }
    unsigned op = 0, dns_count = 0;
    while (op < 64 && scope->operations[op].state) op++;
    for (unsigned i = 0; i < 64; i++) dns_count += scope->operations[i].state && scope->operations[i].kind == 5;
    if (op == 64 || dns_count >= 8) return publish(output, 2, 2);
    uint64_t end;
    if (client_deadline(until, &end)) return 3;
    if (monotonic_ns() >= end) return publish(output, 2, 4);
    unsigned workers = atomic_load_explicit(&resolver_workers, memory_order_relaxed);
    do { if (workers >= 4) return publish(output, 2, 2); }
    while (!atomic_compare_exchange_weak_explicit(&resolver_workers, &workers, workers + 1, memory_order_acq_rel, memory_order_relaxed));
    neoclr_dns_work *work = calloc(1, sizeof(*work));
    uint64_t id = new_operation_id(), root = 0;
    int32_t status = work && id ? neoclr_gc_host_root_create_v1(context, callback, &root) : 0;
    if (!work || !id || status) {
        free(work); atomic_fetch_sub_explicit(&resolver_workers, 1, memory_order_acq_rel);
        return status ? status : publish(output, 2, 2);
    }
    atomic_init(&work->refs, 2); atomic_init(&work->done, 0);
    memcpy(work->name, bytes, (size_t)name->length);
    int failed;
#ifdef _WIN32
    uintptr_t thread = _beginthreadex(NULL, 0, dns_worker, work, 0, NULL);
    failed = !thread;
    if (thread) CloseHandle((HANDLE)thread);
#else
    pthread_t thread;
    pthread_attr_t attr;
    failed = pthread_attr_init(&attr);
    if (!failed) {
        failed = pthread_attr_setdetachstate(&attr, PTHREAD_CREATE_DETACHED);
        if (!failed) failed = pthread_create(&thread, &attr, dns_worker, work);
        pthread_attr_destroy(&attr);
    }
#endif
    if (failed) {
        free(work); atomic_fetch_sub_explicit(&resolver_workers, 1, memory_order_acq_rel);
        neoclr_gc_host_root_release_v1(context, root); return publish(output, 2, 2);
    }
    scope->operations[op].id = id; scope->operations[op].kind = 5;
    scope->operations[op].state = 1; scope->operations[op].deadline = end;
    scope->operations[op].dns = work; scope->operations[op].callback = root;
    return publish(output, 5, id);
}
int32_t neoclr_dns_lookup_v1(const neoclr_aot_text *name, void *cb, neoclr_aot_context *c, void *out) {
    return dns_lookup(name, 0, cb, c, out);
}
int32_t neoclr_dns_lookup_until_v1(const neoclr_aot_text *name, int64_t stamp, void *cb, neoclr_aot_context *c, void *out) {
    uint64_t end; if (deadline_ns(stamp, &end)) return 3;
    return dns_lookup(name, end, cb, c, out);
}
int32_t neoclr_dns_cancel_v1(uint64_t id, neoclr_aot_context *c, int32_t *out) {
    neoclr_socket_scope *s = find_scope(c);
    if (!s || !out) return 3;
    for (unsigned i = 0; i < 64; i++) if (id && s->operations[i].id == id && s->operations[i].kind == 5) {
        *out = s->operations[i].state == 1;
        if (*out) { s->operations[i].state = 2; s->operations[i].tag = 2; s->operations[i].value = 5; release_client(s, i); }
        return 0;
    }
    return 3;
}
int32_t neoclr_dns_result_v1(uint64_t id, neoclr_aot_context *c, void *out) {
    neoclr_socket_scope *s = find_scope(c);
    if (!s || !out) return 3;
    for (unsigned i = 0; i < 64; i++) if (id && s->operations[i].id == id && s->operations[i].kind == 5) {
        if (s->operations[i].state != 3) return 3;
        uint64_t tag = s->operations[i].tag, value = s->operations[i].value;
        if (tag != 2) {
            neoclr_dns_work *work = s->operations[i].dns;
            void *array;
            int32_t status = neoclr_allocate_strings_v1(&c->text, (int32_t)work->count, 0, &array);
            if (status) return status;
            for (unsigned n = 0; n < work->count; n++) {
                uint32_t ip = ntohl(work->addresses[n]); char text[16];
                int length = snprintf(text, sizeof(text), "%u.%u.%u.%u", ip >> 24, (ip >> 16) & 255, (ip >> 8) & 255, ip & 255);
                uint64_t decoded[2] = {0};
                if (length <= 0 || length >= (int)sizeof(text)) return 3;
                status = neoclr_text_decode_utf8_bytes_v1((const unsigned char *)text, (uint64_t)length, &c->text, decoded);
                if (status) return status;
                if (decoded[0] != 4) return 3;
                ((uint64_t *)array)[2 + n] = decoded[1];
            }
            tag = 7; value = (uint64_t)(uintptr_t)array;
        }
        if (neoclr_gc_host_root_release_v1(c, s->operations[i].callback)) return 3;
        release_client(s, i); memset(&s->operations[i], 0, sizeof(s->operations[i]));
        return publish(out, tag, value);
    }
    return 3;
}
int32_t neoclr_dns_addresses_v1(uint32_t tag, uint64_t value, neoclr_aot_context *c, void **out) {
    if (!find_scope(c) || !out || tag != 7 || !value) return 3;
    const uint64_t *source = (const void *)(uintptr_t)value;
    if (source[0] != UINT64_C(0x80000003) || source[1] > 16) return 3;
    void *copy; int32_t status = neoclr_allocate_strings_v1(&c->text, (int32_t)source[1], 0, &copy);
    if (status) return status;
    memcpy((uint64_t *)copy + 2, source + 2, (size_t)source[1] * 8);
    *out = copy; return 0;
}
/* Strict IPv4 syntax matches listener and interpreter; snapshot before admission. */
static int parse_address(const neoclr_aot_text *text, uint32_t *output) {
    if (!text || !text->length || text->length > 15) return 0;
    const unsigned char *bytes = (const void *)(text + 1);
    unsigned at = 0; uint32_t value = 0;
    for (unsigned part = 0; part < 4; part++) {
        unsigned start = at, octet = 0;
        while (at < text->length && bytes[at] >= '0' && bytes[at] <= '9') {
            octet = octet * 10 + bytes[at++] - '0'; if (octet > 255) return 0;
        }
        if (start == at || (at - start > 1 && bytes[start] == '0')) return 0;
        value = (value << 8) | octet;
        if (part < 3 && (at == text->length || bytes[at++] != '.')) return 0;
    }
    if (at != text->length) return 0;
    *output = htonl(value); return 1;
}
static int connect_pending(int error) {
#ifdef _WIN32
    return error == WSAEWOULDBLOCK || error == WSAEINPROGRESS;
#else
    return error == EINPROGRESS || error == EWOULDBLOCK;
#endif
}
static void connect_settle(neoclr_socket_scope *s, unsigned op, uint64_t error) {
    release_client(s, op); s->operations[op].state = 2;
    s->operations[op].tag = 2; s->operations[op].value = error;
}
static void connect_success(neoclr_socket_scope *s, unsigned op) {
    unsigned slot = 0; while (slot < 64 && s->slots[slot].id) slot++;
    uint64_t id = new_operation_id();
    if (slot == 64 || !id) { connect_settle(s, op, 4); return; }
    s->slots[slot].id = id; s->slots[slot].descriptor = s->operations[op].connecting;
    s->slots[slot].listener = 0; s->operations[op].connecting = NEOCLR_INVALID_SOCKET;
    s->operations[op].state = 2; s->operations[op].tag = 5; s->operations[op].value = id;
}
static void connect_next(neoclr_socket_scope *s, unsigned op, uint64_t now) {
    release_client(s, op);
    while (s->operations[op].next_address < s->operations[op].address_count) {
        if (now >= s->operations[op].deadline) { connect_settle(s, op, 10); return; }
        neoclr_socket_descriptor fd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        s->operations[op].connecting = fd;
        int error = fd == NEOCLR_INVALID_SOCKET ? neoclr_os_error() : 0;
        struct sockaddr_in endpoint = {0};
        endpoint.sin_family = AF_INET; endpoint.sin_port = htons(s->operations[op].port);
        endpoint.sin_addr.s_addr = s->operations[op].addresses[s->operations[op].next_address++];
        if (!error && neoclr_os_configure(fd)) error = neoclr_os_error();
        if (!error) {
            if (!connect(fd, (const struct sockaddr *)&endpoint, sizeof(endpoint))) { connect_success(s, op); return; }
            error = neoclr_os_error();
            if (connect_pending(error)) {
                uint64_t attempt = now + UINT64_C(1000000000);
                s->operations[op].attempt_deadline = s->operations[op].next_address < s->operations[op].address_count && attempt < s->operations[op].deadline ? attempt : s->operations[op].deadline;
                return;
            }
        }
        if (s->operations[op].next_address == s->operations[op].address_count) { connect_settle(s, op, error_code(error)); return; }
        release_client(s, op);
    }
}
static int32_t connect_addresses(const neoclr_aot_text *single, const void *array, int32_t port, uint64_t until,
    void *callback, neoclr_aot_context *c, void *out) {
    neoclr_socket_scope *s = find_scope(c);
    if (!s || (!single && !array) || !callback || !out) return 3;
    const uint64_t *items = array;
    uint64_t count = single ? 1 : items[1];
    if (!single && (uint32_t)items[0] != UINT32_C(0x80000003) && (uint32_t)items[0] != UINT32_C(0x80000004)) return 3;
    if (!count || port < 1 || port > 65535) return publish(out, 2, 3);
    if (count > 16) return publish(out, 2, 4);
    uint32_t addresses[16]; unsigned unique = 0;
    for (unsigned i = 0; i < count; i++) {
        if (!single && (uint32_t)items[0] == UINT32_C(0x80000004) && !((const unsigned char *)(items + 2 + count))[i]) return 3;
        const neoclr_aot_text *address = single ? single : (const void *)(uintptr_t)items[2 + i];
        uint32_t ip; if (!parse_address(address, &ip)) return publish(out, 2, 6);
        unsigned n = 0; while (n < unique && addresses[n] != ip) n++;
        if (n == unique) addresses[unique++] = ip;
    }
    unsigned op = 0, resources = 0;
    while (op < 64 && s->operations[op].state) op++;
    for (unsigned i = 0; i < 64; i++) {
        resources += s->slots[i].id != 0;
        resources += s->operations[i].state == 1 && (s->operations[i].kind == 1 || s->operations[i].kind == 4);
    }
    if (op == 64 || resources >= 64) return publish(out, 2, 4);
    uint64_t end; if (client_deadline(until, &end)) return 3;
    if (monotonic_ns() >= end) return publish(out, 2, 10);
    uint64_t id = new_operation_id(), root;
    if (!id) return publish(out, 2, 4);
    int32_t status = neoclr_gc_host_root_create_v1(c, callback, &root);
    if (status) return status;
    s->operations[op].id = id; s->operations[op].callback = root;
    s->operations[op].kind = 4; s->operations[op].state = 1;
    s->operations[op].connecting = NEOCLR_INVALID_SOCKET;
    s->operations[op].deadline = end; s->operations[op].port = (uint16_t)port;
    s->operations[op].address_count = unique;
    memcpy(s->operations[op].addresses, addresses, unique * sizeof(*addresses));
    connect_next(s, op, monotonic_ns());
    return publish(out, 5, id);
}
int32_t neoclr_socket_connect_v1(const neoclr_aot_text *ip, int32_t port, void *cb, neoclr_aot_context *c, void *out) {
    return connect_addresses(ip, NULL, port, 0, cb, c, out);
}
int32_t neoclr_socket_connect_addresses_v1(const void *ips, int32_t port, void *cb, neoclr_aot_context *c, void *out) {
    return connect_addresses(NULL, ips, port, 0, cb, c, out);
}
int32_t neoclr_socket_connect_addresses_until_v1(const void *ips, int32_t port, int64_t stamp, void *cb, neoclr_aot_context *c, void *out) {
    uint64_t end; if (deadline_ns(stamp, &end)) return 3;
    return connect_addresses(NULL, ips, port, end, cb, c, out);
}
static int poll_client(neoclr_socket_scope *s, unsigned i) {
    uint64_t now = monotonic_ns(); if (!now) return 3;
    if (s->operations[i].kind == 5) {
        neoclr_dns_work *work = s->operations[i].dns;
        if (now >= s->operations[i].deadline) {
            s->operations[i].tag = 2; s->operations[i].value = 4;
            s->operations[i].state = 2; release_client(s, i);
        } else if (atomic_load_explicit(&work->done, memory_order_acquire)) {
            s->operations[i].tag = work->error ? 2 : 7;
            s->operations[i].value = work->error; s->operations[i].state = 2;
        }
        return 0;
    }
    if (now >= s->operations[i].deadline) { connect_settle(s, i, 10); return 0; }
    neoclr_socket_descriptor fd = s->operations[i].connecting;
#ifndef _WIN32
    if (fd >= FD_SETSIZE) { connect_settle(s, i, 4); return 0; }
#endif
    fd_set writes, errors; FD_ZERO(&writes); FD_ZERO(&errors); FD_SET(fd, &writes); FD_SET(fd, &errors);
    struct timeval wait = {0, 0};
#ifdef _WIN32
    int ready = select(0, NULL, &writes, &errors, &wait);
#else
    if (fd >= FD_SETSIZE) { connect_settle(s, i, 4); return 0; }
    int ready = select(fd + 1, NULL, &writes, &errors, &wait);
#endif
    if (ready < 0) {
        if (neoclr_os_retry(neoclr_os_error())) return 0;
        connect_settle(s, i, error_code(neoclr_os_error())); return 0;
    }
    if (ready) {
        int error = 0; neoclr_socklen size = sizeof(error);
        if (getsockopt(fd, SOL_SOCKET, SO_ERROR, (void *)&error, &size)) error = neoclr_os_error();
        if (!error) { connect_success(s, i); return 0; }
        if (s->operations[i].next_address == s->operations[i].address_count) connect_settle(s, i, error_code(error));
        else connect_next(s, i, now);
    } else if (now >= s->operations[i].attempt_deadline) connect_next(s, i, now);
    return 0;
}
