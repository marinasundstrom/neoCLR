#include "native-gc.h"
#include <stdlib.h>
#include <stdio.h>
extern void experiment_gc_candidate_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static void cleanup(void *p) { (*(int *)p)++; }
int main(void) {
    uint64_t heap[256] = {0}, foreign[32] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    neoclr_aot_context other = {.text = {(unsigned char *)foreign, sizeof(foreign), 0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, (void **)&root));
    root[0] = 17;
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_v1(&c, root, &handle));
    CHECK(!neoclr_gc_idle_v1(&c, NULL));
    uint64_t counts[6]; experiment_gc_candidate_counts(counts);
    CHECK(counts[0] == 1 && counts[5] == 1);
    /* Pressure must consume the pending mark result, then synchronously reclaim
     * floating new garbage so the original one-retry allocation can succeed. */
    for (unsigned i = 0; i < 200; i++) {
        void *garbage;
        CHECK(!neoclr_gc_allocate_rooted_v1(&c, NULL, 64, NEOCLR_GC_OBJECT, &garbage));
    }
    experiment_gc_candidate_counts(counts);
    CHECK(counts[1] == 1 && counts[3] == 1 && !counts[5] && root[0] == 17);
    CHECK(neoclr_gc_trigger_profile_v1().requests[NEOCLR_GC_TRIGGER_PRESSURE] > 0);
    /* Foreign-context work cancels without interpreting old-context roots. */
    CHECK(!neoclr_gc_idle_v1(&c, NULL));
    CHECK(!neoclr_gc_collect_v1(&other, NULL));
    experiment_gc_candidate_counts(counts);
    CHECK(counts[2] == 1 && !counts[5]);
    neoclr_aot_text *text;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_TEXT, (void **)&text));
    text->length = 1; text->bytes[0] = 'x';
    const neoclr_aot_text *interned;
    CHECK(!neoclr_string_intern_v1(text, &c.text, &interned) && interned == text);
    /* Idle retains its existing cleanup semantics before starting the next epoch. */
    void *owner;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &owner));
    neoclr_gc_resource resource = {0}; int cleaned = 0;
    CHECK(!neoclr_gc_resource_attach_v1(&c, owner, &resource, &cleaned, cleanup));
    CHECK(!neoclr_gc_idle_v1(&c, NULL) && cleaned == 1);
    /* Invocation completion retires intern descriptors; it must cancel first. */
    CHECK(!neoclr_gc_complete_invocation_v1(&c));
    experiment_gc_candidate_counts(counts);
    CHECK(counts[2] == 2 && !counts[5]);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_idle_v1(&c, NULL));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    experiment_gc_candidate_counts(counts);
    CHECK(counts[2] == 3 && !counts[5]);
    CHECK(counts[0] == counts[1] + counts[2]);
    puts("PASS pressure, idle cleanup, foreign context, invocation completion and release");
    return 0;
}
