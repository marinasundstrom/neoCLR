#ifndef NEOCLR_TRACE_CLOCK_H
#define NEOCLR_TRACE_CLOCK_H
#include <stdint.h>
#ifdef __APPLE__
#include <mach/mach_time.h>
#define NEOCLR_TRACE_CLOCK_NAME "mach_absolute_time/ns"
static inline uint64_t neoclr_trace_clock_ns(void) {
    static _Thread_local mach_timebase_info_data_t scale;
    if (!scale.denom && mach_timebase_info(&scale) != KERN_SUCCESS) return 0;
    return (uint64_t)(((__uint128_t)mach_absolute_time() * scale.numer) / scale.denom);
}
#else
#include <time.h>
#define NEOCLR_TRACE_CLOCK_NAME "CLOCK_MONOTONIC"
static inline uint64_t neoclr_trace_clock_ns(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC, &t)) return 0;
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
#endif
#endif
