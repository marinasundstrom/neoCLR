#include "native-gc.h"
#include "../memory/text-arena.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern uint64_t experiment_gc_live_capabilities_v1(uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_live_wait(void);
extern void experiment_gc_store_counts(uint64_t *);
extern int experiment_gc_store_text(neoclr_aot_context *,void *,uint32_t,uint64_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"text arrays line %d\n",__LINE__); abort(); } } while (0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    CHECK((experiment_gc_live_capabilities_v1(1)&15)==15);
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *first,*second,*next;
        unsigned mode=round%4;
        CHECK(!neoclr_allocate_strings_v1(&c.text,mode==2?0:1,mode!=0,(void **)&first));
        CHECK(!neoclr_allocate_strings_v1(&c.text,mode==3?0:1,mode!=0,(void **)&second));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&next));
        next[0]=3;next[1]=UINT64_C(0x636261);
        neoclr_probe_storage slots[]={{&first,8,0},{&second,8,0},{&next,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,3};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,3,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        CHECK(experiment_gc_store_text(&c,first,(uint32_t)first[1],(uintptr_t)next)==3);
        CHECK(experiment_gc_store_text(&c,next,0,0)==3);
        int32_t result=-1;
        int32_t status=write(first,second,(uintptr_t)next,&result,&c);
        experiment_gc_store_counts(after);
        CHECK(status==(mode>=2?8:0));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        if(mode!=2) CHECK(first[2]==(uintptr_t)next && (mode==0 || ((unsigned char *)(first+3))[0]==1));
        if(mode<2) CHECK(second[2]==(uintptr_t)next && (mode==0 || ((unsigned char *)(second+3))[0]==1) && !result);
        if(mode==2) CHECK(!second[2]);
#ifdef NEOCLR_GC_PACED
        CHECK(experiment_gc_live_active(&c));
        CHECK(after[2]-before[2]==(mode<2?2:mode==3?1:0));
        if (mode<2) {
            experiment_gc_live_wait();
            unsigned writes=0;
            while (experiment_gc_live_active(&c) && writes++<100)
                CHECK(!experiment_gc_store_text(&c,first,0,(uintptr_t)next));
            CHECK(!experiment_gc_live_active(&c) && first[2]==(uintptr_t)next);
        }
#endif
        CHECK(after[1]>before[1]); /* Managed temporary array initialization. */
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && *next==3 && next[1]==UINT64_C(0x636261));
        first=second=next=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
int main(void) { CHECK(!native_managed_consumer(live_managed_writer)); puts("PASS 100 managed text-array publication/bounds cycles"); }
#endif
