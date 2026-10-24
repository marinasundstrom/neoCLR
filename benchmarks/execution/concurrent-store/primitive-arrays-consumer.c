#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"primitive arrays line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *owner;
        CHECK(!neoclr_gc_allocate_v1(&c.text,32,NEOCLR_GC_OBJECT,(void **)&owner));
        owner[0]=17;
        neoclr_probe_storage slot={&owner,8,0};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,1};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,&slot,1,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
        unsigned mode=round%6;
        if(mode==1 || mode==2) {
            void *garbage;int32_t status;uint64_t *tail=owner+2;
            do {
                status=neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&garbage);
                if(!status && mode==2) {
                    uint64_t *live=garbage;live[0]=17;
                    CHECK(!experiment_gc_store(&c,tail+1,(uintptr_t)live,8));tail=live;
                }
            } while(!status);
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
        }
        uint64_t pressure_before=neoclr_gc_trigger_profile_v1().requests[NEOCLR_GC_TRIGGER_PRESSURE];
        int32_t result=-1;
        int32_t status=write(owner+1,owner+2,mode,&result,&c);
        CHECK(status==(mode==2?5:mode==3||mode==4?3:mode==5?8:0));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        if(mode==2) CHECK(!owner[1] && !owner[2] && result==-1);
        else {
            CHECK(result==(mode>=3?-1:141));
            uint64_t *first=(void *)(uintptr_t)owner[1],*second=(void *)(uintptr_t)owner[2];
            CHECK(first && second && first!=second);
            CHECK(first[0]==UINT64_C(0x80000001) && first[1]==8);
            CHECK(second[0]==UINT64_C(0x80000002) && second[1]==8);
            unsigned char *bytes=(void *)(first+2),*reserved=(void *)(second+2);
            CHECK(bytes[0]==42 && reserved[0]==43 && reserved[8]==1);
            for(unsigned i=1;i<8;i++) CHECK(bytes[i]==0 && reserved[i]==0 && reserved[8+i]==0);
#ifdef NEOCLR_GC_PACED
            if(mode==0) CHECK(experiment_gc_live_active(&c));
            if(mode==1) CHECK(neoclr_gc_trigger_profile_v1().requests[NEOCLR_GC_TRIGGER_PRESSURE]>pressure_before);
#endif
            CHECK(!neoclr_gc_collect_v1(&c,&frame));
            CHECK(bytes[0]==42 && reserved[0]==43);
        }
        (void)pressure_before;
        owner=NULL;
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
    puts("PASS 120 primitive array allocation/reservation, pressure, exhaustion, initialization and bounds cycles");
}
#endif
