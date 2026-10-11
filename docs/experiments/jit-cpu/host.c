/* Private scalar benchmark host, using the same entry ABI as the JIT. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include "../aot-scalar/abi.h"
static double now(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value)) abort();
    return (double)value.tv_sec + (double)value.tv_nsec / 1000000000.0;
}
int main(int argc, char **argv) {
    if (argc != 3) return 2;
    char *end;
    long count = strtol(argv[1], &end, 10);
    if (end == argv[1] || *end || count < 0 || count > 1000000) return 2;
    long repetitions = strtol(argv[2], &end, 10);
    if (end == argv[2] || *end || repetitions < 1 || repetitions > 1000) return 2;
    printf("{\"invocations\":[");
    for (long i = 0; i < repetitions; ++i) {
        int32_t value = 0;
        double start = now();
        int32_t status = neoclr_entry_v2((int32_t)count, &value);
        double elapsed = now() - start;
        printf("%s{\"status\":%d,\"value\":%d,\"invocationSeconds\":%.9f}", i ? "," : "", status, value, elapsed);
    }
    puts("]}");
    return 0;
}
