#define _POSIX_C_SOURCE 200809L
#include "file-input.h"
#include "file-output.h"
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
#ifdef FILE_LIVE
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern void experiment_gc_candidate_counts(uint64_t *);
#endif
static neoclr_aot_text *text(neoclr_aot_context *c, const void *bytes, size_t length) {
    neoclr_aot_text *value=NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text, 8+length, NEOCLR_GC_TEXT, (void **)&value));
    value->length=length; memcpy(value->bytes, bytes, length); return value;
}
int main(void) {
    char name[]="/tmp/neoclr-file-live-XXXXXX";
    int fd=mkstemp(name); CHECK(fd>=0 && !close(fd));
    void *heap=calloc(1,131072); CHECK(heap);
    neoclr_aot_context c={.text={heap,131072,0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    neoclr_aot_text *path=text(&c,name,strlen(name));
    const unsigned char bytes[]={'a',0xc3,0xa9,0,'z'};
    neoclr_aot_text *value=text(&c,bytes,sizeof(bytes)), *read=NULL;
    neoclr_probe_storage slots[]={{&path,8,0},{&value,8,0},{&read,8,0}};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&c,0,slots,3,"{}",2);
    for (unsigned i=0; i<32; i++) {
#ifdef FILE_LIVE
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        uint64_t epoch[6]; experiment_gc_candidate_counts(epoch); CHECK(epoch[5]);
#endif
        int32_t status=-1;
        CHECK(!neoclr_string_contains_ordinal_v1(value,value,&status) && status==1);
        CHECK(!neoclr_string_contains_ordinal_v1(path,value,&status) && status==0);
        status=77;
        CHECK(neoclr_string_contains_ordinal_v1(NULL,value,&status)==3 && status==77);
        CHECK(!neoclr_file_write_utf8_v1(path,value,5,&status) && status==0);
        CHECK(!neoclr_file_write_utf8_v1(path,value,4,&status) && status==7);
        uint64_t result[2]={99,99};
        CHECK(!neoclr_file_read_utf8_rooted_v1(path,4,&c.text,result) && result[0]==2 && result[1]==7);
        CHECK(!neoclr_file_read_utf8_rooted_v1(path,5,&c.text,result) && result[0]==4);
        read=(neoclr_aot_text *)(uintptr_t)result[1];
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        CHECK(read->length==5 && !memcmp(read->bytes,bytes,5));
        CHECK(value->length==5 && !memcmp(value->bytes,bytes,5));
    }
    CHECK(!unlink(name));
    uint64_t result[2]={99,99};
    CHECK(!neoclr_file_read_utf8_rooted_v1(path,5,&c.text,result) && result[0]==2 && result[1]==3);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
    free(heap);
    puts("PASS rooted file read/write, active epoch, UTF-8/NUL, limits and cleanup");
    return 0;
}
