/* Prototype compiler-facing scalar-store adapter. It never collects. The caller
 * supplies a valid native destination; out-of-arena storage is trusted stack or
 * host storage, not permission to mutate a different managed heap. */
/* These private hints/counters share storage, not lifetimes or admission.
 * The explicit split control preserves the former TLS layout. */
#if defined(NEOCLR_LIVE_SPLIT_TLS) || defined(NEOCLR_LIVE_SPLIT_STORE_TLS)
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
#ifndef NEOCLR_LIVE_HEADER_PROOF
static _Thread_local const void *live_store_last_payload;
#else
static _Thread_local struct {
    const void *payload;
    uint64_t span, bytes, kind;
} live_store_last_header;
#define live_store_last_payload (live_store_last_header.payload)
#endif
#ifndef NEOCLR_LIVE_STORE_RANGES
#define NEOCLR_LIVE_STORE_RANGES 4
#endif
#if NEOCLR_LIVE_STORE_RANGES < 1 || NEOCLR_LIVE_STORE_RANGES > 16
#error "store range count must be between 1 and 16"
#endif
#if NEOCLR_LIVE_STORE_RANGES > 1
static _Thread_local const void *live_store_recent[NEOCLR_LIVE_STORE_RANGES-1];
static _Thread_local unsigned live_store_recent_next;
#endif
#endif
#ifdef NEOCLR_LIVE_WALK_PROFILE
static _Thread_local uint64_t live_store_walk_counts[3]; /* headers visited, max per walk, free headers */
#endif
static _Thread_local uint64_t live_store_lookup_counts[2]; /* proved range/base, checked walks */
#else
#define live_store_lookup_counts (live_owner.store.lookups)
#ifdef NEOCLR_LIVE_WALK_PROFILE
#define live_store_walk_counts (live_owner.store.walks)
#endif
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
#ifdef NEOCLR_LIVE_HEADER_PROOF
#define live_store_last_header (live_owner.store.header)
#define live_store_last_payload (live_owner.store.header.payload)
#else
#define live_store_last_payload (live_owner.store.payload)
#endif
#if NEOCLR_LIVE_STORE_RANGES > 1
#define live_store_recent (live_owner.store.recent)
#define live_store_recent_next (live_owner.store.next)
#endif
#endif
#endif
#ifdef NEOCLR_LIVE_WALK_PROFILE
void experiment_gc_store_walk_counts(uint64_t *out) {
    memcpy(out,live_store_walk_counts,sizeof(live_store_walk_counts));
}
#endif
static void live_store_cancel_epoch(void) {
        candidate_dispose();
        candidate_context=NULL;
        candidate_active=0;
        live_store_counts[3]++;
}
/* Width was validated by the adapter. Constant-size memcpy keeps unaligned
 * destinations legal and preserves native integer truncation/byte order without
 * a second dynamic-size copy. This does not authorize or synchronize a store. */
static inline void live_store_bits(void *destination, uint64_t bits, uint32_t width) {
#ifdef NEOCLR_LIVE_SERIALIZED_STORE
    unsigned char value[8];
    if (width==1) { uint8_t v=(uint8_t)bits; memcpy(value,&v,1); }
    else if (width==2) { uint16_t v=(uint16_t)bits; memcpy(value,&v,2); }
    else if (width==4) { uint32_t v=(uint32_t)bits; memcpy(value,&v,4); }
    else memcpy(value,&bits,8);
    memcpy(destination,value,width);
#else
    if (width==1) { uint8_t v=(uint8_t)bits; memcpy(destination,&v,1); }
    else if (width==2) { uint16_t v=(uint16_t)bits; memcpy(destination,&v,2); }
    else if (width==4) { uint32_t v=(uint32_t)bits; memcpy(destination,&v,4); }
    else memcpy(destination,&bits,8);
#endif
}
#ifdef NEOCLR_LIVE_STORE_HANDLE
#ifdef NEOCLR_LIVE_SPLIT_TLS
#error Store handles require grouped candidate state
#endif
/* Trusted generated/fixture contract, like the existing root-state handle:
 * same native owner thread throughout the borrow; never export or migrate it.
 * Only the state address is borrowed, not a context, epoch or heap proof.
 * Heap ownership and all current validation still run on every store. */
