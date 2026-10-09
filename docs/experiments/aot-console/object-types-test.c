#include "native-gc.h"
#include <stdio.h>
#include <string.h>
static int text_is(const neoclr_aot_text *text, const char *expected) {
    size_t length = strlen(expected);
    return text && text->length == length && memcmp(text->bytes, expected, length) == 0;
}
int main(void) {
    for (int32_t mode = 0; mode < 6; ++mode) {
        uint64_t buffer[257] = {0};
        buffer[256] = 1234567;
        neoclr_aot_context context = {.text = {(unsigned char *)buffer, 2048, 0}};
        int32_t result = -99;
        int32_t status = neoclr_entry_v4(mode, &result, &context);
        if (neoclr_root_probe_depth_v1()) return 1;
        if (mode < 5) {
            if (status || result != 42 || context.fault.code) return 2;
        } else {
            int32_t expected = 6;
            const char *message = "GetType requires a non-null instance";
            if (status != expected || context.fault.code != (uint32_t)expected || result != -99
                || !text_is(context.fault.message, message) || context.fault.frame_count != 1
                || !text_is(context.fault.frames[0].function, "Calculate")) return 3;
        }
        if (neoclr_gc_collect_v1(&context, NULL) || context.text.used || buffer[256] != 1234567) return 4;
    }
    puts("Object types: 42");
    return 0;
}
