#include "native-gc.h"
#include "memory-views.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"byte views line %d: %s\n",__LINE__,#x); abort(); } } while (0)
#ifdef LIVE
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_shadow_gate(int);
extern void experiment_gc_live_wait(void);
extern void experiment_gc_live_counts(uint64_t *);
#endif
static void cleaned(void *state) { ++*(unsigned *)state; }
int main(void) {
    for (unsigned round=0;round<200;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        void *array,*decoy,*forged;
        int reserved=round&1;
        CHECK(!(reserved?neoclr_reserve_bytes_v1(&c.text,32,&array):neoclr_allocate_bytes_v1(&c.text,32,&array)));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,&decoy));
        *(uint64_t *)decoy=17;
        CHECK(!neoclr_gc_allocate_v1(&c.text,48,NEOCLR_GC_OBJECT,&forged));
        ((uint64_t *)forged)[0]=UINT64_C(0x80000001);
        ((uint64_t *)forged)[1]=32;
        unsigned reclaimed=0;neoclr_gc_resource resource={0};
        CHECK(!neoclr_gc_resource_attach_v1(&c,decoy,&resource,&reclaimed,cleaned));
        neoclr_byte_view view,empty,source,destination;
        CHECK(!neoclr_byte_view_create_all_v1(&c,array,NEOCLR_VIEW_SLICE,&view));
        CHECK(!neoclr_byte_view_slice_v1(&c,&view,32,0,&empty));
        uint8_t byte=99;
        if(reserved) CHECK(neoclr_byte_view_get_v1(&c,&view,0,&byte)==3 && byte==99);
        /* A pointer-shaped byte sequence is data, not an ownership edge. */
        uintptr_t pointer=(uintptr_t)decoy;
        for(unsigned i=0;i<32;i++) CHECK(!neoclr_byte_view_set_v1(&c,&view,i,(uint8_t)i));
        for(unsigned i=0;i<sizeof(pointer);i++) CHECK(!neoclr_byte_view_set_v1(&c,&view,i,((uint8_t *)&pointer)[i]));
        neoclr_byte_view unchanged=view;
        CHECK(neoclr_byte_view_create_all_v1(&c,forged,NEOCLR_VIEW_SLICE,&unchanged)==3);
        CHECK(!memcmp(&unchanged,&view,sizeof(view)));
        neoclr_byte_view invalid={forged,0,32,NEOCLR_VIEW_SLICE};
        CHECK(neoclr_byte_view_set_v1(&c,&invalid,0,55)==3);
        CHECK(((unsigned char *)forged)[16]==0);
        invalid.owner=(unsigned char *)array+8;
        CHECK(neoclr_byte_view_set_v1(&c,&invalid,0,55)==3);
        uint64_t allocations=neoclr_gc_statistics_v1().allocations;
        /* Empty windows still retain their owner. Only owner lanes are roots. */
        neoclr_probe_storage slot={&empty.owner,sizeof(empty.owner),0};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,1};
        neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,&slot,1,&map,sizeof(map));
#ifdef LIVE
        uint64_t before[5],after[5];experiment_gc_live_counts(before);
        experiment_gc_shadow_gate(1);
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
#else
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
#endif
        CHECK(!neoclr_byte_view_slice_v1(&c,&view,16,8,&source));
        CHECK(!neoclr_byte_view_slice_v1(&c,&view,18,8,&destination));
        CHECK(!neoclr_byte_view_copy_v1(&c,&source,&destination));
        for(int i=0;i<8;i++) { CHECK(!neoclr_byte_view_get_v1(&c,&destination,i,&byte));CHECK(byte==16+i); }
#ifdef LIVE
        experiment_gc_shadow_gate(0);
#endif
        /* Writes can overlap marker execution; headers and owner never change. */
        for(unsigned i=0;i<1000;i++) CHECK(!neoclr_byte_view_set_v1(&c,&view,31,(uint8_t)i));
#ifdef LIVE
        experiment_gc_live_wait();
#endif
        CHECK(!neoclr_byte_view_set_v1(&c,&view,30,77));
        neoclr_byte_view readonly=view;readonly.kind=NEOCLR_VIEW_READONLY_SLICE;
        CHECK(neoclr_byte_view_set_v1(&c,&readonly,30,12)==3);
        CHECK(neoclr_byte_view_set_v1(&c,&view,32,12)==8);
#ifdef LIVE
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        experiment_gc_live_counts(after);
        CHECK(after[1]==before[1]); /* No payload bytes read by the marker. */
#else
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
#endif
        CHECK(reclaimed==1);
        CHECK(neoclr_gc_statistics_v1().allocations==allocations);
        CHECK(!neoclr_byte_view_get_v1(&c,&view,30,&byte) && byte==77);
        CHECK(!neoclr_byte_view_get_v1(&c,&view,31,&byte) && byte==(uint8_t)999);
        empty.owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 200 byte-view epochs, 200000 writes, reserved/ordinary storage, alias copies, empty-owner retention and false-edge rejection");
}
