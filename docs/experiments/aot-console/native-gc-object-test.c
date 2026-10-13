#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
int main(void) {
    uint64_t heap[32]={0},other[32]={0};
    neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
    neoclr_aot_text_arena foreign={(unsigned char*)other,sizeof(other),0};
    void *out=(void*)(uintptr_t)123,*dead=NULL;
    CHECK(neoclr_allocate_object_rooted_v1(NULL,7,24,&out)==3);
    CHECK(neoclr_allocate_object_rooted_v1(&ctx.text,7,24,NULL)==3);
    CHECK(neoclr_allocate_object_rooted_v1(&ctx.text,7,24,&out)==3);
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,224,NEOCLR_GC_BYTES,&dead));
    neoclr_probe_storage slot={&dead,8,0};neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&ctx,0,&slot,1,"{}",2);
    CHECK(neoclr_allocate_object_rooted_v1(&foreign,7,24,&out)==3 && !foreign.used);
    uint64_t before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_allocate_object_v1(&ctx.text,7,24,&out)==5);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    CHECK(neoclr_allocate_object_rooted_v1(&ctx.text,7,24,&out)==5 && out==(void*)(uintptr_t)123);
    dead=NULL;
    CHECK(!neoclr_allocate_object_rooted_v1(&ctx.text,7,24,&out));
    CHECK(neoclr_gc_statistics_v1().collections==before+2);
    uint64_t *object=out;
    CHECK(object[0]==7 && !object[1] && !object[2]);
    dead=out; /* receiver published before any constructor allocation */
    void *discard=NULL;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,168,NEOCLR_GC_BYTES,&discard));
    CHECK(!neoclr_allocate_object_rooted_v1(&ctx.text,8,24,&out));
    CHECK(object[0]==7 && !object[1] && !object[2]);
    CHECK(((uint64_t*)out)[0]==8 && !((uint64_t*)out)[1] && !((uint64_t*)out)[2]);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
    puts("PASS rooted object initialization, receiver lifetime, exhaustion and reclamation");return 0;
}
