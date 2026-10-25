#include "native-gc.h"
#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
extern int experiment_gc_store_scalar(neoclr_aot_context *,void *,uint32_t,const void *);
extern int experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"scalar line %d\n",__LINE__); abort(); } } while (0)
struct foreign { neoclr_aot_context *context; void *array; };
static void *foreign_store(void *argument) {
    struct foreign *f=argument;
    uint64_t value=17;
    CHECK(experiment_gc_store_scalar(f->context,f->array,0,&value)==3);
    return NULL;
}
int main(void) {
    /* Includes narrow integer/boolean, negative integer, signed zero and NaN bits.
     * The generated caller supplies normalized zero-padded lanes, not references. */
    const uint64_t patterns[]={0,1,255,65535,UINT32_MAX,UINT64_MAX,
        UINT64_C(0x80000000),UINT64_C(0x7fc00001),UINT64_C(0x8000000000000000),
        UINT64_C(0x7ff8000000000001)};
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *array,*child,*reference;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24+4*9,NEOCLR_GC_BYTES,(void **)&array));
        array[0]=UINT64_C(0x80000006);array[1]=4;array[2]=1;
        unsigned char *markers=(unsigned char *)array+24+4*8;
        CHECK(!neoclr_gc_allocate_v1(&c.text,24+4*9,NEOCLR_GC_RECORDS,(void **)&reference));
        reference[0]=UINT64_C(0x80000005);reference[1]=4;reference[2]=1;
        uint64_t value=23;
        CHECK(experiment_gc_store_scalar(&c,reference,0,&value)==3);
        CHECK(experiment_gc_store_scalar(&c,array,4,&value)==3);
        CHECK(experiment_gc_store_scalar(&c,array+1,0,&value)==3);
        CHECK(experiment_gc_store_scalar(&c,array,0,NULL)==3);
        CHECK(experiment_gc_store_scalar(NULL,array,0,&value)==3);
        CHECK(!array[3] && !markers[0]);
        const uint64_t invalid[][3]={{0,4,1},{UINT64_C(0x80000006),4,2},
            {UINT64_C(0x80000006),5,1},{UINT64_C(0x80000006),65537,1}};
        for(unsigned i=0;i<sizeof(invalid)/sizeof(invalid[0]);i++) {
            memcpy(array,invalid[i],24);
            CHECK(experiment_gc_store_scalar(&c,array,0,&value)==3);
            CHECK(!array[3] && !markers[0]);
        }
        array[0]=UINT64_C(0x80000006);array[1]=4;array[2]=1;
        CHECK(!experiment_gc_store_scalar(&c,array,0,&value));
        CHECK(array[3]==value && markers[0]==1);
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&child));
        child[0]=17;
        neoclr_probe_storage slot={&array,8,0};
        neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        pthread_t worker;struct foreign f={&c,array};
        CHECK(!pthread_create(&worker,NULL,foreign_store,&f));
        CHECK(!pthread_join(worker,NULL));
        CHECK(array[3]==value);
        for(unsigned i=0;i<sizeof(patterns)/sizeof(patterns[0]);i++) {
            unsigned index=i%4;
            CHECK(!experiment_gc_store_scalar(&c,array,index,&patterns[i]));
            CHECK(array[3+index]==patterns[i] && markers[index]==1);
        }
        value=(uintptr_t)child;
        CHECK(!experiment_gc_store_scalar(&c,array,0,&value));
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        uint64_t bytes;uint32_t kind;
        CHECK(neoclr_gc_allocation_info_v1(&c,child,&bytes,&kind)!=0);
        CHECK(!experiment_gc_store_scalar(&c,array,1,&value));
        void *stale=array;array=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(experiment_gc_store_scalar(&c,stale,0,&value)==3);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 100 scalar store epochs, bit patterns, bounds, ownership and nonretention");
}
