/* Diagnostic service microbenchmark: excludes compilation, guest dispatch and GC.
 * The linear pool is deliberately a bounded baseline, not an optimized hash table. */
#include "native-gc.h"
#include <stdio.h>
#include <time.h>
static uint64_t storage[131072];
static struct { uint64_t length; unsigned char bytes[16]; } keys[4096];
static double seconds(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (double)now.tv_sec + (double)now.tv_nsec * 1e-9;
}
int main(void) {
    neoclr_aot_context context = { .text = { (unsigned char *)storage, sizeof(storage), 0 } };
    const neoclr_aot_text *result = NULL;
    puts("entries,last_hit_ns,iterations");
    for (unsigned count = 1; count <= 4096; count *= 16) {
        context.text.used = 0;
        for (unsigned i = 0; i < count; ++i) {
            keys[i].length = (uint64_t)snprintf((char *)keys[i].bytes, sizeof(keys[i].bytes), "key-%u", i);
            if (neoclr_string_intern_v1((const neoclr_aot_text *)&keys[i], &context.text, &result)) return 1;
        }
        double start = seconds();
        for (unsigned i = 0; i < 10000; ++i)
            if (neoclr_string_intern_v1((const neoclr_aot_text *)&keys[count-1], &context.text, &result)) return 2;
        double elapsed = seconds() - start;
        if (result != (const neoclr_aot_text *)&keys[count-1]) return 3;
        printf("%u,%.1f,10000\n", count, elapsed * 1e9 / 10000);
    }
}
