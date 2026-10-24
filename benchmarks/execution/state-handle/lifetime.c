#include "native-gc.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); return 1; } } while(0)
void *experiment_state(void);
void experiment_stack(void *,neoclr_probe_frame *,uint32_t,const uint64_t *,uint32_t,const void *,uint32_t);
void experiment_transient(void *,neoclr_probe_frame *,uint32_t,const neoclr_probe_storage *,uint32_t,const void *,uint32_t);
void experiment_leave(void *,neoclr_probe_frame *);
static const char *failure;
static int exercise(void) {
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1() && !neoclr_root_probe_count_v1());
    uint64_t heap[1024]={0}, empty=0;
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}}, other={0};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    void *value=NULL,*second=NULL; uint64_t bytes; uint32_t kind;
    CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&value));
    neoclr_probe_storage slot={&value,8,0};
    neoclr_gc_root_map sm={NEOCLR_GC_STORAGE_MAP_MAGIC,1}, tm={NEOCLR_GC_TRANSIENT_MAP_MAGIC,1}, rm={NEOCLR_GC_ROOT_MAP_MAGIC,0};
    neoclr_probe_frame frame,child;
    neoclr_gc_enter_map_v1(&frame,&c,0,&slot,1,&sm,sizeof(sm));
    void *state=experiment_state(); CHECK(state==experiment_state());
    if (failure && !strcmp(failure,"null")) state=NULL;
    if (failure && !strcmp(failure,"map")) rm.magic=0;
    experiment_stack(state,&frame,0,&empty,0,&rm,sizeof(rm));
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(!neoclr_gc_allocation_info_v1(&c,value,&bytes,&kind));
    void *old=value;
    CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_BYTES,&second));
    value=second; /* Cached state must never imply cached root values. */
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(neoclr_gc_allocation_info_v1(&c,old,&bytes,&kind)==3);
    CHECK(!neoclr_gc_allocation_info_v1(&c,value,&bytes,&kind) && bytes==16);
    neoclr_parked_roots parked={0};
    neoclr_roots_park_v1(&parked);
    if (failure && !strcmp(failure,"parked")) experiment_stack(state,&frame,0,&empty,0,&rm,sizeof(rm));
    neoclr_gc_enter_map_v1(&child,&other,1,&slot,1,&sm,sizeof(sm));
    CHECK(state==experiment_state()); /* Reentry changes head, not thread identity. */
    if (failure && !strcmp(failure,"head")) experiment_leave(state,&frame);
    experiment_stack(state,&child,0,&empty,0,&rm,sizeof(rm));
    experiment_leave(state,&child);
    neoclr_roots_resume_v1(&parked);
    CHECK(state==experiment_state() && neoclr_root_probe_head_v1()==&frame);
    CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_BYTES,&second));
    neoclr_probe_storage temporary={&second,8,0};
    experiment_transient(state,&frame,failure && !strcmp(failure,"phase") ? 4:2,&temporary,1,&tm,sizeof(tm));
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(!neoclr_gc_allocation_info_v1(&c,second,&bytes,&kind) && bytes==24);
    if (failure && !strcmp(failure,"fault")) { c.fault.code=3; c.fault.frame_count=65; }
    experiment_stack(state,&frame,1,&empty,0,&rm,sizeof(rm));
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    CHECK(neoclr_gc_allocation_info_v1(&c,second,&bytes,&kind)==3);
    experiment_leave(state,&frame);
    if (failure && !strcmp(failure,"leave")) experiment_leave(state,&frame);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
    CHECK(!neoclr_gc_collect_v1(&c,NULL));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    return 0;
}
static void *thread(void *unused) { (void)unused; return (void *)(uintptr_t)exercise(); }
int main(int argc,char **argv) {
    failure=argc>1?argv[1]:NULL;
    CHECK(!exercise());
    if(failure) return 2;
    uint64_t count=neoclr_root_probe_count_v1();
    pthread_t a,b; void *ar,*br;
    CHECK(!pthread_create(&a,NULL,thread,NULL) && !pthread_create(&b,NULL,thread,NULL));
    CHECK(!pthread_join(a,&ar) && !pthread_join(b,&br) && !ar && !br);
    CHECK(neoclr_root_probe_count_v1()==count);
    return 0;
}
