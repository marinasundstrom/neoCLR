#include "socket-listener.h"
#include <string.h>
#define CHECK(t) do { if (!(t)) return __LINE__; } while (0)
static struct { uint64_t length; unsigned char bytes[32]; } address;
static const neoclr_aot_text *text(const char *value) {
    address.length = strlen(value); memcpy(address.bytes, value, address.length);
    return (const void *)&address;
}
int main(void) {
    neoclr_aot_context a = {0}, b = {0};
    neoclr_socket_scope first, second;
    uint64_t out[2] = {99, 99}, handle, port;
    CHECK(neoclr_socket_listen_v1(text("127.0.0.1"), 0, 4, &a, out) == 3 && out[0] == 99);
    CHECK(!neoclr_socket_scope_enter_v1(&first, &a));
    CHECK(neoclr_socket_scope_enter_v1(&first, &b) == 3);
    CHECK(neoclr_socket_scope_enter_v1(&second, &a) == 3);
    CHECK(neoclr_socket_listen_v1(NULL, 0, 4, &a, out) == 3 && out[0] == 99);
    for (unsigned i = 0; i < 8; i++) {
        const char *invalid[] = {"", "127.1", "127.00.0.1", "256.0.0.1", "127.0.0.1.", "::1", " 127.0.0.1", "1.2.3.4.5"};
        CHECK(!neoclr_socket_listen_v1(text(invalid[i]), -1, 0, &a, out) && out[0] == 2 && out[1] == 6);
    }
    const int ports[] = {-1, 65536, 0, 0};
    const int backlogs[] = {4, 4, 0, 129};
    for (unsigned i = 0; i < 4; i++) CHECK(!neoclr_socket_listen_v1(text("127.0.0.1"), ports[i], backlogs[i], &a, out) && out[0] == 2 && out[1] == 3);
    CHECK(!neoclr_socket_listen_v1(text("127.0.0.1"), 0, 4, &a, out) && out[0] == 5);
    handle = out[1];
    CHECK(!neoclr_socket_local_port_v1(handle, &a, out) && out[0] == 1 && out[1] > 0 && out[1] <= 65535);
    port = out[1];
    CHECK(!neoclr_socket_listen_v1(text("127.0.0.1"), (int)port, 4, &a, out) && out[0] == 2 && out[1] == 13);
    CHECK(!neoclr_socket_scope_enter_v1(&second, &b));
    CHECK(neoclr_socket_scope_leave_v1(&first) == 3);
    CHECK(!neoclr_socket_local_port_v1(handle, &b, out) && out[0] == 2 && out[1] == 1);
    CHECK(!neoclr_socket_close_v1(handle, &b, out) && out[0] == 0);
    CHECK(!neoclr_socket_local_port_v1(handle, &a, out) && out[0] == 1);
    CHECK(!neoclr_socket_scope_leave_v1(&second));
    CHECK(!neoclr_socket_close_v1(handle, &a, out) && out[0] == 0);
    CHECK(!neoclr_socket_close_v1(handle, &a, out) && out[0] == 0);
    CHECK(!neoclr_socket_local_port_v1(handle, &a, out) && out[0] == 2 && out[1] == 1);
    for (unsigned i = 0; i < 64; i++) {
        CHECK(!neoclr_socket_listen_v1(text("127.0.0.1"), 0, 4, &a, out) && out[0] == 5 && out[1] != handle);
#ifdef _WIN32
        DWORD flags;
        CHECK(GetHandleInformation((HANDLE)first.slots[i].descriptor, &flags) && !(flags & HANDLE_FLAG_INHERIT));
#else
        CHECK((fcntl(first.slots[i].descriptor, F_GETFL) & O_NONBLOCK) != 0);
        CHECK((fcntl(first.slots[i].descriptor, F_GETFD) & FD_CLOEXEC) != 0);
#endif
    }
    CHECK(!neoclr_socket_listen_v1(text("127.0.0.1"), 0, 4, &a, out) && out[0] == 2 && out[1] == 4);
    neoclr_socket_descriptor descriptors[64];
    for (unsigned i = 0; i < 64; i++) descriptors[i] = first.slots[i].descriptor;
    CHECK(!neoclr_socket_scope_leave_v1(&first));
    CHECK(!neoclr_os_start()); /* Keep Winsock initialized while checking closed handles. */
    for (unsigned i = 0; i < 64; i++) {
#ifdef _WIN32
        struct sockaddr_in endpoint;
        int size = sizeof(endpoint);
        CHECK(getsockname(descriptors[i], (struct sockaddr *)&endpoint, &size) == SOCKET_ERROR && WSAGetLastError() == WSAENOTSOCK);
#else
        errno = 0; CHECK(fcntl(descriptors[i], F_GETFD) == -1 && errno == EBADF);
#endif
    }
    CHECK(!neoclr_os_stop());
    CHECK(neoclr_socket_scope_leave_v1(&first) == 3);
    CHECK(!neoclr_socket_scope_enter_v1(&first, &a));
    CHECK(!neoclr_socket_listen_v1(text("127.0.0.1"), (int)port, 4, &a, out) && out[0] == 5);
    CHECK(!neoclr_socket_scope_leave_v1(&first));
    return 0;
}
