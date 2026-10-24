#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_store_counts(uint64_t *);
extern int experiment_gc_static_storage_rooted_v1(neoclr_aot_context *,uint32_t,uint32_t,uint32_t,void **);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"statics line %d\n",__LINE__); abort(); } } while (0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *next,*child,*first=NULL,*second=NULL;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&next));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&child));
        next[0]=child[0]=17;next[1]=(uintptr_t)child;
        neoclr_probe_storage slots[]={{&next,8,0},{&first,8,0},{&second,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,3};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,3,&map,sizeof(map));
        unsigned mode=round%4;
        void *storage;
        if(mode!=3) CHECK(!experiment_gc_static_storage_rooted_v1(&c,0,2,16,&storage));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
        if(mode==1 || mode==2) {
            void *garbage;int32_t status;uint64_t *tail=child;
            do {
                status=neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&garbage);
                if(!status && mode==2) {
                    uint64_t *live=garbage;live[0]=17;
                    CHECK(!experiment_gc_store(&c,tail+1,(uintptr_t)live,8));tail=live;
                }
            } while(!status);
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
        }
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        int32_t result=-1;
        int32_t status=write(&first,&second,(uintptr_t)next,&result,&c);
        experiment_gc_store_counts(after);
        CHECK(status==(mode==2?NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT:0));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        uint64_t *table=c.static_storage;
        CHECK(table && table[0]==2);
        CHECK(((uint64_t *)(uintptr_t)table[1])[1]==(uintptr_t)next);
        if(mode==2) CHECK(!first && !second && !table[2]);
        else {
            CHECK(first==next && second==next && !result);
            CHECK(((uint64_t *)(uintptr_t)table[2])[1]==(uintptr_t)next);
#ifdef NEOCLR_GC_PACED
            if(mode==0) { CHECK(experiment_gc_live_active(&c)); CHECK(after[2]-before[2]==2); }
#endif
            /* Both owners exist now: warm field replacement remains guarded. */
            CHECK(!write(&first,&second,(uintptr_t)next,&result,&c));
        }
        next=first=second=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && *child==17);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_complete_invocation_v1(&c) && !c.static_storage);
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
int main(void) { CHECK(!native_managed_consumer(live_managed_writer)); puts("PASS 100 static cold/warm/publication/pressure/lifetime cycles"); }
#endif
