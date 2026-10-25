#ifdef NEOCLR_GATE_WORKERS
#include "worker-api.h"
int experiment_gc_worker_register(neoclr_aot_context *context, pthread_t thread, gate_writer *writer) {
    if (!candidate_active || candidate_context!=context || candidate_phase) return GATE_INVALID;
    return gate_register_writer(shadow_owner->job->gate,thread,writer);
}
int experiment_gc_worker_target_create(neoclr_aot_context *context, void *payload, live_worker_target *out) {
    if (!out || !candidate_active || candidate_context!=context || candidate_phase) return GATE_INVALID;
    uint64_t bytes;uint32_t kind;
    if (neoclr_gc_allocation_info_v1(context,payload,&bytes,&kind)) return GATE_INVALID;
    /* Other payload shapes require separate registered-worker qualification. */
    if (kind!=NEOCLR_GC_OBJECT) return GATE_INVALID;
    size_t header=8;
    if (bytes<header) return GATE_INVALID;
    shadow_snapshot *s=shadow_owner->job;
    *out=(live_worker_target){s->gate,s->epoch,payload,bytes,kind};
    return GATE_OK;
}
int experiment_gc_worker_write(const gate_writer *writer, const live_worker_target *target,
                               const mutation_patch *patches, size_t count) {
    if (!writer || !target || writer->gate!=target->gate || writer->epoch!=target->epoch ||
        count>GATE_PATCHES || (count && !patches)) return GATE_INVALID;
    if (target->kind!=NEOCLR_GC_OBJECT) return GATE_INVALID;
    size_t header=8;
    for(size_t i=0;i<count;i++) if(patches[i].length && patches[i].offset<header) return GATE_INVALID;
    return gate_write_registered(target->gate,writer,target->payload,target->bytes,
        8,patches,count);
}
int experiment_gc_worker_revoke(neoclr_aot_context *context, const gate_writer *writer) {
    if (!writer || !candidate_active || candidate_context!=context || candidate_phase ||
        writer->gate!=shadow_owner->job->gate) return GATE_INVALID;
    return gate_revoke_writer(writer->gate,writer);
}
#endif
