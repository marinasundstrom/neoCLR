#include "worker-api.h"
#include "participant-roots.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do {if(!(x)){fprintf(stderr,"line %d\n",__LINE__);abort();}} while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_shadow_gate(int);
extern void experiment_gc_live_wait(void);
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    unsigned ready,phase;
    neoclr_aot_context *owner;
    neoclr_gc_participant participant;
    gate_writer writer;
    live_worker_target target;
    uint64_t *parent,*fresh,*white,*root_only;
} worker;
static void *mutate(void *opaque) {
    worker *w=opaque;
    neoclr_aot_context context={.text=w->owner->text};
    uint64_t root=(uintptr_t)w->parent;
    neoclr_probe_storage storage={&root,8,0};
    neoclr_probe_frame frame={.context=&context,.storage=&storage,.storage_count=1};
    CHECK(!pthread_mutex_lock(&w->mutex));
    w->participant=(neoclr_gc_participant){&context,&frame};w->ready=1;
    CHECK(!pthread_cond_signal(&w->condition));
    while(w->phase<1) CHECK(!pthread_cond_wait(&w->condition,&w->mutex));
    CHECK(!pthread_mutex_unlock(&w->mutex));
    live_worker_target rejected;
    CHECK(experiment_gc_worker_target_create(w->owner,w->parent,&rejected)==GATE_INVALID);
    uint64_t zero=0,fresh=(uintptr_t)w->fresh,white=(uintptr_t)w->white;
    mutation_patch invalid={0,8,&zero};
    CHECK(experiment_gc_worker_write(&w->writer,&w->target,&invalid,1)==GATE_INVALID);
    mutation_patch changes[]={{8,8,&zero},{16,8,&fresh}};
    CHECK(!experiment_gc_worker_write(&w->writer,&w->target,changes,2));
    CHECK(!pthread_mutex_lock(&w->mutex));w->ready=2;CHECK(!pthread_cond_signal(&w->condition));
    while(w->phase<2) CHECK(!pthread_cond_wait(&w->condition,&w->mutex));
    CHECK(!pthread_mutex_unlock(&w->mutex));
    mutation_patch insert={8,8,&white};
    CHECK(!experiment_gc_worker_write(&w->writer,&w->target,&insert,1));
    root=(uintptr_t)w->root_only; /* New current root, absent from initial capture. */
    CHECK(!pthread_mutex_lock(&w->mutex));w->ready=3;CHECK(!pthread_cond_signal(&w->condition));
    while(w->phase<3) CHECK(!pthread_cond_wait(&w->condition,&w->mutex));
    CHECK(!pthread_mutex_unlock(&w->mutex));
    volatile unsigned char *bytes=(void *)&frame;
    for(size_t i=0;i<sizeof(frame);i++)bytes[i]=0xa5;
    return NULL;
}
#ifdef NEOCLR_PERSISTENT_MUTATORS
/* An acknowledged completed invocation has returned from mutate(), including
 * its source root frames, and will not access the epoch again. The owner keeps
 * the dispatcher alive across collections; its lock publishes that quiescence. */
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    pthread_t thread;
    worker *job;
    int stop, complete;
} dispatcher;
static void *dispatch_loop(void *opaque) {
    dispatcher *d=opaque;
    CHECK(!pthread_mutex_lock(&d->mutex));
    for (;;) {
        while(!d->job && !d->stop) CHECK(!pthread_cond_wait(&d->condition,&d->mutex));
        if(d->stop) break;
        worker *job=d->job;
        CHECK(!pthread_mutex_unlock(&d->mutex));
        mutate(job);
        job=NULL;
        CHECK(!pthread_mutex_lock(&d->mutex));
        d->job=NULL;d->complete=1;
        CHECK(!pthread_cond_signal(&d->condition));
    }
    CHECK(!pthread_mutex_unlock(&d->mutex));
    return NULL;
}
static void dispatch_start(dispatcher *d) {
    CHECK(!pthread_mutex_init(&d->mutex,NULL));
    CHECK(!pthread_cond_init(&d->condition,NULL));
    CHECK(!pthread_create(&d->thread,NULL,dispatch_loop,d));
}
static void dispatch_submit(dispatcher *d,worker *w) {
    CHECK(!pthread_mutex_lock(&d->mutex));
    CHECK(!d->job && !d->stop);
    d->complete=0;d->job=w;
    CHECK(!pthread_cond_signal(&d->condition));
    CHECK(!pthread_mutex_unlock(&d->mutex));
}
static void dispatch_wait(dispatcher *d) {
    CHECK(!pthread_mutex_lock(&d->mutex));
    while(!d->complete) CHECK(!pthread_cond_wait(&d->condition,&d->mutex));
    CHECK(!d->job);
    CHECK(!pthread_mutex_unlock(&d->mutex));
}
static void dispatch_stop(dispatcher *d) {
    CHECK(!pthread_mutex_lock(&d->mutex));
    CHECK(d->complete && !d->job);d->stop=1;
    CHECK(!pthread_cond_signal(&d->condition));
    CHECK(!pthread_mutex_unlock(&d->mutex));
    CHECK(!pthread_join(d->thread,NULL));
    CHECK(!pthread_cond_destroy(&d->condition));
    CHECK(!pthread_mutex_destroy(&d->mutex));
}
#endif
static uint64_t *object(neoclr_aot_context *c) {
    uint64_t *p=NULL;CHECK(!neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,(void **)&p));p[0]=17;return p;
}
static void lock_ready(worker *w,unsigned ready) {
    CHECK(!pthread_mutex_lock(&w->mutex));
    while(w->ready<ready)CHECK(!pthread_cond_wait(&w->condition,&w->mutex));
}
static void release(worker *w,unsigned phase) {
    w->phase=phase;CHECK(!pthread_cond_signal(&w->condition));CHECK(!pthread_mutex_unlock(&w->mutex));
}
static void *overflow_mutate(void *opaque) {
    worker *w=opaque;
    CHECK(!pthread_mutex_lock(&w->mutex));w->ready=1;CHECK(!pthread_cond_signal(&w->condition));
    while(w->phase<1)CHECK(!pthread_cond_wait(&w->condition,&w->mutex));
    CHECK(!pthread_mutex_unlock(&w->mutex));
    uint64_t last=w->parent[1];unsigned successful=0;
    for(unsigned i=0;i<GATE_WORDS;i++) {
        uint64_t value=(uintptr_t)((i&1)?w->fresh:w->white);
        mutation_patch patch={8,8,&value};
        int result=experiment_gc_worker_write(&w->writer,&w->target,&patch,1);
        if(result==GATE_CANCELLED) {
            CHECK(w->parent[1]==last && successful==GATE_WORDS/2);
            CHECK(experiment_gc_worker_write(&w->writer,&w->target,&patch,1)==GATE_CANCELLED);
            return NULL;
        }
        CHECK(result==GATE_OK);successful++;last=value;
    }
    CHECK(0);return NULL;
}
static void overflow_recovery(void) {
    uint64_t heap[512]={0};neoclr_aot_context context={.text={(void *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
    uint64_t *parent=object(&context),*a=object(&context),*b=object(&context);parent[1]=(uintptr_t)a;
    neoclr_probe_storage slot={&parent,8,0};neoclr_probe_frame frame={.context=&context,.storage=&slot,.storage_count=1};
    worker w={.owner=&context,.parent=parent,.fresh=a,.white=b};
    CHECK(!pthread_mutex_init(&w.mutex,NULL));CHECK(!pthread_cond_init(&w.condition,NULL));
    pthread_t thread;CHECK(!pthread_create(&thread,NULL,overflow_mutate,&w));lock_ready(&w,1);
    experiment_gc_shadow_gate(1);CHECK(!experiment_gc_begin(&context,&frame,NULL,0));
    CHECK(!experiment_gc_worker_register(&context,thread,&w.writer));
    CHECK(!experiment_gc_worker_target_create(&context,parent,&w.target));
    release(&w,1);CHECK(!pthread_join(thread,NULL));
    experiment_gc_shadow_gate(0);CHECK(experiment_gc_finish(&context,&frame,NULL,0)==3);
    CHECK(!neoclr_gc_collect_v1(&context,&frame));
    uint64_t bytes;uint32_t kind;
    CHECK(!neoclr_gc_allocation_info_v1(&context,parent,&bytes,&kind));
    CHECK(!neoclr_gc_allocation_info_v1(&context,(void *)(uintptr_t)parent[1],&bytes,&kind));
    parent=NULL;CHECK(!neoclr_gc_collect_v1(&context,&frame) && !context.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&context.text));
    CHECK(!pthread_cond_destroy(&w.condition));CHECK(!pthread_mutex_destroy(&w.mutex));
}
int main(void) {
    for(unsigned i=0;i<10;i++)overflow_recovery();
#ifdef NEOCLR_PERSISTENT_MUTATORS
    dispatcher dispatchers[4]={0};
    for(unsigned i=0;i<4;i++)dispatch_start(&dispatchers[i]);
#endif
    for(unsigned cycle=0;cycle<100;cycle++) {
        uint64_t heap[4096]={0},initial[16],current[16];
        neoclr_aot_context context={.text={(void *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
        uint64_t *objects[4][7];worker workers[4]={0};pthread_t threads[4];
        for(unsigned i=0;i<4;i++) {
            for(unsigned j=0;j<5;j++)objects[i][j]=object(&context);
            objects[i][0][1]=(uintptr_t)objects[i][1];
            objects[i][2][1]=(uintptr_t)objects[i][3];
            workers[i].owner=&context;workers[i].parent=objects[i][0];
            workers[i].white=objects[i][2];workers[i].root_only=objects[i][4];
            CHECK(!pthread_mutex_init(&workers[i].mutex,NULL));CHECK(!pthread_cond_init(&workers[i].condition,NULL));
        }
        uint64_t *dead=object(&context);
        for(unsigned i=0;i<4;i++) {
#ifdef NEOCLR_PERSISTENT_MUTATORS
            threads[i]=dispatchers[i].thread;
            dispatch_submit(&dispatchers[i],&workers[i]);
#else
            CHECK(!pthread_create(&threads[i],NULL,mutate,&workers[i]));
#endif
        }
        neoclr_gc_participant parts[4];
        for(unsigned i=0;i<4;i++){lock_ready(&workers[i],1);parts[i]=workers[i].participant;}
        size_t initial_count,current_count;
        CHECK(!neoclr_gc_participant_root_capture_v1(&context,parts,4,initial,16,&initial_count));
        neoclr_probe_frame frame={.context=&context,.lanes=initial,.lane_count=(uint32_t)initial_count};
        experiment_gc_shadow_gate(cycle%2==0);
        CHECK(!experiment_gc_begin(&context,&frame,NULL,0));
        for(unsigned i=0;i<4;i++) {
            CHECK(!experiment_gc_worker_register(&context,threads[i],&workers[i].writer));
            CHECK(!experiment_gc_worker_target_create(&context,workers[i].parent,&workers[i].target));
            /* Allocation remains on the heap owner, after initial capture. */
            objects[i][5]=object(&context);objects[i][6]=object(&context);
            objects[i][5][1]=(uintptr_t)objects[i][6];workers[i].fresh=objects[i][5];
            release(&workers[i],1);
        }
        for(unsigned i=0;i<4;i++)lock_ready(&workers[i],2);
        experiment_gc_shadow_gate(0);experiment_gc_live_wait();
        /* Late writes occur after the marker's initial pass has gone idle. */
        for(unsigned i=0;i<4;i++)release(&workers[i],2);
        for(unsigned i=0;i<4;i++)lock_ready(&workers[i],3);
        CHECK(!neoclr_gc_participant_root_capture_v1(&context,parts,4,current,16,&current_count));
        for(unsigned i=0;i<4;i++) {
            CHECK(!experiment_gc_worker_revoke(&context,&workers[i].writer));
            release(&workers[i],3);
#ifdef NEOCLR_PERSISTENT_MUTATORS
            dispatch_wait(&dispatchers[i]);
#else
            CHECK(!pthread_join(threads[i],NULL));
#endif
        }
        frame.lanes=current;frame.lane_count=(uint32_t)current_count;
        CHECK(!experiment_gc_finish(&context,&frame,NULL,0));
        for(unsigned i=0;i<4;i++)for(unsigned j=0;j<7;j++) {
            uint64_t bytes;uint32_t kind;CHECK(!neoclr_gc_allocation_info_v1(&context,objects[i][j],&bytes,&kind));
        }
        uint64_t bytes;uint32_t kind;CHECK(neoclr_gc_allocation_info_v1(&context,dead,&bytes,&kind)==3);
        frame.lane_count=0;CHECK(!neoclr_gc_collect_v1(&context,&frame));CHECK(!context.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&context.text));
        for(unsigned i=0;i<4;i++){CHECK(!pthread_cond_destroy(&workers[i].condition));CHECK(!pthread_mutex_destroy(&workers[i].mutex));}
    }
#ifdef NEOCLR_PERSISTENT_MUTATORS
    for(unsigned i=0;i<4;i++)dispatch_stop(&dispatchers[i]);
    puts("PASS four persistent mutators acknowledged completion across 100 epochs before final shutdown");
#endif
    puts("PASS 100 live epochs, 400 mutators, 2900 liveness checks, current roots, deletion and late insertion, owner allocation and teardown; 10 overflow/recovery cycles");
}
