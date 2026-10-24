/* Prototype compiler-facing scalar-store adapter. It never collects. The caller
 * supplies a valid native destination; out-of-arena storage is trusted stack or
 * host storage, not permission to mutate a different managed heap. */
static _Thread_local uint64_t live_store_counts[5]; /* outside, inactive/new, guarded old, cancelled, rejected */
static void live_store_cancel_epoch(void) {
        candidate_dispose();
        candidate_context=NULL;
        candidate_active=0;
        live_store_counts[3]++;
}
int experiment_gc_store(neoclr_aot_context *context, void *destination,
                       uint64_t bits, uint32_t width) {
    if (!context || owned_heap.arena != &context->text || !valid(&context->text) ||
        !destination || (width!=1 && width!=2 && width!=4 && width!=8) ||
        (candidate_active && candidate_context!=context)) goto rejected;
    uintptr_t address=(uintptr_t)destination, base=(uintptr_t)context->text.data;
    if (address>UINTPTR_MAX-width || base>UINTPTR_MAX-context->text.capacity) goto rejected;
    unsigned char value[8];
    if (width==1) { uint8_t v=(uint8_t)bits; memcpy(value,&v,1); }
    else if (width==2) { uint16_t v=(uint16_t)bits; memcpy(value,&v,2); }
    else if (width==4) { uint32_t v=(uint32_t)bits; memcpy(value,&v,4); }
    else memcpy(value,&bits,8);
    uintptr_t end=base+context->text.capacity;
    if (address>=end || address+width<=base) {
        live_store_counts[0]++;
        memcpy(destination,value,width);
        return 0;
    }
    if (address<base || address-base>=context->text.used) goto rejected;
    neoclr_aot_text_arena *a=&context->text;
    block *b=NULL;
#ifndef NEOCLR_LIVE_LINEAR_STORE
    if (candidate_active) {
        shadow_snapshot *snapshot=shadow_owner->job;
        size_t index=shadow_find(snapshot,address);
        if (index!=SIZE_MAX) {
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
    for (uint64_t at=0;at<a->used;) {
        if (!block_valid(a,at)) goto rejected;
        b=(void *)(a->data+at);
        uintptr_t payload=(uintptr_t)(b+1);
        if (address>=payload && address-payload<b->bytes && (b->state&ALLOCATED))
            goto located;
        at+=b->span;
    }
    goto rejected;
located:;
    uintptr_t payload=(uintptr_t)(b+1);
    uint64_t offset=address-payload;
    if (width>b->bytes-offset) goto rejected;
    if (!candidate_active || shadow_find(shadow_owner->job,address)==SIZE_MAX) {
        /* New allocations are excluded from marker reads and are all
         * traced at remark. No publication exemption for old objects. */
        live_store_counts[1]++;
        memcpy(destination,value,width);
        return 0;
    }
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
        memcpy(destination,value,width);
        return 0;
    }
    if (status) goto rejected;
    live_store_counts[2]++;
    return 0;
rejected:
    live_store_counts[4]++;
    return 3;
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

/* Text/character arrays share the traced pointer-array layout. */
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
