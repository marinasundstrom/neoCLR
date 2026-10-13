#include "native-gc.h"
#include "string-unicode.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
static const unsigned char content[]={'e',0xcc,0x81,0xf0,0x9f,0x99,0x82,0};
static int invoke(int mode,int rooted,const void *input,neoclr_aot_text_arena *a,const void **out) {
    const neoclr_aot_text *t=input;
    const neoclr_aot_text *result=NULL;
    int status;
    if(mode==4) return rooted ? neoclr_string_graphemes_rooted_v1(t,a,out) : neoclr_string_graphemes_v1(t,a,out);
    if(mode==5) return rooted ? neoclr_string_scalars_rooted_v1(t,a,out) : neoclr_string_scalars_v1(t,a,out);
    if(mode==0) status=rooted ? neoclr_string_upper_rooted_v1(t,a,&result) : neoclr_string_upper_v1(t,a,&result);
    else if(mode==1) status=rooted ? neoclr_string_lower_rooted_v1(t,a,&result) : neoclr_string_lower_v1(t,a,&result);
    else if(mode==2) status=rooted ? neoclr_string_grapheme_at_rooted_v1(t,1,a,&result) : neoclr_string_grapheme_at_v1(t,1,a,&result);
    else if(mode==3) status=rooted ? neoclr_string_slice_graphemes_rooted_v1(t,1,2,a,&result) : neoclr_string_slice_graphemes_v1(t,1,2,a,&result);
    else status=rooted ? neoclr_string_from_chars_rooted_v1(input,a,&result) : neoclr_string_from_chars_v1(input,a,&result);
    if(!status) *out=result;
    return status;
}
int main(void) {
    for(int mode=0;mode<7;mode++) {
        uint64_t heap[128]={0},other[128]={0};
        neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
        neoclr_aot_text_arena foreign={(unsigned char*)other,sizeof(other),0};
        void *storage=NULL;
        CHECK(!neoclr_gc_allocate_v1(&ctx.text,8+sizeof(content),NEOCLR_GC_TEXT,&storage));
        neoclr_aot_text *source=storage;
        source->length=sizeof(content);memcpy(source->bytes,content,sizeof(content));
        const void *input=source;
        if(mode==6) CHECK(!neoclr_string_graphemes_v1(source,&ctx.text,&input));
        void *dead=NULL;
        CHECK(!neoclr_gc_allocate_v1(&ctx.text,ctx.text.capacity-ctx.text.used-32,NEOCLR_GC_BYTES,&dead));
        const void *out=(const void*)(uintptr_t)123;
        CHECK(invoke(mode,1,input,&ctx.text,&out)==3);
        neoclr_probe_storage roots[]={{&input,8,0},{&dead,8,0},{&source,8,0}};
        neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame,&ctx,0,roots,3,"{}",2);
        CHECK(invoke(mode,1,input,&foreign,&out)==3 && !foreign.used);
        uint64_t before=neoclr_gc_statistics_v1().collections;
        CHECK(invoke(mode,0,input,&ctx.text,&out)==5);
        CHECK(neoclr_gc_statistics_v1().collections==before);
        CHECK(invoke(mode,1,input,&ctx.text,&out)==5 && out==(const void*)(uintptr_t)123);
        roots[0]=(neoclr_probe_storage){&dead,8,0};roots[2]=roots[0];dead=NULL;
        CHECK(!invoke(mode,1,input,&ctx.text,&out));
        CHECK(neoclr_gc_statistics_v1().collections==before+2);
        if(mode==4 || mode==5) {
            const uint64_t *header=out;
            CHECK(header[1]==(uint64_t)(mode==4 ? 3 : 4));
            if(mode==4) {
                const neoclr_aot_text *part;memcpy(&part,(const unsigned char*)out+16,8);
                CHECK(part->length==3 && !memcmp(part->bytes,content,3));
                memcpy(&part,(const unsigned char*)out+32,8);CHECK(part->length==1 && !part->bytes[0]);
            }
        } else {
            const neoclr_aot_text *t=out;
            size_t length=mode==2 ? 4 : mode==3 ? 5 : sizeof(content);
            CHECK(t->length==length);
            CHECK(!memcmp(t->bytes+(mode==0 ? 1 : 0),content+(mode==2 || mode==3 ? 3 : mode==0 ? 1 : 0),length-(mode==0 ? 1 : 0)));
            if(mode==0) CHECK(t->bytes[0]=='E');
        }
        roots[0]=(neoclr_probe_storage){&out,8,0};
        CHECK(!neoclr_gc_collect_v1(&ctx,&frame));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
        CHECK(!neoclr_gc_entry_check_v1(&ctx));
    }
    /* Exhaustion after publishing some elements must release the temporary frame. */
    uint64_t heap[28]={0};
    neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
    void *storage=NULL;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,8+sizeof(content),NEOCLR_GC_TEXT,&storage));
    neoclr_aot_text *source=storage;source->length=sizeof(content);memcpy(source->bytes,content,sizeof(content));
    neoclr_probe_storage slot={&source,8,0};neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&ctx,0,&slot,0,"{}",2);
    const void *out=(const void*)(uintptr_t)123;
    CHECK(neoclr_string_graphemes_rooted_v1(source,&ctx.text,&out)==5 && out==(const void*)(uintptr_t)123);
    CHECK(neoclr_root_probe_head_v1()==&frame);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    puts("PASS rooted Unicode inputs, partial vectors, faults and reclamation");
    return 0;
}
