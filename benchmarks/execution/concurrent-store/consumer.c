#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_shadow_gate(int);
extern uint64_t experiment_gc_live_capabilities_v1(uint32_t);
typedef int32_t (*writer)(void *,void *,uint64_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"consumer line %d\n",__LINE__); abort(); } } while(0)
void *frame_enter(neoclr_probe_frame *frame, neoclr_aot_context *context) {
    static const neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,0};
    static const neoclr_probe_storage empty={0};
    CHECK(!neoclr_root_probe_head_v1());
    return neoclr_gc_state_enter_v1(frame,context,0,&empty,0,&map,sizeof(map));
}
void frame_leave(void *state,neoclr_probe_frame *frame) {
    CHECK(neoclr_root_probe_head_v1()==frame);
    neoclr_gc_state_leave_v1(state,frame);
    CHECK(!neoclr_root_probe_head_v1());
}
static uint64_t *node(neoclr_aot_context *c) {
    uint64_t *p;
    CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&p));
    p[0]=17; return p;
}
int native_store_consumer(writer write) {
    CHECK(!experiment_gc_live_capabilities_v1(0));
    CHECK(!experiment_gc_live_capabilities_v1(2));
    CHECK((experiment_gc_live_capabilities_v1(1)&7)==7);
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *root=node(&c),*old=node(&c),*old_child=node(&c),*next=node(&c),*next_child=node(&c),*dead=node(&c);
        root[1]=root[2]=(uintptr_t)old;
        old[1]=(uintptr_t)old_child; next[1]=(uintptr_t)next_child;
        neoclr_probe_storage slot={&root,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        experiment_gc_shadow_gate(1);
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        CHECK(write(&c,root+2,0)==3);
        CHECK(!neoclr_root_probe_head_v1()); /* first lowered destination hits a header */
        CHECK(root[1]==(uintptr_t)old && root[2]==(uintptr_t)old);
        CHECK(!write(&c,root,(uintptr_t)next));
        CHECK(!neoclr_root_probe_head_v1());
        CHECK(root[1]==(uintptr_t)next && root[2]==(uintptr_t)next);
        experiment_gc_shadow_gate(0);
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        CHECK(*old==17 && *old_child==17 && *next==17 && *next_child==17);
        CHECK(*dead==UINT64_C(0xdddddddddddddddd));
        root=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
#ifdef CONSUMER_MAIN
extern int32_t mutation_i8(void *,void *,uint64_t);
extern int32_t mutation_i16(void *,void *,uint64_t);
extern int32_t mutation_i32(void *,void *,uint64_t);
extern int32_t mutation_i64(void *,void *,uint64_t);
extern int32_t mutation_f32(void *,void *,uint64_t);
extern int32_t mutation_f64(void *,void *,uint64_t);
extern int32_t framed(void *,void *,uint64_t);
int main(void) {
    CHECK(!native_store_consumer(mutation_i64));
    CHECK(!native_store_consumer(framed));
    writer functions[]={mutation_i8,mutation_i16,mutation_i32,mutation_i64,mutation_f32,mutation_f64};
    unsigned widths[]={1,2,4,8,4,8};
    uint64_t patterns[]={0,UINT64_MAX,UINT64_C(0x8000000000000000),UINT64_C(0x80000000),UINT64_C(0x7ff8000000000123),UINT64_C(0x7fc00123)};
    uint64_t heap[128]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    for(unsigned f=0;f<6;f++) for(unsigned p=0;p<6;p++) {
        unsigned char actual[32],expected[32];
        memset(actual,0xa5,sizeof(actual)); memcpy(expected,actual,sizeof(actual));
        /* Both currently qualified targets are little-endian. */
        memcpy(expected+8,&patterns[p],widths[f]); memcpy(expected+16,&patterns[p],widths[f]);
        CHECK(!functions[f](&c,actual,patterns[p]));
        CHECK(!memcmp(actual,expected,sizeof(actual)));
    }
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    puts("PASS AOT scalar bits and 200 live-GC mutation/reclamation cycles (100 framed)");
}
#endif

typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_store_counts(uint64_t *);
int native_managed_consumer(managed_writer write) {
    CHECK((experiment_gc_live_capabilities_v1(1)&15)==15);
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *root=node(&c),*old=node(&c),*child=node(&c),*next=node(&c),*next_child=node(&c),*dead=node(&c);
        root[1]=root[2]=(uintptr_t)old; old[1]=(uintptr_t)child; next[1]=(uintptr_t)next_child;
        neoclr_probe_storage slots[]={{&root,8,0},{&next,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,2};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,2,&map,sizeof(map));
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        int32_t result=-1;
        CHECK(write(root+3,root+4,0,&result,&c)==3);
        CHECK(neoclr_root_probe_head_v1()==&frame && root[1]==(uintptr_t)old);
        memset(&c.fault,0,sizeof(c.fault));
        uint64_t before[5],after[5];
        experiment_gc_store_counts(before);
        CHECK(!write(root+1,root+2,(uintptr_t)next,&result,&c) && result==0);
        experiment_gc_store_counts(after);
        CHECK(neoclr_root_probe_head_v1()==&frame);
#ifdef NEOCLR_GC_PACED
        CHECK(after[2]-before[2]==2);
        CHECK(experiment_gc_live_active(&c));
        unsigned boundaries=0;
        while (experiment_gc_live_active(&c) && boundaries++<4096)
            CHECK(!neoclr_gc_boundary_v1(&c,&frame));
        CHECK(!experiment_gc_live_active(&c) && boundaries<=4096);
#else
        CHECK(after[1]-before[1]==2);
        CHECK(!experiment_gc_live_active(&c));
#endif
        if (experiment_gc_live_active(&c)) CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        CHECK(root[1]==(uintptr_t)next && root[2]==(uintptr_t)next && *next==17 && *next_child==17);
        CHECK(*dead==UINT64_C(0xdddddddddddddddd));
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(*old==UINT64_C(0xdddddddddddddddd) && *child==UINT64_C(0xdddddddddddddddd));
        root=next=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}

#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
int main(void) {
    CHECK(!native_managed_consumer(live_managed_writer));
    puts("PASS managed value compiler: nested calls, byref writes, faults and 100 live-GC cycles");
}
#endif
