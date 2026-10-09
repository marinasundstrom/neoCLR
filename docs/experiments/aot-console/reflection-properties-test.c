#include "native-gc.h"
#include <stdio.h>
#include <string.h>
static int text_is(const neoclr_aot_text *text, const char *expected) {
    size_t length = strlen(expected);
    return text && text->length == length && memcmp(text->bytes, expected, length) == 0;
}
int main(void) {
    const int32_t expected[] = {42, 3, 5, 4, 6, 7, 7, 0, 0, 4};
    for (int32_t mode = 0; mode < 10; ++mode) {
        uint64_t buffer[257] = {0};
        buffer[256] = 1234567;
        neoclr_aot_context context = {.text = {(unsigned char *)buffer, 2048, 0}};
        int32_t result = -99;
        int32_t status = neoclr_entry_v4(mode, &result, &context);
        if (neoclr_root_probe_depth_v1()) return 1;
        if (mode != 7 && mode != 8) {
            if (status || result != expected[mode] || context.fault.code) return 2;
        } else {
            if (status != 4 || context.fault.code != 4 || result != -99
                || !text_is(context.fault.message, mode == 7 ? "getter failed" : "setter failed")
                || context.fault.frame_count != 3
                || !text_is(context.fault.frames[0].function, mode == 7 ? "Model.get_Broken" : "Model.set_Broken")
                || context.fault.frames[0].instruction != 0
                || !text_is(context.fault.frames[1].function, mode == 7 ? "neoCLR.Runtime.ReflectionPropertyGet" : "neoCLR.Runtime.ReflectionPropertySet")
                || context.fault.frames[1].instruction != (uint32_t)(mode == 7 ? 2 : 4)
                || !text_is(context.fault.frames[2].function, "Calculate")) return 3;
        }
        if (neoclr_gc_collect_v1(&context, NULL) || context.text.used || buffer[256] != 1234567) return 4;
    }
    puts("Reflection properties: 42");
    return 0;
}
