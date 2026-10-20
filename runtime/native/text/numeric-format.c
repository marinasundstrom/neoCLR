/* Private matched floating text adapters; conversion uses bounded stack storage. */
#include "text-arena.h"
#include <string.h>
extern int32_t neoclr_text_allocate_v1(neoclr_aot_text_arena *, uint64_t, neoclr_aot_text **);

extern int32_t neoclr_format_float_bytes_v1(double, int32_t, int32_t, int32_t, int32_t, unsigned char *, size_t, size_t *);
static int32_t floating_format(double value, int32_t single, int32_t code, int32_t precision, int32_t swedish,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **output, int rooted) {
    if (!arena || !output) return 3;
    unsigned char buffer[1024]; size_t length = 0;
    int32_t status = neoclr_format_float_bytes_v1(value, single, code, precision, swedish, buffer, sizeof(buffer), &length);
    if (status) return status;
    neoclr_aot_text *result;
    status = rooted ? neoclr_text_allocate_rooted_values_v1(arena, length, NULL, 0, &result)
                    : neoclr_text_allocate_v1(arena, length, &result);
    if (status) return status;
    memcpy(result->bytes, buffer, length);
    *output = result;
    return 0;
}
#define FLOAT_FORMAT(name, type, single) \
int32_t neoclr_##name##_format_v1(type value, int32_t code, int32_t precision, int32_t swedish, neoclr_aot_text_arena *arena, const neoclr_aot_text **output) { return floating_format(value, single, code, precision, swedish, arena, output, 0); } \
int32_t neoclr_##name##_format_rooted_v1(type value, int32_t code, int32_t precision, int32_t swedish, neoclr_aot_text_arena *arena, const neoclr_aot_text **output) { return floating_format(value, single, code, precision, swedish, arena, output, 1); }
FLOAT_FORMAT(single, float, 1)
FLOAT_FORMAT(double, double, 0)
#undef FLOAT_FORMAT
