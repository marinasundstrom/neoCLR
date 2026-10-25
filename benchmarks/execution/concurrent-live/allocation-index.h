/* Experimental owner-local allocation-start index. Never read by the marker.
 * Entries describe allocated payload bases, not arbitrary observed addresses. */
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
static _Thread_local uint64_t owned_index_counts[3]; /* successful native allocations, current bytes, peak bytes */
void experiment_gc_index_counts(uint64_t *out) { memcpy(out,owned_index_counts,sizeof(owned_index_counts)); }
static void owned_index_destroy(void) {
    free(owned_heap.starts);
    owned_index_counts[1]=0;
}
#ifdef NEOCLR_SHADOW_TEST
static _Thread_local int owned_index_fail_next;
void experiment_gc_index_fail_next(void) { owned_index_fail_next=1; }
#endif
static int owned_index_create(uint64_t capacity) {
    uint64_t words=capacity/512+(capacity%512!=0);
    uint64_t groups=words/64+(words%64!=0);
    if (words+groups>SIZE_MAX/sizeof(uint64_t)) return 0;
    uint64_t *memory;
#ifdef NEOCLR_SHADOW_TEST
    if (owned_index_fail_next) { owned_index_fail_next=0; memory=NULL; }
    else
#endif
    memory=words?calloc((size_t)(words+groups),sizeof(uint64_t)):NULL;
    if (words && !memory) return 0;
    uint64_t bytes=(words+groups)*sizeof(uint64_t);
    owned_index_counts[0]+=memory!=NULL;
    owned_index_counts[1]=bytes;
    if (bytes>owned_index_counts[2]) owned_index_counts[2]=bytes;
    owned_heap.starts=memory;
    owned_heap.start_summary=memory?memory+words:NULL;
    owned_heap.start_words=words;
    owned_heap.start_summary_words=groups;
    return 1;
}
static __attribute__((unused)) void owned_index_reset(void) {
    if (owned_heap.starts)
        memset(owned_heap.starts,0,(size_t)(owned_heap.start_words+owned_heap.start_summary_words)*sizeof(uint64_t));
}
static void owned_index_set(uint64_t offset) {
    uint64_t bit=offset/8, word=bit/64;
    owned_heap.starts[word]|=UINT64_C(1)<<(bit%64);
    owned_heap.start_summary[word/64]|=UINT64_C(1)<<(word%64);
}
static void owned_index_clear(uint64_t offset) {
    uint64_t bit=offset/8, word=bit/64;
    owned_heap.starts[word]&=~(UINT64_C(1)<<(bit%64));
    if (!owned_heap.starts[word])
        owned_heap.start_summary[word/64]&=~(UINT64_C(1)<<(word%64));
}
/* Caller has checked current ownership and offset < used <= capacity. */
static uint64_t owned_index_before(uint64_t offset) {
    uint64_t bit=offset/8, word=bit/64;
    uint64_t bits=owned_heap.starts[word] & (UINT64_MAX>>(63-bit%64));
    if (!bits) {
        uint64_t group=word/64;
        bits=owned_heap.start_summary[group] & ((UINT64_C(1)<<(word%64))-1);
        while (!bits && group) bits=owned_heap.start_summary[--group];
        if (!bits) return UINT64_MAX;
        word=group*64+63-(uint64_t)__builtin_clzll(bits);
        bits=owned_heap.starts[word];
    }
    return (word*64+63-(uint64_t)__builtin_clzll(bits))*8;
}
#endif
#if defined(NEOCLR_LIVE_ALLOCATION_INDEX) && defined(NEOCLR_SHADOW_TEST)
/* Test-only observation of index contents; no store authorization. */
void *experiment_gc_index_before(const neoclr_aot_context *context, uint64_t offset) {
    if (!context || owned_heap.arena!=&context->text || offset>=owned_heap.used) return NULL;
    uint64_t at=owned_index_before(offset);
    return at==UINT64_MAX?NULL:owned_heap.data+at;
}
#endif
