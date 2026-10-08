#include "socket-listener.h"
#include "native-gc.h"
#include <arpa/inet.h>
#include <sys/socket.h>
#include <unistd.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129] = {0}; buffer[128] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}};
    neoclr_socket_scope scope;
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    const struct { uint64_t length; char bytes[16]; } address = {9, "127.0.0.1"};
    uint64_t result[2] = {99, 99};
    CHECK(!neoclr_socket_listen_v1((const void *)&address, 0, 4, &c, result) && result[0] == 5);
    uint64_t listener = result[1];
    CHECK(!neoclr_socket_local_port_v1(listener, &c, result) && result[0] == 1);
    struct sockaddr_in endpoint = {0};
    endpoint.sin_family = AF_INET; endpoint.sin_port = htons((uint16_t)result[1]);
    endpoint.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
    void *callback = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_socket_accept_v1(listener, callback, &c, result) && result[0] == 5);
    uint64_t operation = result[1], root = 999;
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    CHECK(!neoclr_socket_poll_v1(&c, &root) && root == 999);
    CHECK(!neoclr_socket_accept_v1(listener, callback, &c, result) && result[0] == 2 && result[1] == 2);
    result[0] = result[1] = 99;
    CHECK(neoclr_socket_connect_result_v1(operation, &c, result) == 3 && result[0] == 99);
    int peer = socket(AF_INET, SOCK_STREAM, 0);
    CHECK(peer >= 0 && !connect(peer, (const void *)&endpoint, sizeof(endpoint)));
    int ready = 0;
    for (unsigned retry = 0; retry < 1000 && !ready; retry++) ready = neoclr_socket_poll_v1(&c, &root);
    CHECK(ready == 1);
    void *read = NULL;
    CHECK(!neoclr_gc_host_root_read_v1(&c, root, &read) && read == callback);
    CHECK(!neoclr_socket_poll_v1(&c, &root));
    int32_t cancelled = 99;
    CHECK(!neoclr_socket_cancel_v1(operation, &c, &cancelled) && !cancelled);
    CHECK(!neoclr_socket_connect_result_v1(operation, &c, result) && result[0] == 5);
    uint64_t accepted = result[1];
    CHECK(neoclr_socket_connect_result_v1(operation, &c, result) == 3);
    CHECK(neoclr_gc_host_root_read_v1(&c, root, &read) == 3);
    CHECK(!neoclr_socket_accept_v1(accepted, callback, &c, result) && result[0] == 2 && result[1] == 12);
    CHECK(!neoclr_socket_accept_v1(listener, callback, &c, result) && result[0] == 5);
    operation = result[1];
    CHECK(!neoclr_socket_cancel_v1(operation, &c, &cancelled) && cancelled);
    CHECK(!neoclr_socket_cancel_v1(operation, &c, &cancelled) && !cancelled);
    CHECK(neoclr_socket_poll_v1(&c, &root) == 1);
    CHECK(!neoclr_socket_connect_result_v1(operation, &c, result) && result[0] == 2 && result[1] == 5);
    CHECK(!neoclr_socket_accept_v1(listener, callback, &c, result) && result[0] == 5);
    operation = result[1];
    CHECK(!neoclr_socket_close_v1(listener, &c, result));
    CHECK(neoclr_socket_poll_v1(&c, &root) == 1);
    CHECK(!neoclr_socket_connect_result_v1(operation, &c, result) && result[0] == 2 && result[1] == 1);
    CHECK(!neoclr_socket_local_port_v1(accepted, &c, result) && result[0] == 1);
    CHECK(!neoclr_socket_listen_v1((const void *)&address, 0, 4, &c, result) && result[0] == 5);
    CHECK(!neoclr_socket_accept_v1(result[1], callback, &c, result) && result[0] == 5);
    CHECK(neoclr_gc_entry_check_v1(&c) == 3);
    c.fault.code = 4; /* Host teardown must also release an abandoned pending accept. */
    CHECK(neoclr_socket_poll_v1(&c, &root) == -3);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    c.fault.code = 0;
    close(peer);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used && !neoclr_gc_entry_check_v1(&c));
    CHECK(buffer[128] == 1234567);
    return 0;
}