void *experiment_gc_store_state_borrow(void) { return &live_owner; }
int experiment_gc_store_borrowed(void *opaque, neoclr_aot_context *context,
                                void *destination, uint64_t bits, uint32_t width) {
    if (!opaque) return 3;
    live_owner_state *store_state=opaque;
#define live_owner (*store_state)
#else
int experiment_gc_store(neoclr_aot_context *context, void *destination,
                       uint64_t bits, uint32_t width) {
#endif
    if (!context || owned_heap.arena != &context->text || !valid(&context->text) ||
        !destination || (width!=1 && width!=2 && width!=4 && width!=8) ||
        (candidate_active && candidate_context!=context)) goto rejected;
    uintptr_t address=(uintptr_t)destination, base=(uintptr_t)context->text.data;
    if (address>UINTPTR_MAX-width || base>UINTPTR_MAX-context->text.capacity) goto rejected;
    uintptr_t end=base+context->text.capacity;
    if (address>=end || address+width<=base) {
        live_store_counts[0]++;
        live_store_bits(destination,bits,width);
        return 0;
    }
    if (address<base || address-base>=context->text.used) goto rejected;
    neoclr_aot_text_arena *a=&context->text;
    block *b=NULL;
    uint64_t walk_start=0;
#if !defined(NEOCLR_LIVE_LINEAR_STORE) && !defined(NEOCLR_LIVE_REPEAT_SNAPSHOT_LOOKUP)
    int snapshot_allocation=0;
#endif
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE) && defined(NEOCLR_LIVE_HEADER_PROOF)
    int reused_header=0;
#endif
#ifndef NEOCLR_LIVE_LINEAR_STORE
    if (candidate_active) {
        shadow_snapshot *snapshot=shadow_owner->job;
        size_t index=shadow_find(snapshot,address);
        if (index!=SIZE_MAX) {
#ifndef NEOCLR_LIVE_REPEAT_SNAPSHOT_LOOKUP
            snapshot_allocation=1;
#endif
            uint64_t at=snapshot->offsets[index];
            const block *saved=&snapshot->headers[index];
            if (!block_valid(a,at)) goto rejected;
            b=(void *)(a->data+at);
            /* Old allocations cannot move, retire or be reclaimed during this
             * epoch. Validate the identity before trusting detached lookup. */
            if (!(b->state&ALLOCATED) || b->span!=saved->span ||
                b->bytes!=saved->bytes || b->kind!=saved->kind) goto rejected;
            goto located;
        }
    }
#endif
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
    /* The owned exact-base cache revokes proofs on reclamation/reset/release.
     * Never dereference a remembered range unless that current proof survives.
     * Snapshot identity validation above still owns old-epoch allocations. */
    if (live_store_last_payload && known_base(live_store_last_payload)) {
        uintptr_t previous=(uintptr_t)live_store_last_payload;
        if (previous>=base && previous-base>=sizeof(block) && previous-base<a->used && address>=previous) {
            uint64_t at=previous-base-sizeof(block);
            block *cached=(void *)(a->data+at);
#ifdef NEOCLR_LIVE_HEADER_PROOF
            /* The tuple was validated when this hint was installed. Current
             * owned-base admission above protects its address; matching fields
             * preserve the immutable constraints. State and extent stay live.
             * A legitimate metadata transition is revalidated, never assumed. */
            int same_header=cached->span==live_store_last_header.span &&
                cached->bytes==live_store_last_header.bytes &&
                cached->kind==live_store_last_header.kind;
            if (same_header) {
                if (!(cached->state&ALLOCATED) || cached->state>7 ||
                    cached->span>a->used-at) goto rejected;
            } else if (!block_valid(a,at)) goto rejected;
#else
            if (!block_valid(a,at)) goto rejected;
#endif
            if ((cached->state&ALLOCATED) && address-previous<cached->bytes) {
#ifdef NEOCLR_LIVE_HEADER_PROOF
                reused_header=same_header;
#endif
                b=cached;
                live_store_lookup_counts[0]++;
                goto located;
            }
#ifndef NEOCLR_LIVE_FULL_STORE_WALK
            if (cached->state&ALLOCATED) walk_start=at;
#endif
        }
    }
    /* Allocation-base writes can use the existing current owned proof directly.
     * This also seeds the last range for subsequent field initialization. Never
     * derive a header from an unproved destination or skip snapshot checks. */
#ifndef NEOCLR_LIVE_NO_EXACT_STORE
    if (address-base>=sizeof(block) && known_base(destination)) {
        uint64_t at=address-base-sizeof(block);
        if (!block_valid(a,at)) goto rejected;
        b=(void *)(a->data+at);
        if (!(b->state&ALLOCATED)) goto rejected;
        live_store_lookup_counts[0]++;
        goto located;
    }
#endif
#if NEOCLR_LIVE_STORE_RANGES > 1
    for (unsigned i=0;i<NEOCLR_LIVE_STORE_RANGES-1;i++) {
        const void *recent=live_store_recent[i];
        uintptr_t previous=(uintptr_t)recent;
        /* A stale hint is never a proof. Do not inspect its header until the
         * current owned exact-base cache authorizes it. */
        if (!recent || previous<base || previous-base<sizeof(block) ||
            previous-base>=a->used || address<previous || !known_base(recent)) continue;
        uint64_t at=previous-base-sizeof(block);
        if (!block_valid(a,at)) goto rejected;
        block *cached=(void *)(a->data+at);
        if ((cached->state&ALLOCATED) && address-previous<cached->bytes) {
            b=cached;
            live_store_lookup_counts[0]++;
            goto located;
        }
#ifndef NEOCLR_LIVE_FULL_STORE_WALK
        if ((cached->state&ALLOCATED) && at>walk_start) walk_start=at;
#endif
    }
#endif
#endif
    /* An admitted, validated allocation below the destination is also a safe
     * walk anchor. Current ownership revoked stale hints before header access;
     * retain checked span traversal from this boundary, never from user bytes. */
    live_store_lookup_counts[1]++;
#ifdef NEOCLR_LIVE_WALK_PROFILE
    uint64_t walk_steps=0;
#endif
#ifdef NEOCLR_LIVE_ALLOCATION_INDEX
    (void)walk_start;
    uint64_t indexed=owned_index_before(address-base);
    if (indexed==UINT64_MAX || indexed<sizeof(block) || indexed>=a->used) goto rejected;
#ifdef NEOCLR_LIVE_WALK_PROFILE
    live_store_walk_counts[0]++;
    if (!live_store_walk_counts[1]) live_store_walk_counts[1]=1;
    (void)walk_steps;
#endif
    uint64_t at=indexed-sizeof(block);
    if (!block_valid(a,at)) goto rejected;
    b=(void *)(a->data+at);
    if (!(b->state&ALLOCATED) || address-base-indexed>=b->bytes) goto rejected;
    goto located;
#else
    for (uint64_t at=walk_start;at<a->used;) {
#ifdef NEOCLR_LIVE_WALK_PROFILE
        live_store_walk_counts[0]++;
        if (++walk_steps>live_store_walk_counts[1]) live_store_walk_counts[1]=walk_steps;
#endif
        if (!block_valid(a,at)) goto rejected;
        b=(void *)(a->data+at);
#ifdef NEOCLR_LIVE_WALK_PROFILE
        live_store_walk_counts[2]+=!(b->state&ALLOCATED);
#endif
        uintptr_t payload=(uintptr_t)(b+1);
        if (address>=payload && address-payload<b->bytes && (b->state&ALLOCATED))
            goto located;
        at+=b->span;
    }
    goto rejected;
#endif
located:;
    uintptr_t payload=(uintptr_t)(b+1);
    uint64_t offset=address-payload;
    if (width>b->bytes-offset) goto rejected;
#if !defined(NEOCLR_LIVE_NO_STORE_CACHE) && !defined(NEOCLR_LIVE_LINEAR_STORE)
    remember_base((void *)payload);
#if NEOCLR_LIVE_STORE_RANGES > 1
    if (live_store_last_payload!=(void *)payload) {
        live_store_recent[live_store_recent_next]=live_store_last_payload;
        live_store_recent_next=(live_store_recent_next+1)%(NEOCLR_LIVE_STORE_RANGES-1);
    }
#endif
#ifdef NEOCLR_LIVE_HEADER_PROOF
    if (!reused_header) {
        live_store_last_header.span=b->span;
        live_store_last_header.bytes=b->bytes;
        live_store_last_header.kind=b->kind;
    }
#endif
    live_store_last_payload=(void *)payload;
#endif
    /* No callback, allocation or epoch transition occurs between lookup and
     * this store. Reuse its result; the marker cannot change snapshot identity.
     * Linear lookup and the measurement control retain the original search. */
#if defined(NEOCLR_LIVE_LINEAR_STORE) || defined(NEOCLR_LIVE_REPEAT_SNAPSHOT_LOOKUP)
    int new_allocation=!candidate_active || shadow_find(shadow_owner->job,address)==SIZE_MAX;
#else
    int new_allocation=!snapshot_allocation;
#endif
    if (new_allocation) {
        /* New allocations are excluded from marker reads and are all
         * traced at remark. No publication exemption for old objects. */
        live_store_counts[1]++;
        live_store_bits(destination,bits,width);
        return 0;
    }
    unsigned char value[8];
    live_store_bits(value,bits,width);
    mutation_patch patch={offset,width,value};
#ifdef NEOCLR_LIVE_LINEAR_STORE
    int status=experiment_gc_write(context,(void *)payload,&patch,1);
#else
    int status=live_write_known((void *)payload,b->bytes,b->kind,&patch,1);
#endif
    if (status==GATE_CANCELLED) {
        /* No root walk, sweep, cleanup or managed callback here. Discard
         * detached work before writing; future collection needs a normal
         * root-published boundary. Temporaries may be unpublished now. */
        live_store_cancel_epoch();
        live_store_bits(destination,bits,width);
        return 0;
    }
    if (status) goto rejected;
    live_store_counts[2]++;
    return 0;
rejected:
    live_store_counts[4]++;
    return 3;
}
#ifdef NEOCLR_LIVE_STORE_HANDLE
#undef live_owner
/* Ordinary native entry retains current-thread resolution. */
int experiment_gc_store(neoclr_aot_context *context, void *destination,
                       uint64_t bits, uint32_t width) {
    return experiment_gc_store_borrowed(&live_owner,context,destination,bits,width);
}
#endif
void experiment_gc_store_lookup_counts(uint64_t *out) {
    memcpy(out,live_store_lookup_counts,sizeof(live_store_lookup_counts));
}
void experiment_gc_store_counts(uint64_t *out) {
    memcpy(out,live_store_counts,sizeof(live_store_counts));
}

