/* Regressions for combined index validation and intern-root seeding passes. */
#include "native-gc.h"
#include <string.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t memory[256] = {0}, saved[256];
    neoclr_aot_context context = {.text = {(unsigned char *)memory, sizeof(memory), 0}};
    void *entry1, *entry2, *owner1, *owner2, *garbage;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 8, NEOCLR_GC_INTERN, &entry1));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_TEXT, &owner1));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 8, NEOCLR_GC_INTERN, &entry2));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_TEXT, &owner2));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 16, NEOCLR_GC_OBJECT, &garbage));
    memcpy(entry1, &owner1, 8); memcpy(entry2, &owner2, 8);
    ((uint64_t *)owner1)[1] = 123; ((uint64_t *)owner2)[1] = 456;
    /* Corruption in the last descriptor must reject before clearing earlier marks
     * or installing intrusive queue links. Header layout is private to this test. */
    ((uint64_t *)entry1)[-1] = 3;
    uint64_t span = ((uint64_t *)garbage)[-4];
    ((uint64_t *)garbage)[-4] = 0;
    memcpy(saved, memory, sizeof(memory));
    CHECK(neoclr_gc_collect_v1(&context, NULL) == 3);
    CHECK(!memcmp(saved, memory, sizeof(memory)));
    ((uint64_t *)garbage)[-4] = span;
    for (unsigned i = 0; i < 3; ++i) {
        CHECK(!neoclr_gc_collect_v1(&context, NULL));
        CHECK(((uint64_t *)owner1)[1] == 123 && ((uint64_t *)owner2)[1] == 456);
        CHECK(((uint64_t *)entry1)[-1] == 1 && ((uint64_t *)entry2)[-1] == 1);
    }
    /* Remove the private intern classification to verify ordinary reclamation. */
    ((uint64_t *)entry1)[-2] = NEOCLR_GC_TEXT;
    ((uint64_t *)entry2)[-2] = NEOCLR_GC_TEXT;
    CHECK(!neoclr_gc_collect_v1(&context, NULL) && !context.text.used);
    return 0;
}
