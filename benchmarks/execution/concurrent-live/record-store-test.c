#include "native-gc.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int experiment_gc_store_record(neoclr_aot_context *,void *,uint32_t,const void *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern int experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"record line %d\n",__LINE__); abort(); } } while (0)
int main(void) {
    unsigned widths[]={1,2,32,65};
    for (unsigned w=0;w<4;w++) for(unsigned round=0;round<50;round++) {
        uint64_t heap[1024]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        unsigned lanes=widths[w];
        uint64_t *array,*child;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24+4*(8*lanes+1),NEOCLR_GC_RECORDS,(void **)&array));
        array[0]=UINT64_C(0x80000005);array[1]=4;array[2]=lanes;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&child));
        child[0]=17;
        uint64_t values[65];for(unsigned i=0;i<lanes;i++) values[i]=(uintptr_t)child;
        neoclr_probe_storage slot={&array,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        CHECK(experiment_gc_store_record(&c,array,4,values,lanes)==3);
        CHECK(experiment_gc_store_record(&c,array,0,values,lanes+1)==3);
        CHECK(!array[3] && !((unsigned char *)(array+3+4*lanes))[0]);
        CHECK(!experiment_gc_store_record(&c,array,0,values,lanes));
        CHECK(!memcmp(array+3,values,lanes*8));
        CHECK(((unsigned char *)(array+3+4*lanes))[0]==1);
        /* Wide transactions exceed the bounded log and must cancel before copying. */
        if(lanes==65) CHECK(!experiment_gc_live_active(&c));
        if(experiment_gc_live_active(&c)) CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        else CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(*child==17);
        memset(values,0,sizeof(values));
        CHECK(!experiment_gc_store_record(&c,array,0,values,lanes));
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(*child==UINT64_C(0xdddddddddddddddd));
        array=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 200 grouped record writes, validation and wide cancellation");
}
