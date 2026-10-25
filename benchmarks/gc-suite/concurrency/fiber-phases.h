/* Owner-local diagnostic intervals; clock overhead is included. */
typedef struct { uint64_t wall, cpu; } phase_stamp;
static _Thread_local struct { uint64_t count, wall, cpu; } phases[5];
static const char *phase_names[] = {"resume_prepare", "resume_finish", "yield_roots", "scheduler_acquire", "scheduler_wait"};
static phase_stamp phase_begin(void) {
    phase_stamp stamp;
    stamp.cpu = switch_clock(CLOCK_THREAD_CPUTIME_ID);
    stamp.wall = switch_clock(CLOCK_MONOTONIC);
    return stamp;
}
static void phase_end(unsigned id, phase_stamp stamp) {
    uint64_t wall = switch_clock(CLOCK_MONOTONIC);
    uint64_t cpu = switch_clock(CLOCK_THREAD_CPUTIME_ID);
    ++phases[id].count;
    phases[id].wall += wall - stamp.wall;
    phases[id].cpu += cpu - stamp.cpu;
}
static void phase_dump(void) {
    for (unsigned i = 0; i < 5; ++i) {
        fprintf(stderr, "FIBER_PHASE %s %llu %llu %llu\n", phase_names[i],
            (unsigned long long)phases[i].count,
            (unsigned long long)phases[i].wall,
            (unsigned long long)phases[i].cpu);
        phases[i].count = phases[i].wall = phases[i].cpu = 0;
    }
}
