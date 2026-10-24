#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern uint64_t experiment_gc_live_capabilities_v1(uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_store_counts(uint64_t *);
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"construction line %d\n",__LINE__); abort(); } } while (0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
static uint64_t *node(neoclr_aot_context *c) {
    uint64_t *p;
    CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&p));
    p[0]=17;
    return p;
}
int native_managed_consumer(managed_writer write) {
    CHECK((experiment_gc_live_capabilities_v1(1)&15)==15);
    for (unsigned round=0;round<100;round++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *root=node(&c),*next=node(&c),*child=node(&c);
        next[1]=(uintptr_t)child;
        neoclr_probe_storage slots[]={{&root,8,0},{&next,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,2};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,2,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
#ifdef NEOCLR_GC_PACED
        CHECK(experiment_gc_live_active(&c));
#endif
        unsigned pressure=round%3;
        if (pressure) {
            void *garbage;
            int32_t status;
            uint64_t *tail=child;
            do {
                status=neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,&garbage);
                if (!status && pressure==2) {
                    uint64_t *live=garbage;
                    live[0]=17; /* New allocation: not read by the marker. */
                    CHECK(!experiment_gc_store(&c,tail+1,(uintptr_t)live,8));
                    tail=live;
                }
            } while (!status);
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
        }
        uint64_t before[5],after[5];
        uint64_t pressure_before=neoclr_gc_trigger_profile_v1().requests[NEOCLR_GC_TRIGGER_PRESSURE];
        experiment_gc_store_counts(before);
        int32_t result=-1;
        int32_t status=write(root+1,root+2,(uintptr_t)next,&result,&c);
        if (pressure==2) {
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
            CHECK(neoclr_root_probe_head_v1()==&frame && !root[1] && !root[2]);
            CHECK(*next==17 && *child==17);
            root=next=NULL;
            CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
            neoclr_gc_leave_v1(&frame);
            CHECK(!neoclr_gc_heap_release_v1(&c.text));
            continue;
        }
        CHECK(!status && !result);
        experiment_gc_store_counts(after);
        CHECK(neoclr_root_probe_head_v1()==&frame);
        uint64_t *first=(void *)(uintptr_t)root[1],*second=(void *)(uintptr_t)root[2];
        CHECK(first && second && first!=second);
        CHECK(first[1]==(uintptr_t)next && second[1]==(uintptr_t)next);
        CHECK(*next==17 && *child==17);
        CHECK(after[1]>before[1]); /* Constructor writes to newly allocated payload. */
#ifdef NEOCLR_GC_PACED
        if (!pressure) {
            CHECK(experiment_gc_live_active(&c));
            CHECK(after[2]-before[2]==2); /* Publish both objects into old storage. */
        } else {
            CHECK(neoclr_gc_trigger_profile_v1().requests[NEOCLR_GC_TRIGGER_PRESSURE]>pressure_before);
        }
#else
        (void)pressure_before;
#endif
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(first[1]==(uintptr_t)next && second[1]==(uintptr_t)next && *child==17);
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
    puts("PASS managed constructors: 100 cycles: construction, pressure recovery and live-heap exhaustion");
}
#endif
