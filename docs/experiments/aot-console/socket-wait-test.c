/* POSIX readiness is a hint only: no reads, delivery, root transfer or collection.
 * Intercept poll to check its bound without timing-sensitive sleep assertions. */
#include <poll.h>
#include <assert.h>
#include <stdio.h>
static int observed, expected_fd, expected_event, result_mode;
static int wait_test_poll(struct pollfd *, nfds_t, int);
#define poll wait_test_poll
#include "socket-listener.c"
#undef poll
static int wait_test_poll(struct pollfd *fds, nfds_t count, int timeout) {
    ++observed;
    assert(count == 1 && timeout == 1);
    assert(fds[0].fd == expected_fd && fds[0].events == expected_event);
    if (result_mode == 1) { errno = EINTR; return -1; }
    return poll(fds, count, 0);
}
int main(void) {
    uint64_t arena[256] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)arena, sizeof(arena), 0}};
    neoclr_socket_scope scope;
    assert(!neoclr_socket_scope_enter_v1(&scope, &c));
    int pair[2]; assert(!socketpair(AF_UNIX, SOCK_STREAM, 0, pair));
    /* A large descriptor is legal for poll even when select cannot represent it. */
    int fd = fcntl(pair[0], F_DUPFD, 1024); assert(fd >= 1024);
    close(pair[0]); expected_fd = fd;
    scope.slots[0].id = 7; scope.slots[0].descriptor = fd;
    scope.operations[0].state = 1; scope.operations[0].kind = 1;
    scope.operations[0].listener = 7;
    expected_event = POLLIN;
    assert(write(pair[1], "x", 1) == 1);
    uint64_t collections = neoclr_gc_statistics_v1().collections;
    neoclr_scheduler_wait_v1(&c);
    assert(observed == 1 && scope.operations[0].state == 1 && !scope.scheduler_delivery);
    char value; assert(recv(fd, &value, 1, MSG_PEEK) == 1 && value == 'x');
    assert(neoclr_gc_statistics_v1().collections == collections);
    for (unsigned kind = 2; kind <= 3; ++kind) {
        scope.operations[0].kind = kind;
        scope.operations[0].deadline = monotonic_ns() + UINT64_C(1000000000);
        expected_event = kind == 2 ? POLLIN : POLLOUT;
        neoclr_scheduler_wait_v1(&c);
    }
    assert(observed == 3);
    scope.operations[0].kind = 4; scope.operations[0].connecting = fd;
    neoclr_scheduler_wait_v1(&c); assert(observed == 4);
    result_mode = 1; neoclr_scheduler_wait_v1(&c); assert(observed == 5);
    result_mode = 0;
    scope.operations[0].state = 2;
    neoclr_scheduler_wait_v1(&c); assert(observed == 5);
    scope.operations[0].state = 1; scope.operations[0].deadline = 1;
    neoclr_scheduler_wait_v1(&c); assert(observed == 5);
    scope.operations[0].deadline = UINT64_MAX;
    scope.scheduler_delivery = 99;
    neoclr_scheduler_wait_v1(&c); assert(observed == 5);
    scope.scheduler_delivery = 0;
    neoclr_runnable queued = {0}; scope.runnable_head = &queued;
    neoclr_scheduler_wait_v1(&c); assert(observed == 5);
    scope.runnable_head = NULL;
    memset(&scope.operations[0], 0, sizeof(scope.operations[0]));
    memset(&scope.slots[0], 0, sizeof(scope.slots[0]));
    close(fd); close(pair[1]);
    assert(!neoclr_socket_scope_leave_v1(&scope));
    puts("PASS readiness directions, high descriptor, EINTR, deadlines and ownership");
}
