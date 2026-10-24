#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"interface dispatch line %d: %s\n",__LINE__,#x); abort(); } } while(0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    for(unsigned round=0;round<140;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *receiver,*output,*keeper;
        unsigned mode=round%7;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&receiver));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&output));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&keeper));
        receiver[0]=mode==1?2:mode==3?3:mode==4?5:1;
        receiver[1]=42;receiver[2]=0;output[0]=4;output[1]=0;keeper[0]=5;keeper[1]=0;
        neoclr_probe_storage slots[]={{&receiver,8,0},{&output,8,0},{&keeper,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,3};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,3,&map,sizeof(map));
        if(mode>=5) {
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&node)) break;
                if(mode==6) { node[0]=5;node[1]=keeper[1];keeper[1]=(uintptr_t)node; }
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        void *argument=mode==2?NULL:receiver;
        receiver=NULL; /* Generated arguments own receiver lifetime across target GC. */
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        int32_t result=777;
        int32_t status=write(argument,output,0,&result,&c);
        experiment_gc_store_counts(after);
        int32_t expected_status=mode==2?6:mode==3?4:mode==4?3:mode==6?5:0;
        if(status!=expected_status) fprintf(stderr,"round %u mode %u status %d expected %d fault %d\n",round,mode,status,expected_status,c.fault.code);
        CHECK(status==expected_status);
        CHECK(neoclr_root_probe_head_v1()==&frame);
        if(!status) {
            CHECK(result==(mode==1?16:8) && output[1]);
            CHECK(after[1]+after[2]>before[1]+before[2]);
        } else CHECK(result==777 && !output[1]);
        c.fault.code=0;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        if(!status) {
            uint64_t bytes;uint32_t kind;
            CHECK(!neoclr_gc_allocation_info_v1(&c,(void *)(uintptr_t)output[1],&bytes,&kind));
            CHECK(kind==NEOCLR_GC_BYTES && ((uint64_t *)(uintptr_t)output[1])[1]==(uint64_t)result);
        }
        output=keeper=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 140 interface-dispatch cycles: public/private targets, allocation, root retention, null/unmatched receivers, target faults, pressure, exhaustion and cleanup");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
int main(void) { return native_managed_consumer(live_managed_writer); }
#endif
