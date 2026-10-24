#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
extern int experiment_gc_live_active(neoclr_aot_context *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"native text line %d\n",__LINE__); abort(); } } while (0)
typedef int32_t (*managed_writer)(void *,void *,uint64_t,int32_t *,void *);
int native_managed_consumer(managed_writer write) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        neoclr_aot_text *next,*first=NULL,*second=NULL;
        uint64_t *keeper;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&next));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&keeper));
        next->length=3;memcpy(next->bytes,"abc",3);keeper[0]=17;
        neoclr_probe_storage slots[]={{&next,8,0},{&first,8,0},{&second,8,0},{&keeper,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,4};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,4,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
        unsigned mode=round%4;
        if(mode==1 || mode==2) {
            void *garbage;int32_t status;uint64_t *tail=keeper;
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
        int32_t status=write(&first,&second,mode==3?0:(uintptr_t)next,&result,&c);
        CHECK(status==(mode==2?NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT:mode==3?3:0));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        if(mode>=2) CHECK(!first && !second);
        else {
            CHECK(result==9 && first && second && first!=second);
            CHECK(first->length==6 && !memcmp(first->bytes,"abcabc",6));
            CHECK(second->length==9 && !memcmp(second->bytes,"abcabcabc",9));
#ifdef NEOCLR_GC_PACED
            if(mode==0) CHECK(experiment_gc_live_active(&c));
            if(mode==1) CHECK(neoclr_gc_trigger_profile_v1().requests[NEOCLR_GC_TRIGGER_PRESSURE]>pressure_before);
#endif
        }
        (void)pressure_before;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(next->length==3 && !memcmp(next->bytes,"abc",3));
        next=first=second=NULL;keeper=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
typedef int32_t (*native_concat)(uint64_t,uint64_t,void *,void *);
typedef int32_t (*native_count)(uint64_t,void *,void *);
int native_text_alias_consumer(native_concat concat,native_count count) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        neoclr_aot_text *text;
        uint64_t *owner;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&text));
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&owner));
        text->length=3;memcpy(text->bytes,"abc",3);owner[0]=17;
        neoclr_probe_storage slots[]={{&text,8,0},{&owner,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,2};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,2,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        CHECK(!concat((uintptr_t)text,(uintptr_t)text,owner+1,&c));
        CHECK(!count(owner[1],owner+2,&c));
        experiment_gc_store_counts(after);
#ifdef NEOCLR_GC_PACED
        CHECK(after[2]-before[2]==2 && experiment_gc_live_active(&c));
#else
        CHECK(after[1]-before[1]==2);
#endif
        CHECK(*(int32_t *)(owner+2)==6);
        CHECK(neoclr_root_probe_head_v1()==&frame);
        CHECK(!concat((uintptr_t)text,(uintptr_t)text,owner+1,&c));
        text=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        neoclr_aot_text *result=(void *)(uintptr_t)owner[1];
        CHECK(result->length==6 && !memcmp(result->bytes,"abcabc",6));
        owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_managed_writer(void *,void *,uint64_t,int32_t *,void *);
extern int32_t live_native_concat(uint64_t,uint64_t,void *,void *);
extern int32_t live_native_count(uint64_t,void *,void *);
int main(void) {
    CHECK(!native_managed_consumer(live_managed_writer));
    CHECK(!native_text_alias_consumer(live_native_concat,live_native_count));
    puts("PASS 100 native text allocation/pressure/fault and 100 heap-result-alias cycles");
}
#endif
