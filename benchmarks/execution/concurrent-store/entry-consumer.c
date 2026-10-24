#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
typedef int32_t (*entry_fn)(int32_t,int32_t *,neoclr_aot_context *);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"public entry line %d: %s\n",__LINE__,#x); abort(); } } while (0)
int managed_entry_consumer(entry_fn entry) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *old;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&old));
        old[0]=17;old[1]=99;
        uint64_t handle;
        CHECK(!neoclr_gc_host_root_create_v1(&c,old,&handle));
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        int32_t result=777;
        uint64_t used=c.text.used;
        CHECK(entry(1,&result,&c)==3 && result==777);
        CHECK(c.text.used==used && old[1]==99 && experiment_gc_live_active(&c));
        c.fault.code=0;
        CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
        CHECK(!entry(1,&result,&c) && result==42 && !c.fault.code);
        CHECK(!neoclr_root_probe_head_v1() && !experiment_gc_live_active(&c));
        uint64_t extent=c.text.used;
        CHECK(extent && !entry(2,&result,&c) && result==21 && c.text.used==extent);
        CHECK(!experiment_gc_begin(&c,NULL,NULL,0));
        result=777;
        CHECK(entry(0,&result,&c)!=0 && result==777 && c.fault.code);
        CHECK(!neoclr_root_probe_head_v1() && !experiment_gc_live_active(&c));
        CHECK(!entry(3,&result,&c) && result==14 && !c.fault.code);
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 100 public-entry cycles: rejection, allocation, repeat, fault and recovery");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t neoclr_entry_v4(int32_t,int32_t *,neoclr_aot_context *);
int main(void) { return managed_entry_consumer(neoclr_entry_v4); }
#endif
