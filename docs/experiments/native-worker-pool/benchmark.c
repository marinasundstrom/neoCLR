#define _POSIX_C_SOURCE 200809L
#include "native-workers.h"
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) abort(); } while (0)
static uint64_t compute(uint64_t seed) {
    for (unsigned i = 0; i < 20000000; i++) {
        seed ^= seed << 13; seed ^= seed >> 7; seed ^= seed << 17;
    }
    return seed & INT64_MAX;
}
static int work(neoclr_worker_job *j, void *unused) {
    (void)unused;
    neoclr_worker_enter(j);
    uint64_t seed = *(uint64_t *)neoclr_worker_capture(j);
    neoclr_worker_leave(j);
    uint64_t result = compute(seed);
    neoclr_worker_enter(j);
    neoclr_worker_result(j, (neoclr_worker_value){1, (int64_t)result, NULL});
    neoclr_worker_leave(j);
    return 0;
}
static double monotonic(void) {
    struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC, &t));
    return t.tv_sec + t.tv_nsec / 1e9;
}
int main(void) {
    unsigned order[] = {1, 2, 4, 4, 2, 1};
    uint64_t expected = compute(42);
    for (unsigned trial = 0; trial < 6; trial++) {
        neoclr_workers *p = neoclr_workers_create(order[trial], 8, 8192);
        neoclr_worker_handle root, tickets[8], ignored;
        CHECK(p && neoclr_workers_root_new(p, 42, &root));
        clock_t cpu_start = clock(); double start = monotonic();
        for (unsigned i = 0; i < 8; i++) CHECK(neoclr_workers_submit(p, root, work, NULL, &tickets[i]));
        for (unsigned i = 0; i < 8; i++) {
            neoclr_worker_value value; int outcome;
            CHECK(neoclr_workers_wait(p, tickets[i], 10000) == 1);
            CHECK(neoclr_workers_take(p, tickets[i], &outcome, &value, &ignored) == 1);
            CHECK(!outcome && value.tag == 1 && (uint64_t)value.scalar == expected && !ignored.pool);
        }
        double elapsed = monotonic() - start;
        double cpu = (double)(clock() - cpu_start) / CLOCKS_PER_SEC;
        CHECK(neoclr_workers_root_release(p, root));
        CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
        printf("{\"workers\":%u,\"jobs\":8,\"wallSeconds\":%.6f,\"cpuSeconds\":%.6f}\n", order[trial], elapsed, cpu);
    }
}
