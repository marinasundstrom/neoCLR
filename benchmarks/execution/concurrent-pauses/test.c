#include "native-gc.h"
#include <stdlib.h>
extern void experiment_gc_pauses(uint64_t *);
#define CHECK(x) do { if (!(x)) abort(); } while (0)
int main(void) {
    uint64_t heap[128] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(!neoclr_gc_idle_v1(&c,NULL));
    CHECK(!neoclr_gc_idle_v1(&c,NULL));
    CHECK(!neoclr_gc_collect_v1(&c,NULL));
    CHECK(!neoclr_gc_collect_conditional_v1(&c,NULL,NULL,0));
    CHECK(!neoclr_gc_complete_invocation_v1(&c));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    CHECK(neoclr_gc_collect_conditional_v1(NULL,NULL,NULL,0) == 3);
    uint64_t data[74], repeated[74];
    experiment_gc_pauses(data); experiment_gc_pauses(repeated);
    uint64_t sum = 0;
    for (unsigned i=0;i<64;i++) sum += data[i];
    CHECK(sum == 7 && data[64] == 7 && data[67] == 1);
    CHECK(data[65] >= data[66]);
#ifdef NEOCLR_PAUSE_TEST_DELAY
    CHECK(data[66] >= UINT64_C(5000000));
#endif
    CHECK(data[68] == 1 && data[69] == 2 && data[70] == 2 && data[71] == 1 && data[72] == 1);
    for (unsigned i=0;i<74;i++) CHECK(data[i] == repeated[i]);
    return 0;
}
