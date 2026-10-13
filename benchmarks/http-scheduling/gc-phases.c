/* Diagnostic-only phase timing; no collector policy or ownership changes. */
#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
extern uint64_t neoclr_schedule_profile_now(void);
static uint64_t phases[5], stamp, cycles, failures, previous_allocations;
static uint64_t no_new_allocations, boundary_calls, boundary_ns;
static unsigned phase;
static int active;
void neoclr_gc_phase_begin(uint64_t allocations) {
    if (active) abort();
    if (cycles && allocations == previous_allocations) no_new_allocations++;
    previous_allocations = allocations;
    cycles++;
    active = 1; phase = 0;
    stamp = neoclr_schedule_profile_now();
}
void neoclr_gc_phase_next(unsigned next) {
    if (!active || next != phase + 1 || next >= 5) abort();
    uint64_t now = neoclr_schedule_profile_now();
    phases[phase] += now - stamp;
    phase = next; stamp = now;
}
void neoclr_gc_phase_end(int32_t status) {
    if (!active || (!status && phase != 4)) abort();
    phases[phase] += neoclr_schedule_profile_now() - stamp;
    failures += status != 0;
    active = 0;
}
void neoclr_gc_boundary_end(uint64_t started) {
    boundary_calls++;
    boundary_ns += neoclr_schedule_profile_now() - started;
}
static void report(void) {
    const char *path = getenv("NEOCLR_GC_PHASE_OUTPUT");
    if (!path) return;
    if (active) abort();
    FILE *f = fopen(path, "w");
    if (!f) abort();
    fprintf(f, "{\"collections\":%" PRIu64 ",\"failures\":%" PRIu64
        ",\"noInterveningAllocations\":%" PRIu64 ",\"boundaryCalls\":%" PRIu64
        ",\"boundaryValidationNanoseconds\":%" PRIu64
        ",\"indexValidationNanoseconds\":%" PRIu64 ",\"rootSeedingNanoseconds\":%" PRIu64
        ",\"tracingNanoseconds\":%" PRIu64 ",\"ownerCleanupNanoseconds\":%" PRIu64
        ",\"sweepNanoseconds\":%" PRIu64 "}\n",
        cycles, failures, no_new_allocations, boundary_calls, boundary_ns,
        phases[0], phases[1], phases[2], phases[3], phases[4]);
    if (fclose(f)) abort();
}
__attribute__((constructor)) static void initialize(void) { if (atexit(report)) abort(); }
