/* Fixed collector owner serves the real allocator entry used by native callers.
 * Dispatch itself never collects, even on pressure or malformed requests. */
#include "worker-api.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do {if(!(x)){fprintf(stderr,"line %d\n",__LINE__);abort();}} while(0)
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    int start, pending, done;
    neoclr_aot_context context;
    gate_writer writer;
    live_worker_target target;
    neoclr_aot_text_arena *arena;
    uint64_t bytes, used;
    uint32_t kind;
    int32_t status;
    void *result;
    uint64_t *objects[2];
} request_slot;
static _Thread_local request_slot *current;
static int32_t request_allocation(neoclr_aot_text_arena *arena,uint64_t bytes,uint32_t kind,void **out,uint64_t *used) {
    request_slot *s=current;
    CHECK(s && arena==&s->context.text);
    if(!out)return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    CHECK(!pthread_mutex_lock(&s->mutex));
    CHECK(!s->pending && !s->done);
    s->arena=arena;s->bytes=bytes;s->kind=kind;s->pending=1;
    CHECK(!pthread_cond_signal(&s->condition));
    while(s->pending)CHECK(!pthread_cond_wait(&s->condition,&s->mutex));
    int32_t status=s->status;
    if(!status){*out=s->result;*used=s->used;}
    CHECK(!pthread_mutex_unlock(&s->mutex));return status;
}
static void *allocate_and_publish(void *opaque) {
    request_slot *s=opaque;
    CHECK(!pthread_mutex_lock(&s->mutex));
    while(!s->start)CHECK(!pthread_cond_wait(&s->condition,&s->mutex));
    CHECK(!pthread_mutex_unlock(&s->mutex));
    current=s;neoclr_gc_allocation_dispatch_hook=request_allocation;
    void *unchanged=(void *)(uintptr_t)17;
    CHECK(neoclr_gc_allocate_v1(&s->context.text,UINT64_MAX,NEOCLR_GC_OBJECT,&unchanged)==NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
    CHECK(unchanged==(void *)(uintptr_t)17);
    CHECK(neoclr_gc_allocate_v1(&s->context.text,24,0,&unchanged)==NEOCLR_AOT_FAULT_RUNTIME_ERROR);
    CHECK(unchanged==(void *)(uintptr_t)17);
    for(unsigned i=0;i<2;i++) {
        CHECK(!neoclr_gc_allocate_v1(&s->context.text,24,NEOCLR_GC_OBJECT,(void **)&s->objects[i]));
        CHECK(!s->objects[i][0] && !s->objects[i][1] && !s->objects[i][2]);
        s->objects[i][0]=17;
    }
    s->objects[0][1]=(uintptr_t)s->objects[1];
    uint64_t value=(uintptr_t)s->objects[0];mutation_patch patch={8,8,&value};
    CHECK(!experiment_gc_worker_write(&s->writer,&s->target,&patch,1));
    CHECK(neoclr_gc_statistics_v1().allocations==2);
#ifdef NEOCLR_GC_PROFILE
    CHECK(neoclr_gc_profile_v1().allocated_bytes==48 && neoclr_gc_profile_v1().peak_arena_used>0);
#endif
    neoclr_gc_allocation_dispatch_hook=NULL;current=NULL;
    CHECK(!pthread_mutex_lock(&s->mutex));s->done=1;
    CHECK(!pthread_cond_signal(&s->condition));CHECK(!pthread_mutex_unlock(&s->mutex));
    return NULL;
}
int main(void) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[1024]={0},roots[4];
        neoclr_aot_context owner={.text={(void *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&owner.text));
        request_slot slots[4]={0};pthread_t threads[4];
        for(unsigned i=0;i<4;i++) {
            uint64_t *p=NULL;CHECK(!neoclr_gc_allocate_v1(&owner.text,24,NEOCLR_GC_OBJECT,(void **)&p));
            p[0]=17;roots[i]=(uintptr_t)p;
        }
        uint64_t owner_allocations=neoclr_gc_statistics_v1().allocations;
#ifdef NEOCLR_GC_PROFILE
        uint64_t owner_bytes=neoclr_gc_profile_v1().allocated_bytes;
#endif
        neoclr_probe_frame frame={.context=&owner,.lanes=roots,.lane_count=4};
        CHECK(!experiment_gc_begin(&owner,&frame,NULL,0));
        for(unsigned i=0;i<4;i++) {
            request_slot *s=&slots[i];
            /* Empty worker-local arena is a request identity, never allocated
             * locally. The owner service selects its own authoritative arena. */
            CHECK(!pthread_mutex_init(&s->mutex,NULL));CHECK(!pthread_cond_init(&s->condition,NULL));
            CHECK(!pthread_create(&threads[i],NULL,allocate_and_publish,s));
            CHECK(!experiment_gc_worker_register(&owner,threads[i],&s->writer));
            CHECK(!experiment_gc_worker_target_create(&owner,(void *)(uintptr_t)roots[i],&s->target));
            CHECK(!pthread_mutex_lock(&s->mutex));s->start=1;
            CHECK(!pthread_cond_signal(&s->condition));CHECK(!pthread_mutex_unlock(&s->mutex));
        }
        /* One bounded slot per worker; no heap allocation for request messages.
         * Servicing one thread at a time is correctness scaffolding, not the
         * eventual fair scheduler or a performance claim. */
        for(unsigned i=0;i<4;i++) {
            request_slot *s=&slots[i];CHECK(!pthread_mutex_lock(&s->mutex));
            while(!s->done) {
                while(!s->pending && !s->done)CHECK(!pthread_cond_wait(&s->condition,&s->mutex));
                if(s->done)break;
                CHECK(s->arena==&s->context.text);
                CHECK(!neoclr_gc_allocation_dispatch_hook);
                s->status=neoclr_gc_allocate_dispatch_storage_v1(&owner.text,s->bytes,s->kind,&s->result);
                s->used=owner.text.used;
                s->pending=0;CHECK(!pthread_cond_signal(&s->condition));
            }
            CHECK(!pthread_mutex_unlock(&s->mutex));
            CHECK(!pthread_join(threads[i],NULL));
            CHECK(!experiment_gc_worker_revoke(&owner,&s->writer));
        }
        CHECK(neoclr_gc_statistics_v1().allocations==owner_allocations);
#ifdef NEOCLR_GC_PROFILE
        CHECK(neoclr_gc_profile_v1().allocated_bytes==owner_bytes);
#endif
        frame.lane_count=0;CHECK(!experiment_gc_finish(&owner,&frame,NULL,0));
        for(unsigned i=0;i<4;i++) {
            uint64_t bytes;uint32_t kind;
            CHECK(!neoclr_gc_allocation_info_v1(&owner,(void *)(uintptr_t)roots[i],&bytes,&kind));
            for(unsigned j=0;j<2;j++)CHECK(!neoclr_gc_allocation_info_v1(&owner,slots[i].objects[j],&bytes,&kind));
            CHECK(!pthread_cond_destroy(&slots[i].condition));CHECK(!pthread_mutex_destroy(&slots[i].mutex));
        }
        CHECK(!neoclr_gc_collect_v1(&owner,&frame) && !owner.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&owner.text));
    }
    puts("PASS 100 epochs, 1600 owner-dispatched requests, 800 successful allocations, pressure/invalid failures preserve output, 1200 live objects, requester-only accounting and complete reclamation");
}
