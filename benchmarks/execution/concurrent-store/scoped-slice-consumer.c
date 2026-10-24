#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"scoped slice line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
extern void experiment_gc_native_scope_counts(uint64_t *);
typedef int32_t (*slice_fn)(const void *,int32_t,int32_t,void *,void *);
int scoped_slice_consumer(slice_fn slice) {
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        neoclr_aot_text *text;uint64_t *owner;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&text));
        CHECK(!neoclr_gc_allocate_v1(&c.text,40,NEOCLR_GC_OBJECT,(void **)&owner));
        text->length=4;memcpy(text->bytes,"a\xc3\xa9z",4);owner[0]=17;
        neoclr_probe_storage slots[]={{&text,8,0},{&owner,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,2};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,2,&map,sizeof(map));
        unsigned mode=round%6;
        if(mode==5) {
            /* Keep every filler live, forcing the rooted allocator to report exhaustion. */
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,24,NEOCLR_GC_OBJECT,(void **)&node)) break;
                node[0]=17;node[1]=owner[4];owner[4]=(uintptr_t)node;
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        uint64_t before[5],after[5];experiment_gc_native_scope_counts(before);
        owner[1]=111;owner[2]=222;owner[3]=333;
        int32_t start=mode==1?-1:mode==2?2:mode==3?4:1;
        int32_t length=mode==3?0:2;
        int32_t status=slice(mode==4?NULL:text,start,length,owner+1,&c);
        experiment_gc_native_scope_counts(after);
        CHECK(after[0]==before[0]+1 && after[1]==before[1]+1 && after[4]==0);
        CHECK(!experiment_gc_live_active(&c));
        CHECK(neoclr_root_probe_head_v1()==&frame && owner[3]==333);
        if(mode==4 || mode==5) {
            CHECK(status==(mode==4?3:5) && owner[1]==111 && owner[2]==222);
        } else {
            CHECK(!status);
            if(mode==1 || mode==2) {
                CHECK((uint32_t)owner[1]==2 && owner[2]==(mode==1?1:2));
            } else {
                CHECK((uint32_t)owner[1]==4);
                neoclr_aot_text *result=(void *)(uintptr_t)owner[2];
                CHECK(result->length==(mode==3?0:2));
                if(mode==0) CHECK(!memcmp(result->bytes,"\xc3\xa9",2));
                text=NULL;
                CHECK(!neoclr_gc_collect_v1(&c,&frame));
                uint64_t bytes;uint32_t kind;
                CHECK(!neoclr_gc_allocation_info_v1(&c,result,&bytes,&kind) && kind==NEOCLR_GC_TEXT);
            }
        }
        text=NULL;owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_idle_v1(&c,&frame));
#ifdef NEOCLR_GC_PACED
        CHECK(experiment_gc_live_active(&c));
#endif
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 120 scoped slice cycles: UTF-8 results, invalid range/boundary, empty, null, exhaustion, aliases, cleanup and resumption");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_scoped_slice(const void *,int32_t,int32_t,void *,void *);
int main(void) { return scoped_slice_consumer(live_scoped_slice); }
#endif
