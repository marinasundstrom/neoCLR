#include "text-arena.h"
#include <string.h>
#define CHECK(test) do { if (!(test)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129]; buffer[128] = 42;
    neoclr_aot_text_arena arena = {(unsigned char *)buffer, 1024, 0};
    struct { uint64_t length; char bytes[8]; } a = {1, "e"}, b = {2, "́"}, sep = {3, "•"}, empty = {0, ""};
    struct { uint64_t kind, length; const void *parts[3]; unsigned char init[3]; } array = {
        UINT64_C(0x80000004), 3, {&a, &empty, &b}, {1, 1, 1}};
    const neoclr_aot_text *output = NULL;
    CHECK(!neoclr_string_join_parts_v1(&array, 3, (void *)&sep, 9, &arena, &output));
    CHECK(output->length == 9 && !memcmp(output->bytes, "e••́", 9));
    const neoclr_aot_text *saved = output;
    uint64_t used = arena.used;
    array.parts[0] = &b;
    CHECK(!memcmp(saved->bytes, "e••́", 9));
    for (int i = 0; i < 6; i++) {
        int count = i == 0 ? -1 : i == 1 ? 4 : 3;
        int bytes = i == 2 ? -1 : i == 3 ? 65537 : i == 4 ? 9 : 11;
        CHECK(neoclr_string_join_parts_v1(&array, count, (void *)&sep, bytes, &arena, &output) == 3);
        CHECK(output == saved && arena.used == used);
    }
    array.init[2] = 0;
    CHECK(neoclr_string_join_parts_v1(&array, 3, (void *)&sep, 10, &arena, &output) == 3);
    CHECK(!neoclr_string_join_parts_v1(&array, 2, (void *)&sep, 5, &arena, &output));
    CHECK(output->length == 5 && !memcmp(output->bytes, "́•", 5));
    CHECK(!neoclr_string_join_parts_v1(&array, 0, (void *)&sep, 0, &arena, &output));
    CHECK(output->length == 0);
    saved = output; used = arena.used;
    CHECK(neoclr_string_join_parts_v1(NULL, 0, (void *)&sep, 0, &arena, &output) == 3);
    CHECK(neoclr_string_join_parts_v1(&array, 0, NULL, 0, &arena, &output) == 3);
    array.parts[0] = NULL;
    CHECK(neoclr_string_join_parts_v1(&array, 1, (void *)&sep, 0, &arena, &output) == 3);
    CHECK(output == saved && arena.used == used);
    neoclr_aot_text_arena exhausted = {(unsigned char *)buffer, 0, 0};
    CHECK(neoclr_string_join_parts_v1(&array, 0, (void *)&sep, 0, &exhausted, &output) == 5);
    CHECK(output == saved && exhausted.used == 0 && buffer[128] == 42);
    /* Join result size is independent of StringBuilder's 65536-byte quota. */
    uint64_t large_buffer[16384];
    struct { uint64_t length; char bytes[33000]; } large = {33000, {0}};
    memset(large.bytes, 'x', sizeof(large.bytes));
    struct { uint64_t kind, length; const void *parts[2]; } pair = {
        UINT64_C(0x80000003), 2, {&large, &large}};
    neoclr_aot_text_arena large_arena = {(unsigned char *)large_buffer, sizeof(large_buffer), 0};
    CHECK(!neoclr_string_join_parts_v1(&pair, 2, (void *)&sep, 66003, &large_arena, &output));
    CHECK(output->length == 66003 && output->bytes[32999] == 'x' &&
          !memcmp(output->bytes + 33000, "•", 3) && output->bytes[33003] == 'x');
    return 0;
}
