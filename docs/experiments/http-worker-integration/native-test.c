#include "http-workers.h"
#include "socket-os.h"
#include <stdatomic.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed %d: %s\n", __LINE__, #x); abort(); } } while (0)
typedef struct { atomic_int arrivals, port; int count, mismatch, occupied; } trial;
static int entry(void *state) {
    trial *t = state;
    int ordinal = atomic_fetch_add(&t->arrivals, 1);
    int fd = -1;
    int status = neoclr_http_workers_listener(0x7f000001, t->occupied, 8 + (t->mismatch ? ordinal : 0), &fd);
    if (t->mismatch || t->occupied) {
        if (status < 0) return 1;
        while (!neoclr_http_workers_stopping()) sched_yield();
        CHECK(!close(fd));
        return 0;
    }
    CHECK(status == 1 && fd >= 0);
    struct sockaddr_in endpoint;
    socklen_t size = sizeof(endpoint);
    CHECK(!getsockname(fd, (struct sockaddr *)&endpoint, &size));
    int port = ntohs(endpoint.sin_port), expected = 0;
    if (!atomic_compare_exchange_strong(&t->port, &expected, port)) CHECK(expected == port);
    int untouched = 99;
    CHECK(neoclr_http_workers_listener(0x7f000001, 0, 8, &untouched) == 0 && untouched == 99);
    while (atomic_load(&t->arrivals) < t->count) sched_yield();
    CHECK(!close(fd));
    return 0;
}
int main(void) {
    CHECK(neoclr_http_workers_run(0, entry, NULL) == 2);
    CHECK(neoclr_http_workers_run(5, entry, NULL) == 2);
    CHECK(neoclr_http_workers_run(1, NULL, NULL) == 2);
    for (int repeat = 0; repeat < 2; repeat++) {
        for (int count = 1; count <= 4; count++) {
            trial t = {.count = count};
            CHECK(!neoclr_http_workers_run(count, entry, &t));
            CHECK(atomic_load(&t.arrivals) == count && atomic_load(&t.port));
        }
        trial mismatch = {.count = 4, .mismatch = 1};
        CHECK(neoclr_http_workers_run(4, entry, &mismatch) == 2);
        int occupied = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
        struct sockaddr_in endpoint = {.sin_family = AF_INET};
        endpoint.sin_addr.s_addr = htonl(0x7f000001);
        CHECK(occupied >= 0 && !bind(occupied, (struct sockaddr *)&endpoint, sizeof(endpoint)) && !listen(occupied, 8));
        socklen_t size = sizeof(endpoint);
        CHECK(!getsockname(occupied, (struct sockaddr *)&endpoint, &size));
        trial unavailable = {.count = 4, .occupied = ntohs(endpoint.sin_port)};
        CHECK(neoclr_http_workers_run(4, entry, &unavailable) == 2);
        CHECK(!close(occupied));
    }
    puts("PASS shared listener startup, independent duplicates, restart, mismatch and bind-failure teardown");
}