/* Trusted generated source is a compiler-owned stack snapshot, never an alias
 * into the destination. Publish payload and initialization marker together. */
int experiment_gc_store_record(neoclr_aot_context *context, void *array,
    uint32_t index, const void *source, uint32_t width) {
    if (!context || owned_heap.arena!=&context->text || !array || !source ||
        (candidate_active && candidate_context!=context)) return 3;
    uint64_t bytes; uint32_t kind;
    if (neoclr_gc_allocation_info_v1(context,array,&bytes,&kind) ||
        kind!=NEOCLR_GC_RECORDS || bytes<24) return 3;
    uint64_t header[3]; memcpy(header,array,24);
    uint64_t length=header[1],lanes=header[2];
    if ((uint32_t)header[0]!=UINT32_C(0x80000005) ||
        (header[0]>>32)>NEOCLR_NATIVE_TYPE_LIMIT ||
        ((header[0]>>32) && lanes!=1) || !lanes || lanes>NEOCLR_NATIVE_LAYOUT_LANES ||
        width!=lanes || length>65536 || index>=length ||
        length>(bytes-24)/(lanes*8+1)) return 3;
    unsigned char initialized=1;
    mutation_patch patches[2]={{24+index*lanes*8,lanes*8,source},
        {24+length*lanes*8+index,1,&initialized}};
    if (candidate_active && shadow_find(shadow_owner->job,(uintptr_t)array)!=SIZE_MAX) {
        int status=live_write_known(array,bytes,kind,patches,2);
        if (status==GATE_CANCELLED) live_store_cancel_epoch();
        else if (status) return 3;
        else { live_store_counts[2]++; return 0; }
    }
    for (unsigned i=0;i<2;i++)
        memcpy((unsigned char *)array+patches[i].offset,patches[i].source,patches[i].length);
    live_store_counts[1]++;
    return 0;
}

