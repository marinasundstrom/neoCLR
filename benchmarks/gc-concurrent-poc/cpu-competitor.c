/* Bounded arithmetic contention; owns no runtime state and allocates no hot data. */
#include <assert.h>
#include <signal.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
static volatile sig_atomic_t stop;
static void stopped(int signal_number) { (void)signal_number; stop = 1; }
static uint64_t ns(clockid_t clock) {
    struct timespec t;
    assert(!clock_gettime(clock, &t));
    return (uint64_t)t.tv_sec * 1000000000 + t.tv_nsec;
}
int main(int argc, char **argv) {
    assert(argc == 2);
    unsigned duration = (unsigned)strtoul(argv[1], NULL, 10);
    assert(duration >= 100 && duration <= 60000);
    assert(signal(SIGTERM, stopped) != SIG_ERR);
    uint64_t start = ns(CLOCK_MONOTONIC), cpu = ns(CLOCK_PROCESS_CPUTIME_ID);
    uint64_t value = 1, iterations = 0;
    puts("ready"); fflush(stdout);
    while (!stop && ns(CLOCK_MONOTONIC) - start < (uint64_t)duration * 1000000) {
        for (unsigned i = 0; i < 65536; i++) value = value * UINT64_C(6364136223846793005) + 1;
        iterations += 65536;
    }
    printf("{\"elapsedNs\":%llu,\"cpuNs\":%llu,\"iterations\":%llu,\"checksum\":%llu}\n",
        (unsigned long long)(ns(CLOCK_MONOTONIC) - start),
        (unsigned long long)(ns(CLOCK_PROCESS_CPUTIME_ID) - cpu),
        (unsigned long long)iterations, (unsigned long long)value);
    return 0;
}
