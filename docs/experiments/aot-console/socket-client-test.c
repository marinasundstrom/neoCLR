#include "socket-listener.h"
#include "native-gc.h"
#include <string.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "client contract line %d\n", __LINE__); return __LINE__; } } while (0)
void neoclr_dns_test_hold(unsigned);
unsigned neoclr_dns_test_workers(void);
static int poll(neoclr_aot_context *c, uint64_t *root) {
    for (unsigned i = 0; i < 5000; i++) {
        int ready = neoclr_socket_poll_v1(c, root);
        if (ready) return ready;
        neoclr_os_pause();
    }
    return 0;
}
int main(void) {
    uint64_t heap[8192] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_socket_scope s;
    CHECK(!neoclr_socket_scope_enter_v1(&s, &c));
    void *cb;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &cb));
    ((uint64_t *)cb)[1] = 1;
    uint64_t cbroot;
    CHECK(!neoclr_gc_host_root_create_v1(&c, cb, &cbroot));
    const struct { uint64_t length; char bytes[16]; } ip = {9, "127.0.0.1"}, bad = {3, "a b"}, fallback = {9, "127.0.0.2"};
    uint64_t out[2] = {99, 99}, root = 0, id;
    CHECK(!neoclr_dns_lookup_v1((const void *)&bad, cb, &c, out) && out[0] == 2 && out[1] == 1);
    CHECK(!neoclr_dns_lookup_until_v1((const void *)&ip, 0, cb, &c, out) && out[0] == 2 && out[1] == 4);
    CHECK(!neoclr_dns_lookup_v1((const void *)&ip, cb, &c, out) && out[0] == 5);
    id = out[1];
    CHECK(neoclr_dns_result_v1(id, &c, out) == 3);
    CHECK(poll(&c, &root) == 1);
    CHECK(!neoclr_socket_poll_v1(&c, &root));
    int32_t dns_status = neoclr_dns_result_v1(id, &c, out);
    if (dns_status || out[0] != 7) fprintf(stderr, "dns status %d tag %llu value %llu\n", dns_status, (unsigned long long)out[0], (unsigned long long)out[1]);
    CHECK(!dns_status && out[0] == 7);
    void *snapshot = (void *)(uintptr_t)out[1], *copy;
    uint64_t snapshot_root;
    CHECK(!neoclr_gc_host_root_create_v1(&c, snapshot, &snapshot_root));
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    CHECK(!neoclr_dns_addresses_v1((uint32_t)out[0], out[1], &c, &copy) && copy != snapshot);
    uint64_t *strings = copy;
    CHECK(strings[1] == 1);
    const neoclr_aot_text *text = (const void *)(uintptr_t)strings[2];
    CHECK(text->length == 9 && !memcmp(text + 1, "127.0.0.1", 9));
    CHECK(neoclr_dns_result_v1(id, &c, out) == 3);
    CHECK(!neoclr_gc_host_root_release_v1(&c, snapshot_root));
    /* Frozen worker proves cancellation/deadline delivery and scope teardown do
     * not join blocking work or let workers retain the guest context. */
    neoclr_dns_test_hold(1);
    for (unsigned i = 0; i < 4; i++) {
        CHECK(!neoclr_dns_lookup_v1((const void *)&ip, cb, &c, out) && out[0] == 5);
        id = out[1]; int32_t cancelled = 0;
        if (i == 1) {
            for (unsigned j = 0; j < 64; j++) if (s.operations[j].id == id) s.operations[j].deadline = 1;
        } else CHECK(!neoclr_dns_cancel_v1(id, &c, &cancelled) && cancelled);
        CHECK(poll(&c, &root) == 1);
        CHECK(!neoclr_dns_result_v1(id, &c, out) && out[0] == 2 && out[1] == (i == 1 ? 4u : 5u));
    }
    CHECK(neoclr_dns_test_workers() == 4);
    CHECK(!neoclr_dns_lookup_v1((const void *)&ip, cb, &c, out) && out[0] == 2 && out[1] == 2);
    CHECK(!neoclr_gc_host_root_release_v1(&c, cbroot));
    CHECK(!neoclr_socket_scope_leave_v1(&s));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    neoclr_dns_test_hold(0);
    for (unsigned i = 0; i < 5000 && neoclr_dns_test_workers(); i++) neoclr_os_pause();
    CHECK(!neoclr_dns_test_workers());
    CHECK(!neoclr_socket_scope_enter_v1(&s, &c));
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &cb));
    ((uint64_t *)cb)[1] = 1;
    CHECK(!neoclr_gc_host_root_create_v1(&c, cb, &cbroot));
    CHECK(!neoclr_socket_listen_v1((const void *)&ip, 0, 4, &c, out) && out[0] == 5);
    uint64_t listener = out[1];
    CHECK(!neoclr_socket_local_port_v1(listener, &c, out) && out[0] == 1);
    int32_t port = (int32_t)out[1];
    uint64_t addresses[] = {UINT64_C(0x1280000003), 3, (uint64_t)(uintptr_t)&fallback, (uint64_t)(uintptr_t)&ip, (uint64_t)(uintptr_t)&ip};
    CHECK(!neoclr_socket_connect_addresses_until_v1(addresses, port, 0, cb, &c, out) && out[0] == 2 && out[1] == 10);
    CHECK(!neoclr_socket_connect_addresses_v1(addresses, port, cb, &c, out) && out[0] == 5);
    id = out[1];
    CHECK(neoclr_socket_connect_result_v1(id, &c, out) == 3);
    CHECK(poll(&c, &root) == 1);
    CHECK(!neoclr_socket_connect_result_v1(id, &c, out) && out[0] == 5);
    CHECK(!neoclr_socket_close_v1(out[1], &c, out));
    /* Obtain an unused ephemeral port, then close before requesting refusal.
     * Bound-but-not-listening ports can time out instead on macOS. */
    neoclr_socket_descriptor refused = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    CHECK(refused != NEOCLR_INVALID_SOCKET);
    struct sockaddr_in endpoint = {0}; endpoint.sin_family = AF_INET; endpoint.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    CHECK(!bind(refused, (const void *)&endpoint, sizeof(endpoint)));
    neoclr_socklen size = sizeof(endpoint);
    CHECK(!getsockname(refused, (void *)&endpoint, &size));
    neoclr_os_close(refused);
    CHECK(!neoclr_socket_connect_v1((const void *)&ip, ntohs(endpoint.sin_port), cb, &c, out) && out[0] == 5);
    id = out[1]; CHECK(poll(&c, &root) == 1);
    int32_t connect_status = neoclr_socket_connect_result_v1(id, &c, out);
    if (connect_status || out[0] != 2 || out[1] != 7) fprintf(stderr, "connect status %d tag %llu value %llu\n", connect_status, (unsigned long long)out[0], (unsigned long long)out[1]);
    CHECK(!connect_status && out[0] == 2 && out[1] == 7);
    CHECK(!neoclr_gc_host_root_release_v1(&c, cbroot));
    CHECK(!neoclr_socket_scope_leave_v1(&s));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used && !neoclr_gc_entry_check_v1(&c));
    return 0;
}
