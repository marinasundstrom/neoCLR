/* Private text/arena adapter for allocation-free pinned IANA queries. */
#include "text-arena.h"
#include <stdint.h>
#include <stddef.h>
#include <string.h>
extern int32_t neoclr_text_allocate_v1(neoclr_aot_text_arena *, uint64_t, neoclr_aot_text **);
extern int32_t neoclr_tz_exists_v1(const unsigned char *, size_t);
extern int32_t neoclr_tz_offset_v1(const unsigned char *, size_t, int64_t);
extern const unsigned char *neoclr_tz_version_v1(size_t *);
int32_t neoclr_time_zone_exists_v1(const neoclr_aot_text *id, int32_t *output) {
    if (!id || !output) return 3;
    *output = neoclr_tz_exists_v1(id->bytes, (size_t)id->length);
    return 0;
}
int32_t neoclr_time_zone_offset_v1(const neoclr_aot_text *id, int64_t ticks, int32_t *output) {
    if (!id || !output) return 3;
    *output = neoclr_tz_offset_v1(id->bytes, (size_t)id->length, ticks);
    return 0;
}
int32_t neoclr_time_zone_version_v1(neoclr_aot_text_arena *arena, const neoclr_aot_text **output) {
    if (!arena || !output) return 3;
    size_t length = 0;
    const unsigned char *bytes = neoclr_tz_version_v1(&length);
    if (!bytes) return 3;
    neoclr_aot_text *result;
    int32_t status = neoclr_text_allocate_v1(arena, length, &result);
    if (status) return status;
    memcpy(result->bytes, bytes, length);
    *output = result;
    return 0;
}
