#include "root-probe.h"
#include <stdlib.h>
#include <string.h>
#ifdef NEOCLR_NATIVE_GC
#include "native-gc.h"
#endif
/* Resolve one thread-local object for the related hot publication state. This
 * changes only private storage: frame chains, counters and search hints retain
 * their independent lifetimes and are never shared between native threads. */
#if defined(NEOCLR_NATIVE_GC) && defined(NEOCLR_GC_SITE_PROFILE)
/* Optional diagnostic sink; absent from ordinary runtime builds. */
extern void neoclr_gc_site_profile_v1(uint32_t function, uint32_t instruction);
#endif
static _Thread_local struct {
    neoclr_probe_frame *head;
    uint64_t calls;
    uint32_t depth;
    uint32_t spill_marker_offsets[256];
} probe_state;
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
#ifdef NEOCLR_NATIVE_GC
    /* Collection enumerates fault roots separately. Publication only needs the
     * current fault shape, not a temporary array of root descriptors. */
    if (!context) abort();
    const neoclr_aot_fault *fault = context;
    if (fault->code && fault->frame_count > 64) abort();
#else
    neoclr_probe_storage slots[65];
    int32_t count = neoclr_probe_fault_roots_v1(context, slots, 65);
    if (count < 0) abort();
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
    frame->previous = probe_state.head;
    frame->context = context;
    frame->function = function;
    if (!storage || !plan || strlen(plan) != length) abort();
    frame->storage = storage; frame->storage_count = count;
    frame->storage_plan = plan; frame->storage_length = length;
    probe_state.head = frame;
    probe_state.depth++;
}
void neoclr_probe_leave_v1(neoclr_probe_frame *frame) {
    if (!frame || probe_state.head != frame || !probe_state.depth) abort();
    observe_fault(frame->context);
    probe_state.head = frame->previous;
    probe_state.depth--;
    memset(frame, 0, sizeof(*frame));
}
/* Cache a search hint, never a validation result or an image pointer. Every
 * publication still checks the current string length and marker bytes. A hash
 * collision, mutable plan or reused image address can only cause another search.
 * The bounded thread-local table needs neither allocation nor image teardown. */
static int has_spill_marker(const char *plan, uint32_t length) {
    static const char marker[] = "requiredSpillLanes";
    const uint32_t size = sizeof(marker) - 1;
    uintptr_t key = (uintptr_t)plan;
    uint32_t bucket = (uint32_t)((key >> 4) ^ (key >> 12)) & 255;
    uint32_t hint = probe_state.spill_marker_offsets[bucket];
    if (hint && length >= size && hint - 1 <= length - size &&
        !memcmp(plan + hint - 1, marker, size)) return 1;
    const char *found = strstr(plan, marker);
    if (!found) return 0;
    probe_state.spill_marker_offsets[bucket] = (uint32_t)(found - plan) + 1;
    return 1;
}
#ifdef NEOCLR_NATIVE_GC
/* Check current descriptor bytes without reading slot values: once at owned
 * entry, or at every publication for mutable storage/transients. Accumulate
 * failures so the compiler can vectorize the scan. */
