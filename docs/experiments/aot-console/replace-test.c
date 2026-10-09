#include "text-arena.h"
#include <assert.h>
#include <string.h>

struct text { uint64_t length; unsigned char bytes[16]; };
#define TEXT(x) ((const neoclr_aot_text *)&(x))
int main(void) {
    struct text input = {3, "aba"}, search = {1, "a"}, replacement = {2, "XY"};
    struct text missing = {1, "z"}, empty = {0, ""};
    uint64_t storage[8] = {0};
    neoclr_aot_text_arena arena = {(unsigned char *)storage, sizeof(storage), 0};
    const neoclr_aot_text *output = TEXT(missing);
    assert(neoclr_string_replace_ordinal_v1(TEXT(input), TEXT(search), TEXT(replacement), &arena, &output) == 0);
    assert(output->length == 5 && memcmp(output->bytes, "XYbXY", 5) == 0);
    assert(input.length == 3 && memcmp(input.bytes, "aba", 3) == 0);
    uint64_t used = arena.used;
    assert(neoclr_string_replace_ordinal_v1(TEXT(input), TEXT(missing), TEXT(replacement), &arena, &output) == 0);
    assert(output == TEXT(input) && arena.used == used);
    assert(neoclr_string_replace_ordinal_v1(TEXT(input), TEXT(search), TEXT(search), &arena, &output) == 0);
    assert(output == TEXT(input) && arena.used == used);
    output = TEXT(missing);
    assert(neoclr_string_replace_ordinal_v1(TEXT(input), TEXT(empty), TEXT(replacement), &arena, &output) == 3);
    assert(output == TEXT(missing) && arena.used == used);
    arena.capacity = arena.used;
    assert(neoclr_string_replace_ordinal_v1(TEXT(input), TEXT(search), TEXT(replacement), &arena, &output) == 5);
    assert(output == TEXT(missing) && arena.used == used);
    assert(neoclr_string_replace_ordinal_v1(NULL, TEXT(search), TEXT(replacement), &arena, &output) == 3);
    assert(output == TEXT(missing) && arena.used == used);
    return 0;
}
