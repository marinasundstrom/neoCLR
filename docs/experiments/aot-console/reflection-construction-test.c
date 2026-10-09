#include "native-gc.h"
#include <stdio.h>
#include <string.h>
static int text_is(const neoclr_aot_text *text, const char *expected) {
    size_t length = strlen(expected);
    return text && text->length == length && memcmp(text->bytes, expected, length) == 0;
}
int main(void) {
    const int32_t expected[] = {42, 3, 4, 2};
    for (int32_t mode = 0; mode < 5; ++mode) {
        uint64_t buffer[257] = {0};
        buffer[256] = 1234567;
        neoclr_aot_context context = {.text = {(unsigned char *)buffer, 2048, 0}};
        int32_t result = -99;
        int32_t status = neoclr_entry_v4(mode, &result, &context);
        if (neoclr_root_probe_depth_v1()) return 1;
        if (mode < 4) {
            if (status || result != expected[mode] || context.fault.code) return 2;
        } else {
            if (status != 4 || context.fault.code != 4 || result != -99
                || !text_is(context.fault.message, "constructor failed") || context.fault.frame_count != 3
                || !text_is(context.fault.frames[0].function, "Broken..ctor") || context.fault.frames[0].instruction != 0
                || !text_is(context.fault.frames[1].function, "neoCLR.Runtime.ReflectionConstruct") || context.fault.frames[1].instruction != 0
                || !text_is(context.fault.frames[2].function, "Calculate") || context.fault.frames[2].instruction != 32) return 3;
        }
        if (neoclr_gc_collect_v1(&context, NULL) || context.text.used || buffer[256] != 1234567) return 4;
    }
    puts("Reflection construction: 42");
    return 0;
}
