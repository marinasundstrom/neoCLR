#include "native-gc.h"
#include "memory-views.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"UTF-8 conversion line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
typedef int32_t (*convert_fn)(const void *,void *,void *);
typedef int32_t (*slice_fn)(const void *,int32_t,int32_t,void *,void *);
static void text_matches(uint64_t pointer,const char *expected,size_t length) {
    neoclr_aot_text *text=(void *)(uintptr_t)pointer;
    CHECK(text && text->length==length && !memcmp(text->bytes,expected,length));
}
int utf8_conversion_consumer(convert_fn encode,convert_fn decode,slice_fn slice) {
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        unsigned mode=round%10;
        const char *expected=(round&1 || mode==8)?"a\xc3\xa9z":"abcd";
        int32_t length=mode==7?0:4;
        neoclr_aot_text *text;void *array;uint64_t *owner;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&text));
        CHECK(!neoclr_gc_allocate_v1(&c.text,64,NEOCLR_GC_OBJECT,(void **)&owner));
        CHECK(!(mode==4?neoclr_reserve_bytes_v1(&c.text,length,&array):neoclr_allocate_bytes_v1(&c.text,length,&array)));
        text->length=length;memcpy(text->bytes,expected,length);
        memcpy((unsigned char *)array+16,expected,length);
        if(mode==3) ((unsigned char *)array)[16]=0xff;
        owner[0]=17;for(unsigned j=1;j<=6;j++) owner[j]=777+j;
        neoclr_probe_storage slots[]={{&text,8,0},{&array,8,0},{&owner,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,3};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,3,&map,sizeof(map));
        if(mode==1 || mode==2) {
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&node)) break;
                if(mode==2) { node[0]=17;node[1]=owner[7];owner[7]=(uintptr_t)node; }
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        int32_t encoded=encode(mode==6?NULL:text,owner+1,&c);
        int32_t decoded=decode(mode==6?NULL:mode==9?(void *)(uintptr_t)owner[1]:array,owner+2,&c);
        const void *input=mode==8?text:mode==6?NULL:array;
        int32_t offset=mode==8?(int32_t)(NEOCLR_VIEW_UTF8_OFFSET|1):mode==5?5:0;
        int32_t count=mode==6?0:mode==8?2:length;
        int32_t sliced=slice(input,offset,count,owner+4,&c);
        CHECK(encoded==(mode==2?5:mode==6?3:0));
        CHECK(decoded==(mode==2?5:mode==4 || mode==6?3:0));
        /* Preserve existing wrapper fault mapping for invalid windows. */
        CHECK(sliced==(mode==2?5:mode==4 || mode==5?3:0));
        CHECK(owner[6]==783 && neoclr_root_probe_head_v1()==&frame);
        if(encoded) CHECK(owner[1]==778);
        if(decoded) CHECK(owner[2]==779 && owner[3]==780);
        if(sliced) CHECK(owner[4]==781 && owner[5]==782);
        experiment_gc_store_counts(after);
        CHECK(after[1]+after[2]>=before[1]+before[2]+(unsigned)(!encoded+!decoded+!sliced));
        text=NULL;array=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        if(!encoded) {
            const uint64_t *result=(void *)(uintptr_t)owner[1];
            CHECK((uint32_t)result[0]==UINT32_C(0x80000001) && result[1]==(uint64_t)length);
            CHECK(!memcmp((const unsigned char *)result+16,expected,length));
        }
        if(!decoded) {
            CHECK((uint32_t)owner[2]==(mode==3?2:4));
            if(mode==3) CHECK(owner[3]==1);else text_matches(owner[3],expected,length);
        }
        if(!sliced) {
            CHECK((uint32_t)owner[4]==(mode==3?2:4));
            if(mode==3) CHECK(owner[5]==1);else text_matches(owner[5],mode==8?expected+1:expected,count);
        }
        owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 120 UTF-8 cycles: roundtrip bytes, malformed input, reserved reads, text windows, empty/null, bounds, pressure, exhaustion, aliases and cleanup");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_utf8_conversion_0(const void *,void *,void *);
extern int32_t live_utf8_conversion_1(const void *,void *,void *);
extern int32_t live_utf8_conversion_2(const void *,int32_t,int32_t,void *,void *);
int main(void) { return utf8_conversion_consumer(live_utf8_conversion_0,live_utf8_conversion_1,live_utf8_conversion_2); }
#endif
