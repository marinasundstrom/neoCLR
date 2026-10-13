#include "entry-arguments.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
#ifdef _WIN32
#define ARG(x) L##x
#else
#define ARG(x) x
#endif
int main(void) {
    neoclr_process_char program[]=ARG("program"), empty[]=ARG(""), tail[]=ARG("tail");
#ifdef _WIN32
    neoclr_process_char unicode[]=L"\u00e9\U0001f642";
    neoclr_process_char bad[]={0xd800,0};
#else
    neoclr_process_char unicode[]="\xc3\xa9\xf0\x9f\x99\x82";
    neoclr_process_char bad[]={(char)0xff,0};
#endif
    neoclr_process_char *argv[]={program,unicode,empty,tail};
    uint64_t heap[128]={0};
    neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
    void *dead=NULL,*out=(void*)(uintptr_t)123;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,sizeof(heap)-32,NEOCLR_GC_BYTES,&dead));
    uint64_t before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_process_arguments_v1(4,argv,&ctx,&out)==5 && out==(void*)(uintptr_t)123);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    CHECK(!neoclr_process_arguments_rooted_v1(4,argv,&ctx,&out));
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    CHECK(!neoclr_root_probe_head_v1() && !neoclr_root_probe_depth_v1());
    CHECK(((uint64_t*)out)[1]==3);
    const neoclr_aot_text *text=(const neoclr_aot_text*)(uintptr_t)((uint64_t*)out)[2];
    CHECK(text->length==6 && !memcmp(text->bytes,"\xc3\xa9\xf0\x9f\x99\x82",6));
    text=(const neoclr_aot_text*)(uintptr_t)((uint64_t*)out)[3];CHECK(!text->length);
    neoclr_probe_storage root={&out,8,0};neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&ctx,0,&root,1,"{}",2);
    CHECK(!neoclr_gc_collect_v1(&ctx,&frame));
    text=(const neoclr_aot_text*)(uintptr_t)((uint64_t*)out)[4];CHECK(text->length==4 && !memcmp(text->bytes,"tail",4));
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    /* Reclaim only after the array and first string exist: earlier elements must survive. */
    ctx.text.capacity=256;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,80,NEOCLR_GC_BYTES,&dead));
    before=neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_process_arguments_rooted_v1(4,argv,&ctx,&out));
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    text=(const neoclr_aot_text*)(uintptr_t)((uint64_t*)out)[2];
    CHECK(text->length==6 && !memcmp(text->bytes,"\xc3\xa9\xf0\x9f\x99\x82",6));
    CHECK(!neoclr_root_probe_head_v1());
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    ctx.text.capacity=sizeof(heap);
    /* Native input failure after partial publication must unwind the frame. */
    argv[3]=bad;out=(void*)(uintptr_t)123;
    CHECK(neoclr_process_arguments_rooted_v1(4,argv,&ctx,&out)==3 && out==(void*)(uintptr_t)123);
    CHECK(!neoclr_root_probe_head_v1());
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    argv[3]=NULL;
    CHECK(neoclr_process_arguments_rooted_v1(4,argv,&ctx,&out)==3 && out==(void*)(uintptr_t)123);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    argv[3]=tail;ctx.text.capacity=192;
    CHECK(neoclr_process_arguments_rooted_v1(4,argv,&ctx,&out)==5 && out==(void*)(uintptr_t)123);
    CHECK(!neoclr_root_probe_head_v1());
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    CHECK(!neoclr_process_arguments_rooted_v1(1,argv,&ctx,&out) && ((uint64_t*)out)[1]==0);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&ctx));
    puts("PASS native argv ownership, Unicode, partial failure and final reclamation");
    return 0;
}
