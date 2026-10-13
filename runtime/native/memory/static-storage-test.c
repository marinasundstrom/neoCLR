#include "../gc/native-gc.h"
#include <string.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t heap[512] = {0}, other_heap[512] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_aot_context other = {.text = {(unsigned char *)other_heap, sizeof(other_heap), 0}};
    uint64_t *first = NULL, *second = NULL, *again = NULL, *isolated = NULL;
    CHECK(!neoclr_static_storage_v1(&c, 0, 2, 24, (void **)&first));
    CHECK(!neoclr_static_storage_v1(&c, 1, 2, 24, (void **)&second));
    CHECK(first != second && first[1] == 0 && second[1] == 0);
    first[1] = 40; second[1] = 2;
    void *child;
    CHECK(!neoclr_allocate_object_v1(&c.text, 0, 16, &child));
    ((uint64_t *)child)[1] = 42;
    first[2] = (uintptr_t)child;
    CHECK(!neoclr_static_storage_v1(&other, 0, 2, 24, (void **)&isolated));
    CHECK(!isolated[1] && !isolated[2]);
    for (unsigned i = 0; i < 20; i++) {
        void *temporary;
        CHECK(!neoclr_allocate_object_v1(&c.text, 0, 16, &temporary));
#ifdef NEOCLR_NATIVE_GC
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
#endif
        CHECK(!neoclr_static_storage_v1(&c, 0, 2, 24, (void **)&again));
        CHECK(first == again && first[1] + second[1] == 42);
        CHECK(((uint64_t *)(uintptr_t)first[2])[1] == 42);
    }
    void *unchanged = child;
    CHECK(neoclr_static_storage_v1(&c, 2, 2, 24, &unchanged) == 3 && unchanged == child);
    CHECK(neoclr_static_storage_v1(&c, 0, 3, 24, &unchanged) == 3 && unchanged == child);
#ifdef NEOCLR_NATIVE_GC
    const neoclr_aot_text *text, *canonical;
    CHECK(!neoclr_int32_to_string_v1(42, &c.text, &text));
    CHECK(!neoclr_string_intern_v1(text, &c.text, &canonical) && text == canonical);
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_v1(&c, (void *)text, &handle));
    CHECK(!neoclr_gc_complete_invocation_v1(&c));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && text->length == 2);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
#endif
    c.static_storage = NULL; other.static_storage = NULL;
#ifdef NEOCLR_NATIVE_GC
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_collect_v1(&other, NULL) && !other.text.used);
    neoclr_gc_statistics stats = neoclr_gc_statistics_v1();
    CHECK(stats.allocations == stats.reclaimed_allocations);
#endif
    return 0;
}
