/* Diagnostic-only Windows worker counters, never linked into shipping hosts. */
#include "socket-os.h"
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static uint64_t started, calls, ready, empty, errors, polling, pauses, waiting;
uint64_t neoclr_wait_profile_now(void) { return neoclr_os_monotonic_ns(); }
void neoclr_wait_profile_poll(uint64_t start, int32_t result) {
    polling += neoclr_wait_profile_now() - start;
    calls++;
    if (result == 1) ready++;
    else if (!result) empty++;
    else errors++;
}
void neoclr_wait_profile_pause(void) {
    uint64_t start = neoclr_wait_profile_now();
    neoclr_os_pause();
    waiting += neoclr_wait_profile_now() - start;
    pauses++;
}
static uint64_t ticks(FILETIME time) {
    return ((uint64_t)time.dwHighDateTime << 32) | time.dwLowDateTime;
}
static void report(void) {
    FILETIME created, exited, kernel, user;
    if (!GetProcessTimes(GetCurrentProcess(), &created, &exited, &kernel, &user)) abort();
    const char *path = getenv("NEOCLR_WAIT_PROFILE_OUTPUT");
    if (!path) abort();
    FILE *file = fopen(path, "w");
    if (!file) abort();
    neoclr_gc_profile p = neoclr_gc_profile_v1();
    neoclr_gc_statistics s = neoclr_gc_statistics_v1();
    fprintf(file, "{\"wallNanoseconds\":%" PRIu64 ",\"processCpuNanoseconds\":%" PRIu64
        ",\"polls\":%" PRIu64 ",\"ready\":%" PRIu64 ",\"empty\":%" PRIu64
        ",\"errors\":%" PRIu64 ",\"pollNanoseconds\":%" PRIu64
        ",\"pauses\":%" PRIu64 ",\"pauseNanoseconds\":%" PRIu64
        ",\"gcNanoseconds\":%" PRIu64 ",\"collections\":%" PRIu64
        ",\"gcFailures\":%" PRIu64 ",\"allocations\":%" PRIu64
        ",\"reclaimed\":%" PRIu64 "}\n",
        neoclr_wait_profile_now() - started, (ticks(kernel) + ticks(user)) * 100,
        calls, ready, empty, errors, polling, pauses, waiting,
        p.nanoseconds, p.attempts, p.failures, s.allocations, s.reclaimed_allocations);
    if (fclose(file)) abort();
}
void neoclr_wait_profile_start(void) {
    started = neoclr_wait_profile_now();
    if (!started || atexit(report)) abort();
}
