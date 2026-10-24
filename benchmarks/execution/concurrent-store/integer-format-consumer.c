#include "native-gc.h"
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"integer format line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_store_counts(uint64_t *);
typedef int32_t (*format32_fn)(int32_t,void *,void *);
typedef int32_t (*format64_fn)(uint64_t,void *,void *);
int integer_format_consumer(format32_fn narrow,format64_fn wide,format64_fn unsigned_wide) {
    const int32_t small[]={INT32_MIN,INT32_MAX,0,-1};
    const int64_t large[]={INT64_MIN,INT64_MAX,0,-1};
    const uint64_t positive[]={UINT64_MAX,0,INT64_MAX,1};
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *owner;
        CHECK(!neoclr_gc_allocate_v1(&c.text,40,NEOCLR_GC_OBJECT,(void **)&owner));
        owner[0]=17;owner[1]=777;owner[2]=888;owner[3]=999;
        neoclr_probe_storage slots[]={{&owner,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,1};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,1,&map,sizeof(map));
        unsigned mode=round%3;
        if(mode) {
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&node)) break;
                if(mode==2) { node[0]=17;node[1]=owner[4];owner[4]=(uintptr_t)node; }
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        uint64_t before[5],after[5];experiment_gc_store_counts(before);
        unsigned i=round%4;
        int32_t status=narrow(small[i],owner+1,&c);
        if(mode==2) {
            CHECK(status==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && owner[1]==777);
            CHECK(wide((uint64_t)large[i],owner+2,&c)==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && owner[2]==888);
            CHECK(unsigned_wide(positive[i],owner+3,&c)==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && owner[3]==999);
        } else {
            CHECK(!status);
            CHECK(!wide((uint64_t)large[i],owner+2,&c));
            CHECK(!unsigned_wide(positive[i],owner+3,&c));
            char expected[3][32];
            snprintf(expected[0],32,"%" PRId32,small[i]);
            snprintf(expected[1],32,"%" PRId64,large[i]);
            snprintf(expected[2],32,"%" PRIu64,positive[i]);
            CHECK(!neoclr_gc_collect_v1(&c,&frame));
            for(unsigned j=0;j<3;j++) {
                neoclr_aot_text *text=(void *)(uintptr_t)owner[j+1];
                uint64_t bytes;uint32_t kind;
                CHECK(!neoclr_gc_allocation_info_v1(&c,text,&bytes,&kind) && kind==NEOCLR_GC_TEXT);
                CHECK(text->length==strlen(expected[j]) && !memcmp(text->bytes,expected[j],text->length));
            }
            experiment_gc_store_counts(after);
            CHECK(after[1]+after[2]>=before[1]+before[2]+3);
        }
        CHECK(neoclr_root_probe_head_v1()==&frame);
        owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        neoclr_gc_leave_v1(&frame);CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    puts("PASS 120 integer formatting cycles: extrema, heap aliases, garbage pressure, exhaustion, retention and cleanup");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_integer_format_0(int32_t,void *,void *);
extern int32_t live_integer_format_1(uint64_t,void *,void *);
extern int32_t live_integer_format_2(uint64_t,void *,void *);
int main(void) { return integer_format_consumer(live_integer_format_0,live_integer_format_1,live_integer_format_2); }
#endif
