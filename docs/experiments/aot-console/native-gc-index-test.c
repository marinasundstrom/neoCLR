#include "native-gc.h"
#include <assert.h>
#include <stdlib.h>
#include <stdint.h>
int main(void) {
    uint64_t *buffer = calloc(131073, sizeof(uint64_t)); assert(buffer);
    buffer[131072] = 1234567;
    neoclr_aot_context ctx = {.text = {(unsigned char *)buffer, 1048576, 0}};
    uint64_t root = 0;
    neoclr_probe_storage storage = {&root, 8, 2};
    neoclr_probe_frame frame = {.context = &ctx, .storage = &storage, .storage_count = 1};
    uint64_t *previous = NULL;
    for (unsigned i = 0; i < 4096; i++) {
        uint64_t *value = NULL; void *garbage = NULL;
        assert(!neoclr_gc_allocate_v1(&ctx.text, 24 + i % 8, NEOCLR_GC_OBJECT, (void **)&value));
        value[1] = (uintptr_t)previous; value[2] = i;
        previous = value;
        assert(!neoclr_gc_allocate_v1(&ctx.text, 8 + i % 24, NEOCLR_GC_BYTES, &garbage));
    }
    root = (uintptr_t)&previous[2]; /* Retain reverse chain via an interior root. */
    for (unsigned round = 0; round < 4; round++) {
        assert(!neoclr_gc_collect_v1(&ctx, &frame));
        uint64_t *value = previous;
        for (unsigned i = 4096; i; i--) {
            assert(value && value[2] == i - 1);
            value = (void *)(uintptr_t)value[1];
        }
        assert(!value);
        for (unsigned i = 0; i < 512; i++) {
            void *garbage = NULL;
            assert(!neoclr_gc_allocate_v1(&ctx.text, 8, NEOCLR_GC_BYTES, &garbage));
        }
    }
    root = 0;
    assert(!neoclr_gc_collect_v1(&ctx, &frame) && !ctx.text.used);
    for (int offset = -32; offset <= 16; offset++) {
        unsigned char *value = NULL;
        assert(!neoclr_gc_allocate_v1(&ctx.text, 9, NEOCLR_GC_BYTES, (void **)&value));
        root = (uintptr_t)value + offset;
        assert(!neoclr_gc_collect_v1(&ctx, &frame));
        assert((ctx.text.used != 0) == (offset >= 0 && offset < 9));
        root = 0;
        assert(!neoclr_gc_collect_v1(&ctx, &frame) && !ctx.text.used);
    }
    assert(buffer[131072] == 1234567);
    free(buffer);
}
