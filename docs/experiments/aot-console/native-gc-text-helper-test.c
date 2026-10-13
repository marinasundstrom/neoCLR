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
    /* UTF-8 slicing retains a heap source even when absent from caller roots.
     * Two copies contain multibyte scalars and an embedded NUL. */
    static const struct { uint64_t length; unsigned char bytes[9]; } unicode = {
        9, {'a',0xc3,0xa9,0xf0,0x9f,0x98,0x80,0,'z'}
    };
    literal = (const void *)&unicode;
    CHECK(!neoclr_string_concat_v1(literal,literal,&c.text,&left));
    CHECK(!neoclr_gc_allocate_v1(&c.text,408,NEOCLR_GC_BYTES,&dead));
    uint64_t outcome[2] = {99,99};
    neoclr_probe_storage live = {&dead,8,0};
    neoclr_probe_enter_v3(&frame,&c,0,&live,1,"{}",2);
    before=neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_string_slice_utf8_rooted_v1(left,2,99,&c.text,outcome));
    CHECK(outcome[0]==2 && outcome[1]==1);
    CHECK(!neoclr_string_slice_utf8_rooted_v1(left,2,0,&c.text,outcome));
    CHECK(outcome[0]==2 && outcome[1]==2);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    /* A genuinely live arena cannot fit the result: retry preserves output. */
    outcome[0]=99; outcome[1]=99;
    CHECK(neoclr_string_slice_utf8_rooted_v1(left,1,7,&c.text,outcome)==5);
    CHECK(outcome[0]==99 && outcome[1]==99 && left->length==18);
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    dead=NULL;
    CHECK(neoclr_string_slice_utf8_v1(left,1,7,&c.text,outcome)==5);
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    CHECK(!neoclr_string_slice_utf8_rooted_v1(left,1,7,&c.text,outcome));
    CHECK(neoclr_gc_statistics_v1().collections==before+2 && outcome[0]==4);
    result=(const void *)(uintptr_t)outcome[1];
    CHECK(result->length==7 && !memcmp(result->bytes,unicode.bytes+1,7));
    CHECK(left->length==18 && !memcmp(left->bytes,unicode.bytes,9));
    CHECK(neoclr_root_probe_head_v1()==&frame);
    neoclr_probe_leave_v1(&frame);
    uint64_t saved_outcome[2]; memcpy(saved_outcome,outcome,sizeof(outcome));
    CHECK(neoclr_string_slice_utf8_rooted_v1(left,0,0,&c.text,outcome)==3);
    CHECK(!memcmp(outcome,saved_outcome,sizeof(outcome)));
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    return 0;
}
