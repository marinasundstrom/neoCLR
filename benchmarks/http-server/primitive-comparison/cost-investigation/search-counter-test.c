/* Include only the generated diagnostic runtime so its private counters can be
 * checked without adding a production ABI. Test the unchanged first-fit policy. */
#include "diagnostic-gc.c"
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    uint64_t heap[64] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
    void *a, *b, *retained;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_BYTES, &a));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_BYTES, &b));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_BYTES, &retained));
    CHECK(allocation_search.attempts == 3 && allocation_search.steps == 0);
    CHECK(allocation_search.appends == 3 && allocation_search.peak == 144);
    neoclr_probe_storage slot = {&retained, 8, 0};
    neoclr_probe_frame frame = {.context = &context, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&context, &frame));
    void *reused, *appended;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_BYTES, &reused));
    CHECK(reused == a && allocation_search.steps == 1 && allocation_search.fits == 1);
    CHECK(!neoclr_gc_allocate_v1(&context.text, 64, NEOCLR_GC_BYTES, &appended));
    CHECK(allocation_search.attempts == 5 && allocation_search.steps == 3);
    CHECK(allocation_search.appends == 4 && allocation_search.peak == 240);
    CHECK(!neoclr_gc_collect_v1(&context, NULL) && !context.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&context.text));
    CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
    CHECK(!allocation_search.attempts && !allocation_search.steps && !allocation_search.peak);
    CHECK(!neoclr_gc_heap_release_v1(&context.text));
    return 0;
}
