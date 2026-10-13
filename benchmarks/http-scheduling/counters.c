/* Diagnostic-only counters. This translation unit is never in a shipping host. */
#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/resource.h>
static uint64_t calls, ready, empty, errors, elapsed, streak, longest, pauses, paused;
uint64_t neoclr_schedule_profile_now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + t.tv_nsec;
}
void neoclr_schedule_profile_poll(uint64_t start, int32_t result) {
    elapsed += neoclr_schedule_profile_now() - start;
    calls++;
    if (result == 1) { ready++; streak = 0; }
    else if (!result) { empty++; if (++streak > longest) longest = streak; }
    else { errors++; streak = 0; }
}
void neoclr_schedule_profile_paused(uint64_t start) {
    pauses++;
    paused += neoclr_schedule_profile_now() - start;
}
static void report(void) {
    const char *path = getenv("NEOCLR_SCHEDULING_PROFILE_OUTPUT");
    if (!path) return;
    struct rusage usage;
    if (getrusage(RUSAGE_SELF, &usage)) abort();
    FILE *file = fopen(path, "w");
    if (!file) abort();
    fprintf(file, "{\"polls\":%" PRIu64 ",\"ready\":%" PRIu64
        ",\"empty\":%" PRIu64 ",\"errors\":%" PRIu64
        ",\"pollNanoseconds\":%" PRIu64 ",\"longestEmptyStreak\":%" PRIu64
        ",\"pauses\":%" PRIu64 ",\"pauseNanoseconds\":%" PRIu64
        ",\"processUserSeconds\":%.6f,\"processSystemSeconds\":%.6f}\n",
        calls, ready, empty, errors, elapsed, longest, pauses, paused,
        usage.ru_utime.tv_sec + usage.ru_utime.tv_usec / 1e6,
        usage.ru_stime.tv_sec + usage.ru_stime.tv_usec / 1e6);
    if (fclose(file)) abort();
}
__attribute__((constructor)) static void initialize(void) { if (atexit(report)) abort(); }
