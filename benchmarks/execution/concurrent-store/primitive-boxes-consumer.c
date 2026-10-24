#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
extern uint64_t experiment_gc_live_capabilities_v1(uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_store_counts(uint64_t *);
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"primitive boxes line %d\n",__LINE__); abort(); } } while (0)
typedef int32_t (*box_int)(int32_t,void *,void *);
typedef int32_t (*box_bool)(uint8_t,void *,void *);
typedef int32_t (*query)(uintptr_t,void *,void *);
int managed_primitive_box_consumer(const void **functions) {
    box_int integer=(box_int)functions[0];
    box_bool boolean=(box_bool)functions[1];
    query is_int=(query)functions[2],unbox_int=(query)functions[3];
    query is_bool=(query)functions[4],unbox_bool=(query)functions[5];
    query type_handle=(query)functions[6];
    int32_t (*make_function)(void *,void *)=(int32_t (*)(void *,void *))functions[7];
    CHECK((experiment_gc_live_capabilities_v1(1)&15)==15);
    for (unsigned round=0;round<120;round++) {
        uint64_t heap[128]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *root;
        CHECK(!neoclr_gc_allocate_v1(&c.text,64,NEOCLR_GC_OBJECT,(void **)&root));
        root[0]=1;
        neoclr_probe_storage slots[]={{&root,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,1};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,1,&map,sizeof(map));
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
#ifdef NEOCLR_GC_PACED
        CHECK(experiment_gc_live_active(&c));
#endif
        unsigned pressure=round%3;
        if (pressure) {
            void *garbage;
            int32_t status;
            uint64_t *tail=root+2;
            do {
                status=neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&garbage);
                if (!status && pressure==2) {
                    uint64_t *live=garbage;
                    live[0]=17;
                    CHECK(!experiment_gc_store(&c,tail+1,(uintptr_t)live,8));
                    tail=live;
                }
            } while (!status);
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
        }
        int32_t value=round%4==0?INT32_MIN:round%4==1?INT32_MAX:round%4==2?-1:0;
        uint8_t truth=round%2;
        uint64_t before[5],after[5];
        experiment_gc_store_counts(before);
        int32_t status=integer(value,root+1,&c);
        if (pressure==2) {
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && !root[1]);
            CHECK(boolean(truth,root+2,&c)==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && !root[2]);
        } else {
            CHECK(!status && root[1]);
            CHECK(!boolean(truth,root+2,&c) && root[2] && root[1]!=root[2]);
            experiment_gc_store_counts(after);
            CHECK(after[1]>before[1]);
#ifdef NEOCLR_GC_PACED
            if (!pressure) CHECK(after[2]-before[2]==2);
#endif
            CHECK(!is_int(root[1],root+4,&c) && root[4]==root[1]);
            CHECK(!unbox_int(root[1],root+5,&c) && *(int32_t *)(root+5)==value);
            CHECK(!is_bool(root[2],root+4,&c) && root[4]==root[2]);
            CHECK(!unbox_bool(root[2],root+6,&c) && *(uint8_t *)(root+6)==truth);
            CHECK(!is_int(root[2],root+4,&c) && !root[4]);
            CHECK(!is_bool(root[1],root+4,&c) && !root[4]);
            CHECK(!is_int(0,root+4,&c) && !root[4]);
            CHECK(!is_bool(1,root+4,&c) && !root[4]); /* Tagged text needs no payload read. */
            int32_t untouched=-77;
            CHECK(unbox_int(root[2],&untouched,&c)==11 && untouched==-77);
            CHECK(unbox_int(0,&untouched,&c)==6 && untouched==-77);
            uint8_t byte=99;
            CHECK(unbox_bool(root[1],&byte,&c)==11 && byte==99);
            CHECK(unbox_bool(0,&byte,&c)==6 && byte==99);
            uint64_t int_token=0,bool_token=0,class_token=0,text_token=0;
            CHECK(!type_handle(root[1],&int_token,&c) && int_token);
            CHECK(!type_handle(root[2],&bool_token,&c) && bool_token && bool_token!=int_token);
            CHECK(!type_handle((uintptr_t)root,&class_token,&c) && class_token);
            CHECK(!type_handle(1,&text_token,&c) && text_token);
            CHECK(class_token!=int_token && class_token!=bool_token && class_token!=text_token);
            CHECK(text_token!=int_token && text_token!=bool_token);
            CHECK(!type_handle(root[1],root+4,&c) && root[4]==int_token);
            uint64_t token=77,unknown[2]={12345,0};
            CHECK(type_handle(0,&token,&c)==6 && token==77);
            CHECK(type_handle((uintptr_t)unknown,&token,&c)==3 && token==77);
            CHECK(!make_function(root+7,&c) && root[7]);
            CHECK(!type_handle(root[7],&token,&c) && token && token!=int_token);
            CHECK(!neoclr_gc_collect_v1(&c,&frame));
            CHECK(!type_handle(root[7],root+4,&c) && root[4]==token);
            CHECK(!unbox_int(root[1],&untouched,&c) && untouched==value);
            CHECK(!unbox_bool(root[2],&byte,&c) && byte==truth);
            CHECK(integer(7,root+8,&c)==3); /* Heap header/extent is not a result slot. */
        }
        CHECK(neoclr_root_probe_head_v1()==&frame);
        root=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_box_0(int32_t,void *,void *);
extern int32_t live_box_1(uint8_t,void *,void *);
extern int32_t live_box_2(uintptr_t,void *,void *);
extern int32_t live_box_3(uintptr_t,void *,void *);
extern int32_t live_box_4(uintptr_t,void *,void *);
extern int32_t live_box_5(uintptr_t,void *,void *);
extern int32_t live_box_6(uintptr_t,void *,void *);
extern int32_t live_box_7(void *,void *);
int main(void) {
    const void *functions[]={live_box_0,live_box_1,live_box_2,live_box_3,live_box_4,live_box_5,live_box_6,live_box_7};
    CHECK(!managed_primitive_box_consumer(functions));
    puts("PASS primitive boxes: 120 cycles, heap aliases, extrema, queries, faults, pressure, exhaustion and cleanup");
}
#endif
