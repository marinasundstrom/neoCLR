#include "native-gc.h"
#include <stdio.h>
#include <string.h>
static int text_is(const neoclr_aot_text *text, const char *expected) {
    size_t length = strlen(expected);
    return text && text->length == length && memcmp(text->bytes, expected, length) == 0;
}
int main(void) {
    const int32_t statuses[] = {0, 11, 6, 8, 3, 3, 3, 3};
    for (int32_t mode = 0; mode < 8; ++mode) {
        uint64_t buffer[257] = {0};
        buffer[256] = 1234567;
        neoclr_aot_context context = {.text = {(unsigned char *)buffer, 2048, 0}};
        int32_t result = -99;
        int32_t status = neoclr_entry_v4(mode, &result, &context);
        if (neoclr_root_probe_depth_v1()) return 1;
        if (status != statuses[mode] || context.fault.code != (uint32_t)statuses[mode]) return 2;
        if (mode == 0) {
            if (result != 42) return 3;
        } else {
            if (result != -99 || context.fault.frame_count != (uint32_t)(mode < 4 ? 2 : 1)) return 4;
            if (mode < 4 && (!text_is(context.fault.frames[0].function, mode == 3
                    ? "neoCLR.Runtime.ReflectionArrayGet" : "neoCLR.Runtime.ReflectionArrayCreate")
                    || context.fault.frames[0].instruction != (uint32_t)(mode == 3 ? 3 : 18))) return 5;
            if (!text_is(context.fault.frames[context.fault.frame_count - 1].function, "Calculate")) return 6;
            if (mode == 4 && !text_is(context.fault.message, "array access requires a non-null array")) return 7;
            if (mode == 5 && !text_is(context.fault.message, "array construction requires runtime TypeInfo")) return 8;
            if (mode >= 6 && !text_is(context.fault.message, "array service requires a vector type")) return 9;
        }
        if (neoclr_gc_collect_v1(&context, NULL) || context.text.used || buffer[256] != 1234567) return 10;
    }
    puts("Reflection arrays: 42");
    return 0;
}
