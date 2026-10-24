#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_candidate_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"entry boundary line %d: %s\n",__LINE__,#x); abort(); } } while (0)
int main(void) {
    for (unsigned round=0; round<100; round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *object;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&object));
        object[0]=17; object[1]=42;
        uint64_t handle;
        CHECK(!neoclr_gc_host_root_create_v1(&c,object,&handle));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(experiment_gc_live_active(&c));
        /* Rejected reentry must preserve both the marker and retained objects. */
        CHECK(neoclr_gc_entry_check_v1(&c)==3);
        CHECK(experiment_gc_live_active(&c) && object[1]==42);
        CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
        uint64_t before[6],after[6];
        experiment_gc_candidate_counts(before);
        CHECK(!neoclr_gc_entry_check_v1(&c));
        experiment_gc_candidate_counts(after);
        CHECK(!experiment_gc_live_active(&c));
        CHECK(after[2]==before[2]+1 && after[1]==before[1]);
        CHECK(object[1]==42); /* Admission cancels; it does not collect. */
        /* Owned arenas require collector-coordinated reclamation before reuse.
         * A direct cursor reset would violate their bookkeeping contract. */
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&object));
        object[0]=17; object[1]=round;
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        CHECK(!neoclr_gc_complete_invocation_v1(&c));
        CHECK(!experiment_gc_live_active(&c));
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 100 entry rejection, cancellation, arena reuse and completion cycles");
}
