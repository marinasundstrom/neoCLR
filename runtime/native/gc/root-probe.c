#include "root-probe.h"
#include <stdlib.h>
#include <string.h>
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
static _Thread_local uint64_t calls;
static _Thread_local uint32_t depth;
static _Thread_local neoclr_probe_frame *head;
int32_t neoclr_probe_fault_roots_v1(const void *context, neoclr_probe_storage *output,
                                   uint32_t capacity) {
    if (!context) return -1;
    const neoclr_aot_fault *fault = context;
    if (!fault->code) return 0;
    if (fault->frame_count > 64 || !output || capacity < 1 + fault->frame_count) return -1;
    output[0] = (neoclr_probe_storage){&fault->message, 8, 0};
    for (uint32_t i = 0; i < fault->frame_count; i++)
        output[i + 1] = (neoclr_probe_storage){&fault->frames[i].function, 8, 0};
    return (int32_t)(1 + fault->frame_count);
}
static void observe_fault(const void *context) {
    neoclr_probe_storage slots[65];
    int32_t count = neoclr_probe_fault_roots_v1(context, slots, 65);
    if (count < 0) abort();
#ifndef NEOCLR_NATIVE_GC
    volatile uint64_t observed = 0;
    for (int32_t i = 0; i < count; i++) {
        uint64_t word;
        memcpy(&word, slots[i].address, sizeof(word));
        observed ^= word;
    }
    (void)observed;
#endif
}
void neoclr_probe_enter_v3(neoclr_probe_frame *frame, const void *context, uint32_t function,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (!frame || !context || function >= NEOCLR_NATIVE_FUNCTION_LIMIT) abort();
    memset(frame, 0, sizeof(*frame));
    frame->previous = head;
    frame->context = context;
    frame->function = function;
    if (!storage || !plan || strlen(plan) != length) abort();
    frame->storage = storage; frame->storage_count = count;
    frame->storage_plan = plan; frame->storage_length = length;
    head = frame;
    depth++;
}
void neoclr_probe_leave_v1(neoclr_probe_frame *frame) {
    if (!frame || head != frame || !depth) abort();
    observe_fault(frame->context);
    head = frame->previous;
    depth--;
    memset(frame, 0, sizeof(*frame));
}
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length) {
    if (!frame || head != frame || instruction >= 8192 || lane_count > 8192 || !lanes ||
        !plan || strlen(plan) != length || !strstr(plan, "requiredSpillLanes")) abort();
    frame->transient = NULL; frame->transient_plan = NULL;
    frame->transient_count = frame->transient_length = frame->transient_phase = 0;
    frame->instruction = instruction;
    frame->lanes = lanes;
    frame->lane_count = lane_count;
    frame->plan = plan;
    frame->length = length;
    /* The collector consumes root values after publication. Keep descriptor and
     * fault validation here, but reserve volatile observation for probe-only builds. */
#ifndef NEOCLR_NATIVE_GC
    volatile uint64_t value = 0;
#endif
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        observe_fault(f->context);
        for (uint32_t i = 0; i < f->storage_count; i++) {
            const neoclr_probe_storage *s = &f->storage[i];
            if (!s->address || (s->read_bytes != 4 && s->read_bytes != 8) || s->flags > 2) abort();
#ifndef NEOCLR_NATIVE_GC
            uint64_t word = 0;
            memcpy(&word, s->address, s->read_bytes);
            value ^= word; /* Observe the slot only; never follow a borrowed pointee. */
#endif
        }
        for (uint32_t i = 0; i < f->transient_count; i++) {
            const neoclr_probe_storage *s = &f->transient[i];
            if (!s->address || (s->read_bytes != 4 && s->read_bytes != 8)) abort();
#ifndef NEOCLR_NATIVE_GC
            uint64_t word = 0; memcpy(&word, s->address, s->read_bytes); value ^= word;
#endif
        }
#ifndef NEOCLR_NATIVE_GC
        for (uint32_t i = 0; i < f->lane_count; i++) value ^= f->lanes[i];
#endif
    }
#ifndef NEOCLR_NATIVE_GC
    (void)value;
#endif
    calls++;
#ifdef NEOCLR_NATIVE_GC
    /* Complete snapshot: audited allocating helpers additionally protect their temporaries. */
#ifdef NEOCLR_GC_PACED
    if (neoclr_gc_boundary_v1((neoclr_aot_context *)frame->context, head)) abort();
#else
    if (neoclr_gc_collect_v1((neoclr_aot_context *)frame->context, head)) abort();
