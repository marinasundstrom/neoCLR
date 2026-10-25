/* Diagnostic acquisition phases; only used by a copied socket host. */
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
typedef struct { uint64_t wall, cpu; } scheduler_stamp;
static _Thread_local struct { uint64_t count, wall, cpu; } scheduler_phases[3];
static uint64_t scheduler_clock(clockid_t id) {
    struct timespec t;
    if (clock_gettime(id, &t)) abort();
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
static scheduler_stamp scheduler_begin(void) {
    scheduler_stamp s;
    s.cpu = scheduler_clock(CLOCK_THREAD_CPUTIME_ID);
    s.wall = scheduler_clock(CLOCK_MONOTONIC);
    return s;
}
static void scheduler_end(unsigned id, scheduler_stamp s) {
    uint64_t wall = scheduler_clock(CLOCK_MONOTONIC);
    uint64_t cpu = scheduler_clock(CLOCK_THREAD_CPUTIME_ID);
    ++scheduler_phases[id].count;
    scheduler_phases[id].wall += wall - s.wall;
    scheduler_phases[id].cpu += cpu - s.cpu;
}
void neoclr_diagnostic_scheduler_dump(void) {
    const char *names[] = {"poll", "idle_gc", "delivery_roots"};
    for (unsigned i = 0; i < 3; ++i) {
        fprintf(stderr, "SCHEDULER_PHASE %s %llu %llu %llu\n", names[i],
            (unsigned long long)scheduler_phases[i].count,
            (unsigned long long)scheduler_phases[i].wall,
            (unsigned long long)scheduler_phases[i].cpu);
        scheduler_phases[i].count = scheduler_phases[i].wall = scheduler_phases[i].cpu = 0;
    }
}
