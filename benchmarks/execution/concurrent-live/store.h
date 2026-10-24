/* Prototype compiler-facing scalar-store adapter. It never collects. The caller
 * supplies a valid native destination; out-of-arena storage is trusted stack or
 * host storage, not permission to mutate a different managed heap. */
static _Thread_local uint64_t live_store_counts[5]; /* outside, inactive/new, guarded old, cancelled, rejected */
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
    for (uint64_t at=0;at<a->used;) {
        if (!block_valid(a,at)) goto rejected;
        block *b=(void *)(a->data+at);
        uintptr_t payload=(uintptr_t)(b+1);
        if (address>=payload && address-payload<b->bytes && (b->state&ALLOCATED)) {
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
            int status=experiment_gc_write(context,(void *)payload,&patch,1);
            if (status==GATE_CANCELLED) {
                /* No root walk, sweep, cleanup or managed callback here. Discard
                 * detached work before writing; future collection needs a normal
                 * root-published boundary. Temporaries may be unpublished now. */
                candidate_dispose();
                candidate_context=NULL;
                candidate_active=0;
                live_store_counts[3]++;
                memcpy(destination,value,width);
                return 0;
            }
            if (status) goto rejected;
            live_store_counts[2]++;
            return 0;
        }
        at+=b->span;
    }
rejected:
    live_store_counts[4]++;
    return 3;
}
void experiment_gc_store_counts(uint64_t *out) {
    memcpy(out,live_store_counts,sizeof(live_store_counts));
}
