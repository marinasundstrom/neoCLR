#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static int text(neoclr_aot_text_arena *arena, const char *bytes, const neoclr_aot_text **out) {
    void *p = NULL;
    int result = neoclr_gc_allocate_v1(arena, 8 + strlen(bytes), NEOCLR_GC_TEXT, &p);
    if (result) return result;
    neoclr_aot_text *t = p;
    t->length = strlen(bytes);
    memcpy(t->bytes, bytes, (size_t)t->length);
    *out = t;
    return 0;
}
static int invoke(int mode, int rooted, const neoclr_aot_text *a,
    const neoclr_aot_text *b, const neoclr_aot_text *c,
    neoclr_aot_text_arena *arena, const neoclr_aot_text **out) {
    if (mode < 4) return rooted ? neoclr_path_combine_unix_rooted_v1(a,b,arena,out) : neoclr_path_combine_unix_v1(a,b,arena,out);
    if (mode == 4) return rooted ? neoclr_path_file_name_unix_rooted_v1(a,arena,out) : neoclr_path_file_name_unix_v1(a,arena,out);
    return rooted ? neoclr_string_replace_ordinal_rooted_v1(a,b,c,arena,out) : neoclr_string_replace_ordinal_v1(a,b,c,arena,out);
}
int main(void) {
    const char *left[] = {"abc", "", "abc", "abc", "/a/b", "ababa", "ababa"};
    const char *right[] = {"def", "def", "", "/def", "", "ba", "ba"};
    const char *replacement[] = {"", "", "", "", "", "XYZ", ""};
    const char *expected[] = {"abc/def", "def", "abc", "/def", "b", "aXYZXYZ", "a"};
    for (int mode = 0; mode < 7; mode++) {
        uint64_t heap[64] = {0}, other[64] = {0};
        neoclr_aot_context ctx = {.text = {(unsigned char *)heap,sizeof(heap),0}};
        neoclr_aot_text_arena foreign = {(unsigned char *)other,sizeof(other),0};
        const neoclr_aot_text *a=NULL,*b=NULL,*c=NULL,*out=(const neoclr_aot_text *)(uintptr_t)123;
        CHECK(!text(&ctx.text,left[mode],&a) && !text(&ctx.text,right[mode],&b) && !text(&ctx.text,replacement[mode],&c));
        void *dead=NULL;
        CHECK(!neoclr_gc_allocate_v1(&ctx.text,ctx.text.capacity-ctx.text.used-32,NEOCLR_GC_BYTES,&dead));
        CHECK(invoke(mode,1,a,b,c,&ctx.text,&out)==3);
        neoclr_probe_storage slot={&dead,8,0};
        neoclr_probe_frame frame;
        neoclr_probe_enter_v3(&frame,&ctx,0,&slot,1,"{}",2);
        CHECK(invoke(mode,1,a,b,c,&foreign,&out)==3 && !foreign.used);
        uint64_t before=neoclr_gc_statistics_v1().collections;
        CHECK(invoke(mode,0,a,b,c,&ctx.text,&out)==5);
        CHECK(neoclr_gc_statistics_v1().collections==before);
        /* Retain all inputs for the exhaustion control, including unused ones. */
        neoclr_probe_storage slots[]={{&dead,8,0},{&a,8,0},{&b,8,0},{&c,8,0}};
        neoclr_probe_leave_v1(&frame);
        neoclr_probe_enter_v3(&frame,&ctx,0,slots,4,"{}",2);
        CHECK(invoke(mode,1,a,b,c,&ctx.text,&out)==5 && out==(const neoclr_aot_text *)(uintptr_t)123);
        neoclr_probe_leave_v1(&frame);
        dead=NULL;
        neoclr_probe_enter_v3(&frame,&ctx,0,&slot,1,"{}",2);
        CHECK(!invoke(mode,1,a,b,c,&ctx.text,&out));
        CHECK(neoclr_gc_statistics_v1().collections==before+2);
        CHECK(out->length==strlen(expected[mode]) && !memcmp(out->bytes,expected[mode],(size_t)out->length));
        CHECK(neoclr_root_probe_head_v1()==&frame);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
        CHECK(!neoclr_gc_entry_check_v1(&ctx));
    }
    puts("PASS borrowed path/replacement inputs, pressure, exhaustion and reclamation");
    return 0;
}
