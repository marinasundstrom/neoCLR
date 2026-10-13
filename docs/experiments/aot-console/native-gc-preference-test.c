#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
extern int32_t neoclr_system_culture_name_v1(neoclr_aot_text_arena *, const neoclr_aot_text **);
extern int32_t neoclr_system_culture_name_rooted_v1(neoclr_aot_text_arena *, const neoclr_aot_text **);
extern int32_t neoclr_time_zone_version_v1(neoclr_aot_text_arena *, const neoclr_aot_text **);
extern int32_t neoclr_time_zone_version_rooted_v1(neoclr_aot_text_arena *, const neoclr_aot_text **);
int main(void) {
    for(int mode=0;mode<2;mode++) {
        int32_t (*raw)(neoclr_aot_text_arena *,const neoclr_aot_text **) = mode ? neoclr_time_zone_version_v1 : neoclr_system_culture_name_v1;
        int32_t (*rooted)(neoclr_aot_text_arena *,const neoclr_aot_text **) = mode ? neoclr_time_zone_version_rooted_v1 : neoclr_system_culture_name_rooted_v1;
        uint64_t heap[512]={0},other[512]={0};
        neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
        neoclr_aot_text_arena foreign={(unsigned char*)other,sizeof(other),0};
        const neoclr_aot_text *out=NULL;
        CHECK(!raw(&ctx.text,&out));
        size_t length=(size_t)out->length;unsigned char *baseline=malloc(length+1);CHECK(baseline);
        memcpy(baseline,out->bytes,length);
        CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
        void *dead=NULL;CHECK(!neoclr_gc_allocate_v1(&ctx.text,sizeof(heap)-32,NEOCLR_GC_BYTES,&dead));
        out=(const neoclr_aot_text*)(uintptr_t)123;
        CHECK(rooted(&ctx.text,&out)==3 && out==(const neoclr_aot_text*)(uintptr_t)123);
        neoclr_probe_frame frame;neoclr_probe_storage slot={&dead,8,0};
        neoclr_probe_enter_v3(&frame,&ctx,0,&slot,1,"{}",2);
        CHECK(rooted(&foreign,&out)==3 && !foreign.used);
        uint64_t before=neoclr_gc_statistics_v1().collections;
        CHECK(raw(&ctx.text,&out)==5 && neoclr_gc_statistics_v1().collections==before);
        CHECK(rooted(&ctx.text,&out)==5 && out==(const neoclr_aot_text*)(uintptr_t)123);
        dead=NULL;CHECK(!rooted(&ctx.text,&out));
        CHECK(neoclr_gc_statistics_v1().collections==before+2);
        CHECK(out->length==length && !memcmp(out->bytes,baseline,length));
        free(baseline);
        CHECK(neoclr_root_probe_head_v1()==&frame);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used && !neoclr_gc_entry_check_v1(&ctx));
    }
    puts("PASS rooted native preferences/version, exhaustion and cleanup");
    return 0;
}