/* Text, character and callable-descriptor arrays share traced pointer storage. */
int experiment_gc_store_text(neoclr_aot_context *context, void *array,
    uint32_t index, uint64_t value) {
    if (!context || owned_heap.arena!=&context->text || !array ||
        (candidate_active && candidate_context!=context)) return 3;
    uint64_t bytes; uint32_t kind;
    if (neoclr_gc_allocation_info_v1(context,array,&bytes,&kind) ||
        kind!=NEOCLR_GC_STRINGS || bytes<16) return 3;
    uint64_t header[2]; memcpy(header,array,16);
    uint32_t tag=(uint32_t)header[0];
    unsigned reserved=tag==UINT32_C(0x80000004);
    uint64_t length=header[1];
    if ((!reserved && tag!=UINT32_C(0x80000003)) || index>=length ||
        length>(bytes-16)/(reserved?9:8)) return 3;
    unsigned char initialized=1;
    mutation_patch patches[2]={{16+(uint64_t)index*8,8,&value},
        {16+length*8+index,1,&initialized}};
    unsigned count=reserved?2:1;
    if (candidate_active && shadow_find(shadow_owner->job,(uintptr_t)array)!=SIZE_MAX) {
        int status=live_write_known(array,bytes,kind,patches,count);
        if (status==GATE_CANCELLED) live_store_cancel_epoch();
        else if (status) return 3;
        else { live_store_counts[2]++; return 0; }
    }
    for (unsigned i=0;i<count;i++)
        memcpy((unsigned char *)array+patches[i].offset,patches[i].source,patches[i].length);
    live_store_counts[1]++;
    return 0;
}

