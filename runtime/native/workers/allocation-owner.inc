#ifdef NEOCLR_WORKER_READ_EPOCHS
extern int32_t experiment_gc_begin(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *,const neoclr_probe_frame *,neoclr_gc_conditional_root *,uint32_t);
extern void experiment_gc_shadow(uint64_t *);
extern void experiment_gc_live_counts(uint64_t *);
#endif
/* Existing compiled exclusive lease remains held logically by the requester.
 * The pool gate publishes request/response; this service never invokes managed
 * code. Collection-owner mode consumes only copied root values while paused. The external arena is checked by the normal
 * allocator; it is not acquired as a thread-owned heap with persistent caches. */
static void *compiled_allocation_owner(void *opaque) {
    neoclr_workers *p=opaque;
    lock(p);
    for (;;) {
        while (!p->allocation.pending && !p->allocation.stop)
            if (pthread_cond_wait(&p->allocation.ready,&p->gate)) abort();
        if (p->allocation.stop) break;
        if (p->compiled_running!=1 || !p->compiled_exclusive ||
            p->allocation.arena!=&p->compiled_exclusive->context.text ||
            neoclr_gc_allocation_dispatch_hook) abort();
#ifdef NEOCLR_WORKER_COLLECTION_OWNER
        if (p->allocation.pending>=2) {
            neoclr_aot_context *context=&p->allocation.collector_context;
            context->text=*p->allocation.arena;
            neoclr_probe_frame frame={.context=context,.lanes=p->allocation.roots,
                .lane_count=p->allocation.root_count};
#ifdef NEOCLR_WORKER_READ_EPOCHS
            if (p->allocation.pending==3) {
                if (p->allocation.epoch_active) abort();
                p->allocation.status=experiment_gc_begin(context,&frame,NULL,0);
                if (!p->allocation.status) {
                    p->allocation.epoch_active=1;++p->allocation.epoch_started;
                }
            } else if (p->allocation.epoch_active) {
                p->allocation.status=experiment_gc_finish(context,&frame,NULL,0);
                p->allocation.epoch_active=0;++p->allocation.epoch_finished;
                uint64_t live[5];experiment_gc_live_counts(live);
                if (live[4] || live[3]!=p->allocation.epoch_finished) abort();
                p->allocation.epoch_payload_bytes=live[1];
                ++p->allocation.collections;
            } else
#endif
            {
                if (p->allocation.pending!=2) abort();
                p->allocation.status=neoclr_gc_collect_conditional_v1(context,&frame,NULL,0);
                ++p->allocation.collections;
            }
            *p->allocation.arena=context->text;
        } else
#endif
        {
            if (p->allocation.pending!=1) abort();
#ifdef NEOCLR_WORKER_READ_EPOCHS
            if (p->allocation.epoch_active) abort();
#endif
            p->allocation.status=neoclr_gc_allocate_dispatch_storage_v1(
                p->allocation.arena,p->allocation.bytes,p->allocation.kind,&p->allocation.result);
            ++p->allocation.requests;
#ifdef NEOCLR_WORKER_PACED
            if (!p->allocation.status) {
                ++p->allocation.allocations;
                p->allocation.allocated_bytes+=p->allocation.bytes;
            }
#endif
        }
        p->allocation.used=p->allocation.arena->used;
        p->allocation.pending=0;
        pthread_cond_signal(&p->allocation.ready);
    }
    if (p->allocation.pending) abort();
#ifdef NEOCLR_WORKER_READ_EPOCHS
    if (p->allocation.epoch_active) abort();
    uint64_t stats[5];experiment_gc_shadow(stats); /* Join persistent marker before pool teardown. */
#endif
    unlock(p);return NULL;
}
static int32_t compiled_allocate(neoclr_aot_text_arena *arena,uint64_t bytes,
    uint32_t kind,void **output,uint64_t *used) {
    compiled_participant *self=compiled_current;
    if (!self || arena!=&self->context.text || !output || !used) return 3;
    neoclr_workers *p=self->job->pool;
    lock(p);
    if (!self->exclusive || p->compiled_exclusive!=self || p->compiled_running!=1 ||
        !p->allocation.started || p->allocation.pending || p->allocation.stop) abort();
    p->allocation.arena=arena;p->allocation.bytes=bytes;p->allocation.kind=kind;
    p->allocation.pending=1;
    pthread_cond_signal(&p->allocation.ready);
    while (p->allocation.pending)
        if (pthread_cond_wait(&p->allocation.ready,&p->gate)) abort();
    int32_t status=p->allocation.status;
    if (!status) {*output=p->allocation.result;*used=p->allocation.used;}
    p->allocation.arena=NULL;p->allocation.result=NULL;
    unlock(p);return status;
}
int neoclr_workers_allocation_requests(neoclr_workers *p,uint64_t *requests) {
    if (!requests || !pthread_equal(p->owner,pthread_self())) return 0;
    lock(p);*requests=p->allocation.requests;unlock(p);return 1;
}

#ifdef NEOCLR_WORKER_COLLECTION_OWNER
/* Caller holds pool gate and exclusive lease; values stay valid until response. */
static int32_t compiled_collect_on_owner(compiled_participant *self,const uint64_t *values,uint32_t count,unsigned operation) {
    neoclr_workers *p=self->job->pool;
    if (!self->exclusive || p->compiled_exclusive!=self || p->compiled_running!=1 ||
        !p->allocation.started || p->allocation.pending || p->allocation.stop) abort();
    p->allocation.arena=&self->context.text;
    p->allocation.roots=values;p->allocation.root_count=count;p->allocation.pending=operation;
    pthread_cond_signal(&p->allocation.ready);
    while (p->allocation.pending)
        if (pthread_cond_wait(&p->allocation.ready,&p->gate)) abort();
    int32_t status=p->allocation.status;
    p->allocation.arena=NULL;p->allocation.roots=NULL;p->allocation.root_count=0;
    return status;
}
int neoclr_workers_collection_requests(neoclr_workers *p,uint64_t *requests) {
    if (!requests || !pthread_equal(p->owner,pthread_self())) return 0;
    lock(p);*requests=p->allocation.collections;unlock(p);return 1;
}
#endif

#ifdef NEOCLR_WORKER_READ_EPOCHS
int neoclr_workers_read_epoch_statistics(neoclr_workers *p,uint64_t *started,uint64_t *finished,uint64_t *readers,uint64_t *payload_bytes) {
    if (!started || !finished || !readers || !payload_bytes || !pthread_equal(p->owner,pthread_self())) return 0;
    lock(p);*started=p->allocation.epoch_started;*finished=p->allocation.epoch_finished;
    *readers=p->allocation.epoch_readers;*payload_bytes=p->allocation.epoch_payload_bytes;unlock(p);return 1;
}
#endif
