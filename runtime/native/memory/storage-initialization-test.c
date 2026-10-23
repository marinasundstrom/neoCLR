#include "../gc/native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    const int lengths[] = {0, 1, 9, 33};
    for (unsigned n = 0; n < sizeof(lengths) / sizeof(lengths[0]); n++)
    for (int mode = 0; mode < 10; mode++) {
        uint64_t heap[1024];
        memset(heap, 0xa5, sizeof(heap));
        neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
        int length = lengths[n];
        for (int reuse = 0; reuse < 2; reuse++) {
            void *out = NULL;
            uint64_t bytes = 0, header = 0, data = 0, flags = 0;
            switch (mode) {
            case 0:
                CHECK(!neoclr_allocate_object_v1(&c.text, 7, 8 + 8 * length, &out));
                header = 8; data = 8 * length;
                CHECK(*(uint64_t *)out == 7);
                break;
            case 1:
                CHECK(!neoclr_static_storage_v1(&c, 1, 3, 8 + 8 * length, &out));
                header = 8; data = 8 * length;
                CHECK(*(uint64_t *)out == 1);
                CHECK(((uint64_t *)c.static_storage)[0] == 3);
                CHECK(!((uint64_t *)c.static_storage)[1] && !((uint64_t *)c.static_storage)[3]);
                break;
            case 2: case 3:
                CHECK(!(mode == 2 ? neoclr_allocate_bytes_v1(&c.text, length, &out)
                    : neoclr_reserve_bytes_v1(&c.text, length, &out)));
                header = 16; data = length * (mode == 2 ? 1 : 2);
                break;
            case 4: case 5:
                CHECK(!neoclr_allocate_strings_v1(&c.text, length, mode == 5, &out));
                header = 16; data = length * (mode == 4 ? 8 : 9);
                break;
            case 6:
                CHECK(!neoclr_reserve_records_v1(&c.text, length, 2, &out));
                header = 24; data = length * 17;
                break;
            case 7:
                CHECK(!neoclr_allocate_references_v1(&c.text, length, &out));
                header = 24; data = length * 8; flags = length;
                break;
            case 8: case 9:
                CHECK(!neoclr_allocate_scalars_v1(&c.text, length, mode == 9, &out));
                header = 24; data = length * (mode == 8 ? 8 : 9);
                flags = mode == 8 ? length : 0;
                break;
            }
            unsigned char *raw = out;
            for (uint64_t i = 0; i < data; i++) CHECK(raw[header + i] == 0);
            for (uint64_t i = 0; i < flags; i++) CHECK(raw[header + data + i] == 1);
            bytes = header + data + flags;
            memset(out, 0xff, (size_t)bytes);
            c.static_storage = NULL;
#ifdef NEOCLR_NATIVE_GC
            CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
#else
            c.text.used = 0;
#endif
        }
    }
    puts("PASS fresh dirty storage and reused object/array/static initialization");
    return 0;
}
