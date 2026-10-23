/* Include the diagnostic implementation to test its bounded private table. */
#include "../../serve/profile-native.c"
#include <assert.h>
#include <pthread.h>
#include <string.h>
static uint64_t expected;
neoclr_gc_profile neoclr_gc_profile_v1(void) { return (neoclr_gc_profile){0}; }
neoclr_gc_statistics neoclr_gc_statistics_v1(void) { return (neoclr_gc_statistics){0}; }
uint64_t neoclr_root_probe_count_v1(void) { return expected; }
static void *other_thread(void *ignored) {
    (void)ignored;
    for (unsigned i = 0; i < SITE_CAPACITY; i++) assert(!sites[i].count);
    neoclr_gc_site_profile_v1(7, 9);
    return NULL;
}
int main(int argc, char **argv) {
    assert(argc == 2);
    if (!strcmp(argv[1], "function")) neoclr_gc_site_profile_v1(NEOCLR_NATIVE_FUNCTION_LIMIT, 0);
    else if (!strcmp(argv[1], "instruction")) neoclr_gc_site_profile_v1(0, 8192);
    else {
        /* Fill the table with distinct keys, exercising collision resolution and repeated hits. */
        for (unsigned i = 0; i < SITE_CAPACITY; i++) {
            neoclr_gc_site_profile_v1(i / 8192, i % 8192);
            neoclr_gc_site_profile_v1(i / 8192, i % 8192);
        }
        if (!strcmp(argv[1], "capacity")) neoclr_gc_site_profile_v1(3, 0);
        else if (!strcmp(argv[1], "overflow")) {
            sites[0].count = UINT64_MAX;
            neoclr_gc_site_profile_v1(sites[0].function, sites[0].instruction);
        } else {
            pthread_t thread;
            assert(!pthread_create(&thread, NULL, other_thread, NULL));
            assert(!pthread_join(thread, NULL));
            expected = SITE_CAPACITY * 2;
            if (!strcmp(argv[1], "mismatch")) expected++;
            FILE *file = tmpfile(); assert(file);
            report_sites(file);
            assert(!fclose(file));
            for (unsigned i = 0; i < SITE_CAPACITY; i++) {
                assert(sites[i].count == 2);
                assert(sites[i].function == i / 8192 && sites[i].instruction == i % 8192);
            }
            assert(!strcmp(argv[1], "valid"));
            return 0;
        }
    }
    return 1; /* Negative cases must abort, not merely return failure. */
}