/* Typed compiler contract: source is one normalized, zero-padded stack lane.
 * Primitive allocations are never read by the marker. Keep current ownership,
 * exact allocation admission and shape/extent checks, but no mutation gate is
 * needed for this nontraced payload. This is not a raw-address store exemption. */
int experiment_gc_store_scalar(neoclr_aot_context *context, void *array,
    uint32_t index, const void *source) {
    if (!context || owned_heap.arena!=&context->text || !valid(&context->text) ||
        !array || !source || (candidate_active && candidate_context!=context)) return 3;
    uint64_t bytes; uint32_t kind;
    if (neoclr_gc_allocation_info_v1(context,array,&bytes,&kind) ||
        kind!=NEOCLR_GC_BYTES || bytes<24) return 3;
    uint64_t header[3]; memcpy(header,array,24);
    uint64_t length=header[1];
    if ((uint32_t)header[0]!=UINT32_C(0x80000006) ||
        (header[0]>>32)>NEOCLR_NATIVE_TYPE_LIMIT || header[2]!=1 ||
        length>65536 || index>=length || length>(bytes-24)/9) return 3;
    memcpy((unsigned char *)array+24+(uint64_t)index*8,source,8);
    ((unsigned char *)array)[24+length*8+index]=1;
    return 0;
}
