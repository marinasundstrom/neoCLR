#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#ifdef NEOCLR_TEST_THREADS
#include <pthread.h>
#endif
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int check(const uint64_t *expected, const uint64_t *failures) {
    neoclr_gc_trigger_profile p = neoclr_gc_trigger_profile_v1();
#ifndef NEOCLR_GC_PROFILE
    (void)expected; (void)failures;
#endif
    for (unsigned i = 0; i < NEOCLR_GC_TRIGGER_COUNT; i++) {
#ifdef NEOCLR_GC_PROFILE
        CHECK(p.requests[i] == expected[i] && p.failures[i] == failures[i]);
#else
        CHECK(!p.requests[i] && !p.failures[i]);
#endif
    }
    return 0;
}
static void boundary(neoclr_probe_frame *f) {
    uint64_t empty = 0;
    const neoclr_gc_root_map map = {NEOCLR_GC_ROOT_MAP_MAGIC, 0};
    neoclr_gc_stack_map_v1(f, 0, &empty, 0, &map, sizeof(map));
}
#ifdef NEOCLR_WORKER_GC
static int32_t reject(neoclr_aot_context *c, const neoclr_probe_frame *head) {
    (void)c; (void)head; return 3;
}
#endif
static int exercise(void) {
    uint64_t e[NEOCLR_GC_TRIGGER_COUNT] = {0}, f[NEOCLR_GC_TRIGGER_COUNT] = {0};
#define EXPECT(reason) do { e[NEOCLR_GC_TRIGGER_##reason]++; CHECK(!check(e, f)); } while (0)
    uint64_t heap[8192] = {0}, small[16] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}}, other = {0};
    neoclr_aot_context tiny = {.text = {(unsigned char *)small, sizeof(small), 0}};
    neoclr_probe_storage slot = {0}; neoclr_probe_frame frame, child;
    neoclr_gc_trigger_profile_reset_v1(); neoclr_gc_profile_reset_v1();
    neoclr_probe_enter_v3(&frame, &c, 0, &slot, 0, "{}", 2);
    boundary(&frame); EXPECT(CONTEXT);
    CHECK(neoclr_gc_boundary_v1(NULL, &frame) == 3); CHECK(!check(e, f));
    /* Reset diagnostics must not reset the 4096-publication policy budget. */
    for (unsigned i = 0; i < 2048; i++) boundary(&frame);
    neoclr_gc_trigger_profile_reset_v1(); memset(e, 0, sizeof(e));
    for (unsigned i = 0; i < 2047; i++) boundary(&frame);
    CHECK(!check(e, f)); boundary(&frame); EXPECT(PUBLICATION);
    void *value;
    for (unsigned i = 0; i < 256; i++) CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &value));
    boundary(&frame); EXPECT(ALLOCATION);
    CHECK(!neoclr_gc_collect_v1(&c, &frame)); EXPECT(DIRECT);
    boundary(&frame); EXPECT(EPOCH);
    CHECK(!neoclr_gc_idle_v1(&c, &frame)); EXPECT(IDLE);
    CHECK(!neoclr_gc_collect_managed_v1(&c, NULL)); EXPECT(MANAGED);
    boundary(&frame); EXPECT(EPOCH);
    for (unsigned i = 0; i < 256; i++) CHECK(!neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_BYTES, &value));
    CHECK(!neoclr_gc_collect_v1(&c, &frame)); EXPECT(DIRECT);
    neoclr_probe_enter_v3(&child, &other, 1, &slot, 0, "{}", 2);
    boundary(&child); EXPECT(CONTEXT); /* Context wins over epoch/allocation. */
    neoclr_probe_leave_v1(&child); boundary(&frame); EXPECT(CONTEXT);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_allocate_v1(&tiny.text, 64, NEOCLR_GC_BYTES, &value));
    CHECK(!neoclr_gc_allocate_rooted_v1(&tiny, NULL, 64, NEOCLR_GC_BYTES, &value)); EXPECT(PRESSURE);
    CHECK(neoclr_gc_allocate_rooted_v1(&tiny, NULL, 1024, NEOCLR_GC_BYTES, &value) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
    CHECK(!check(e, f)); /* An impossible block never requests collection. */
    CHECK(!neoclr_gc_collect_conditional_v1(&tiny, NULL, NULL, 0)); CHECK(!check(e, f));
    CHECK(neoclr_gc_collect_v1(NULL, NULL) == 3); f[NEOCLR_GC_TRIGGER_DIRECT]++; EXPECT(DIRECT);
#ifdef NEOCLR_WORKER_GC
    uint64_t attempts = neoclr_gc_profile_v1().attempts;
    neoclr_gc_worker_collect_hook = reject;
    CHECK(neoclr_gc_collect_v1(&tiny, NULL) == 3); f[NEOCLR_GC_TRIGGER_DIRECT]++; EXPECT(DIRECT);
    CHECK(neoclr_gc_profile_v1().attempts == attempts); /* Request != collection entry. */
    neoclr_gc_worker_collect_hook = NULL;
#endif
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    neoclr_gc_profile pause = neoclr_gc_profile_v1();
    neoclr_gc_trigger_profile_reset_v1(); memset(e, 0, sizeof(e)); memset(f, 0, sizeof(f));
    CHECK(!check(e, f));
    neoclr_gc_statistics after = neoclr_gc_statistics_v1();
    neoclr_gc_profile after_pause = neoclr_gc_profile_v1();
    CHECK(!memcmp(&before, &after, sizeof(before)) && !memcmp(&pause, &after_pause, sizeof(pause)));
    return 0;
}
#ifdef NEOCLR_TEST_THREADS
static void *worker(void *unused) {
    (void)unused;
    uint64_t zero[NEOCLR_GC_TRIGGER_COUNT] = {0};
    return (void *)(uintptr_t)(check(zero, zero) || exercise());
}
#endif
int main(void) {
    CHECK(!exercise());
#ifdef NEOCLR_TEST_THREADS
    CHECK(neoclr_gc_collect_v1(NULL, NULL) == 3);
    neoclr_gc_trigger_profile before = neoclr_gc_trigger_profile_v1();
    pthread_t threads[2];
    for (unsigned i = 0; i < 2; i++) CHECK(!pthread_create(&threads[i], NULL, worker, NULL));
    for (unsigned i = 0; i < 2; i++) { void *result; CHECK(!pthread_join(threads[i], &result) && !result); }
    neoclr_gc_trigger_profile after = neoclr_gc_trigger_profile_v1();
    CHECK(!memcmp(&before, &after, sizeof(before)));
#endif
    return 0;
}
