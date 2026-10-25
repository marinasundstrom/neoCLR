/* Existing native array layouts, not a new managed record contract. */
#include "worker-api.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_shadow_gate(int);
extern void experiment_gc_live_wait(void);
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    unsigned ready, phase;
    gate_writer writer;
    live_worker_target target;
    uint64_t *old_child, *new_child;
    size_t header, lanes;
    int marked;
} array_worker;
static void *mutate_array(void *opaque) {
    array_worker *w=opaque;
    CHECK(!pthread_mutex_lock(&w->mutex));
    w->ready=1;CHECK(!pthread_cond_signal(&w->condition));
    while(!w->phase) CHECK(!pthread_cond_wait(&w->condition,&w->mutex));
    CHECK(!pthread_mutex_unlock(&w->mutex));
    uint64_t value=(uintptr_t)w->new_child,zero=0;
    unsigned char initialized=1;
    mutation_patch invalid_header={w->header-8,8,&zero};
    CHECK(experiment_gc_worker_write(&w->writer,&w->target,&invalid_header,1)==GATE_INVALID);
    /* A failing grouped transaction must not publish even its first valid patch. */
    mutation_patch invalid[]={{w->header,8,&value},{w->target.bytes,1,&initialized}};
    CHECK(experiment_gc_worker_write(&w->writer,&w->target,invalid,2)==GATE_INVALID);
    uint64_t before;
    memcpy(&before,(unsigned char *)w->target.payload+w->header,8);
    CHECK(before==(uintptr_t)w->old_child);
    mutation_patch publish[]={{w->header,8,&value},{w->header+w->lanes*8,1,&initialized}};
    CHECK(!experiment_gc_worker_write(&w->writer,&w->target,publish,w->marked?2:1));
    return NULL;
}
static uint64_t *object(neoclr_aot_context *context) {
    uint64_t *p=NULL;
    CHECK(!neoclr_gc_allocate_v1(&context->text,24,NEOCLR_GC_OBJECT,(void **)&p));p[0]=17;return p;
}
static void run_case(unsigned shape,unsigned delayed) {
    uint64_t heap[1024]={0};
    neoclr_aot_context context={.text={(void *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
    uint64_t *old_child=object(&context),*new_child=object(&context),*descendant=object(&context),*array=NULL;
    new_child[1]=(uintptr_t)descendant;
    /* Plain reference array, reserved reference array, or two-lane value array. */
    unsigned kind=shape==2?NEOCLR_GC_RECORDS:NEOCLR_GC_STRINGS;
    size_t header=shape==2?24:16,lanes=shape==2?2:1;
    size_t bytes=header+lanes*8+(shape!=0);
    CHECK(!neoclr_gc_allocate_v1(&context.text,bytes,kind,(void **)&array));
    array[0]=shape==2?UINT64_C(0x80000005):shape==1?UINT64_C(0x80000004):UINT64_C(0x80000003);
    array[1]=1;if(shape==2)array[2]=lanes;
    /* For delayed deletion, old value is initialized. For late publication it
     * is absent, so only the barrier can make the new graph reachable. */
    memcpy((unsigned char *)array+header,&old_child,8);
    if(shape) ((unsigned char *)array)[header+lanes*8]=delayed;
    uint64_t *root=array;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&context,.storage=&slot,.storage_count=1};
    array_worker w={.old_child=old_child,.new_child=new_child,.header=header,.lanes=lanes,.marked=shape!=0};
    CHECK(!pthread_mutex_init(&w.mutex,NULL));CHECK(!pthread_cond_init(&w.condition,NULL));
    pthread_t thread;CHECK(!pthread_create(&thread,NULL,mutate_array,&w));
    CHECK(!pthread_mutex_lock(&w.mutex));
    while(!w.ready)CHECK(!pthread_cond_wait(&w.condition,&w.mutex));
    experiment_gc_shadow_gate(delayed);
    CHECK(!experiment_gc_begin(&context,&frame,NULL,0));
    if(!delayed)experiment_gc_live_wait();
    CHECK(!experiment_gc_worker_register(&context,thread,&w.writer));
    CHECK(!experiment_gc_worker_target_create(&context,array,&w.target));
    w.phase=1;CHECK(!pthread_cond_signal(&w.condition));CHECK(!pthread_mutex_unlock(&w.mutex));
    CHECK(!pthread_join(thread,NULL));
    CHECK(!experiment_gc_worker_revoke(&context,&w.writer));
    /* Initial snapshot keeps the array alive. Avoid rescanning it through a
     * current root, which would mask a missing insertion barrier. */
    root=NULL;
    experiment_gc_shadow_gate(0);
    CHECK(!experiment_gc_finish(&context,&frame,NULL,0));
    uint64_t allocation_bytes;uint32_t allocation_kind;
    CHECK(!neoclr_gc_allocation_info_v1(&context,old_child,&allocation_bytes,&allocation_kind));
    CHECK(!neoclr_gc_allocation_info_v1(&context,new_child,&allocation_bytes,&allocation_kind));
    CHECK(!neoclr_gc_allocation_info_v1(&context,descendant,&allocation_bytes,&allocation_kind));
    CHECK(array[1]==1 && (!shape || ((unsigned char *)array)[header+lanes*8]==1));
    root=NULL;CHECK(!neoclr_gc_collect_v1(&context,&frame) && !context.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&context.text));
    CHECK(!pthread_cond_destroy(&w.condition));CHECK(!pthread_mutex_destroy(&w.mutex));
}
int main(int argc,char **argv) {
    CHECK(argc==1 || (argc==2 && strlen(argv[1])==1 && argv[1][0]>='0' && argv[1][0]<='2'));
    unsigned first=argc==2?(unsigned)(argv[1][0]-'0'):0,last=argc==2?first+1:3;
    for(unsigned round=0;round<50;round++)
        for(unsigned shape=first;shape<last;shape++)
            for(unsigned delayed=0;delayed<2;delayed++)run_case(shape,delayed);
    printf("PASS %u registered array epochs: layouts %u..%u, delayed deletion, late publication, grouped bounds rejection and reclamation\n",100*(last-first),first,last-1);
}
