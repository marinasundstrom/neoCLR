#include "native-gc.h"
#include <string.h>
#define CHECK(test) do { if (!(test)) return __LINE__; } while (0)
static void *allocate(neoclr_aot_context *c, uint64_t bytes, uint32_t kind) {
    void *p = NULL;
    return neoclr_gc_allocate_v1(&c->text, bytes, kind, &p) ? NULL : p;
}
static int conditional_roots(void) {
    uint64_t storage[256] = {0}, foreign_storage[32] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    neoclr_aot_context foreign = {.text = {(unsigned char *)foreign_storage, sizeof(foreign_storage), 0}};
    uint64_t *owner = allocate(&c, 16, NEOCLR_GC_OBJECT);
    uint64_t *value = allocate(&c, 24, NEOCLR_GC_OBJECT);
    CHECK(owner && value);
    value[1] = (uintptr_t)owner; value[2] = 42;
    neoclr_gc_conditional_root edge = {owner, value, -1};
    void *root = owner;
    neoclr_probe_storage slot = {&root, 8, 0};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, &edge, 1));
    CHECK(edge.retained && value[2] == 42);
    root = NULL;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, &edge, 1));
    CHECK(!edge.retained && !edge.owner && !edge.value && !c.text.used);
    /* Cleared entries cannot accidentally retain a reused allocation address. */
    CHECK(allocate(&c, 16, NEOCLR_GC_OBJECT) == owner);
    CHECK(!neoclr_gc_collect_conditional_v1(&c, NULL, &edge, 1) && !c.text.used);

    uint64_t *a = allocate(&c, 16, NEOCLR_GC_OBJECT);
    uint64_t *b = allocate(&c, 16, NEOCLR_GC_OBJECT);
    uint64_t *d = allocate(&c, 24, NEOCLR_GC_OBJECT);
    CHECK(a && b && d);
    d[1] = (uintptr_t)a; d[2] = 63;
    neoclr_gc_conditional_root chain[] = {{b, d, -1}, {a, b, -1}};
    root = a;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, chain, 2));
    CHECK(chain[0].retained && chain[1].retained && d[2] == 63);
    /* Independently rooted callback keeps its owner and downstream graph live. */
    root = d;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, chain, 2));
    CHECK(chain[0].retained && chain[1].retained);
    root = NULL;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, chain, 2));
    CHECK(!chain[0].retained && !chain[1].retained && !c.text.used);

    a = allocate(&c, 16, NEOCLR_GC_OBJECT);
    b = allocate(&c, 16, NEOCLR_GC_OBJECT);
    CHECK(a && b);
    chain[0] = (neoclr_gc_conditional_root){a, b, -1};
    chain[1] = (neoclr_gc_conditional_root){b, a, -1};
    CHECK(!neoclr_gc_collect_conditional_v1(&c, NULL, chain, 2) && !c.text.used);
    CHECK(!chain[0].owner && !chain[1].owner);

    /* Several dependents share a key, but a rooted value does not by itself
     * retain that key when there is no managed back reference. */
    a = allocate(&c, 16, NEOCLR_GC_OBJECT);
    b = allocate(&c, 16, NEOCLR_GC_OBJECT);
    d = allocate(&c, 16, NEOCLR_GC_OBJECT);
    CHECK(a && b && d);
    b[1] = 17; d[1] = 19;
    chain[0] = (neoclr_gc_conditional_root){a, b, -1};
    chain[1] = (neoclr_gc_conditional_root){a, d, -1};
    root = a;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, chain, 2));
    CHECK(chain[0].retained && chain[1].retained && b[1] == 17 && d[1] == 19);
    root = b;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, chain, 2));
    CHECK(!chain[0].retained && !chain[1].retained && b[1] == 17);
    root = NULL;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, NULL, chain, 2) && !c.text.used);

    /* A failure discovered while tracing a dependent must not publish results. */
    a = allocate(&c, 16, NEOCLR_GC_OBJECT);
    b = allocate(&c, 8, NEOCLR_GC_STRINGS);
    CHECK(a && b);
    edge = (neoclr_gc_conditional_root){a, b, -1};
    root = a;
    uint64_t collections = neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_gc_collect_conditional_v1(&c, &frame, &edge, 1) == 3);
    CHECK(edge.owner == a && edge.value == b && edge.retained == -1 &&
        neoclr_gc_statistics_v1().collections == collections);
    root = NULL;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, NULL, &edge, 1) && !c.text.used);

    a = allocate(&c, 16, NEOCLR_GC_OBJECT);
    void *alien = allocate(&foreign, 16, NEOCLR_GC_OBJECT);
    CHECK(a && alien);
    neoclr_gc_conditional_root invalid[] = {
        {alien, a, -1}, {a, alien, -1}, {(char *)a + 1, a, -1}, {NULL, a, -1}
    };
    uint64_t before = c.text.used;
    for (unsigned i = 0; i < sizeof(invalid) / sizeof(invalid[0]); i++) {
        neoclr_gc_conditional_root saved = invalid[i];
        CHECK(neoclr_gc_collect_conditional_v1(&c, NULL, &invalid[i], 1) == 3);
        CHECK(invalid[i].owner == saved.owner && invalid[i].value == saved.value &&
            invalid[i].retained == -1 && c.text.used == before);
    }
    CHECK(neoclr_gc_collect_conditional_v1(&c, NULL, NULL, 1) == 3);
    CHECK(neoclr_gc_collect_conditional_v1(&c, NULL, &edge, NEOCLR_GC_HOST_ROOT_LIMIT + 1) == 3);
    edge = (neoclr_gc_conditional_root){a, NULL, -1};
    root = a;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, &frame, &edge, 1) && edge.retained);
    root = NULL;
    CHECK(!neoclr_gc_collect_conditional_v1(&c, NULL, &edge, 1) && !edge.retained && !c.text.used);
    CHECK(!neoclr_gc_collect_v1(&foreign, NULL) && !foreign.text.used);
    return 0;
}
int main(void) {
    uint64_t buffer[129];
    buffer[128] = UINT64_C(0x1122334455667788);
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}};
    uint64_t *a = allocate(&c, 24, NEOCLR_GC_OBJECT);
    uint64_t *b = allocate(&c, 24, NEOCLR_GC_OBJECT);
    CHECK(a && b);
    a[1] = (uintptr_t)b; b[1] = (uintptr_t)a; a[2] = 42;
    uint64_t interior = (uintptr_t)&a[2];
    neoclr_probe_storage slot = {&interior, 8, 2};
    neoclr_probe_frame frame = {.context = &c, .storage = &slot, .storage_count = 1};
    uint64_t used = c.text.used;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == used && a[2] == 42 && b[1] == (uintptr_t)a);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 0); /* unreachable cycle */
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == 2);
    uint64_t *text = allocate(&c, 16, NEOCLR_GC_TEXT);
    uint64_t *dead = allocate(&c, 16, NEOCLR_GC_TEXT);
    uint64_t *strings = allocate(&c, 34, NEOCLR_GC_STRINGS);
    CHECK(text && dead && strings);
    text[0] = 1; ((char *)(text + 1))[0] = 'x';
    strings[0] = UINT64_C(0x80000004); strings[1] = 2;
    strings[2] = (uintptr_t)text; strings[3] = (uintptr_t)dead;
    ((unsigned char *)strings)[32] = 1; /* Only initialized element zero is traced. */
    interior = (uintptr_t)strings;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && text[0] == 1 && neoclr_gc_statistics_v1().reclaimed_allocations == 3);
    void *reuse = allocate(&c, 16, NEOCLR_GC_BYTES);
    CHECK(reuse == dead); /* free-list reuse without moving retained data */
    interior = 0;
    c.fault.code = 4; c.fault.message = (const void *)text;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && text[0] == 1);
    c.fault.code = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 0);
    /* Atomic bytes that resemble an address do not trace their payload. */
    uint64_t *atomic = allocate(&c, 16, NEOCLR_GC_BYTES);
    dead = allocate(&c, 16, NEOCLR_GC_TEXT);
    CHECK(atomic && dead); atomic[1] = (uintptr_t)dead; interior = (uintptr_t)atomic;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 48);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    /* Live-set exhaustion must neither collect inside allocation nor publish output. */
    void *large = allocate(&c, 992, NEOCLR_GC_BYTES);
    CHECK(large && c.text.used == 1024);
    void *output = (void *)(uintptr_t)1;
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_TEXT, &output) == 5 && output == (void *)(uintptr_t)1);
    interior = (uintptr_t)large;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && c.text.used == 1024);
    CHECK(neoclr_gc_allocate_v1(&c.text, 8, NEOCLR_GC_TEXT, &output) == 5);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    /* Typed reference-array tags preserve storage tracing and reject malformed
     * type indices or multi-lane records disguised as reference arrays. */
    uint64_t *references = allocate(&c, 42, NEOCLR_GC_RECORDS);
    uint64_t *child = allocate(&c, 16, NEOCLR_GC_OBJECT);
    dead = allocate(&c, 16, NEOCLR_GC_OBJECT);
    CHECK(references && child && dead);
    references[0] = (UINT64_C(512) << 32) | UINT64_C(0x80000005);
    references[1] = 2; references[2] = 1;
    references[3] = (uintptr_t)child; references[4] = (uintptr_t)dead;
    ((unsigned char *)references)[40] = 1;
    interior = (uintptr_t)references;
    uint64_t reclaimed = neoclr_gc_statistics_v1().reclaimed_allocations;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == reclaimed + 1);
    child[1] = 42;
    for (uint64_t type = 1; type <= 512; type += 511) {
        references[0] = (type << 32) | UINT64_C(0x80000005);
        CHECK(!neoclr_gc_collect_v1(&c, &frame) && child[1] == 42);
    }
    references[0] = (UINT64_C(513) << 32) | UINT64_C(0x80000005);
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    references[0] = (UINT64_C(1) << 32) | UINT64_C(0x80000006);
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    references[0] = (UINT64_C(1) << 32) | UINT64_C(0x80000005);
    references[1] = 0; references[2] = 2;
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    references[1] = 2; references[2] = 1;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && child[1] == 42);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    /* A malformed descriptor may interrupt traversal with other objects queued. */
    uint64_t *owner = allocate(&c, 24, NEOCLR_GC_OBJECT);
    uint64_t *queued = allocate(&c, 16, NEOCLR_GC_OBJECT);
    uint64_t *bad = allocate(&c, 16, NEOCLR_GC_STRINGS);
    CHECK(owner && queued && bad);
    owner[1] = (uintptr_t)queued; owner[2] = (uintptr_t)bad;
    interior = (uintptr_t)owner;
    uint64_t collections = neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_gc_collect_v1(&c, &frame) == 3);
    CHECK(neoclr_gc_statistics_v1().collections == collections);
    CHECK(allocate(&c, 8, NEOCLR_GC_BYTES)); /* queue links were cleared on failure */
    bad[0] = UINT64_C(0x80000003); bad[1] = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame));
    CHECK(owner[1] == (uintptr_t)queued && owner[2] == (uintptr_t)bad);
    interior = 0;
    CHECK(!neoclr_gc_collect_v1(&c, &frame) && !c.text.used);
    /* Two independently parked native chains must remain precise strong roots. */
    neoclr_probe_frame first, second;
    neoclr_parked_roots first_roots = {0}, second_roots = {0};
    uint64_t *retained = allocate(&c, 16, NEOCLR_GC_OBJECT);
    uint64_t *other = allocate(&c, 16, NEOCLR_GC_OBJECT);
    CHECK(retained && other);
    retained[1] = 42; other[1] = 17;
    neoclr_probe_storage first_slot = {&retained, 8, 0};
    neoclr_probe_storage second_slot = {&other, 8, 0};
    neoclr_probe_enter_v3(&first, &c, 1, &first_slot, 1, "{}", 2);
    neoclr_roots_park_v1(&first_roots);
    CHECK(!neoclr_root_probe_head_v1() && neoclr_root_probe_depth_v1() == 0);
    CHECK(neoclr_gc_entry_check_v1(&c) == 3);
    CHECK(neoclr_gc_owned_roots_check_v1(&c, NULL, 0) == 3);
    neoclr_probe_enter_v3(&second, &c, 2, &second_slot, 1, "{}", 2);
    neoclr_roots_park_v1(&second_roots);
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    CHECK(retained[1] == 42 && other[1] == 17);
    neoclr_roots_resume_v1(&first_roots); /* deliberately resume out of parking order */
    CHECK(neoclr_root_probe_head_v1() == &first && neoclr_root_probe_depth_v1() == 1);
    neoclr_probe_leave_v1(&first);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && other[1] == 17);
    neoclr_roots_discard_v1(&second_roots); /* terminal invocation abandonment */
    CHECK(!neoclr_roots_parked_v1());
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    CHECK(buffer[128] == UINT64_C(0x1122334455667788));
    int conditional_status = conditional_roots();
    if (conditional_status) return conditional_status;
    return 0;
}
