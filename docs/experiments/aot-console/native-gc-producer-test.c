#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while(0)
#ifndef _WIN32
#include <unistd.h>
#include <fcntl.h>
static int reads, closes;
static ssize_t counted_read(int fd, void *bytes, size_t count) { ++reads; return read(fd,bytes,count); }
static int counted_close(int fd) { ++closes; return close(fd); }
#define read counted_read
#define close counted_close
#include "file-input.c"
#undef read
#undef close
#endif
int main(void) {
    const unsigned char content[]={'a',0xc3,0xa9,0,'z'};
    unsigned char *native=malloc(sizeof(content));CHECK(native);
    memcpy(native,content,sizeof(content));
    uint64_t heap[64]={0}, foreign_heap[64]={0};
    neoclr_aot_context ctx={.text={(unsigned char*)heap,sizeof(heap),0}};
    neoclr_aot_text_arena foreign={(unsigned char*)foreign_heap,sizeof(foreign_heap),0};
    void *dead=NULL;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,sizeof(heap)-32,NEOCLR_GC_BYTES,&dead));
    uint64_t result[2]={99,99};
    CHECK(neoclr_text_decode_utf8_host_bytes_rooted_v1(native,sizeof(content),&ctx.text,result)==3 && result[0]==99 && result[1]==99);
    neoclr_probe_storage slot={&dead,8,0};neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&ctx,0,&slot,1,"{}",2);
    CHECK(neoclr_text_decode_utf8_host_bytes_rooted_v1(native,sizeof(content),&foreign,result)==3 && !foreign.used);
    uint64_t before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_text_decode_utf8_bytes_v1(native,sizeof(content),&ctx.text,result)==5);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    CHECK(neoclr_text_decode_utf8_host_bytes_rooted_v1(native,sizeof(content),&ctx.text,result)==5 && result[0]==99 && result[1]==99);
    dead=NULL;
    CHECK(!neoclr_text_decode_utf8_host_bytes_rooted_v1(native,sizeof(content),&ctx.text,result));
    CHECK(neoclr_gc_statistics_v1().collections==before+2);
    const neoclr_aot_text *text=(const neoclr_aot_text*)(uintptr_t)result[1];
    CHECK(result[0]==4 && text->length==sizeof(content) && !memcmp(text->bytes,content,sizeof(content)));
    CHECK(!memcmp(native,content,sizeof(content)));
    native[0]=0xff;before=neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_text_decode_utf8_host_bytes_rooted_v1(native,sizeof(content),&ctx.text,result) && result[0]==2);
    CHECK(neoclr_gc_statistics_v1().collections==before);
    free(native);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
#ifndef _WIN32
    char name[]="/tmp/neoclr-rooted-input-XXXXXX";
    int fd=mkstemp(name);CHECK(fd>=0);
    CHECK(write(fd,content,sizeof(content))==(ssize_t)sizeof(content));CHECK(!close(fd));
    void *storage=NULL;
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,8+strlen(name),NEOCLR_GC_TEXT,&storage));
    neoclr_aot_text *path=storage;path->length=strlen(name);memcpy(path->bytes,name,strlen(name));
    CHECK(!neoclr_gc_allocate_v1(&ctx.text,ctx.text.capacity-ctx.text.used-32,NEOCLR_GC_BYTES,&dead));
    neoclr_probe_storage roots[]={{&path,8,0},{&dead,8,0}};
    neoclr_probe_enter_v3(&frame,&ctx,0,roots,2,"{}",2);
    result[0]=result[1]=99;reads=closes=0;before=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_file_read_utf8_v1(path,100,&ctx.text,result)==5);
    CHECK(reads==2 && closes==1 && neoclr_gc_statistics_v1().collections==before);
    reads=closes=0;
    CHECK(neoclr_file_read_utf8_rooted_v1(path,100,&ctx.text,result)==5);
    CHECK(reads==2 && closes==1 && result[0]==99 && result[1]==99);
    /* Even the managed path may die now: no reads of it follow file acquisition. */
    roots[0]=roots[1];dead=NULL;reads=closes=0;
    CHECK(!neoclr_file_read_utf8_rooted_v1(path,100,&ctx.text,result));
    CHECK(reads==2 && closes==1 && neoclr_gc_statistics_v1().collections==before+2);
    text=(const neoclr_aot_text*)(uintptr_t)result[1];
    CHECK(result[0]==4 && text->length==sizeof(content) && !memcmp(text->bytes,content,sizeof(content)));
    neoclr_probe_leave_v1(&frame);
    CHECK(!unlink(name));
    CHECK(!neoclr_gc_collect_v1(&ctx,NULL) && !ctx.text.used);
#endif
    CHECK(!neoclr_gc_entry_check_v1(&ctx));
    puts("PASS stable native decode and supported file producer lifetime");
    return 0;
}
