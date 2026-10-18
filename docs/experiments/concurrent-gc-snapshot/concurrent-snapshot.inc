/* Experimental POSIX snapshot tracer. Included after synchronous tracing so both
 * paths use the same graph walker. Only the owner accesses the live heap. */
#include <pthread.h>
#include <stdlib.h>
#define SNAPSHOT_MAX_BYTES (8u * 1024u * 1024u)
#define SNAPSHOT_MAX_ROOTS 65536u
typedef struct {
    pthread_t thread;
    pthread_mutex_t lock;
    pthread_cond_t wake;
    int stop;
    _Atomic int phase; /* 0 idle, 1 queued/tracing, 2 completed */
    neoclr_aot_context *owner, copy;
    uintptr_t source;
    unsigned char *heap;
    uint64_t capacity, roots[SNAPSHOT_MAX_ROOTS];
    uint32_t root_count;
    int32_t status;
} snapshot_job;
static _Thread_local snapshot_job *snapshot;
#ifdef NEOCLR_GC_SNAPSHOT_TEST
static _Atomic int snapshot_test_hold, snapshot_test_entered;
#endif
static void *snapshot_worker(void *argument) {
    snapshot_job *job = argument;
    pthread_mutex_lock(&job->lock);
    for (;;) {
        while (!job->stop && atomic_load_explicit(&job->phase, memory_order_acquire) != 1)
            pthread_cond_wait(&job->wake, &job->lock);
        if (job->stop) break;
        pthread_mutex_unlock(&job->lock);
#ifdef NEOCLR_GC_SNAPSHOT_TEST
        atomic_store(&snapshot_test_entered, 1);
        while (atomic_load(&snapshot_test_hold)) { struct timespec t={0,1000000}; nanosleep(&t,NULL); }
#endif
        snapshot_source = job->source;
        snapshot_mark_only = 1;
        neoclr_probe_frame frame = {.context=&job->copy, .lanes=job->roots, .lane_count=job->root_count};
        job->status = collect_conditional(&job->copy, &frame, NULL, 0);
        snapshot_mark_only = 0;
        pthread_mutex_lock(&job->lock);
        atomic_store_explicit(&job->phase, 2, memory_order_release);
        pthread_cond_broadcast(&job->wake);
    }
    pthread_mutex_unlock(&job->lock);
    return NULL;
}
static int32_t snapshot_discard(void) {
    snapshot_job *job = snapshot;
    if (!job) return 0;
    pthread_mutex_lock(&job->lock);
    job->stop = 1;
    pthread_cond_broadcast(&job->wake);
    pthread_mutex_unlock(&job->lock);
    if (pthread_join(job->thread,NULL)) abort();
    pthread_cond_destroy(&job->wake);
    pthread_mutex_destroy(&job->lock);
    free(job->heap);
    free(job);
    snapshot = NULL;
    return 0;
}
static int snapshot_add(snapshot_job *job, uint64_t word) {
    if (job->root_count == SNAPSHOT_MAX_ROOTS) return 0;
    job->roots[job->root_count++] = word;
    return 1;
}
static int snapshot_slots(snapshot_job *job, const neoclr_probe_storage *slots, uint32_t count) {
    for (uint32_t i=0;i<count;i++) {
        if (slots[i].read_bytes != 8) continue;
        uint64_t word; memcpy(&word,slots[i].address,8);
        if (!snapshot_add(job,word)) return 0;
    }
    return 1;
}
static int snapshot_frames(snapshot_job *job, const neoclr_probe_frame *frame) {
    for (;frame;frame=frame->previous) {
        if (frame->context != job->owner) continue;
        if (!snapshot_slots(job,frame->storage,frame->storage_count) ||
            !snapshot_slots(job,frame->transient,frame->transient_count)) return 0;
        for (uint32_t i=0;i<frame->lane_count;i++)
            if (!snapshot_add(job,frame->lanes[i])) return 0;
    }
    return 1;
}
static int32_t snapshot_poll(neoclr_aot_context *context) {
    snapshot_job *job = snapshot;
    if (!job || atomic_load_explicit(&job->phase,memory_order_acquire) != 2) return 0;
    if (job->owner != context) return snapshot_discard();
    if (job->status) { int32_t status=job->status; snapshot_discard(); return status; }
    neoclr_aot_text_arena *a=&context->text;
    if (a->data != (void *)job->source || a->used < job->copy.text.used || !blocks_valid(a)) {
        snapshot_discard(); return 3;
    }
    /* Validate the old allocated descriptors before changing live state. No old
     * allocation can have been freed/reused: every blocking GC discards the job. */
    for (uint64_t at=0;at<job->copy.text.used;) {
        const block *old=(const void *)(job->heap+at);
        if (old->state & ALLOCATED) {
            const block *live=(const void *)(a->data+at);
            if (live->span != old->span || live->bytes != old->bytes || live->kind != old->kind || live->state != ALLOCATED) {
                snapshot_discard(); return 3;
            }
        }
        at+=old->span;
    }
    /* Preserve all post-snapshot allocations, including allocations in old free
     * spans. Only allocations present in the snapshot may become reclaimable. */
    for (uint64_t at=0;at<a->used;) {
        block *live=(void *)(a->data+at);
        if (live->state) live->state=ALLOCATED|MARKED;
        at+=live->span;
    }
    for (uint64_t at=0;at<job->copy.text.used;) {
        const block *old=(const void *)(job->heap+at);
        if (old->state & ALLOCATED)
            ((block *)(void *)(a->data+at))->state=ALLOCATED|(old->state & MARKED);
        at+=old->span;
    }
    int32_t status=finish_collection(context,NULL,0);
    atomic_store_explicit(&job->phase,0,memory_order_release);
    return status;
}
static int32_t snapshot_start(neoclr_aot_context *context,const neoclr_probe_frame *head) {
    if (!context || resource_cleanup_active || head != neoclr_root_probe_head_v1() ||
        (head && head->context != context) || !blocks_valid(&context->text)) return 3;
    if (snapshot && snapshot->owner != context) snapshot_discard();
    if (snapshot && atomic_load_explicit(&snapshot->phase,memory_order_acquire) != 0) return 0;
    if (!context->text.used) return 0;
    if (context->text.used > SNAPSHOT_MAX_BYTES) return neoclr_gc_collect_v1(context,head);
    if (!snapshot) {
        snapshot_job *job=calloc(1,sizeof(*job));
        if (!job) return neoclr_gc_collect_v1(context,head);
        if (pthread_mutex_init(&job->lock,NULL)) { free(job); return neoclr_gc_collect_v1(context,head); }
        if (pthread_cond_init(&job->wake,NULL)) { pthread_mutex_destroy(&job->lock); free(job); return neoclr_gc_collect_v1(context,head); }
        if (pthread_create(&job->thread,NULL,snapshot_worker,job)) {
            pthread_cond_destroy(&job->wake); pthread_mutex_destroy(&job->lock); free(job);
            return neoclr_gc_collect_v1(context,head);
        }
        snapshot=job;
    }
    snapshot_job *job=snapshot;
    job->owner=context; job->root_count=0;
    if (!snapshot_frames(job,head)) return neoclr_gc_collect_v1(context,head);
    /* Owned host/parked roots are deliberately strong for this automatic cycle.
     * Exact dependent lifetime/abandonment is handled by blocking collections. */
    for (host_root *root=host_head;root;root=root->next)
        if (root->context==context && !root->expired) {
            if (!snapshot_add(job,(uintptr_t)root->owner) || !snapshot_add(job,(uintptr_t)root->value))
                return neoclr_gc_collect_v1(context,head);
        }
    for (const neoclr_parked_roots *p=neoclr_roots_parked_v1();p;p=p->next)
        if (!p->expired && (!p->owner_context || p->owner_context==context)) {
            if (!snapshot_add(job,(uintptr_t)p->owner) || !snapshot_frames(job,p->head))
                return neoclr_gc_collect_v1(context,head);
        }
    if (job->capacity < context->text.used) {
        unsigned char *heap=realloc(job->heap,(size_t)context->text.used);
        if (!heap) return neoclr_gc_collect_v1(context,head);
        job->heap=heap; job->capacity=context->text.used;
    }
    job->source=(uintptr_t)context->text.data;
    job->copy=*context;
    memcpy(job->heap,context->text.data,(size_t)context->text.used);
    job->copy.text.data=job->heap;
    job->copy.text.capacity=job->capacity;
    pthread_mutex_lock(&job->lock);
    atomic_store_explicit(&job->phase,1,memory_order_release);
    pthread_cond_signal(&job->wake);
    pthread_mutex_unlock(&job->lock);
    return 0;
}
