#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int32_t format(int mode, int rooted, neoclr_aot_text_arena *arena, const neoclr_aot_text **out) {
    switch (mode) {
    case 0: return rooted ? neoclr_int32_to_string_rooted_v1(INT32_MIN, arena, out) : neoclr_int32_to_string_v1(INT32_MIN, arena, out);
    case 1: return rooted ? neoclr_int32_to_string_rooted_v1(INT32_MAX, arena, out) : neoclr_int32_to_string_v1(INT32_MAX, arena, out);
    case 2: return rooted ? neoclr_int64_to_string_rooted_v1(INT64_MIN, arena, out) : neoclr_int64_to_string_v1(INT64_MIN, arena, out);
    case 3: return rooted ? neoclr_int64_to_string_rooted_v1(INT64_MAX, arena, out) : neoclr_int64_to_string_v1(INT64_MAX, arena, out);
    case 4: return rooted ? neoclr_uint64_to_string_rooted_v1(UINT64_MAX, arena, out) : neoclr_uint64_to_string_v1(UINT64_MAX, arena, out);
    default: return rooted ? neoclr_uint64_to_string_rooted_v1(0, arena, out) : neoclr_uint64_to_string_v1(0, arena, out);
    }
}
int main(void) {
    const char *expected[] = {"-2147483648", "2147483647", "-9223372036854775808", "9223372036854775807", "18446744073709551615", "0"};
    for (int mode = 0; mode < 6; mode++) {
        uint64_t heap[16] = {0}, other_heap[16] = {0};
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        neoclr_aot_context other = {.text = {(unsigned char *)other_heap, sizeof(other_heap), 0}};
        void *live = NULL, *dead = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_BYTES, &live));
        CHECK(!neoclr_gc_allocate_v1(&c.text, 48, NEOCLR_GC_BYTES, &dead));
        memset(live, 0xa5, 16);
        const neoclr_aot_text *out = (const void *)(uintptr_t)123;
        uint64_t before = neoclr_gc_statistics_v1().collections;
        CHECK(format(mode, 1, &c.text, &out) == 3 && out == (const void *)(uintptr_t)123);
        neoclr_probe_storage slots[] = {{&live, 8, 0}, {&dead, 8, 0}};
        neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame, &c, 0, slots, 2, "{}", 2);
        CHECK(format(mode, 1, &other.text, &out) == 3 && !other.text.used);
        CHECK(format(mode, 0, &c.text, &out) == 5 && out == (const void *)(uintptr_t)123);
        CHECK(neoclr_gc_statistics_v1().collections == before);
        CHECK(format(mode, 1, &c.text, &out) == 5 && out == (const void *)(uintptr_t)123);
        CHECK(neoclr_gc_statistics_v1().collections == before + 1);
        dead = NULL;
        CHECK(!format(mode, 1, &c.text, &out));
        CHECK(neoclr_gc_statistics_v1().collections == before + 2);
        CHECK(out->length == strlen(expected[mode]) && !memcmp(out->bytes, expected[mode], (size_t)out->length));
        for (int i = 0; i < 16; i++) CHECK(((unsigned char *)live)[i] == 0xa5);
        CHECK(neoclr_root_probe_head_v1() == &frame);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        CHECK(!neoclr_gc_entry_check_v1(&c));
    }
    puts("PASS rooted integer formatting, pressure retry, live exhaustion and final reclamation");
    return 0;
}
