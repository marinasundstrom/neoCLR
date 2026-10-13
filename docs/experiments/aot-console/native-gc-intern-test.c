#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
int main(void) {
    uint64_t heap[32]={0},other[32]={0};
    neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
    neoclr_aot_text_arena foreign={(unsigned char*)other,sizeof(other),0};
    void *storage=NULL,*dead=NULL;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,16,NEOCLR_GC_TEXT,&storage));
    neoclr_aot_text *input=storage;input->length=4;memcpy(input->bytes,"a\0bc",4);
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,ctx.text.capacity-ctx.text.used-32,NEOCLR_GC_BYTES,&dead));
    const neoclr_aot_text *out=(const neoclr_aot_text*)(uintptr_t)123;
    CHECK(neoclr_string_intern_rooted_v1(input,&ctx.text,&out)==3);
    neoclr_probe_storage slot={&dead,8,0};neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&ctx,0,&slot,1,"{}",2);
    CHECK(neoclr_string_intern_rooted_v1(input,&foreign,&out)==3 && !foreign.used);
    uint64_t before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_string_intern_v1(input,&ctx.text,&out)==5);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    CHECK(neoclr_string_intern_rooted_v1(input,&ctx.text,&out)==5 && out==(const neoclr_aot_text*)(uintptr_t)123);
    dead=NULL;CHECK(!neoclr_string_intern_rooted_v1(input,&ctx.text,&out) && out==input);
    CHECK(neoclr_gc_statistics_v1().collections==before+2);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL));
    CHECK(!memcmp(input->bytes,"a\0bc",4));
    static const struct { uint64_t length; unsigned char bytes[4]; } equal={4,{'a',0,'b','c'}};
    before=neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_string_intern_rooted_v1((const neoclr_aot_text*)&equal,&ctx.text,&out) && out==input);
    CHECK(neoclr_gc_statistics_v1().collections==before); /* lookup needs no allocation boundary */
    static const struct { uint64_t length; } too_large={1048577};
    CHECK(neoclr_string_intern_rooted_v1((const neoclr_aot_text*)&too_large,&ctx.text,&out)==10 && out==input);
    CHECK(!neoclr_gc_entry_check_v1(&ctx));ctx.text.used=0; /* admitted entry reset releases pool */
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    puts("PASS rooted intern ownership, identity, exhaustion, limits and reset");return 0;
}
