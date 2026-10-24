#define _POSIX_C_SOURCE 200809L
#include "gate.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) abort(); } while (0)
static double now(clockid_t id) {
    struct timespec t; CHECK(!clock_gettime(id,&t));
    return t.tv_sec + t.tv_nsec / 1e9;
}
__attribute__((noinline)) static void direct(uint64_t *p, uint64_t value) {
    p[1] = value;
}
int main(void) {
    const unsigned iterations = 10000000;
    unsigned modes[] = {0,1,2,2,1,0};
    for (unsigned trial = 0; trial < sizeof(modes)/sizeof(modes[0]); trial++) {
        unsigned mode = modes[trial];
        mutation_gate g; uint64_t epoch = 0, data[] = {17,0}, words[GATE_WORDS]; size_t count;
        CHECK(!gate_init(&g));
        if (mode == 2) CHECK(!gate_begin(&g,&epoch));
        double wall = now(CLOCK_MONOTONIC), cpu = now(CLOCK_PROCESS_CPUTIME_ID);
        for (uint64_t i = 1; i <= iterations; i++) {
            if (!mode) direct(data,i);
            else {
                mutation_patch patch = {8,8,&i};
                CHECK(!gate_write(&g,data,sizeof(data),8,&patch,1));
                if (mode == 2) CHECK(!gate_drain(&g,epoch,words,GATE_WORDS,&count) && count == 2);
            }
        }
        cpu = now(CLOCK_PROCESS_CPUTIME_ID) - cpu;
        wall = now(CLOCK_MONOTONIC) - wall;
        CHECK(data[1] == iterations);
        if (mode == 2) CHECK(!gate_stop(&g));
        CHECK(!gate_destroy(&g));
        printf("{\"mode\":%u,\"iterations\":%u,\"wallSeconds\":%.9f,\"cpuSeconds\":%.9f}\n",mode,iterations,wall,cpu);
    }
}
