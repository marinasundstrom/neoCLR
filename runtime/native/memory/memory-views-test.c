#include "memory-views.h"
#ifdef NEOCLR_NATIVE_GC
#include "../gc/native-gc.h"
#endif
#include <string.h>
#include <limits.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed at %d: %s\n", __LINE__, #x); return 1; } } while (0)
static int copy_tests(void) {
    uint64_t heap[512] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *arrays[2];
    CHECK(!neoclr_allocate_bytes_v1(&c.text, 6, &arrays[0]));
    CHECK(!neoclr_reserve_bytes_v1(&c.text, 6, &arrays[1]));
    uint64_t used = c.text.used;
#ifdef NEOCLR_NATIVE_GC
    uint64_t allocations = neoclr_gc_statistics_v1().allocations;
#endif
    /* All windows of six bytes, both overlap directions, separate owners,
     * initialized/reserved storage and every source/destination capability. */
    for (int src_array = 0; src_array < 2; src_array++) {
        for (int dst_array = 0; dst_array < 2; dst_array++) {
            for (uint64_t src_kind = 0; src_kind < 4; src_kind++) {
                for (uint64_t dst_kind = 0; dst_kind < 4; dst_kind++) {
                    for (int start = 0; start <= 6; start++) {
                        for (int count = 0; count <= 6 - start; count++) {
                            for (int target = 0; target <= 6; target++) {
                                for (int capacity = 0; capacity <= 6 - target; capacity++) {
                                    unsigned char before[2][12], expected[2][12];
                                    for (int a = 0; a < 2; a++) {
                                        unsigned char *data = (unsigned char *)arrays[a] + 16;
                                        for (int i = 0; i < 6; i++) data[i] = (unsigned char)(10 * a + i + 1);
                                        if (a) memset(data + 6, 1, 6);
                                        memcpy(before[a], data, a ? 12 : 6);
                                        memcpy(expected[a], before[a], a ? 12 : 6);
                                    }
                                    neoclr_byte_view source, destination;
                                    CHECK(!neoclr_byte_view_create_v1(&c, arrays[src_array], start, count, src_kind, &source));
                                    CHECK(!neoclr_byte_view_create_v1(&c, arrays[dst_array], target, capacity, dst_kind, &destination));
                                    neoclr_byte_view source_before = source, destination_before = destination;
                                    int32_t status = (dst_kind & 1) ? 3 : count > capacity ? 8 : 0;
                                    if (!status) {
                                        /* Independent snapshot oracle, not an in-place copy. */
                                        for (int i = 0; i < count; i++)
                                            expected[dst_array][target + i] = before[src_array][start + i];
                                    }
                                    CHECK(neoclr_byte_view_copy_v1(&c, &source, &destination) == status);
                                    CHECK(!memcmp(&source, &source_before, sizeof(source)));
                                    CHECK(!memcmp(&destination, &destination_before, sizeof(destination)));
                                    for (int a = 0; a < 2; a++)
                                        CHECK(!memcmp((unsigned char *)arrays[a] + 16, expected[a], a ? 12 : 6));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    unsigned char *pending = (unsigned char *)arrays[1] + 16;
    neoclr_byte_view source, destination;
    CHECK(!neoclr_byte_view_create_v1(&c, arrays[1], 0, 3, 1, &source));
    CHECK(!neoclr_byte_view_create_v1(&c, arrays[1], 2, 4, 2, &destination));
    /* An overlapping write must not initialize a later unread source slot. */
    memset(pending + 6, 0, 6);
    pending[6] = pending[7] = 1;
    unsigned char before[12]; memcpy(before, pending, sizeof(before));
    CHECK(neoclr_byte_view_copy_v1(&c, &source, &destination) == 3);
    CHECK(!memcmp(before, pending, sizeof(before)));
    pending[8] = 1;
    CHECK(!neoclr_byte_view_copy_v1(&c, &source, &destination));
    CHECK(pending[8] && pending[9] && pending[10] && !pending[11]);
    CHECK(pending[2] == before[0] && pending[3] == before[1] && pending[4] == before[2]);
    CHECK(!neoclr_byte_view_copy_v1(&c, &source, &destination));
    CHECK(!neoclr_byte_view_create_v1(&c, NULL, 0, 0, 0, &destination));
    CHECK(!neoclr_byte_view_copy_v1(&c, &destination, &destination));
    CHECK(neoclr_byte_view_copy_v1(&c, &destination, &source) == 3); /* readonly target */
    CHECK(neoclr_byte_view_copy_v1(&c, NULL, &destination) == 3);
    CHECK(neoclr_byte_view_copy_v1(&c, &source, NULL) == 3);
    CHECK(neoclr_byte_view_copy_v1(NULL, &source, &destination) == 3);
    neoclr_byte_view bad = source; bad.offset = UINT32_MAX;
    CHECK(neoclr_byte_view_copy_v1(&c, &bad, &destination) == 8);
    CHECK(neoclr_byte_view_copy_v1(&c, &destination, &bad) == 8);
    CHECK(c.text.used == used);
#ifdef NEOCLR_NATIVE_GC
    CHECK(neoclr_gc_statistics_v1().allocations == allocations);
    neoclr_aot_context foreign = {0};
    CHECK(neoclr_byte_view_copy_v1(&foreign, &source, &destination) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(neoclr_byte_view_copy_v1(&c, &source, &destination) == 3);
#endif
    return 0;
}
int main(void) {
    CHECK(!copy_tests());
    uint64_t heap[512] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    void *array;
    CHECK(!neoclr_reserve_bytes_v1(&c.text, 5, &array));
    uint64_t used = c.text.used;
#ifdef NEOCLR_NATIVE_GC
    uint64_t allocations = neoclr_gc_statistics_v1().allocations;
#endif
    neoclr_byte_view view, nested, readonly, persistent;
    CHECK(!neoclr_byte_view_create_v1(&c, array, 1, 3, NEOCLR_VIEW_MEMORY, &persistent));
    CHECK(!neoclr_byte_view_convert_v1(&c, &persistent, NEOCLR_VIEW_SLICE, &view));
    CHECK(!neoclr_byte_view_slice_v1(&c, &view, 1, 1, &nested));
    CHECK(nested.owner == array && nested.offset == 2 && nested.length == 1);
    uint8_t value = 99;
    CHECK(neoclr_byte_view_get_v1(&c, &nested, 0, &value) == 3 && value == 99);
    CHECK(!neoclr_byte_view_set_v1(&c, &nested, 0, 42));
    CHECK(!neoclr_byte_view_get_v1(&c, &persistent, 1, &value) && value == 42);
    CHECK(((unsigned char *)array)[18] == 42 && ((unsigned char *)array)[23]);
    CHECK(!((unsigned char *)array)[21] && !((unsigned char *)array)[22]);
    CHECK(!neoclr_byte_view_convert_v1(&c, &persistent, NEOCLR_VIEW_READONLY_MEMORY, &readonly));
    CHECK(neoclr_byte_view_set_v1(&c, &readonly, 1, 7) == 3);
    CHECK(!neoclr_byte_view_get_v1(&c, &readonly, 1, &value) && value == 42);
    CHECK(!neoclr_byte_view_set_v1(&c, &view, 1, 43));
    CHECK(!neoclr_byte_view_get_v1(&c, &readonly, 1, &value) && value == 43);
    neoclr_byte_view unchanged = nested;
    CHECK(neoclr_byte_view_convert_v1(&c, &readonly, NEOCLR_VIEW_MEMORY, &nested) == 3);
    CHECK(!memcmp(&unchanged, &nested, sizeof(nested)));
    CHECK(neoclr_byte_view_convert_v1(&c, &view, NEOCLR_VIEW_MEMORY, &nested) == 3);
    CHECK(!memcmp(&unchanged, &nested, sizeof(nested)));
    CHECK(neoclr_byte_view_slice_v1(&c, &view, 3, 1, &nested) == 8);
    CHECK(neoclr_byte_view_get_v1(&c, &nested, 1, &value) == 8);
    CHECK(neoclr_byte_view_slice_v1(&c, &view, INT32_MAX, INT32_MAX, &nested) == 8);
    CHECK(neoclr_byte_view_slice_v1(&c, &view, -1, 0, &nested) == 8);
    CHECK(neoclr_byte_view_slice_v1(&c, &view, 0, -1, &nested) == 8);
    CHECK(!memcmp(&unchanged, &nested, sizeof(nested)));
    CHECK(neoclr_byte_view_create_v1(&c, array, 4, 2, 0, &nested) == 8);
    CHECK(neoclr_byte_view_create_v1(&c, array, 0, 0, 4, &nested) == 3);
    CHECK(!memcmp(&unchanged, &nested, sizeof(nested)));
    CHECK(neoclr_byte_view_set_v1(&c, &view, 3, 100) == 8);
    CHECK(neoclr_byte_view_get_v1(&c, &view, -1, &value) == 8 && value == 43);
    CHECK(neoclr_byte_view_get_v1(&c, &view, INT32_MAX, &value) == 8 && value == 43);
    CHECK(!neoclr_byte_view_slice_v1(&c, &readonly, 3, 0, &readonly));
    CHECK(readonly.owner == array && readonly.offset == 4 && !readonly.length);
    CHECK(!neoclr_byte_view_convert_v1(&c, &readonly, NEOCLR_VIEW_READONLY_SLICE, &readonly));
    CHECK(neoclr_byte_view_set_v1(&c, &readonly, 0, 1) == 8);
    CHECK(!neoclr_byte_view_create_v1(&c, NULL, 0, 0, NEOCLR_VIEW_SLICE, &nested));
    CHECK(!nested.owner && !nested.length && !nested.offset);
    CHECK(neoclr_byte_view_get_v1(&c, &nested, 0, &value) == 8);
    CHECK(neoclr_byte_view_create_v1(&c, NULL, 1, 0, 0, &nested) == 8);
    CHECK(neoclr_byte_view_create_v1(NULL, array, 0, 0, 0, &nested) == 3);
    CHECK(neoclr_byte_view_get_v1(&c, &view, 0, NULL) == 3);
    CHECK(neoclr_byte_view_slice_v1(&c, NULL, 0, 0, &nested) == 3);
    const int allowed[4][4] = {{1,1,0,0}, {0,1,0,0}, {1,1,1,1}, {0,1,0,1}};
    for (uint64_t from = 0; from < 4; from++) {
        neoclr_byte_view source;
        CHECK(!neoclr_byte_view_create_v1(&c, array, 1, 3, from, &source));
        for (uint64_t to = 0; to < 4; to++) {
            neoclr_byte_view destination = source;
            int32_t status = neoclr_byte_view_convert_v1(&c, &source, to, &destination);
            CHECK(status == (allowed[from][to] ? 0 : 3));
            CHECK(destination.owner == source.owner && destination.offset == source.offset && destination.length == source.length);
            CHECK(destination.kind == (allowed[from][to] ? to : from));
        }
    }
    CHECK(c.text.used == used);
#ifdef NEOCLR_NATIVE_GC
    CHECK(neoclr_gc_statistics_v1().allocations == allocations);
    /* Only an empty view owns the array here. No hidden native-stack scanning. */
    neoclr_probe_storage slot = {&readonly.owner, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(!neoclr_byte_view_get_v1(&c, &persistent, 1, &value) && value == 43);
    neoclr_aot_context foreign = {0};
    CHECK(neoclr_byte_view_get_v1(&foreign, &persistent, 1, &value) == 3 && value == 43);
    neoclr_byte_view interior = persistent; interior.owner = (unsigned char *)array + 8;
    CHECK(neoclr_byte_view_get_v1(&c, &interior, 0, &value) == 3);
    readonly.owner = NULL;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    CHECK(neoclr_byte_view_get_v1(&c, &persistent, 1, &value) == 3 && value == 43);
#endif
    /* Zero-initialized arrays, malformed descriptors and extents. */
    CHECK(!neoclr_allocate_bytes_v1(&c.text, 3, &array));
    CHECK(!neoclr_byte_view_create_v1(&c, array, 0, 3, 0, &view));
    CHECK(!neoclr_byte_view_get_v1(&c, &view, 2, &value) && !value);
    neoclr_byte_view bad = view; bad.offset = UINT32_MAX;
    CHECK(neoclr_byte_view_set_v1(&c, &bad, 0, 1) == 8);
    bad = view; bad.kind = UINT64_MAX;
    CHECK(neoclr_byte_view_get_v1(&c, &bad, 0, &value) == 3);
    uint64_t original_count; memcpy(&original_count, (unsigned char *)array + 8, 8);
    uint64_t excessive = UINT64_MAX; memcpy((unsigned char *)array + 8, &excessive, 8);
    CHECK(neoclr_byte_view_get_v1(&c, &view, 0, &value) == 3);
    uint64_t too_large = 4; memcpy((unsigned char *)array + 8, &too_large, 8);
    CHECK(neoclr_byte_view_get_v1(&c, &view, 0, &value) == 3);
    memcpy((unsigned char *)array + 8, &original_count, 8);
#ifdef NEOCLR_NATIVE_GC
    /* A persistent descriptor stored in a managed object retains its byte owner. */
    void *holder;
    CHECK(!neoclr_allocate_object_v1(&c.text, 0, 8 + sizeof(neoclr_byte_view), &holder));
    CHECK(!neoclr_byte_view_create_v1(&c, array, 1, 2, NEOCLR_VIEW_MEMORY, &persistent));
    memcpy((unsigned char *)holder + 8, &persistent, sizeof(persistent));
    neoclr_probe_storage holder_slot = {&holder, 8, 0};
    neoclr_probe_frame holder_frame = {.context = &c, .storage = &holder_slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&c, &holder_frame));
    neoclr_byte_view stored; memcpy(&stored, (unsigned char *)holder + 8, sizeof(stored));
    CHECK(!neoclr_byte_view_set_v1(&c, &stored, 0, 77));
    CHECK(!neoclr_byte_view_get_v1(&c, &stored, 0, &value) && value == 77);
    holder = NULL;
    CHECK(!neoclr_gc_collect_v1(&c, &holder_frame) && !c.text.used);
#endif
    return 0;
}
