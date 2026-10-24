#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"managed views line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern void experiment_gc_store_counts(uint64_t *);
extern int experiment_gc_live_active(neoclr_aot_context *);
typedef int32_t (*create_fn)(uint64_t,void *,void *);
typedef int32_t (*window_fn)(uint64_t,int32_t,int32_t,void *,void *);
typedef int32_t (*slice_fn)(uint64_t,int32_t,int32_t,int32_t,int32_t,void *,void *);
typedef int32_t (*get_fn)(uint64_t,int32_t,int32_t,int32_t,void *,void *);
typedef int32_t (*set_fn)(uint64_t,int32_t,int32_t,int32_t,int32_t,void *,void *);
typedef int32_t (*length_fn)(uint64_t,int32_t,int32_t,void *,void *);
#define VIEW(p) (p)[0],(int32_t)(p)[1],(int32_t)(p)[2]
int managed_byte_view_consumer(void *const *functions) {
    create_fn create=(create_fn)functions[0],utf8=(create_fn)functions[6];
    window_fn window=(window_fn)functions[1];
    slice_fn slice=(slice_fn)functions[2];
    get_fn get=(get_fn)functions[3];
    set_fn set=(set_fn)functions[4];
    length_fn length=(length_fn)functions[5];
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[256]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        void *array;neoclr_aot_text *text;uint64_t *owner;
        CHECK(!(round&1?neoclr_reserve_bytes_v1(&c.text,8,&array):neoclr_allocate_bytes_v1(&c.text,8,&array)));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&text));
        CHECK(!neoclr_gc_allocate_v1(&c.text,80,NEOCLR_GC_OBJECT,(void **)&owner));
        text->length=3;memcpy(text->bytes,"abc",3);owner[0]=17;
        neoclr_probe_storage slots[]={{&array,8,0},{&text,8,0},{&owner,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,3};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,3,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
        CHECK(!((create_fn)functions[7])((uintptr_t)array,owner+1,&c));
        CHECK(owner[1]==(uintptr_t)array && owner[2]==0 && owner[3]==8);
        CHECK(!((create_fn)functions[8])((uintptr_t)array,owner+1,&c));
        CHECK(owner[1]==(uintptr_t)array && owner[2]==0 && owner[3]==8);
        CHECK(!create((uintptr_t)array,owner+1,&c));
        CHECK(owner[1]==(uintptr_t)array && owner[2]==0 && owner[3]==8);
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        CHECK(!set(VIEW(owner+1),3,88,owner+9,&c));
        experiment_gc_store_counts(after);
        CHECK(after[1]==before[1] && after[2]==before[2] && owner[9]==0);
        CHECK(!get(VIEW(owner+1),3,owner+7,&c));
        experiment_gc_store_counts(after);
#ifdef NEOCLR_GC_PACED
        CHECK(after[2]-before[2]==1 && experiment_gc_live_active(&c));
#else
        CHECK(after[1]-before[1]==1);
#endif
        CHECK(owner[7]==88);
        CHECK(!window((uintptr_t)array,2,4,owner+1,&c));
        CHECK(owner[2]==2 && owner[3]==4);
        CHECK(!length(VIEW(owner+1),owner+8,&c) && owner[8]==4);
        CHECK(!get(VIEW(owner+1),1,owner+7,&c) && owner[7]==88);
        CHECK(!slice(VIEW(owner+1),1,2,owner+1,&c));
        CHECK(owner[2]==3 && owner[3]==2);
        CHECK(get(VIEW(owner+1),2,owner+7,&c)==8 && owner[7]==88);
        CHECK(set(VIEW(owner+1),-1,22,owner+9,&c)==8 && owner[9]==0);
        CHECK(window((uintptr_t)array,-1,1,owner+4,&c)==8);
        CHECK(!owner[4] && !owner[5] && !owner[6]);
        CHECK(!utf8((uintptr_t)text,owner+4,&c));
        CHECK(owner[4]==(uintptr_t)text && owner[5]==UINT32_C(0x80000000) && owner[6]==3);
        CHECK(set(VIEW(owner+4),0,42,owner+9,&c)==3 && text->bytes[0]=='a');
        CHECK(neoclr_root_probe_head_v1()==&frame);
        /* Only heap-published descriptors retain both owners across completion. */
        array=NULL;text=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(!get(VIEW(owner+1),0,owner+7,&c) && owner[7]==88);
        CHECK(((neoclr_aot_text *)(uintptr_t)owner[4])->length==3);
        owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_view_0(uint64_t,void *,void *);
extern int32_t live_view_1(uint64_t,int32_t,int32_t,void *,void *);
extern int32_t live_view_2(uint64_t,int32_t,int32_t,int32_t,int32_t,void *,void *);
extern int32_t live_view_3(uint64_t,int32_t,int32_t,int32_t,void *,void *);
extern int32_t live_view_4(uint64_t,int32_t,int32_t,int32_t,int32_t,void *,void *);
extern int32_t live_view_5(uint64_t,int32_t,int32_t,void *,void *);
extern int32_t live_view_6(uint64_t,void *,void *);
extern int32_t live_view_7(uint64_t,void *,void *);
extern int32_t live_view_8(uint64_t,void *,void *);
int main(void) {
    void *functions[]={(void *)live_view_0,(void *)live_view_1,(void *)live_view_2,(void *)live_view_3,(void *)live_view_4,(void *)live_view_5,(void *)live_view_6,(void *)live_view_7,(void *)live_view_8};
    CHECK(!managed_byte_view_consumer(functions));
    puts("PASS 100 generated byte-view publication, mutation, fault and lifetime cycles");
}
#endif
