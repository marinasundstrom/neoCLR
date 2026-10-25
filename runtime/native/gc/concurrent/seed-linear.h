#ifdef NEOCLR_LIVE_LINEAR_SEED
/* The current heap's spans were validated by build_index; detached offsets are
 * ordered snapshot metadata. Validate all old identities before applying marks.
 * Free snapshot blocks may have split/merged or gained allocations. */
static int candidate_validate_old_linear(neoclr_aot_text_arena *a, const shadow_snapshot *s) {
    uint64_t at=0;
    for(size_t i=0;i<s->count;i++) {
        if(i && s->offsets[i]<=s->offsets[i-1]) return 3;
        const block *old=&s->headers[i];
        if(!(old->state & ALLOCATED)) continue;
        while(at<a->used && at<s->offsets[i]) {
            const block *b=(const void *)(a->data+at);
            at+=b->span;
        }
        if(at>=a->used || at!=s->offsets[i]) return 3;
        const block *now=(const void *)(a->data+at);
        if(!(now->state & ALLOCATED) || now->span!=old->span ||
            now->bytes!=old->bytes || now->kind!=old->kind) return 3;
    }
    return 0;
}
#endif
