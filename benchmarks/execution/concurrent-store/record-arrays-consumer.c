#include "native-gc.h"
#include "../memory/text-arena.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"record arrays line %d: %s\n",__LINE__,#x); abort(); } } while(0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    for(unsigned round=0;round<140;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *first,*second,*next,*child,*keeper;
        unsigned mode=round%7;
        CHECK(!neoclr_reserve_records_v1(&c.text,mode==2?0:mode==4?2:1,2,(void **)&first));
        CHECK(!neoclr_reserve_records_v1(&c.text,mode==3?0:1,2,(void **)&second));
        if(mode==0) { ((unsigned char *)(first+5))[0]=1; ((unsigned char *)(second+5))[0]=1; }
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&next));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&child));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&keeper));
        next[0]=child[0]=keeper[0]=17;next[1]=(uintptr_t)child;
        if(mode==4) first[5]=(uintptr_t)next; /* An uninitialized lane must not retain a graph. */
        neoclr_probe_storage slots[]={{&first,8,0},{&second,8,0},{&next,8,0},{&keeper,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,4};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,4,&map,sizeof(map));
        if(mode>=5) {
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&node)) break;
                if(mode==6) { node[0]=17;node[1]=keeper[1];keeper[1]=(uintptr_t)node; }
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        int32_t result=-1;
        int32_t status=write(first,second,(uintptr_t)next,&result,&c);
        experiment_gc_store_counts(after);
        CHECK(status==(mode==2 || mode==3?8:mode==4?3:mode==6?5:0));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        if(mode==0 || mode==1 || mode==3 || mode==5) {
            CHECK(first[3]==(uintptr_t)next && first[4]==42 && ((unsigned char *)(first+5))[0]==1);
            CHECK(after[1]+after[2]>before[1]+before[2]);
        }
        if(!status) CHECK(second[3]==(uintptr_t)next && second[4]==42 && ((unsigned char *)(second+5))[0]==1 && !result);
        if(mode==2 || mode==4 || mode==6) CHECK(!second[3] && !((unsigned char *)(second+5))[0]);
        if(mode==4) CHECK(!first[3] && !((unsigned char *)(first+7))[0] && !((unsigned char *)(first+7))[1]);
        /* Remove the original root: only an initialized record field retains the graph. */
        uint64_t *expected=next;next=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        if(!status || mode==3) {
            uint64_t bytes;uint32_t kind;
            CHECK(!neoclr_gc_allocation_info_v1(&c,expected,&bytes,&kind));
            CHECK(*expected==17 && *child==17);
        } else {
            uint64_t bytes;uint32_t kind;
            CHECK(neoclr_gc_allocation_info_v1(&c,expected,&bytes,&kind)!=0);
        }
        first=second=keeper=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 140 record-array cycles: multi-lane references, initialization, uninitialized read, bounds, pressure, exhaustion, retention and cleanup");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
int main(void) { return native_managed_consumer(live_managed_writer); }
#endif
