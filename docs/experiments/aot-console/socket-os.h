#ifndef NEOCLR_SOCKET_OS_H
#define NEOCLR_SOCKET_OS_H
/* Private OS boundary. Socket identity and operation/root policy stay in the
 * shared adapter; this header only translates handles, errors and system calls. */
#include <stdint.h>
#ifdef _WIN32
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <winsock2.h>
#include <windows.h>
typedef SOCKET neoclr_socket_descriptor;
typedef int neoclr_socklen;
#define NEOCLR_INVALID_SOCKET INVALID_SOCKET
static inline int neoclr_os_start(void) {
    WSADATA data;
    int status = WSAStartup(MAKEWORD(2, 2), &data);
    if (status) return status;
    if (data.wVersion != MAKEWORD(2, 2)) { WSACleanup(); return WSAVERNOTSUPPORTED; }
    return 0;
}
static inline int neoclr_os_stop(void) { return WSACleanup(); }
static inline int neoclr_os_error(void) { return WSAGetLastError(); }
static inline int neoclr_os_retry(int error) { return error == WSAEWOULDBLOCK || error == WSAEINTR; }
static inline int neoclr_os_close(neoclr_socket_descriptor fd) { return closesocket(fd); }
static inline int neoclr_os_configure(neoclr_socket_descriptor fd) {
    u_long one = 1;
    if (ioctlsocket(fd, FIONBIO, &one)) return -1;
    if (!SetHandleInformation((HANDLE)fd, HANDLE_FLAG_INHERIT, 0)) {
        WSASetLastError(WSAEACCES); return -1;
    }
    return 0;
}
static inline int64_t neoclr_os_transfer(neoclr_socket_descriptor fd, unsigned char *buffer,
                                       uint32_t count, int sending) {
    return sending ? send(fd, (const char *)buffer, (int)count, 0) : recv(fd, (char *)buffer, (int)count, 0);
}
static inline uint64_t neoclr_os_monotonic_ns(void) {
    LARGE_INTEGER counter, frequency;
    if (!QueryPerformanceCounter(&counter) || !QueryPerformanceFrequency(&frequency)
        || counter.QuadPart < 0 || frequency.QuadPart <= 0
        || (uint64_t)frequency.QuadPart > UINT64_MAX / UINT64_C(1000000000)) return 0;
    uint64_t ticks = (uint64_t)counter.QuadPart, hz = (uint64_t)frequency.QuadPart;
    return (ticks / hz) * UINT64_C(1000000000) + (ticks % hz) * UINT64_C(1000000000) / hz;
}
static inline void neoclr_os_pause(void) { Sleep(1); }
static inline uint64_t neoclr_os_error_code(int error) {
    switch (error) {
        case WSAECONNREFUSED: return 7;
        case WSAECONNRESET: return 8;
        case WSAEACCES: return 9;
        case WSAETIMEDOUT: return 10;
        case WSAEADDRINUSE: return 13;
        default: return 11;
    }
}
#else
#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <unistd.h>
#include <time.h>
typedef int neoclr_socket_descriptor;
typedef socklen_t neoclr_socklen;
#define NEOCLR_INVALID_SOCKET (-1)
static inline int neoclr_os_start(void) { return 0; }
static inline int neoclr_os_stop(void) { return 0; }
static inline int neoclr_os_error(void) { return errno; }
static inline int neoclr_os_retry(int error) { return error == EAGAIN || error == EWOULDBLOCK || error == EINTR; }
static inline int neoclr_os_close(neoclr_socket_descriptor fd) { return close(fd); }
static inline int neoclr_os_configure(neoclr_socket_descriptor fd) {
    int flags = fcntl(fd, F_GETFL), fdflags = fcntl(fd, F_GETFD);
    return flags < 0 || fdflags < 0 || fcntl(fd, F_SETFL, flags | O_NONBLOCK) < 0 ||
        fcntl(fd, F_SETFD, fdflags | FD_CLOEXEC) < 0 ? -1 : 0;
}
static inline int64_t neoclr_os_transfer(neoclr_socket_descriptor fd, unsigned char *buffer,
                                       uint32_t count, int sending) {
#ifdef SO_NOSIGPIPE
    int one = 1;
    if (sending && setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &one, sizeof(one))) return -2;
#endif
    int flags = 0;
#ifdef MSG_NOSIGNAL
    flags = MSG_NOSIGNAL;
#endif
    return sending ? send(fd, buffer, count, flags) : recv(fd, buffer, count, 0);
}
static inline uint64_t neoclr_os_monotonic_ns(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now)) return 0;
    return (uint64_t)now.tv_sec * UINT64_C(1000000000) + (uint64_t)now.tv_nsec;
}
static inline void neoclr_os_pause(void) { const struct timespec pause = {0, 1000000}; nanosleep(&pause, NULL); }
static inline uint64_t neoclr_os_error_code(int error) {
    switch (error) {
        case ECONNREFUSED: return 7;
        case ECONNRESET: case EPIPE: return 8;
        case EACCES: case EPERM: return 9;
        case ETIMEDOUT: return 10;
        case EADDRINUSE: return 13;
        default: return 11;
    }
}
#endif
#endif