#endif
#endif
}
void neoclr_probe_transient_v2(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (head != frame || (phase != 1 && phase != 2 && phase != 3) || !storage || !plan || strlen(plan) != length) abort();
    frame->transient = storage; frame->transient_count = count;
    frame->transient_plan = plan; frame->transient_length = length; frame->transient_phase = phase;
#ifndef NEOCLR_NATIVE_GC
    volatile uint64_t observed = 0;
#endif
    for (uint32_t i = 0; i < count; i++) {
        if (!storage[i].address || (storage[i].read_bytes != 4 && storage[i].read_bytes != 8)) abort();
#ifndef NEOCLR_NATIVE_GC
        uint64_t word = 0; memcpy(&word, storage[i].address, storage[i].read_bytes);
        observed ^= word;
#endif
    }
#ifndef NEOCLR_NATIVE_GC
    (void)observed;
#endif
}
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return head; }
uint64_t neoclr_root_probe_count_v1(void) { return calls; }
uint32_t neoclr_root_probe_depth_v1(void) { return depth; }

static _Thread_local neoclr_parked_roots *parked;
void neoclr_roots_park_v1(neoclr_parked_roots *record) {
    if (!record || record->head || record->next || record->owner_context ||
        record->owner || record->expired) abort();
    for (neoclr_parked_roots *p = parked; p; p = p->next) if (p == record) abort();
    record->head = head;
    record->next = parked;
    parked = record;
    head = NULL;
    depth = 0;
}
void neoclr_roots_resume_v1(neoclr_parked_roots *record) {
    if (!record || head || record->expired) abort();
    neoclr_parked_roots **p = &parked;
    while (*p && *p != record) p = &(*p)->next;
    if (!*p) abort();
    *p = record->next;
    head = record->head;
    memset(record, 0, sizeof(*record));
    depth = 0;
    for (const neoclr_probe_frame *f = head; f; f = f->previous) depth++;
}
const neoclr_parked_roots *neoclr_roots_parked_v1(void) { return parked; }

void neoclr_roots_discard_v1(neoclr_parked_roots *record) {
    neoclr_parked_roots **p = &parked;
    while (*p && *p != record) p = &(*p)->next;
    if (!*p) abort();
    *p = record->next;
    memset(record, 0, sizeof(*record));
}

#ifdef NEOCLR_NATIVE_GC
/* Capture published logical frames, never machine-stack bytes or local values.
 * Saved frames are active native stack records; names belong to the immutable
 * image, not the managed heap. With a complete current-context boundary this
 * single allocation may collect/retry. No managed intermediate survives across
 * it. Publish output only after a complete owned snapshot has been written. */
int32_t neoclr_stack_trace_capture_v1(int32_t skip, neoclr_aot_context *context,
    const neoclr_aot_text **output, const neoclr_aot_text *const *names, uint32_t count) {
    if (skip < 0 || !context || !output || !names) return 3;
    const neoclr_probe_frame *frames[64];
    uint32_t selected = 0;
    uint64_t length = 0;
    int truncated = 0;
    const char marker[] = "   ... stack trace truncated\n";
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        if (f->context != context) continue;
        if (skip) { --skip; continue; }
        if (f->function >= count || !names[f->function]) return 3;
        uint64_t name_length = names[f->function]->length;
        uint64_t digits = 1;
        for (uint32_t n = f->instruction; n >= 10; n /= 10) ++digits;
        uint64_t extra = 6 + name_length + 14 + digits + 2;
        if (selected == 64 || name_length > 65536 || extra > 65536 - (sizeof(marker) - 1) - length) {
            truncated = 1;
            break;
        }
        frames[selected++] = f;
        length += extra;
    }
    if (truncated) length += sizeof(marker) - 1;
    void *allocation = NULL;
    /* A host may capture an ancestor context while another context is active.
     * Preserve that noncollecting path: it is not a current-context retry boundary. */
    int32_t status = !head || head->context == context
        ? neoclr_gc_allocate_rooted_v1(context, head, 8 + length, NEOCLR_GC_TEXT, &allocation)
        : neoclr_gc_allocate_v1(&context->text, 8 + length, NEOCLR_GC_TEXT, &allocation);
    if (status) return status;
    neoclr_aot_text *text = allocation;
    text->length = length;
    unsigned char *cursor = (unsigned char *)text->bytes;
    for (uint32_t i = 0; i < selected; i++) {
        const neoclr_probe_frame *f = frames[i];
        const neoclr_aot_text *name = names[f->function];
        memcpy(cursor, "   at ", 6); cursor += 6;
        memcpy(cursor, name->bytes, (size_t)name->length); cursor += name->length;
        memcpy(cursor, " [instruction ", 14); cursor += 14;
        char digits[10]; unsigned used = 0; uint32_t n = f->instruction;
        do { digits[used++] = (char)('0' + n % 10); n /= 10; } while (n);
        while (used) *cursor++ = (unsigned char)digits[--used];
        *cursor++ = ']'; *cursor++ = '\n';
    }
    if (truncated) { memcpy(cursor, marker, sizeof(marker) - 1); cursor += sizeof(marker) - 1; }
    if ((uint64_t)(cursor - (unsigned char *)text->bytes) != length) abort();
    *output = text;
    return 0;
}
#endif
