/* Private diagnostic accounting must not alter heap ownership or collection. */
#include "native-gc.h"
#include <string.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t memory[64] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)memory, sizeof(memory), 0}};
    neoclr_gc_profile_reset_v1();
    void *live, *dead;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_TEXT, &live));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_TEXT, &dead));
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_v1(&context, live, &handle));
    CHECK(!neoclr_gc_collect_v1(&context, NULL));
    neoclr_gc_profile measured = neoclr_gc_profile_v1();
#ifdef NEOCLR_GC_PROFILE
    CHECK(measured.allocated_bytes == 32 && measured.live_bytes == 16 && measured.live_blocks == 1);
    CHECK(measured.peak_live_bytes == 16 && measured.peak_arena_used > 32);
    CHECK(measured.attempts == 1 && measured.failures == 0);
#endif
    uint64_t used = context.text.used;
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    neoclr_gc_profile_reset_v1();
    CHECK(context.text.used == used);
    CHECK(neoclr_gc_statistics_v1().collections == before.collections);
    CHECK(!neoclr_gc_collect_v1(&context, NULL));
    CHECK(neoclr_gc_collect_v1(NULL, NULL) != 0);
    measured = neoclr_gc_profile_v1();
#ifdef NEOCLR_GC_PROFILE
    CHECK(measured.allocated_bytes == 0 && measured.peak_arena_used == 0);
    CHECK(measured.live_bytes == 16 && measured.live_blocks == 1 && measured.peak_live_bytes == 16);
    CHECK(measured.attempts == 2 && measured.failures == 1);
    CHECK(measured.nanoseconds >= measured.maximum_nanoseconds);
    uint64_t total = 0;
    for (unsigned i = 0; i < NEOCLR_GC_PAUSE_BUCKETS; i++) total += measured.pause_buckets[i];
    CHECK(total == measured.attempts);
    unsigned maximum_bucket = 0;
    for (uint64_t ns = measured.maximum_nanoseconds; ns > 1; ns >>= 1) maximum_bucket++;
    CHECK(measured.pause_buckets[maximum_bucket] > 0);
    for (unsigned i = maximum_bucket + 1; i < NEOCLR_GC_PAUSE_BUCKETS; i++) CHECK(!measured.pause_buckets[i]);
#else
    neoclr_gc_profile zero = {0};
    CHECK(!memcmp(&measured, &zero, sizeof(zero)));
#endif
    CHECK(!neoclr_gc_host_root_release_v1(&context, handle));
    CHECK(!neoclr_gc_collect_v1(&context, NULL));
    CHECK(!context.text.used);
    measured = neoclr_gc_profile_v1();
    CHECK(!measured.live_bytes && !measured.live_blocks);
    return 0;
}
