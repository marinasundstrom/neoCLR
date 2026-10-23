/* Execution-host facts: shared platform implementation with interpreter. */
#include "text-arena.h"
#include "../../../runtime/native/environment/platform.h"
#include <string.h>
extern int32_t neoclr_text_allocate_v1(neoclr_aot_text_arena *, uint64_t, neoclr_aot_text **);
static int32_t platform_info(int32_t selector, neoclr_aot_text_arena *arena,
                            const neoclr_aot_text **output, int rooted) {
    if (!arena || !output) return 3;
    char buffer[4096];
    int length = neoclr_platform_info(selector, buffer, sizeof(buffer));
    if (length < 0) length = 0;
    neoclr_aot_text *text;
    int32_t status = rooted
        ? neoclr_text_allocate_rooted_values_v1(arena, (uint64_t)length, NULL, 0, &text)
        : neoclr_text_allocate_v1(arena, (uint64_t)length, &text);
    if (!status) {
        memcpy(text->bytes, buffer, (size_t)length);
        *output = text;
    }
    return status;
}
int32_t neoclr_environment_platform_v1(int32_t selector, neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return platform_info(selector, arena, output, 0);
}
int32_t neoclr_environment_platform_rooted_v1(int32_t selector, neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    return platform_info(selector, arena, output, 1);
}