static void validate_storage(const neoclr_probe_storage *storage, uint32_t count,
                             int check_flags) {
    uint32_t invalid = 0;
    for (uint32_t i = 0; i < count; i++) {
        const neoclr_probe_storage *s = &storage[i];
        invalid |= (s->address == NULL) |
            ((s->read_bytes != 4) & (s->read_bytes != 8)) |
            (check_flags ? (s->flags > 2) : 0);
    }
    if (invalid) abort();
}
void neoclr_gc_enter_owned_v1(neoclr_probe_frame *frame, const void *context, uint32_t function,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (!storage) abort();
    validate_storage(storage, count, 1);
    neoclr_probe_enter_v3(frame, context, function, storage, count, plan, length);
    frame->reserved = NEOCLR_GC_OWNED_STORAGE;
}
#endif
static void stack_roots(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t lane_count, const char *plan, uint32_t length, uint32_t map_flags) {
    if (!frame || probe_state.head != frame || instruction >= 8192 || lane_count > 8192 || !lanes || !plan) abort();
    if (map_flags) {
        if (length != sizeof(neoclr_gc_root_map)) abort();
        neoclr_gc_root_map map;
        memcpy(&map, plan, sizeof(map)); /* Accept unaligned read-only image data. */
        if (map.magic != NEOCLR_GC_ROOT_MAP_MAGIC || map.lane_count != lane_count) abort();
    } else if (strlen(plan) != length || !has_spill_marker(plan, length)) abort();
    frame->reserved = (frame->reserved & ~(NEOCLR_GC_OWNED_TRANSIENT | NEOCLR_GC_TYPED_PLAN)) | map_flags;
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
#else
    const void *previous_context = NULL;
#endif
    for (const neoclr_probe_frame *f = probe_state.head; f; f = f->previous) {
#ifdef NEOCLR_NATIVE_GC
        /* No guest code or collection runs during this scan. Consecutive frames
         * may share a context; validate it once within this publication only.
         * Always reject NULL, including a corrupted first frame. */
        if (!f->context || f->context != previous_context) observe_fault(f->context);
        previous_context = f->context;
        if (!(f->reserved & NEOCLR_GC_OWNED_STORAGE))
            validate_storage(f->storage, f->storage_count, 1);
        if (!(f->reserved & NEOCLR_GC_OWNED_TRANSIENT))
            validate_storage(f->transient, f->transient_count, 0);
#else
        observe_fault(f->context);
        for (uint32_t i = 0; i < f->storage_count; i++) {
            const neoclr_probe_storage *s = &f->storage[i];
            if (!s->address || (s->read_bytes != 4 && s->read_bytes != 8) || s->flags > 2) abort();
            uint64_t word = 0;
            memcpy(&word, s->address, s->read_bytes);
            value ^= word; /* Observe the slot only; never follow a borrowed pointee. */
        }
        for (uint32_t i = 0; i < f->transient_count; i++) {
            const neoclr_probe_storage *s = &f->transient[i];
            if (!s->address || (s->read_bytes != 4 && s->read_bytes != 8)) abort();
            uint64_t word = 0; memcpy(&word, s->address, s->read_bytes); value ^= word;
        }
        for (uint32_t i = 0; i < f->lane_count; i++) value ^= f->lanes[i];
#endif
    }
#ifndef NEOCLR_NATIVE_GC
    (void)value;
#endif
    probe_state.calls++;
#if defined(NEOCLR_NATIVE_GC) && defined(NEOCLR_GC_SITE_PROFILE)
    neoclr_gc_site_profile_v1(frame->function, instruction);
#endif
#ifdef NEOCLR_NATIVE_GC
#ifdef NEOCLR_WORKER_GC
    if (neoclr_gc_worker_boundary_hook) {
        uint32_t worker_state = frame->reserved & ~(NEOCLR_GC_OWNED_STORAGE | NEOCLR_GC_TYPED_PLAN);
        if (worker_state < 0x100 || worker_state > 0x101) abort();
        neoclr_gc_worker_boundary_hook((neoclr_aot_context *)frame->context, probe_state.head, frame->reserved & 1);
        return;
    }
#endif
    /* Complete snapshot: audited allocating helpers additionally protect their temporaries. */
#ifdef NEOCLR_GC_PACED
#ifdef NEOCLR_GC_SITE_PROFILE
    /* The external diagnostic sink intervened after the head check. Retain the
     * checked handoff in instrumented builds rather than extending that proof. */
    if (neoclr_gc_boundary_v1((neoclr_aot_context *)frame->context, probe_state.head)) abort();
#else
    if (neoclr_gc_published_boundary_v1((neoclr_aot_context *)frame->context, probe_state.head)) abort();
#endif
#else
    if (neoclr_gc_collect_v1((neoclr_aot_context *)frame->context, probe_state.head)) abort();
#endif
#endif
}
void neoclr_probe_stack_roots_v2(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t count, const char *plan, uint32_t length) {
    stack_roots(frame, instruction, lanes, count, plan, length, 0);
}
#ifdef NEOCLR_NATIVE_GC
void neoclr_gc_stack_map_v1(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t count, const void *map, uint32_t length) {
    stack_roots(frame, instruction, lanes, count, map, length, NEOCLR_GC_TYPED_PLAN);
}
#endif
#ifdef NEOCLR_WORKER_GC
_Thread_local neoclr_gc_worker_boundary neoclr_gc_worker_boundary_hook;
void neoclr_gc_worker_stack_roots_v1(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t count, const char *plan, uint32_t length, uint32_t exclusive) {
    if (!frame || exclusive > 1) abort();
    frame->reserved = (frame->reserved & NEOCLR_GC_OWNED_STORAGE) | 0x100 | exclusive;
    neoclr_probe_stack_roots_v2(frame, instruction, lanes, count, plan, length);
}
void neoclr_gc_worker_stack_map_v1(neoclr_probe_frame *frame, uint32_t instruction,
    const uint64_t *lanes, uint32_t count, const void *map, uint32_t length, uint32_t exclusive) {
    if (!frame || exclusive > 1) abort();
    frame->reserved = (frame->reserved & NEOCLR_GC_OWNED_STORAGE) | 0x100 | exclusive;
    neoclr_gc_stack_map_v1(frame, instruction, lanes, count, map, length);
}
#endif
void neoclr_probe_transient_v2(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    if (probe_state.head != frame || (phase != 1 && phase != 2 && phase != 3) || !storage || !plan || strlen(plan) != length) abort();
    frame->reserved &= ~NEOCLR_GC_OWNED_TRANSIENT;
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
#ifdef NEOCLR_NATIVE_GC
void neoclr_gc_transient_owned_v1(neoclr_probe_frame *frame, uint32_t phase,
    const neoclr_probe_storage *storage, uint32_t count, const char *plan, uint32_t length) {
    neoclr_probe_transient_v2(frame, phase, storage, count, plan, length);
    frame->reserved |= NEOCLR_GC_OWNED_TRANSIENT;
}
#endif
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return probe_state.head; }
uint64_t neoclr_root_probe_count_v1(void) { return probe_state.calls; }
uint32_t neoclr_root_probe_depth_v1(void) { return probe_state.depth; }

static _Thread_local neoclr_parked_roots *parked;
void neoclr_roots_park_v1(neoclr_parked_roots *record) {
    if (!record || record->head || record->next || record->owner_context ||
        record->owner || record->expired) abort();
    for (neoclr_parked_roots *p = parked; p; p = p->next) if (p == record) abort();
    record->head = probe_state.head;
    record->next = parked;
    parked = record;
    probe_state.head = NULL;
    probe_state.depth = 0;
}
void neoclr_roots_resume_v1(neoclr_parked_roots *record) {
    if (!record || probe_state.head || record->expired) abort();
    neoclr_parked_roots **p = &parked;
    while (*p && *p != record) p = &(*p)->next;
    if (!*p) abort();
    *p = record->next;
    probe_state.head = record->head;
    memset(record, 0, sizeof(*record));
    probe_state.depth = 0;
    for (const neoclr_probe_frame *f = probe_state.head; f; f = f->previous) probe_state.depth++;
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
    for (const neoclr_probe_frame *f = probe_state.head; f; f = f->previous) {
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
    int32_t status = !probe_state.head || probe_state.head->context == context
        ? neoclr_gc_allocate_rooted_v1(context, probe_state.head, 8 + length, NEOCLR_GC_TEXT, &allocation)
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
