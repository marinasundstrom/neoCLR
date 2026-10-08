#include "text-arena.h"
#include <string.h>
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
#define CHECK(test) do { if (!(test)) return __LINE__; } while (0)
#define TEXT(name, value) const struct { uint64_t length; unsigned char bytes[sizeof(value)]; } name = {sizeof(value) - 1, value}
#define VIEW(value) ((const neoclr_aot_text *)&(value))
int main(void) {
    uint64_t storage[129]; storage[128] = UINT64_C(0x1122334455667788);
    neoclr_aot_context ctx = {.text = {(unsigned char *)storage, 1024, 0}};
    TEXT(left, "資料\0part"); TEXT(right, "leaf\0.txt"); TEXT(root, "/");
    TEXT(empty, ""); TEXT(short_text, "b");
    const neoclr_aot_text *result = NULL;
    CHECK(!neoclr_path_combine_unix_v1(VIEW(left), VIEW(right), &ctx.text, &result));
    CHECK(result->length == left.length + 1 + right.length);
    CHECK(!memcmp(result->bytes, left.bytes, left.length));
    CHECK(result->bytes[left.length] == '/');
    CHECK(!memcmp(result->bytes + left.length + 1, right.bytes, right.length));
    const neoclr_aot_text *combined = result;
    CHECK(!neoclr_path_file_name_unix_v1(combined, &ctx.text, &result));
    CHECK(result != combined && result->length == right.length && !memcmp(result->bytes, right.bytes, right.length));
    CHECK(!neoclr_path_combine_unix_v1(VIEW(empty), result, &ctx.text, &combined));
    CHECK(combined != result && combined->length == result->length && !memcmp(combined->bytes, result->bytes, result->length));
    CHECK(!neoclr_path_file_name_unix_v1(VIEW(root), &ctx.text, &result) && !result->length);
    const neoclr_aot_text *sentinel = (const void *)(uintptr_t)1;
    uint64_t before = ctx.text.used;
    CHECK(neoclr_path_combine_unix_v1(NULL, VIEW(right), &ctx.text, &sentinel) == 3);
    CHECK(neoclr_path_combine_unix_v1(VIEW(left), NULL, &ctx.text, &sentinel) == 3);
    CHECK(neoclr_path_combine_unix_v1(VIEW(left), VIEW(right), NULL, &sentinel) == 3);
    CHECK(neoclr_path_combine_unix_v1(VIEW(left), VIEW(right), &ctx.text, NULL) == 3);
    CHECK(neoclr_path_file_name_unix_v1(NULL, &ctx.text, &sentinel) == 3);
    CHECK(neoclr_path_file_name_unix_v1(VIEW(left), NULL, &sentinel) == 3);
    CHECK(neoclr_path_file_name_unix_v1(VIEW(left), &ctx.text, NULL) == 3);
    CHECK(ctx.text.used == before && sentinel == (const void *)(uintptr_t)1);
    const struct { uint64_t length; unsigned char bytes[1]; } huge = {UINT64_MAX, {'x'}};
    CHECK(neoclr_path_combine_unix_v1(VIEW(huge), VIEW(short_text), &ctx.text, &sentinel) == 5);
    CHECK(neoclr_path_combine_unix_v1(VIEW(short_text), VIEW(huge), &ctx.text, &sentinel) == 5);
    CHECK(ctx.text.used == before && sentinel == (const void *)(uintptr_t)1);
    neoclr_aot_text_arena full = {NULL, 0, 0};
    CHECK(neoclr_path_combine_unix_v1(VIEW(left), VIEW(right), &full, &sentinel) == 5);
    CHECK(neoclr_path_file_name_unix_v1(VIEW(right), &full, &sentinel) == 5);
    CHECK(!full.used && sentinel == (const void *)(uintptr_t)1);
#ifdef NEOCLR_NATIVE_GC
    uint64_t owner = (uintptr_t)combined;
    neoclr_probe_storage slot = {&owner, 8, 2};
    neoclr_probe_frame frame = {.context = &ctx, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&ctx, &frame));
    CHECK(combined->length == right.length && !memcmp(combined->bytes, right.bytes, right.length));
    owner = 0;
    CHECK(!neoclr_gc_collect_v1(&ctx, &frame) && !ctx.text.used);
#endif
    CHECK(storage[128] == UINT64_C(0x1122334455667788));
    return 0;
}
