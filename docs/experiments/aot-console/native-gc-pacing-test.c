#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#ifdef NEOCLR_TEST_THREADS
#include <pthread.h>
#endif
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
static int cleanup_publication;
enum { ALLOCATION_BUDGET = 256, PUBLICATION_BUDGET = 1024 };
static void boundary(neoclr_probe_frame *frame) {
    uint64_t empty=0;
    neoclr_probe_stack_roots_v2(frame,0,&empty,0,"{\"requiredSpillLanes\":[]}",25);
}
typedef struct {
    neoclr_aot_context *context;
    neoclr_probe_frame *frame;
    int32_t checked, published;
} cleanup_check;
static void check_cleanup_boundary(void *argument) {
    cleanup_check *state = argument;
    state->checked = neoclr_gc_boundary_v1(state->context, state->frame);
    state->published = neoclr_gc_published_boundary_v1(state->context, state->frame);
    if (cleanup_publication) boundary(state->frame); /* Must abort inside cleanup. */
}
static int exercise(void) {
    uint64_t heap[4096]={0};
    neoclr_aot_context c={.text={(unsigned char*)heap,sizeof(heap),0}},other={0};
    void *root=NULL;neoclr_probe_storage slot={&root,8,0};neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&c,0,&slot,1,"{}",2);
    boundary(&frame);
    uint64_t before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_gc_boundary_v1(&other,&frame)==3);
    CHECK(neoclr_gc_boundary_v1(&c,NULL)==3);
    neoclr_probe_frame stale = frame;
    CHECK(neoclr_gc_boundary_v1(&c,&stale)==3);
    CHECK(neoclr_gc_published_boundary_v1(NULL,&frame)==3);
    CHECK(neoclr_gc_published_boundary_v1(&c,NULL)==3);
    CHECK(neoclr_gc_published_boundary_v1(&other,&frame)==3);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    for (int i=0;i<ALLOCATION_BUDGET-1;i++) CHECK(!neoclr_allocate_object_rooted_v1(&c.text,1,16,&root));
    boundary(&frame);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    CHECK(!neoclr_allocate_object_rooted_v1(&c.text,1,16,&root));
    boundary(&frame);
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    CHECK(c.text.used && ((uint64_t*)root)[0]==1);
    root=NULL;
    /* An unrelated empty ownership scope must not reset this context's budget. */
    boundary(&frame);
    CHECK(!neoclr_gc_heap_acquire_v1(&other.text));
    CHECK(!neoclr_gc_heap_release_v1(&other.text));
    neoclr_parked_roots parked = {0};
    neoclr_roots_park_v1(&parked);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    neoclr_roots_resume_v1(&parked);
    CHECK(neoclr_root_probe_head_v1()==&frame && neoclr_root_probe_depth_v1()==1);
    for(int i=0;i<PUBLICATION_BUDGET-2;i++) boundary(&frame);
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    boundary(&frame);
    CHECK(neoclr_gc_statistics_v1().collections==before+2 && !c.text.used);
    /* Explicit collections cannot starve this context's cleanup budget. */
    CHECK(!neoclr_gc_collect_v1(&other,NULL));
    before=neoclr_gc_statistics_v1().collections;
    boundary(&frame);
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    CHECK(!neoclr_allocate_object_rooted_v1(&c.text,1,16,&root));
    cleanup_check state = {.context=&c, .frame=&frame, .checked=-1, .published=-1};
    neoclr_gc_resource resource = {0};
    CHECK(!neoclr_gc_resource_attach_v1(&c,root,&resource,&state,check_cleanup_boundary));
    before=neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_gc_resource_release_v1(&c,&resource));
    CHECK(state.checked==3 && state.published==3);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    boundary(&frame); /* Cleanup guard is restored after callback exit. */
    root=NULL;
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_idle_v1(&c,NULL) && !c.text.used);
    return 0;
}

#ifdef NEOCLR_TEST_THREADS
static void *worker(void *argument) {
    int *status = argument;
    neoclr_gc_statistics initial = neoclr_gc_statistics_v1();
    *status = initial.allocations || initial.collections ? 1 : exercise();
    return NULL;
}
#endif
int main(int argc, char **argv) {
    if (argc > 1) {
        if (argc != 2 || strcmp(argv[1], "cleanup-publication")) return 2;
        cleanup_publication = 1;
    }
    CHECK(!exercise());
#ifdef NEOCLR_TEST_THREADS
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    pthread_t threads[2];
    int status[2] = {-1, -1};
    for (unsigned i = 0; i < 2; i++) CHECK(!pthread_create(&threads[i], NULL, worker, &status[i]));
    for (unsigned i = 0; i < 2; i++) CHECK(!pthread_join(threads[i], NULL) && !status[i]);
    neoclr_gc_statistics after = neoclr_gc_statistics_v1();
    CHECK(before.allocations == after.allocations && before.collections == after.collections);
#endif
    return 0;
}
