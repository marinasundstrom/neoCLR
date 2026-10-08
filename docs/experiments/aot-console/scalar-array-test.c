#include "text-arena.h"
#include <assert.h>
#include <string.h>
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
int main(void) {
    uint64_t buffer[129];
    buffer[128] = 1234567;
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 1024, 0}};
    void *output = (void *)(uintptr_t)123;
    for (int reserved = 0; reserved <= 1; reserved++) {
        assert(!neoclr_allocate_scalars_v1(&ctx.text, 2, reserved, &output));
        uint64_t *words = output;
        assert(words[0] == UINT64_C(0x80000006) && words[1] == 2 && words[2] == 1);
        assert(!words[3] && !words[4]);
        unsigned char *markers = (unsigned char *)(words + 5);
        assert(markers[0] == !reserved && markers[1] == !reserved);
    }
    output = (void *)(uintptr_t)123;
    uint64_t before = ctx.text.used;
    assert(neoclr_allocate_scalars_v1(&ctx.text, -1, 0, &output) == 3);
    assert(neoclr_allocate_scalars_v1(&ctx.text, 65537, 0, &output) == 7);
    assert(neoclr_allocate_scalars_v1(&ctx.text, 1, 2, &output) == 3);
    assert(neoclr_allocate_scalars_v1(&ctx.text, 65536, 0, &output) == 5);
    assert(output == (void *)(uintptr_t)123 && ctx.text.used == before);
#ifdef NEOCLR_NATIVE_GC
    assert(!neoclr_gc_collect_v1(&ctx, NULL) && !ctx.text.used);
    void *child = NULL;
    assert(!neoclr_gc_allocate_v1(&ctx.text, 8, NEOCLR_GC_BYTES, &child));
    assert(!neoclr_allocate_scalars_v1(&ctx.text, 1, 0, &output));
    ((uint64_t *)output)[3] = (uintptr_t)child;
    uint64_t root = (uintptr_t)output;
    neoclr_probe_storage slot = {&root, 8, 2};
    neoclr_probe_frame frame = {.context = &ctx, .storage = &slot, .storage_count = 1};
    uint64_t reclaimed = neoclr_gc_statistics_v1().reclaimed_allocations;
    assert(!neoclr_gc_collect_v1(&ctx, &frame));
    assert(neoclr_gc_statistics_v1().reclaimed_allocations == reclaimed + 1);
    assert(((uint64_t *)output)[3] == (uintptr_t)child); /* Numeric bits unchanged. */
    root = 0;
    assert(!neoclr_gc_collect_v1(&ctx, &frame) && !ctx.text.used);
#endif
    assert(buffer[128] == 1234567);
}
