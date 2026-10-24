#include "native-gc.h"
#include "../memory/text-arena.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"callable arrays line %d: %s\n",__LINE__,#x); abort(); } } while(0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *first,*second,*next,*keeper;
        unsigned mode=round%6;
        CHECK(!neoclr_allocate_strings_v1(&c.text,mode==2?0:1,mode!=0,(void **)&first));
        CHECK(!neoclr_allocate_strings_v1(&c.text,mode==3?0:1,mode!=0,(void **)&second));
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&next));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&keeper));
        next[0]=1; next[1]=42; next[2]=0; keeper[0]=17; keeper[1]=0;
        neoclr_probe_storage slots[]={{&first,8,0},{&second,8,0},{&next,8,0},{&keeper,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,4};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,4,&map,sizeof(map));
        if(mode>=4) {
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&node)) break;
                if(mode==5) { node[0]=17;node[1]=keeper[1];keeper[1]=(uintptr_t)node; }
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        int32_t result=-1;
        if(mode==1) {
            CHECK(write(first,second,0,&result,&c)==3 && result==-1);
            c.fault.code=0;
        }
        int32_t status=write(first,second,(uintptr_t)next,&result,&c);
        CHECK(status==(mode==2 || mode==3?8:mode==5?5:0));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        if(!status) CHECK(result==42 && first[2] && first[2]==second[2]);
        if(mode==2 || mode==5) CHECK(!second[2]);
        uint64_t *expected=next;next=NULL;c.fault.code=0;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        if(!status || mode==3) {
            uint64_t bytes;uint32_t kind;
            CHECK(!neoclr_gc_allocation_info_v1(&c,expected,&bytes,&kind));
            CHECK(((uint64_t *)(uintptr_t)first[2])[2]==(uintptr_t)expected);
            CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
            CHECK(!write(first,second,0,&result,&c) && result==42);
            CHECK(!neoclr_gc_collect_v1(&c,&frame));
            CHECK(expected[2] && !neoclr_gc_allocation_info_v1(&c,(void *)(uintptr_t)expected[2],&bytes,&kind));
        }
        if(!status) {
            CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&next));
            next[0]=1;next[1]=43;next[2]=0;
            CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
            CHECK(!write(first,second,(uintptr_t)next,&result,&c) && result==43);
            next=NULL;
            CHECK(!neoclr_gc_collect_v1(&c,&frame));
            CHECK(!write(first,second,0,&result,&c) && result==43);
        }
        first=second=keeper=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 120 callable-array cycles: normal/reserved arrays, uninitialized reads, bounds, pressure, exhaustion, replacement and retained receivers, invocation after GC and cleanup");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
int main(void) { return native_managed_consumer(live_managed_writer); }
#endif
