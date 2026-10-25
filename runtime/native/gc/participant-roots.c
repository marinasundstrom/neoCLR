#include "participant-roots.h"
#include <string.h>
#include <stdlib.h>

static int append(uint64_t *output, size_t *count, uint64_t value) {
    if (*count == SIZE_MAX) return 3;
    if (output) output[*count] = value;
    ++*count;
    return 0;
}
static int slots(const neoclr_probe_storage *storage, uint32_t count, int flags,
                 uint64_t *output, size_t *used) {
    if (count && !storage) return 3;
    for (uint32_t i=0;i<count;i++) {
        const neoclr_probe_storage *s=&storage[i];
        if (!s->address || (s->read_bytes!=4 && s->read_bytes!=8) || (flags && s->flags>2)) return 3;
        /* Match mark_slots: discriminators are not roots; borrow addresses are
         * copied as words, never followed to the borrowed pointee. */
        if (s->read_bytes==8) {
            uint64_t value=0;
            if(output) memcpy(&value,s->address,8);
            if (append(output,used,value)) return 3;
        }
    }
    return 0;
}
static int walk(const neoclr_aot_context *owner, const neoclr_gc_participant *parts,
                unsigned count, uint64_t *output, size_t *used) {
    if (!owner || (count && !parts) || count>17) return 3;
    *used=0;
    for(unsigned i=0;i<count;i++) {
        const neoclr_aot_context *context=parts[i].context;
        if (!context || context->text.data!=owner->text.data ||
            context->text.capacity!=owner->text.capacity) return 3;
        const neoclr_probe_frame *slow=parts[i].frames,*fast=slow;
        while(fast && fast->previous) {
            slow=slow->previous;fast=fast->previous->previous;
            if(slow==fast) return 3;
        }
        neoclr_probe_storage fault[65];
        int32_t faults=neoclr_probe_fault_roots_v1(context,fault,65);
        if(faults<0 || slots(fault,(uint32_t)faults,0,output,used) ||
            append(output,used,(uintptr_t)context->static_storage)) return 3;
        for(const neoclr_probe_frame *f=parts[i].frames;f;f=f->previous) {
            if(f->context!=context || (f->lane_count && !f->lanes) || f->lane_count>8192 ||
                slots(f->storage,f->storage_count,1,output,used) ||
                slots(f->transient,f->transient_count,0,output,used)) return 3;
            for(uint32_t n=0;n<f->lane_count;n++)
                if(append(output,used,output?f->lanes[n]:0)) return 3;
        }
    }
    return 0;
}
int neoclr_gc_participant_root_count_v1(const neoclr_aot_context *owner, const neoclr_gc_participant *parts,
                           unsigned count, size_t *result) {
    size_t used;
    if(!result || walk(owner,parts,count,NULL,&used)) return 3;
    *result=used;
    return 0;
}
int neoclr_gc_participant_root_capture_v1(const neoclr_aot_context *owner, const neoclr_gc_participant *parts,
                             unsigned count, uint64_t *output, size_t capacity, size_t *result) {
    size_t used;
    if(!result || walk(owner,parts,count,NULL,&used) || used>capacity || (used && !output)) return 3;
    /* No mutation can occur between these passes under the caller's rendezvous.
     * The second pass cannot newly fail under that contract. */
    if(walk(owner,parts,count,output,&used)) return 3;
    *result=used;
    return 0;
}

int neoclr_gc_participant_root_snapshot_v1(const neoclr_aot_context *owner,
    const neoclr_gc_participant *parts, unsigned count, neoclr_gc_root_buffer *buffer, size_t *result) {
    size_t used;
    if (!buffer || !result || (!!buffer->values != !!buffer->capacity) ||
        walk(owner,parts,count,NULL,&used) || used>UINT32_MAX || used>SIZE_MAX/sizeof(uint64_t)) return 3;
    if (used>buffer->capacity) {
        size_t capacity=64;
        while (capacity<used && capacity<=SIZE_MAX/2/sizeof(uint64_t)) capacity*=2;
        if (capacity<used) capacity=used;
        uint64_t *values=realloc(buffer->values,capacity*sizeof(uint64_t));
        if (!values) return 3;
        buffer->values=values;buffer->capacity=capacity;++buffer->growths;
    }
    /* Under the required rendezvous the validated graph cannot change between
     * sizing and copying. A failed second pass means the caller broke ownership. */
    if (walk(owner,parts,count,buffer->values,&used)) abort();
    ++buffer->captures;*result=used;return 0;
}
void neoclr_gc_root_buffer_dispose_v1(neoclr_gc_root_buffer *buffer) {
    if (!buffer) return;
    free(buffer->values);memset(buffer,0,sizeof(*buffer));
}
