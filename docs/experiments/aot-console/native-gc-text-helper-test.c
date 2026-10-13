#include "native-gc.h"
#include <string.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while (0)
static const struct { uint64_t length; char bytes[4]; } word = {3,"abc"};
int main(void) {
    uint64_t heap[64] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    const neoclr_aot_text *literal = (const void *)&word, *left = NULL, *right = NULL, *result = NULL;
    CHECK(!neoclr_string_concat_v1(literal,literal,&c.text,&left));
    CHECK(!neoclr_string_concat_v1(literal,literal,&c.text,&right));
    void *dead = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text,344,NEOCLR_GC_BYTES,&dead));
    /* Deliberately absent from caller storage: the helper must protect inputs. */
    neoclr_probe_frame frame;
    neoclr_probe_storage empty = {0};
    neoclr_probe_enter_v3(&frame,&c,0,&empty,0,"{}",2);
    uint64_t before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_string_concat_v1(left,right,&c.text,&result)==5 && !result);
    CHECK(!neoclr_string_concat_rooted_v1(left,right,&c.text,&result));
    CHECK(neoclr_gc_statistics_v1().collections==before+1 && result->length==12);
    CHECK(!memcmp(result->bytes,"abcabcabcabc",12) && left->length==6 && right->length==6);
    CHECK(neoclr_root_probe_head_v1()==&frame);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    /* Joining protects the parts array, and tracing it protects its strings. */
    void *parts=NULL;
    CHECK(!neoclr_allocate_strings_v1(&c.text,2,0,&parts));
    CHECK(!neoclr_string_concat_v1(literal,literal,&c.text,&left));
    CHECK(!neoclr_string_concat_v1(literal,literal,&c.text,&right));
    ((const void **)parts)[2]=left; ((const void **)parts)[3]=right;
    CHECK(!neoclr_gc_allocate_v1(&c.text,288,NEOCLR_GC_BYTES,&dead));
    neoclr_probe_enter_v3(&frame,&c,0,&empty,0,"{}",2);
    before=neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_string_join_parts_rooted_v1(parts,2,literal,15,&c.text,&result));
    CHECK(neoclr_gc_statistics_v1().collections==before+1 && result->length==15);
    CHECK(!memcmp(result->bytes,"abcabcabcabcabc",15));
    const neoclr_aot_text *saved=result;
    CHECK(neoclr_string_join_parts_rooted_v1(parts,2,literal,14,&c.text,&result)==3 && result==saved);
    neoclr_probe_leave_v1(&frame);
    CHECK(neoclr_string_concat_rooted_v1(literal,literal,&c.text,&result)==3 && result==saved);
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    return 0;
}
