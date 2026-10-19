#include "text-arena.h"
#include <stdint.h>
#include <string.h>
#define CHECK(c) do { if (!(c)) return __LINE__; } while (0)
int main(void) {
    struct { uint64_t length; unsigned char bytes[8]; } source = {3, {'a', '\r', 'b'}},
        replacement = {1, {'|'}}, unchanged = {1, {'\r'}};
    uint64_t storage[32];
    neoclr_aot_text_arena arena = {(unsigned char *)storage, sizeof(storage), 0};
    const neoclr_aot_text *input = (const neoclr_aot_text *)&source;
    const neoclr_aot_text *output = NULL;
    CHECK(!neoclr_string_replace_line_endings_v1(input, (const neoclr_aot_text *)&replacement, &arena, &output));
    CHECK(output->length == 3 && !memcmp(output->bytes, "a|b", 3));
    arena.capacity = 0; arena.used = 0;
    output = input;
    CHECK(neoclr_string_replace_line_endings_v1(input, (const neoclr_aot_text *)&replacement, &arena, &output) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
    CHECK(output == input && arena.used == 0);
    CHECK(!neoclr_string_replace_line_endings_v1(input, (const neoclr_aot_text *)&unchanged, &arena, &output));
    CHECK(output == input && arena.used == 0);
    replacement.length = UINT64_MAX;
    CHECK(neoclr_string_replace_line_endings_v1(input, (const neoclr_aot_text *)&replacement, &arena, &output) == NEOCLR_AOT_FAULT_RUNTIME_ERROR);
    CHECK(output == input && arena.used == 0);
    CHECK(neoclr_string_replace_line_endings_v1(NULL, (const neoclr_aot_text *)&unchanged, &arena, &output) == NEOCLR_AOT_FAULT_RUNTIME_ERROR);
    return 0;
}
