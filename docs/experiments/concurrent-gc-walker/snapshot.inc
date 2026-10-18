/* Bounded capture for this experiment only. No parked frames are supported yet. */
#include <stdlib.h>
#define SNAPSHOT_BYTES (1024u * 1024u)
#define SNAPSHOT_ROOTS 4096u
#define SNAPSHOT_PAIRS 1024u
typedef struct {
    neoclr_aot_context copy;
    uintptr_t source;
    uint64_t roots[SNAPSHOT_ROOTS];
    neoclr_gc_conditional_root pairs[SNAPSHOT_PAIRS];
    unsigned roots_count, pairs_count;
    atomic_bool entered, proceed, tracing;
} snapshot;
static bool add_root(snapshot *s, uint64_t value) {
    if (s->roots_count == SNAPSHOT_ROOTS) return false;
    s->roots[s->roots_count++] = value;
    return true;
}
static bool copy_slots(snapshot *s, const neoclr_probe_storage *slots, unsigned count) {
    for (unsigned i = 0; i < count; i++) {
        if (slots[i].read_bytes != 8) continue;
        uint64_t word;
        memcpy(&word, slots[i].address, 8);
        if (!add_root(s, word)) return false;
    }
    return true;
}
static void *relocate(snapshot *s, const void *value) {
    if (!value) return NULL;
    return s->copy.text.data + ((uintptr_t)value - s->source);
}
static bool capture(snapshot *s, neoclr_aot_context *context) {
    if (!context || !blocks_valid(&context->text) || context->text.used > SNAPSHOT_BYTES ||
        neoclr_roots_parked_v1() || resource_cleanup_active) return false;
    for (neoclr_gc_resource *r = resource_head; r; r = r->next)
        if (r->context == context && (!r->owner || !validated_bases(&context->text, r->owner, NULL)))
            return false;
    *s = (snapshot){0};
    atomic_init(&s->entered, false);
    atomic_init(&s->proceed, false);
    atomic_init(&s->tracing, false);
    s->copy = *context;
    s->source = (uintptr_t)context->text.data;
    s->copy.text.capacity = context->text.used;
    s->copy.text.data = malloc(context->text.used ? context->text.used : 8);
    if (!s->copy.text.data) return false;
    memcpy(s->copy.text.data, context->text.data, context->text.used);
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous) {
        if (f->context != context) continue;
        if (!copy_slots(s, f->storage, f->storage_count) ||
            !copy_slots(s, f->transient, f->transient_count)) goto failed;
        for (unsigned i = 0; i < f->lane_count; i++)
            if (!add_root(s, f->lanes[i])) goto failed;
    }
    for (host_root *r = host_head; r; r = r->next) {
        if (r->context != context || r->expired) continue;
        if (!validated_bases(&context->text, r->value, r->conditional ? r->owner : NULL)) goto failed;
        if (r->conditional) {
            if (s->pairs_count == SNAPSHOT_PAIRS) goto failed;
            s->pairs[s->pairs_count++] = (neoclr_gc_conditional_root){
                relocate(s, r->owner), relocate(s, r->value), 0};
        } else if (!add_root(s, (uintptr_t)r->value)) goto failed;
    }
    return true;
failed:
    free(s->copy.text.data);
    s->copy.text.data = NULL;
    return false;
}
static int trace_snapshot(void *argument, const atomic_bool *cancel) {
    snapshot *s = argument;
    /* Dedicated worker TLS must have no live application registrations. */
    if (host_head || resource_head || neoclr_roots_parked_v1()) return 3;
    atomic_store(&s->entered, true);
    while (!atomic_load(&s->proceed) && !atomic_load(cancel)) sched_yield();
    if (atomic_load(cancel)) return 7;
    experiment_snapshot = 1;
    experiment_source = s->source;
    experiment_cancel = cancel;
    neoclr_probe_frame frame = {.context = &s->copy, .lanes = s->roots, .lane_count = s->roots_count};
    atomic_store(&s->tracing, true);
    int status = collect_conditional(&s->copy, &frame, s->pairs, s->pairs_count);
    experiment_snapshot = 0;
    experiment_source = 0;
    experiment_cancel = NULL;
    return atomic_load(cancel) ? 7 : status;
}
