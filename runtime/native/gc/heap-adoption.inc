#ifdef NEOCLR_EXCLUSIVE_HEAP_ADOPTION
int32_t neoclr_gc_heap_adopt_exclusive_v1(neoclr_aot_text_arena *target) {
    if (owned_heap.arena || gc_state.cleanup_active || host_head || resource_head ||
        neoclr_root_probe_head_v1() || neoclr_roots_parked_v1() || !blocks_valid(target)) return 3;
    /* Acquire fresh TLS/index storage without changing the caller's live cursor.
     * All fallible work precedes descriptor changes. The temporary cursor is
     * rebound before any allocation, root admission or collector operation. */
    neoclr_aot_text_arena empty = *target;
    empty.used = 0;
    int32_t status = neoclr_gc_heap_acquire_v1(&empty);
    if (status) return status;
    owned_heap.arena = target;
    owned_heap.used = target->used;
    uint64_t *next = &owned_heap.free_head;
    for (uint64_t at = 0; at < target->used;) {
        block *b = (void *)(target->data + at);
        if (!b->state) {
            *next = at + 1;
            next = &b->bytes;
        } else {
            remember_base(b + 1);
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
            owned_index_set((unsigned char *)(b + 1) - target->data);
#endif
        }
        at += b->span;
    }
    *next = 0;
    gc_state.pacing.context = NULL; /* A previous invocation cannot authorize pacing. */
    return 0;
}
#endif
