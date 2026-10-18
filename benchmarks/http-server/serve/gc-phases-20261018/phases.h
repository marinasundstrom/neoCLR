/* Diagnostic only: owner-thread GC phase timing, sampled without allocation. */
#include <time.h>
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
#ifndef GC_PHASE_SAMPLE
#define GC_PHASE_SAMPLE 64
#endif
static uint64_t phase_calls, phase_samples, phase_errors;
static uint64_t phase_wall[6], phase_cpu[6], phase_last_wall, phase_last_cpu;
static unsigned phase_current;
static int phase_active;
static uint64_t phase_stamp(clockid_t clock) {
    struct timespec t;
    if (clock_gettime(clock, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
static void phase_next(unsigned next) {
    if (!phase_active) return;
    uint64_t wall = phase_stamp(CLOCK_MONOTONIC);
    uint64_t cpu = phase_stamp(CLOCK_THREAD_CPUTIME_ID);
    phase_wall[phase_current] += wall - phase_last_wall;
    phase_cpu[phase_current] += cpu - phase_last_cpu;
    phase_last_wall = wall;
    phase_last_cpu = cpu;
    phase_current = next;
}
static void phase_begin(void) {
    phase_calls++;
    phase_active = phase_calls % GC_PHASE_SAMPLE == 0;
    if (!phase_active) return;
    phase_samples++;
    phase_current = 0;
    phase_last_wall = phase_stamp(CLOCK_MONOTONIC);
    phase_last_cpu = phase_stamp(CLOCK_THREAD_CPUTIME_ID);
}
static void phase_end(int status) {
    phase_errors += status != 0;
    phase_next(0);
    phase_active = 0;
}
static void phase_report(void) {
    const char *path = getenv("NEOCLR_GC_PHASES");
    if (!path) abort();
    FILE *f = fopen(path, "w");
    if (!f) abort();
    fprintf(f, "{\"stride\":%u,\"calls\":%"PRIu64",\"samples\":%"PRIu64",\"errors\":%"PRIu64",\"phases\":[", GC_PHASE_SAMPLE, phase_calls, phase_samples, phase_errors);
    const char *names[] = {"index", "validate-roots", "seed-roots", "trace", "cleanup", "sweep"};
    for (unsigned i = 0; i < 6; i++)
        fprintf(f, "%s{\"name\":\"%s\",\"wallNs\":%"PRIu64",\"cpuNs\":%"PRIu64"}", i ? "," : "", names[i], phase_wall[i], phase_cpu[i]);
    fputs("]}\n", f);
    if (fclose(f)) abort();
}
__attribute__((constructor)) static void phase_initialize(void) {
    if (atexit(phase_report)) abort();
}
