#define _POSIX_C_SOURCE 200809L
#define _DARWIN_C_SOURCE
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"text finish line %d: %s\n",__LINE__,#x); abort(); } } while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int experiment_gc_live_active(neoclr_aot_context *);
typedef int32_t (*unary_fn)(const void *,void *,void *);
typedef int32_t (*compare_fn)(const void *,const void *,void *,void *);
typedef int32_t (*void_fn)(const void *,void *,void *);
static const struct { uint64_t length;char bytes[1048577]; } too_large={1048577,{0}};
static const struct { uint64_t length;char bytes[3]; } equal={2,"gc"},greater={3,"gcd"},less={1,"a"};
int text_finish_consumer(unary_fn intern,compare_fn compare,unary_fn line,void_fn no_result_line) {
    FILE *capture=tmpfile();CHECK(capture);int saved=dup(STDOUT_FILENO);CHECK(saved>=0);
    CHECK(!fflush(stdout) && dup2(fileno(capture),STDOUT_FILENO)>=0);
    for(unsigned round=0;round<120;round++) {
        uint64_t heap[512]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t handles[128];unsigned handle_count=0;
        unsigned mode=round%6;neoclr_aot_text *text,*duplicate;uint64_t *owner;
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&text));
        CHECK(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_TEXT,(void **)&duplicate));
        CHECK(!neoclr_gc_allocate_v1(&c.text,40,NEOCLR_GC_OBJECT,(void **)&owner));
        text->length=duplicate->length=2;memcpy(text->bytes,"gc",2);memcpy(duplicate->bytes,"gc",2);
        owner[0]=17;owner[1]=777;owner[2]=888;owner[3]=999;
        neoclr_probe_storage slots[]={{&text,8,0},{&duplicate,8,0},{&owner,8,0}};
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,3};neoclr_probe_frame frame;
        neoclr_gc_enter_map_v1(&frame,&c,0,slots,3,&map,sizeof(map));
        if(mode==2 || mode==3) {
            for(;;) {
                uint64_t *node;
                if(neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_OBJECT,(void **)&node)) break;
                /* Use a strong handle for each tiny filler to consume the final intern-sized block. */
                if(mode==3) { CHECK(handle_count<128);node[0]=17;CHECK(!neoclr_gc_host_root_create_v1(&c,node,&handles[handle_count++])); }
            }
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        int32_t status=intern(mode==4?NULL:mode==5?(const void *)&too_large:text,owner+1,&c);
        CHECK(status==(mode==3?5:mode==4?3:mode==5?10:0));
        if(status) CHECK(owner[1]==777);
        else {
            CHECK(owner[1]==(uintptr_t)text);
            uint64_t allocations=neoclr_gc_statistics_v1().allocations;
            CHECK(!intern(duplicate,owner+1,&c) && owner[1]==(uintptr_t)text);
            CHECK(neoclr_gc_statistics_v1().allocations==allocations);
        }
        CHECK(!compare(text,&equal,owner+3,&c) && *(int32_t *)(owner+3)==0);
        CHECK(!compare(text,&greater,owner+3,&c) && *(int32_t *)(owner+3)==-1);
        CHECK(!compare(text,&less,owner+3,&c) && *(int32_t *)(owner+3)==1);
        CHECK(compare(NULL,text,owner+3,&c)==3 && *(int32_t *)(owner+3)==1);
        CHECK(!line(text,owner+2,&c) && *(int32_t *)(owner+2)==0);
        CHECK(!no_result_line(text,NULL,&c));
        CHECK(line(NULL,owner+2,&c)==3 && *(int32_t *)(owner+2)==0);
        CHECK(no_result_line(NULL,NULL,&c)==3);
        if(round==119) {
            CHECK(!close(STDOUT_FILENO));
            CHECK(line(text,owner+2,&c)==3 && *(int32_t *)(owner+2)==0);
            CHECK(!fpurge(stdout)); /* Discard failed buffered output before restoring the capture. */
            CHECK(dup2(fileno(capture),STDOUT_FILENO)>=0);clearerr(stdout);
        }
        CHECK(neoclr_root_probe_head_v1()==&frame);
        CHECK(neoclr_gc_complete_invocation_v1(&c)==3);
        /* Release all strong filler handles before checking invocation pool lifetime. */
        for(unsigned i=0;i<handle_count;i++) CHECK(!neoclr_gc_host_root_release_v1(&c,handles[i]));
        neoclr_aot_text *expected=text;text=duplicate=NULL;owner=NULL;
        CHECK(!neoclr_gc_collect_v1(&c,&frame));
        if(!status) {
            uint64_t bytes;uint32_t kind;
            CHECK(!neoclr_gc_allocation_info_v1(&c,expected,&bytes,&kind) && kind==NEOCLR_GC_TEXT);
            CHECK(expected->length==2 && !memcmp(expected->bytes,"gc",2));
        }
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        neoclr_gc_leave_v1(&frame);
        CHECK(!neoclr_gc_complete_invocation_v1(&c));
        CHECK(!experiment_gc_live_active(&c));
        CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
    CHECK(!fflush(stdout));rewind(capture);
    for(unsigned i=0;i<240;i++) { char bytes[3];CHECK(fread(bytes,1,3,capture)==3 && !memcmp(bytes,"gc\n",3)); }
    CHECK(fgetc(capture)==EOF);
    CHECK(dup2(saved,STDOUT_FILENO)>=0 && !close(saved));fclose(capture);
    puts("PASS 120 intern/ordinal/console cycles: identity, retention, limits, pressure, aliases, exact output, unit/void, output errors and cleanup");
    return 0;
}
#ifdef MANAGED_MAIN
extern int32_t live_text_finish_0(const void *,void *,void *);
extern int32_t live_text_finish_1(const void *,const void *,void *,void *);
extern int32_t live_text_finish_2(const void *,void *,void *);
extern int32_t live_text_finish_3(const void *,void *,void *);
int main(void) { return text_finish_consumer(live_text_finish_0,live_text_finish_1,live_text_finish_2,live_text_finish_3); }
#endif
