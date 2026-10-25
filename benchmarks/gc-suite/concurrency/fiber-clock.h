/* Diagnostic timestamps at both ends of a transfer on one owner thread.
 * No guest execution interval is charged to the carrier transfer. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static _Thread_local struct {
    uint64_t start_wall,start_cpu,wall,cpu,count,last_resumes;
    int pending;
} switch_profile;
static uint64_t switch_clock(clockid_t id) {
    struct timespec t;if(clock_gettime(id,&t)) abort();
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
static void switch_begin(void) {
    if(switch_profile.pending) abort();
    switch_profile.pending=1;
    switch_profile.start_cpu=switch_clock(CLOCK_THREAD_CPUTIME_ID);
    switch_profile.start_wall=switch_clock(CLOCK_MONOTONIC);
}
static void switch_arrive(void) {
    uint64_t wall=switch_clock(CLOCK_MONOTONIC),cpu=switch_clock(CLOCK_THREAD_CPUTIME_ID);
    if(!switch_profile.pending) abort();
    switch_profile.wall+=wall-switch_profile.start_wall;
    switch_profile.cpu+=cpu-switch_profile.start_cpu;
    ++switch_profile.count;switch_profile.pending=0;
}
static void switch_dump(uint64_t resumes) {
    if(switch_profile.pending || resumes<switch_profile.last_resumes ||
        switch_profile.count!=2*(resumes-switch_profile.last_resumes)) abort();
    fprintf(stderr,"FIBER_SWITCH %llu %llu %llu %llu\n",
        (unsigned long long)(resumes-switch_profile.last_resumes),
        (unsigned long long)switch_profile.count,
        (unsigned long long)switch_profile.wall,
        (unsigned long long)switch_profile.cpu);
    switch_profile.last_resumes=resumes;
    switch_profile.count=switch_profile.wall=switch_profile.cpu=0;
}
