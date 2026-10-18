#define NEOCLR_GC_CONCURRENT_SNAPSHOT
#define NEOCLR_GC_SNAPSHOT_TEST
#include "native-gc.c"
#include <stdio.h>
#include <assert.h>
static pthread_t owner_thread;
static void cleanup(void *state) { assert(pthread_equal(owner_thread,pthread_self())); ++*(int *)state; }
static void complete(neoclr_aot_context *c) {
    for(int n=0;n<5000;n++) {
        if(atomic_load(&snapshot->phase)==2) { assert(!snapshot_poll(c)); return; }
        struct timespec t={0,1000000}; nanosleep(&t,NULL);
    }
    abort();
}
int main(void) {
    owner_thread=pthread_self();
    uint64_t arena[4096]={0}; neoclr_aot_context c={.text={(void*)arena,sizeof(arena),0}};
    void *parent,*child,*dead,*fresh;
    assert(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&parent));
    assert(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&child));
    assert(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&dead));
    ((uint64_t*)parent)[1]=(uintptr_t)child; *(uint64_t*)child=42;
    void *root=parent; neoclr_probe_storage slot={&root,8,0}; neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&c,0,&slot,1,"{}",2);
    atomic_store(&snapshot_test_hold,1);
    assert(!snapshot_start(&c,&frame));
    for(int n=0;!atomic_load(&snapshot_test_entered);n++) {assert(n<5000);struct timespec t={0,1000000};nanosleep(&t,NULL);}
    ((uint64_t*)parent)[1]=0; root=child;
    assert(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&fresh));
    ((uint64_t*)fresh)[1]=(uintptr_t)child;
    assert(atomic_load(&snapshot->phase)==1);
    atomic_store(&snapshot_test_hold,0); complete(&c);
    assert(*(uint64_t*)child==42 && ((uint64_t*)fresh)[1]==(uintptr_t)child);
    assert(*(uint64_t*)dead==UINT64_C(0xdddddddddddddddd));
    neoclr_probe_leave_v1(&frame); assert(!neoclr_gc_collect_v1(&c,NULL)); assert(!c.text.used);
    puts("mutator progressed during outstanding snapshot; old/new reachable objects survived");
    uint64_t handle=0;
    int cleaned=0; neoclr_gc_resource resource={0};
    assert(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&parent));
    assert(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&child));
    assert(!neoclr_gc_host_root_create_owned_v1(&c,parent,child,&handle));
    assert(!neoclr_gc_resource_attach_v1(&c,parent,&resource,&cleaned,cleanup));
    for(int n=0;n<3;n++) {assert(!snapshot_start(&c,NULL));complete(&c);assert(c.text.used && !cleaned);}
    puts("COUNTEREXAMPLE: three snapshot cycles retain an unrooted conditional owner");
    assert(!neoclr_gc_collect_v1(&c,NULL)); assert(!c.text.used);
    assert(cleaned==1);
    assert(neoclr_gc_host_root_read_v1(&c,handle,&root)==3);
    assert(!neoclr_gc_host_root_release_v1(&c,handle));
    puts("synchronous oracle expires the same owner and reclaims all memory");
}
