#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static const unsigned char content[] = {'a',0xc3,0xa9,0xf0,0x9f,0x98,0x80,0,'z'};
int main(void) {
    for (int empty = 0; empty < 2; empty++) {
        uint64_t heap[32] = {0}, foreign_heap[32] = {0};
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        neoclr_aot_context foreign = {.text = {(unsigned char *)foreign_heap, sizeof(foreign_heap), 0}};
        uint64_t length = empty ? 0 : sizeof(content);
        void *storage = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text, 8 + length, NEOCLR_GC_TEXT, &storage));
        neoclr_aot_text *source = storage;
        source->length = length;
        memcpy(source->bytes, content, (size_t)length);
        void *dead = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text, c.text.capacity - c.text.used - 32, NEOCLR_GC_BYTES, &dead));
        const void *out = (const void *)(uintptr_t)123;
        uint64_t before = neoclr_gc_statistics_v1().collections;
        CHECK(neoclr_utf8_encode_rooted_v1(source, &c.text, &out) == 3 && out == (const void *)(uintptr_t)123);
        neoclr_probe_storage slot = {&dead,8,0};
        neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
        CHECK(neoclr_utf8_encode_rooted_v1(source, &foreign.text, &out) == 3 && !foreign.text.used);
        CHECK(neoclr_utf8_encode_v1(source, &c.text, &out) == 5);
        CHECK(neoclr_gc_statistics_v1().collections == before);
        CHECK(neoclr_utf8_encode_rooted_v1(source, &c.text, &out) == 5 && out == (const void *)(uintptr_t)123);
        dead = NULL; /* Source is intentionally absent from the caller roots. */
        CHECK(!neoclr_utf8_encode_rooted_v1(source, &c.text, &out));
        CHECK(neoclr_gc_statistics_v1().collections == before + 2);
        CHECK(((const uint64_t *)out)[0] == UINT64_C(0x80000003) && ((const uint64_t *)out)[1] == length);
        CHECK(!memcmp((const unsigned char *)out + 16, content, (size_t)length));
        CHECK(source->length == length && !memcmp(source->bytes, content, (size_t)length));
        CHECK(neoclr_root_probe_head_v1() == &frame);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        for (int reserved = 0; reserved < 2; reserved++) {
            void *array = NULL;
            CHECK(!(reserved ? neoclr_reserve_bytes_v1(&c.text, (int32_t)length, &array) : neoclr_allocate_bytes_v1(&c.text, (int32_t)length, &array)));
            unsigned char *bytes = (unsigned char *)array + 16;
            memcpy(bytes, content, (size_t)length);
            if (reserved) memset(bytes + length, 1, (size_t)length);
            CHECK(!neoclr_gc_allocate_v1(&c.text, c.text.capacity - c.text.used - 32, NEOCLR_GC_BYTES, &dead));
            uint64_t result[2] = {99,99};
            CHECK(neoclr_utf8_decode_rooted_v1(array, &c.text, result) == 3 && result[0] == 99 && result[1] == 99);
            neoclr_probe_enter_v3(&frame, &c, 0, &slot, 1, "{}", 2);
            before = neoclr_gc_statistics_v1().collections;
            if (length) {
                bytes[0] = 0xff;
                if (reserved) {
                    bytes[length + length - 1] = 0;
                    CHECK(neoclr_utf8_decode_rooted_v1(array, &c.text, result) == 3 && result[0] == 99 && result[1] == 99);
                    bytes[length + length - 1] = 1;
                }
                CHECK(!neoclr_utf8_decode_rooted_v1(array, &c.text, result) && (uint32_t)result[0] == 2 && result[1] == 1);
                result[0] = result[1] = 99;
                bytes[0] = content[0];
            }
            CHECK(neoclr_utf8_decode_rooted_v1(array, &foreign.text, result) == 3 && !foreign.text.used);
            CHECK(neoclr_utf8_decode_v1(array, &c.text, result) == 5);
            CHECK(neoclr_text_decode_utf8_bytes_v1(bytes, length, &c.text, result) == 5);
            CHECK(neoclr_gc_statistics_v1().collections == before);
            CHECK(neoclr_utf8_decode_rooted_v1(array, &c.text, result) == 5 && result[0] == 99 && result[1] == 99);
            dead = NULL; /* Interior bytes must retain the whole array owner. */
            CHECK(!neoclr_utf8_decode_rooted_v1(array, &c.text, result));
            CHECK(neoclr_gc_statistics_v1().collections == before + 2 && (uint32_t)result[0] == 4);
            const neoclr_aot_text *text = (const void *)(uintptr_t)result[1];
            CHECK(text->length == length && !memcmp(text->bytes, content, (size_t)length));
            CHECK(!memcmp(bytes, content, (size_t)length));
            CHECK(neoclr_root_probe_head_v1() == &frame);
            neoclr_probe_leave_v1(&frame);
            CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
            CHECK(!neoclr_gc_entry_check_v1(&c));
        }
    }
    puts("PASS rooted UTF-8 borrowed-input retention, strict errors, pressure and reclamation");
    return 0;
}
