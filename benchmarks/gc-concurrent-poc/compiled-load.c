/* Fixed offered arrivals; queue delay is measured even when execution falls behind.
 * One arrival is a batch of eight callbacks, not an HTTP request. */
#include <assert.h>
#include <errno.h>
#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>
static uint64_t ns(clockid_t clock);
enum { BASE_RATE = 3000, BURST_RATE = 6000, TOTAL = 21000, BIN_COUNT = 40 };
static uint64_t response[TOTAL], peak_queue, recovery, cpu_previous;
static uint64_t bin_cpu[BIN_COUNT], bin_completed[BIN_COUNT];
static unsigned current_bin, completed_count;
static uint64_t due(unsigned i) {
    if (i < 6000) return ((uint64_t)i * 1000000000 + BASE_RATE - 1) / BASE_RATE;
    if (i < 12000) return 2000000000 + ((uint64_t)(i - 6000) * 1000000000 + BURST_RATE - 1) / BURST_RATE;
    return 3000000000 + ((uint64_t)(i - 12000) * 1000000000 + BASE_RATE - 1) / BASE_RATE;
}
static uint64_t arrivals(uint64_t t) {
    uint64_t count;
    if (t < 2000000000) count = t * BASE_RATE / 1000000000 + 1;
    else if (t < 3000000000) count = 6000 + (t - 2000000000) * BURST_RATE / 1000000000 + 1;
    else count = 12000 + (t - 3000000000) * BASE_RATE / 1000000000 + 1;
    return count < TOTAL ? count : TOTAL;
}
static void arrival(unsigned i, uint64_t start) {
    assert(i < TOTAL && arrivals(due(i)) == i + 1);
    if (!i) cpu_previous = ns(CLOCK_PROCESS_CPUTIME_ID);
    uint64_t deadline = start + due(i), now;
    while ((now = ns(CLOCK_MONOTONIC)) < deadline) {
        uint64_t left = deadline - now;
        struct timespec delay = {left / 1000000000, left % 1000000000};
        assert(!nanosleep(&delay, NULL) || errno == EINTR);
    }
    uint64_t waiting = arrivals(now - start) - i;
    if (waiting > peak_queue) peak_queue = waiting;
}
static void completed(unsigned i, uint64_t start, uint64_t end) {
    assert(i == completed_count++);
    uint64_t elapsed = end - start;
    response[i] = elapsed - due(i);
    unsigned bin = elapsed / 500000000;
    assert(bin < BIN_COUNT); /* Fail rather than silently truncate a stalled run. */
    uint64_t cpu = ns(CLOCK_PROCESS_CPUTIME_ID);
    bin_cpu[bin] += cpu - cpu_previous;
    cpu_previous = cpu;
    bin_completed[bin]++;
    current_bin = bin;
    if (!recovery && elapsed >= 3000000000 && arrivals(elapsed) == i + 1)
        recovery = elapsed - 3000000000;
}
static int order(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}
static void report_load(void) {
    assert(completed_count == TOTAL);
    qsort(response, TOTAL, sizeof(*response), order);
    printf("{\"offeredBatches\":%u,\"baseBatchesPerSecond\":%u,\"burstBatchesPerSecond\":%u,\"responseP99Ns\":%llu,\"maxResponseNs\":%llu,\"peakQueuedIncludingActive\":%llu,\"recoveryAfterBurstNs\":%llu,\"bins\":[",
        TOTAL, BASE_RATE, BURST_RATE, (unsigned long long)response[TOTAL * 99 / 100 - 1],
        (unsigned long long)response[TOTAL - 1], (unsigned long long)peak_queue, (unsigned long long)recovery);
    for (unsigned i = 0; i <= current_bin; i++)
        printf("%s{\"startMs\":%u,\"completedBatches\":%llu,\"cpuNs\":%llu}", i ? "," : "", i * 500,
               (unsigned long long)bin_completed[i], (unsigned long long)bin_cpu[i]);
    puts("]}");
}
#define LOAD_ARRIVAL(i, start) arrival(i, start)
#define LOAD_COMPLETE(i, start, end) completed(i, start, end)
#define LOAD_REPORT() report_load()
#include "compiled-callback.c"
