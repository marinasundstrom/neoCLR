#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"callbacks line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
typedef int32_t (*callback_fn)(uint64_t,neoclr_aot_context *);
int managed_callback_consumer(callback_fn invoke) {
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *receiver,*descriptor;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&receiver));
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&descriptor));
        unsigned mode=round%6;
        receiver[0]=17;receiver[1]=11;
        descriptor[0]=mode==4?17:UINT32_MAX;
        descriptor[1]=mode==1?2:mode==2?999:1;
        descriptor[2]=(uintptr_t)receiver;
        uint64_t handle;CHECK(!neoclr_gc_host_root_create_v1(&c,descriptor,&handle));
        if(mode==0 && round%12==0) {
            void *garbage;
            while(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,&garbage)) {}
        }
        uint64_t collections=neoclr_gc_statistics_v1().collections;
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0) && experiment_gc_live_active(&c));
        if(mode==3) CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
        neoclr_probe_frame frame;
        neoclr_probe_storage empty={0};
        if(mode==5) neoclr_probe_enter_v3(&frame,&c,1,&empty,0,"{}",2);
        int32_t status=invoke(handle,&c);
        CHECK(status==(mode==0?0:mode==1?1:3));
        CHECK((uint32_t)receiver[1]==(mode==0?42:11));
        if(mode==0) {
            CHECK(receiver[2]);
            if(round%12==0) CHECK(neoclr_gc_statistics_v1().collections>collections);
        }
        CHECK(neoclr_root_probe_head_v1()==(mode==5?&frame:NULL));
        if(mode==5) neoclr_probe_leave_v1(&frame);
        if(mode==1) CHECK(c.fault.code==1);
        c.fault.code=0;
        if(mode!=3) {
            CHECK(!neoclr_gc_collect_v1(&c,NULL));
            uint64_t bytes;uint32_t kind;
            CHECK(!neoclr_gc_allocation_info_v1(&c,receiver,&bytes,&kind));
            if(mode==0) CHECK(!neoclr_gc_allocation_info_v1(&c,(void *)(uintptr_t)receiver[2],&bytes,&kind) && kind==NEOCLR_GC_BYTES);
            CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
        }
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_entry_check_v1(&c));
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 120 generated callback cycles: unit/no-result, receiver writes, faults, invalid handles/identity/descriptor, frame rejection and lifetime");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t neoclr_invoke_void_callback_v1(uint64_t,neoclr_aot_context *);
int main(void) { return managed_callback_consumer(neoclr_invoke_void_callback_v1); }
#endif
