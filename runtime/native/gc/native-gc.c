#include "native-gc.h"
#include <string.h>
#include <stdatomic.h>
#ifdef NEOCLR_GC_PROFILE
#include <stdlib.h>
#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#else
#include <time.h>
#endif
#endif
static _Thread_local neoclr_gc_profile profile;
neoclr_gc_profile neoclr_gc_profile_v1(void) { return profile; }
void neoclr_gc_profile_reset_v1(void) {
#ifdef NEOCLR_GC_PROFILE
    profile = (neoclr_gc_profile){0};
#endif
}
/* Allocation descriptors live in the same bounded buffer as payloads. Free
 * blocks coalesce; payload addresses never move. Object words are conservative
 * candidates; text/byte payloads are atomic, String arrays have explicit slots. */
typedef struct { uint64_t span, bytes, kind, state; } block;
_Static_assert(sizeof(block) == 32, "private allocation descriptor");
enum { ALLOCATED = 1, MARKED = 2, SCANNED = 4 };
static _Thread_local neoclr_gc_statistics statistics;
/* One exclusive runtime-owned arena per native thread. No per-object metadata
 * or allocation; an address-ordered free list is maintained by allocation and sweep.
 * Free blocks reuse their otherwise-unused bytes word for the next offset + 1;
 * zero terminates the list. Live object descriptors and payloads are unchanged. */
enum { OWNED_BASE_CACHE_SIZE = 64 };
static _Thread_local struct {
    neoclr_aot_text_arena *arena;
    unsigned char *data;
    uint64_t capacity, used, free_head;
    const void *bases[OWNED_BASE_CACHE_SIZE];
} owned_heap;
/* Positive entries only: insertion requires a newly allocated payload or an
 * exact-base proof from the checked walk. Collisions only cause misses. */
static unsigned base_slot(const void *value) {
    uintptr_t word = (uintptr_t)value >> 3;
    return (unsigned)((word ^ (word >> 6)) & (OWNED_BASE_CACHE_SIZE - 1));
}
static void remember_base(const void *value) {
    if (value) owned_heap.bases[base_slot(value)] = value;
}
static int known_base(const void *value) {
    return !value || owned_heap.bases[base_slot(value)] == value;
}
static int valid(const neoclr_aot_text_arena *a) {
    return a && (a != owned_heap.arena ||
        (a->data == owned_heap.data && a->capacity == owned_heap.capacity && a->used == owned_heap.used)) &&
        a->used <= a->capacity && !(a->used & 7) &&
        (!a->capacity || a->data) && !((uintptr_t)a->data & 7);
}
static int block_valid(const neoclr_aot_text_arena *a, uint64_t at) {
    if (a->used - at < sizeof(block)) return 0;
    const block *b = (const void *)(a->data + at);
    return b->span >= sizeof(block) + 8 && !(b->span & 7) && b->span <= a->used - at &&
        b->state <= 7 && (!b->state || ((b->state & ALLOCATED) &&
            b->bytes && b->bytes <= b->span - sizeof(block) &&
            b->kind >= NEOCLR_GC_TEXT && b->kind <= NEOCLR_GC_INTERN));
}
static int blocks_valid(const neoclr_aot_text_arena *a) {
    if (!valid(a)) return 0;
    uint64_t next_free = a == owned_heap.arena ? owned_heap.free_head : 0;
    for (uint64_t at = 0; at < a->used;) {
        if (!block_valid(a, at)) return 0;
        const block *b = (const void *)(a->data + at);
        if (a == owned_heap.arena && !b->state) {
            if (next_free != at + 1) return 0;
            next_free = b->bytes;
        }
        at += b->span;
    }
    return !next_free;
}
int32_t neoclr_gc_heap_acquire_v1(neoclr_aot_text_arena *arena) {
    if (owned_heap.arena || !valid(arena) || arena->used) return 3;
    owned_heap.arena = arena;
    owned_heap.data = arena->data;
    owned_heap.capacity = arena->capacity;
    owned_heap.used = owned_heap.free_head = 0;
    return 0;
}
int32_t neoclr_gc_heap_release_v1(neoclr_aot_text_arena *arena) {
    if (!arena || owned_heap.arena != arena) return 3;
    int32_t status = blocks_valid(arena) ? 0 : 3;
    /* Always relinquish on the owning thread, including after host corruption.
     * Failure never repairs or mutates the caller's arena. */
    memset(&owned_heap, 0, sizeof(owned_heap));
    return status;
}
typedef struct host_root {
    neoclr_aot_context *context;
    void *value, *owner;
    int conditional, expired;
    uint64_t handle;
    struct host_root *previous, *next;
} host_root;
static _Thread_local host_root host_roots[NEOCLR_GC_HOST_ROOT_LIMIT];
static _Thread_local host_root *host_head;
static _Atomic uint64_t next_host_handle = 1;
/* Validate every span while finding exact bases. Never publish a result or
 * stop validation early: a matching prefix cannot hide a corrupt trailing block.
 * Null values retain their existing admitted-root meaning. */
static int validated_bases(const neoclr_aot_text_arena *arena, const void *value, const void *owner) {
    if (!valid(arena)) return 0;
    int owned = arena == owned_heap.arena;
    if (owned && known_base(value) && known_base(owner)) return 1;
    int found_value = !value, found_owner = !owner;
    uint64_t at = 0;
    while (at < arena->used && !(found_value && found_owner)) {
        if (!block_valid(arena, at)) return 0;
        const block *b = (const void *)(arena->data + at);
        if (b->state & ALLOCATED) {
            found_value |= value == (const void *)(b + 1);
            found_owner |= owner == (const void *)(b + 1);
        }
        at += b->span;
    }
    /* Once both bases match, the remaining walk only needs integrity checks. */
    for (; at < arena->used;) {
        if (!block_valid(arena, at)) return 0;
        const block *b = (const void *)(arena->data + at);
        at += b->span;
    }
    if (owned && found_value && found_owner) {
        remember_base(value);
        remember_base(owner);
    }
    return found_value && found_owner;
}
int32_t neoclr_gc_allocation_info_v1(neoclr_aot_context *context, const void *value,
    uint64_t *bytes, uint32_t *kind) {
    if (!context || !value || !bytes || !kind || !validated_bases(&context->text, value, NULL)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    const block *b = (const block *)value - 1;
    *bytes = b->bytes;
    *kind = (uint32_t)b->kind;
    return 0;
}
static _Thread_local neoclr_gc_resource *resource_head;
static _Thread_local int resource_cleanup_active;
static void resource_cleanup(neoclr_gc_resource **link) {
    neoclr_gc_resource *resource = *link;
    void *state = resource->state;
    void (*cleanup)(void *) = resource->cleanup;
    *link = resource->next;
    *resource = (neoclr_gc_resource){0};
    resource_cleanup_active = 1;
    cleanup(state);
    resource_cleanup_active = 0;
}
int32_t neoclr_gc_resource_attach_v1(neoclr_aot_context *context, void *owner,
    neoclr_gc_resource *resource, void *state, void (*cleanup)(void *)) {
    if (resource_cleanup_active || !context || !owner || !resource || !cleanup ||
        !validated_bases(&context->text, owner, NULL)) return 3;
    unsigned count = 0;
    for (neoclr_gc_resource *r = resource_head; r; r = r->next) {
        if (r == resource) return 3;
        count += r->context == context;
    }
    if (resource->context || resource->owner || resource->state || resource->cleanup || resource->next)
        return 3;
    if (count >= 1024) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    *resource = (neoclr_gc_resource){context, owner, state, cleanup, resource_head};
    resource_head = resource;
    return 0;
}
int32_t neoclr_gc_resource_release_v1(neoclr_aot_context *context, neoclr_gc_resource *resource) {
    if (resource_cleanup_active || !context || !resource) return 3;
    for (neoclr_gc_resource **link = &resource_head; *link; link = &(*link)->next)
        if (*link == resource && (*link)->context == context) {
            resource_cleanup(link);
            return 0;
        }
    return 3;
}
static host_root *find_host_root(neoclr_aot_context *context, uint64_t handle) {
    if (!context || !handle) return NULL;
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context && root->handle == handle) return root;
    return NULL;
}
static int32_t host_root_create(neoclr_aot_context *context, void *owner, void *value,
    int conditional, uint64_t *output) {
    if (!context || !output || (conditional && !owner) ||
        !validated_bases(&context->text, value, conditional ? owner : NULL))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    host_root *available = NULL;
    for (uint32_t i = 0; i < NEOCLR_GC_HOST_ROOT_LIMIT; i++)
        if (!host_roots[i].context) { available = &host_roots[i]; break; }
    if (!available) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    uint64_t handle = atomic_load_explicit(&next_host_handle, memory_order_relaxed);
    do {
        if (handle == UINT64_MAX) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    } while (!atomic_compare_exchange_weak_explicit(&next_host_handle, &handle, handle + 1,
                memory_order_relaxed, memory_order_relaxed));
    *available = (host_root){.context = context, .value = value, .owner = owner,
        .conditional = conditional, .handle = handle, .next = host_head};
    if (host_head) host_head->previous = available;
    host_head = available;
    *output = handle;
    return 0;
}
int32_t neoclr_gc_host_root_create_v1(neoclr_aot_context *context, void *value, uint64_t *output) {
    return host_root_create(context, NULL, value, 0, output);
}
int32_t neoclr_gc_host_root_create_owned_v1(neoclr_aot_context *context, void *owner,
    void *value, uint64_t *output) {
    return host_root_create(context, owner, value, 1, output);
}
int32_t neoclr_gc_host_root_replace_v1(neoclr_aot_context *context, uint64_t handle, void *value) {
    host_root *root = find_host_root(context, handle);
    if (!root || root->expired || !validated_bases(&context->text, value, NULL)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    root->value = value;
    return 0;
}
int32_t neoclr_gc_host_root_read_v1(neoclr_aot_context *context, uint64_t handle, void **output) {
    host_root *root = find_host_root(context, handle);
    if (!root || root->expired || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    *output = root->value;
    return 0;
}
int32_t neoclr_gc_host_root_release_v1(neoclr_aot_context *context, uint64_t handle) {
    host_root *root = find_host_root(context, handle);
    if (!root) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (root->previous) root->previous->next = root->next;
    else host_head = root->next;
    if (root->next) root->next->previous = root->previous;
    *root = (host_root){0};
    return 0;
}
static int32_t callback_read(neoclr_aot_context *context, uint64_t handle, void **output) {
    host_root *root = find_host_root(context, handle);
    if (!root || root->conditional || !output || !root->value || context->fault.code ||
        !validated_bases(&context->text, root->value, NULL)) return 3;
    const block *allocation = (const block *)root->value - 1;
    uint64_t kind;
    if (allocation->kind != NEOCLR_GC_OBJECT || allocation->bytes != 24) return 3;
    memcpy(&kind, root->value, 8);
    if (kind != UINT32_MAX) return 3;
    *output = root->value;
    return 0;
}
int32_t neoclr_gc_callback_read_v1(neoclr_aot_context *context, uint64_t handle, void **output) {
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context == context) return 3;
    return callback_read(context, handle, output);
}
int32_t neoclr_gc_callback_read_suspended_v1(neoclr_aot_context *context, uint64_t handle,
    void **output, const neoclr_probe_frame *boundary) {
    if (!boundary || neoclr_root_probe_head_v1() != boundary || boundary->context != context) return 3;
    return callback_read(context, handle, output);
}
int32_t neoclr_gc_roots_park_owned_v1(neoclr_aot_context *context, void *owner,
    neoclr_parked_roots *record) {
    if (!context || !owner || !record || !validated_bases(&context->text, owner, NULL) || record->head || record->next ||
        record->owner_context || record->owner || record->expired) return 3;
    for (const neoclr_parked_roots *p = neoclr_roots_parked_v1(); p; p = p->next)
        if (p == record) return 3;
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context != context) return 3;
    neoclr_roots_park_v1(record);
    record->owner_context = context;
    record->owner = owner;
    return 0;
}
int32_t neoclr_gc_complete_invocation_v1(neoclr_aot_context *context) {
    if (!context || !blocks_valid(&context->text)) return 3;
    for (const neoclr_probe_frame *frame = neoclr_root_probe_head_v1(); frame; frame = frame->previous)
        if (frame->context == context) return 3;
    /* Removing pool ownership never frees an exposed String. Any host/managed
     * roots still retain the original value normally until their own release. */
    for (uint64_t at = 0; at < context->text.used;) {
        block *b = (void *)(context->text.data + at);
        if ((b->state & ALLOCATED) && b->kind == NEOCLR_GC_INTERN)
            b->kind = NEOCLR_GC_BYTES;
        at += b->span;
    }
    context->static_storage = NULL;
    return 0;
}

int32_t neoclr_gc_entry_check_v1(neoclr_aot_context *context) {
    for (neoclr_gc_resource *r = resource_head; r; r = r->next)
        if (r->context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (!context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (const neoclr_parked_roots *p = neoclr_roots_parked_v1(); p; p = p->next) {
        if (p->owner_context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
        for (const neoclr_probe_frame *f = p->head; f; f = f->previous)
            if (f->context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    }

    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (const neoclr_probe_frame *frame = neoclr_root_probe_head_v1(); frame; frame = frame->previous)
        if (frame->context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return 0;
}
int32_t neoclr_gc_owned_roots_check_v1(neoclr_aot_context *context, const uint64_t *handles, uint32_t count) {
    for (neoclr_gc_resource *r = resource_head; r; r = r->next)
        if (r->context == context) return 3;
    if (!context || count > NEOCLR_GC_HOST_ROOT_LIMIT || (count && !handles)) return 3;
    for (const neoclr_parked_roots *p = neoclr_roots_parked_v1(); p; p = p->next) {
        if (p->owner_context == context) return 3;
        for (const neoclr_probe_frame *f = p->head; f; f = f->previous)
            if (f->context == context) return 3;
    }
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context == context) return 3;
    for (uint32_t i = 0; i < count; i++) {
        if (!find_host_root(context, handles[i])) return 3;
        for (uint32_t j = 0; j < i; j++) if (handles[j] == handles[i]) return 3;
    }
    uint32_t actual = 0;
    for (host_root *r = host_head; r; r = r->next) actual += r->context == context;
    return actual == count ? 0 : 3;
}
int32_t neoclr_gc_allocate_v1(neoclr_aot_text_arena *a, uint64_t bytes,
                             uint32_t kind, void **output) {
    if (!output || !valid(a) || !bytes || kind < NEOCLR_GC_TEXT || kind > NEOCLR_GC_INTERN)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    int oversized = bytes > UINT64_MAX - sizeof(block) - 7;
    uint64_t needed = oversized ? UINT64_MAX : sizeof(block) + ((bytes + 7) & ~UINT64_C(7));
    block *chosen = NULL;
    int owned = a == owned_heap.arena;
    uint64_t *free_link = &owned_heap.free_head;
    /* The owned list preserves first-fit address order while skipping live blocks.
     * Validate visited descriptors/links before mutation; collection and release
     * reconcile the complete list against the authoritative block walk. */
    if (owned) {
        while (*free_link) {
            uint64_t at = *free_link - 1;
            if (at >= a->used || (at & 7) || !block_valid(a, at)) return 3;
            block *b = (void *)(a->data + at);
            uint64_t next = b->bytes;
            if (b->state || (next && ((next - 1) < at + b->span ||
                (next - 1) >= a->used || ((next - 1) & 7)))) return 3;
            if (b->span >= needed) { chosen = b; break; }
            free_link = &b->bytes;
        }
    } else {
        /* External arenas still validate every descriptor, even after a fit. */
        for (uint64_t at = 0; at < a->used;) {
            if (!block_valid(a, at)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
            block *b = (void *)(a->data + at);
            if (!chosen && !b->state && b->span >= needed) chosen = b;
            at += b->span;
        }
    }
    if (oversized) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    if (chosen) {
        uint64_t next = owned ? chosen->bytes : 0;
        if (chosen->span - needed >= sizeof(block) + 8) {
            block *rest = (void *)((unsigned char *)chosen + needed);
            *rest = (block){chosen->span - needed, next, 0, 0};
            chosen->span = needed;
            if (owned) *free_link = (unsigned char *)rest - a->data + 1;
        } else if (owned) *free_link = next;
    } else {
        if (needed > a->capacity - a->used) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
        chosen = (void *)(a->data + a->used);
        chosen->span = needed;
        a->used += needed;
    }
    if (owned) {
        owned_heap.used = a->used;
    }
    chosen->bytes = bytes; chosen->kind = kind; chosen->state = ALLOCATED;
    memset(chosen + 1, 0, (size_t)(chosen->span - sizeof(block)));
    if (owned) remember_base(chosen + 1);
    *output = chosen + 1;
    statistics.allocations++;
#ifdef NEOCLR_GC_PROFILE
    profile.allocated_bytes += bytes;
    if (a->used > profile.peak_arena_used) profile.peak_arena_used = a->used;
#endif
    return 0;
}
/* Opt-in admission for callers that have published every live reference.
 * Keep the arena-only allocator non-collecting: native helpers may retain
 * unregistered intermediates between its calls. Never retry a whole operation. */
int32_t neoclr_gc_allocate_rooted_values_v1(neoclr_aot_context *context,
    const neoclr_probe_frame *boundary, const void *const *values, uint32_t count,
    uint64_t bytes, uint32_t kind, void **output) {
    if (count > 16 || (count && !values) || !context || resource_cleanup_active || boundary != neoclr_root_probe_head_v1() ||
        (boundary && boundary->context != context)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    int32_t status = neoclr_gc_allocate_v1(&context->text, bytes, kind, output);
    if (status != NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT) return status;
    /* An impossible single block cannot benefit from collection. Arithmetic
     * mirrors the allocator, including descriptor and alignment overhead. */
    if (bytes > UINT64_MAX - sizeof(block) - 7 ||
        sizeof(block) + ((bytes + 7) & ~UINT64_C(7)) > context->text.capacity) return status;
    /* Private helper roots are visible only to this collection, not logical stack
     * tracing. Values include every borrowed input and intermediate needed after
     * allocation; their slots remain live on this synchronous C call's stack. */
    neoclr_probe_storage slots[16];
    for (uint32_t i = 0; i < count; i++) slots[i] = (neoclr_probe_storage){&values[i], 8, 0};
    neoclr_probe_frame helper = {.previous = (neoclr_probe_frame *)boundary,
        .context = context, .storage = slots, .storage_count = count};
    status = neoclr_gc_collect_v1(context, count ? &helper : boundary);
    if (status) return status;
    return neoclr_gc_allocate_v1(&context->text, bytes, kind, output);
}

int32_t neoclr_gc_allocate_rooted_v1(neoclr_aot_context *context, const neoclr_probe_frame *boundary,
    uint64_t bytes, uint32_t kind, void **output) {
    return neoclr_gc_allocate_rooted_values_v1(context, boundary, NULL, 0, bytes, kind, output);
}

/* Sparse byte-range index rebuilt from validated spans for each collection.
 * Small heaps use 2 KiB; dense heaps temporarily use another 32 KiB.
 * No allocation, retained addresses or heap ABI change.
 * A bucket points to the block containing its first byte, including free spans.
 * Interior/tagged candidates still require the exact allocated payload bounds. */
enum { INTERN_SEED_LIMIT = 32 };
typedef struct {
    block **start, **interns;
    unsigned capacity, shift, intern_count;
    uint64_t blocks, atomic_marks;
    int reset_required;
} block_index;
/* Record candidates without mutating heap state. Overflow retains the original
 * full seed walk; these pointers live only through this synchronous collection. */
static void index_seed(block_index *index, block *b) {
    index->reset_required |= b->state > ALLOCATED;
    if (b->state && b->kind == NEOCLR_GC_INTERN && index->intern_count <= INTERN_SEED_LIMIT) {
        if (index->intern_count < INTERN_SEED_LIMIT)
            index->interns[index->intern_count] = b;
        index->intern_count++;
    }
}
static int build_index(neoclr_aot_text_arena *a, block_index *index) {
    if (!valid(a)) return 0;
    uint64_t blocks = 0;
    index->shift = 6;
    while (a->used && ((a->used - 1) >> index->shift) >= index->capacity) index->shift++;
    unsigned bucket = 0;
    uint64_t next_free = a == owned_heap.arena ? owned_heap.free_head : 0;
    for (uint64_t at = 0; at < a->used;) {
        if (!block_valid(a, at)) return 0;
        block *b = (void *)(a->data + at);
        if (a == owned_heap.arena && !b->state) {
            if (next_free != at + 1) return 0;
            next_free = b->bytes;
        }
        blocks++;
        index_seed(index, b);
        uint64_t end = at + b->span;
        while (bucket < index->capacity && ((uint64_t)bucket << index->shift) < end)
            index->start[bucket++] = b;
        at = end;
    }
    index->blocks = blocks;
    return !next_free;
}
/* Index lookup preserves exact payload-base validation for host owners/values.
 * Only use after blocks_valid and before sweep changes block spans. */
static block *indexed_block(neoclr_aot_text_arena *a, const block_index *index, uint64_t word) {
    uintptr_t base = (uintptr_t)a->data;
    if (word < base || word - base >= a->used) return NULL;
    uint64_t offset = word - base;
    block *first = index->start[offset >> index->shift];
    for (uint64_t at = (uintptr_t)first - base; at < a->used;) {
        block *b = (void *)(a->data + at);
        if (offset < at + b->span) return b;
        at += b->span;
    }
    return NULL;
}
static int indexed_live_base(neoclr_aot_text_arena *a, const block_index *index, const void *value) {
    if (!value) return 1;
    block *b = indexed_block(a, index, (uintptr_t)value);
    return b && (b->state & ALLOCATED) && value == (void *)(b + 1);
}
static void mark(neoclr_aot_text_arena *a, block_index *index, uint64_t word, block **pending) {
    block *b = indexed_block(a, index, word);
    if (b && b->state && word >= (uintptr_t)(b + 1) &&
        word - (uintptr_t)(b + 1) < b->bytes && !(b->state & MARKED)) {
        /* Atomic payloads have no children: retain them without worklist traffic.
         * Count newly marked leaves so dependent/parked owners discovered late
         * still drive the fixed point even when no tracing work was enqueued. */
        if (b->kind == NEOCLR_GC_TEXT || b->kind == NEOCLR_GC_BYTES) {
            b->state = ALLOCATED | MARKED | SCANNED;
            index->atomic_marks++;
        } else {
            b->state = (uint64_t)(uintptr_t)*pending | ALLOCATED | MARKED;
            *pending = b;
        }
    }
}
static void mark_slots(neoclr_aot_text_arena *a, block_index *index, const neoclr_probe_storage *slots, uint32_t count, block **pending) {
    for (uint32_t i = 0; i < count; i++) {
        if (slots[i].read_bytes != 8) continue; /* 32-bit discriminators are not pointers. */
        uint64_t word; memcpy(&word, slots[i].address, 8); mark(a, index, word, pending);
    }
}
static void mark_frames(neoclr_aot_context *context, block_index *index,
    const neoclr_probe_frame *head, block **pending) {
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        if (f->context != context) continue;
        mark_slots(&context->text, index, f->storage, f->storage_count, pending);
        mark_slots(&context->text, index, f->transient, f->transient_count, pending);
        for (uint32_t i = 0; i < f->lane_count; i++)
            mark(&context->text, index, f->lanes[i], pending);
    }
}
#ifdef NEOCLR_WORKER_GC
_Thread_local neoclr_gc_worker_collector neoclr_gc_worker_collect_hook;
static _Thread_local const neoclr_gc_participant *worker_participants;
static _Thread_local unsigned worker_participant_count;
int32_t neoclr_gc_collect_participants_v1(neoclr_aot_context *context, const neoclr_probe_frame *head,
    const neoclr_gc_participant *participants, unsigned count) {
    if (worker_participants || !participants || count > 17) return 3;
    worker_participants = participants; worker_participant_count = count;
    int32_t status = neoclr_gc_collect_conditional_v1(context, head, NULL, 0);
    worker_participants = NULL; worker_participant_count = 0;
    return status;
}
int32_t neoclr_gc_worker_roots_check_v1(neoclr_aot_context *context, uint64_t handle) {
    unsigned found = 0;
    if (neoclr_roots_parked_v1()) return 3;
    for (neoclr_gc_resource *r = resource_head; r; r = r->next) if (r->context == context) return 3;
    for (host_root *r = host_head; r; r = r->next) if (r->context == context) {
        if (r->handle != handle || r->conditional) return 3;
        found++;
    }
    return found == 1 ? 0 : 3;
}
#endif
int32_t neoclr_gc_collect_v1(neoclr_aot_context *context, const neoclr_probe_frame *head) {
#ifdef NEOCLR_WORKER_GC
    if (neoclr_gc_worker_collect_hook) return neoclr_gc_worker_collect_hook(context, head);
#endif
    return neoclr_gc_collect_conditional_v1(context, head, NULL, 0);
}
static int32_t collect_indexed(neoclr_aot_context *context,
    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t conditional_count,
    block_index index) {
    neoclr_aot_text_arena *a = &context->text;
    if (conditional_count > NEOCLR_GC_HOST_ROOT_LIMIT || (conditional_count && !conditional))
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (neoclr_gc_resource *r = resource_head; r; r = r->next)
        if (r->context == context && (!r->owner || !indexed_live_base(a, &index, r->owner)))
            return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (uint32_t i = 0; i < conditional_count; i++)
        if ((!conditional[i].owner && conditional[i].value) ||
            !indexed_live_base(a, &index, conditional[i].owner) || !indexed_live_base(a, &index, conditional[i].value))
            return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context && (!indexed_live_base(a, &index, root->value) ||
            (root->conditional && !root->expired &&
                (!root->owner || !indexed_live_base(a, &index, root->owner))))) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (const neoclr_parked_roots *p = neoclr_roots_parked_v1(); p; p = p->next)
        if (p->owner_context == context && !p->expired &&
            (!p->owner || !indexed_live_base(a, &index, p->owner))) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    neoclr_probe_storage fault[65];
    int32_t count = neoclr_probe_fault_roots_v1(context, fault, 65);
    if (count < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (neoclr_parked_roots *p = (neoclr_parked_roots *)neoclr_roots_parked_v1(); p; p = p->next)
        if (p->owner_context == context) p->traced = 0;
    block *pending = NULL;
    /* Header validation recorded a bounded list of intern seeds. Clean marks
     * need no reset walk. Stale marks or overflow retain the original full walk,
     * where mark() only enqueues the current block until the worklist runs. */
    if (index.reset_required || index.intern_count > INTERN_SEED_LIMIT) {
        for (uint64_t at = 0; at < a->used;) {
            block *b = (void *)(a->data + at);
            b->state &= ALLOCATED;
            if ((b->state & ALLOCATED) && b->kind == NEOCLR_GC_INTERN)
                mark(a, &index, (uintptr_t)(b + 1), &pending);
            at += b->span;
        }
    } else {
        for (unsigned i = 0; i < index.intern_count; i++)
            mark(a, &index, (uintptr_t)(index.interns[i] + 1), &pending);
    }
    mark_slots(a, &index, fault, (uint32_t)count, &pending);
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context && !root->conditional)
            mark(a, &index, (uintptr_t)root->value, &pending);
    mark(a, &index, (uintptr_t)context->static_storage, &pending);
    mark_frames(context, &index, head, &pending);
#ifdef NEOCLR_WORKER_GC
    for (unsigned n = 0; n < worker_participant_count; n++) {
        const neoclr_gc_participant *part = &worker_participants[n];
        neoclr_probe_storage diagnostics[65];
        int32_t slots = neoclr_probe_fault_roots_v1(part->context, diagnostics, 65);
        if (slots < 0) return 3;
        mark_slots(a, &index, diagnostics, (uint32_t)slots, &pending);
        for (const neoclr_probe_frame *f = part->frames; f; f = f->previous) {
            if (f->context != part->context) return 3;
            mark_slots(a, &index, f->storage, f->storage_count, &pending);
            mark_slots(a, &index, f->transient, f->transient_count, &pending);
            for (uint32_t i = 0; i < f->lane_count; i++) mark(a, &index, f->lanes[i], &pending);
        }
    }
#endif

    for (const neoclr_parked_roots *p = neoclr_roots_parked_v1(); p; p = p->next)
        if (!p->owner_context) mark_frames(context, &index, p->head, &pending);
    /* Each newly marked reference-bearing allocation enters the worklist once.
     * The sparse index bounds address searches to a bucket plus its crossing block. */
    for (;;) {
        uint64_t atomic_before = index.atomic_marks;
        while (pending) {
            block *b = pending;
            pending = (block *)(uintptr_t)(b->state & ~UINT64_C(7));
            b->state = ALLOCATED | MARKED | SCANNED;
            const unsigned char *data = (const void *)(b + 1);
            if (b->kind == NEOCLR_GC_INTERN) {
                if (b->bytes != 8) goto invalid_descriptor;
                uint64_t owner; memcpy(&owner, data, 8);
                mark(a, &index, owner, &pending);
            } else if (b->kind == NEOCLR_GC_OBJECT) {
                for (uint64_t offset = 8; offset + 8 <= b->bytes; offset += 8) {
                    uint64_t word; memcpy(&word, data + offset, 8); mark(a, &index, word, &pending);
                }
            } else if (b->kind == NEOCLR_GC_RECORDS) {
                uint64_t kind, length, lanes;
                if (b->bytes < 24) goto invalid_descriptor;
                memcpy(&kind, data, 8); memcpy(&length, data + 8, 8); memcpy(&lanes, data + 16, 8);
                if ((kind & UINT64_C(0xffffffff)) != UINT64_C(0x80000005) ||
                    (kind >> 32) > NEOCLR_NATIVE_TYPE_LIMIT || ((kind >> 32) && lanes != 1) || !lanes || lanes > NEOCLR_NATIVE_LAYOUT_LANES ||
                    length > 65536 || length > (b->bytes - 24) / (lanes * 8 + 1)) goto invalid_descriptor;
                uint64_t markers = 24 + length * lanes * 8;
                for (uint64_t i = 0; i < length; i++) {
                    if (!data[markers + i]) continue;
                    for (uint64_t lane = 0; lane < lanes; lane++) {
                        uint64_t word; memcpy(&word, data + 24 + (i * lanes + lane) * 8, 8);
                        mark(a, &index, word, &pending);
                    }
                }
            } else if (b->kind == NEOCLR_GC_STRINGS) {
                uint64_t kind, length;
                if (b->bytes < 16) goto invalid_descriptor;
                memcpy(&kind, data, 8); memcpy(&length, data + 8, 8);
                kind &= UINT64_C(0xffffffff);
                int reserved = kind == UINT64_C(0x80000004);
                if ((!reserved && kind != UINT64_C(0x80000003)) ||
                    length > (b->bytes - 16) / (reserved ? 9 : 8)) goto invalid_descriptor;
                for (uint64_t i = 0; i < length; i++) {
                    if (reserved && !data[16 + 8 * length + i]) continue;
                    uint64_t word; memcpy(&word, data + 16 + 8 * i, 8); mark(a, &index, word, &pending);
                }
            }
        }
        // Values can reveal further owners. Reach a fixed point independent of
        // entry order; an unrooted cycle never supplies its own first mark.
        for (uint32_t i = 0; i < conditional_count; i++) {
            const void *owner = conditional[i].owner;
            if (owner && (((const block *)owner - 1)->state & MARKED))
                mark(a, &index, (uintptr_t)conditional[i].value, &pending);
        }
        for (host_root *root = host_head; root; root = root->next)
            if (root->context == context && root->conditional && !root->expired &&
                (((const block *)root->owner - 1)->state & MARKED))
                mark(a, &index, (uintptr_t)root->value, &pending);
        /* Parked frame contents are immutable during collection. Once their
         * owner is marked, scan this registration only once; newly discovered
         * owners still activate other registrations on subsequent passes. */
        for (neoclr_parked_roots *p = (neoclr_parked_roots *)neoclr_roots_parked_v1(); p; p = p->next)
            if (p->owner_context == context && !p->expired && !p->traced &&
                (((const block *)p->owner - 1)->state & MARKED)) {
                p->traced = 1;
                mark_frames(context, &index, p->head, &pending);
            }
        if (!pending && atomic_before == index.atomic_marks) break;
    }
    for (neoclr_gc_resource **link = &resource_head; *link;) {
        neoclr_gc_resource *r = *link;
        if (r->context == context && !(((const block *)r->owner - 1)->state & MARKED))
            resource_cleanup(link);
        else
            link = &r->next;
    }
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context && root->conditional && !root->expired &&
            !(((const block *)root->owner - 1)->state & MARKED)) {
            root->owner = root->value = NULL;
            root->expired = 1;
        }
    for (neoclr_parked_roots *p = (neoclr_parked_roots *)neoclr_roots_parked_v1(); p; p = p->next)
        if (p->owner_context == context && !p->expired &&
            !(((const block *)p->owner - 1)->state & MARKED)) {
            p->head = NULL;
            p->owner = NULL;
            p->expired = 1;
        }
    for (uint32_t i = 0; i < conditional_count; i++) {
        void *owner = conditional[i].owner;
        conditional[i].retained = owner && (((const block *)owner - 1)->state & MARKED);
        if (!conditional[i].retained)
            conditional[i].owner = conditional[i].value = NULL;
    }
#ifdef NEOCLR_GC_PROFILE
    uint64_t live_bytes = 0, live_blocks = 0;
#endif
    /* Sweep and coalesce in one forward walk. Preserve the original span before
     * extending a preceding free run; no later lookup uses the marking index. */
    block *free_run = NULL, *last_free = NULL;
    uint64_t free_at = 0, free_head = 0;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        uint64_t span = b->span;
        if (b->state && !(b->state & MARKED)) {
            statistics.reclaimed_allocations++; statistics.reclaimed_bytes += b->bytes;
            /* Poison reclaimed contents so stale reads fail functional stress tests. */
            memset(b + 1, 0xdd, (size_t)(span - sizeof(block)));
            b->state = 0; b->bytes = 0; b->kind = 0;
        } else b->state &= ALLOCATED;
#ifdef NEOCLR_GC_PROFILE
        if (b->state) { live_bytes += b->bytes; live_blocks++; }
#endif
        if (b->state) {
            if (free_run && a == owned_heap.arena) {
                if (last_free) last_free->bytes = free_at + 1;
                else free_head = free_at + 1;
                last_free = free_run;
            }
            free_run = NULL;
        } else if (free_run) {
            free_run->span += span;
        } else {
            free_run = b;
            free_at = at;
            if (a == owned_heap.arena) b->bytes = 0;
        }
        at += span;
    }
    if (free_run) a->used = free_at;
    if (a == owned_heap.arena) {
        memset(owned_heap.bases, 0, sizeof(owned_heap.bases));
        owned_heap.used = a->used;
        owned_heap.free_head = free_head;
    }
    statistics.collections++;
#ifdef NEOCLR_GC_PROFILE
    profile.live_bytes = live_bytes; profile.live_blocks = live_blocks;
    if (live_bytes > profile.peak_live_bytes) profile.peak_live_bytes = live_bytes;
#endif
    return 0;
invalid_descriptor:
    /* Do not leave private queue links in public quiescent heap state on failure.
     * Nothing has been swept; a repaired descriptor can be collected again. */
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at); b->state &= ALLOCATED; at += b->span;
    }
    return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
}
/* Keep the larger stack reservation off the common small-heap path, including
 * compilers that would otherwise inline the helper and reserve it unconditionally. */
#ifdef _MSC_VER
__declspec(noinline)
#else
__attribute__((noinline))
#endif
static int32_t collect_dense(neoclr_aot_context *context,
    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {
    block *starts[4096], *interns[INTERN_SEED_LIMIT];
    block_index index = {.start = starts, .interns = interns, .capacity = 4096};
    /* The caller has validated every span in this synchronous collection.
     * No guest code, cleanup callback or heap mutation runs between the walks.
     * Rebuild only bucket starts; do not repeat validation or population counting. */
    neoclr_aot_text_arena *a = &context->text;
    index.shift = 6;
    while (a->used && ((a->used - 1) >> index.shift) >= index.capacity) index.shift++;
    unsigned bucket = 0;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        index_seed(&index, b);
        uint64_t end = at + b->span;
        while (bucket < index.capacity && ((uint64_t)bucket << index.shift) < end)
            index.start[bucket++] = b;
        at = end;
    }
    return collect_indexed(context, head, conditional, count, index);
}
static int32_t collect_conditional(neoclr_aot_context *context,
    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {
    if (!context || resource_cleanup_active) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    block *starts[256], *interns[INTERN_SEED_LIMIT];
    block_index index = {.start = starts, .interns = interns, .capacity = 256};
    if (!build_index(&context->text, &index)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    /* Population, not arena size: a few large atomic buffers do not warrant a
     * denser table. Rebuild only before any marks or cleanup state are changed. */
    if (index.blocks > 2048) return collect_dense(context, head, conditional, count);
    return collect_indexed(context, head, conditional, count, index);
}
int32_t neoclr_gc_collect_conditional_v1(neoclr_aot_context *context,
    const neoclr_probe_frame *head, neoclr_gc_conditional_root *conditional, uint32_t count) {
#ifdef NEOCLR_GC_PROFILE
#ifdef _WIN32
    static _Thread_local LARGE_INTEGER frequency;
    LARGE_INTEGER before, after;
    if (!frequency.QuadPart &&
        (!QueryPerformanceFrequency(&frequency) || frequency.QuadPart <= 0)) abort();
    if (!QueryPerformanceCounter(&before)) abort();
#else
    struct timespec before, after;
    if (clock_gettime(CLOCK_MONOTONIC, &before)) abort();
#endif
#endif
    int32_t status = collect_conditional(context, head, conditional, count);
#ifdef NEOCLR_GC_PROFILE
#ifdef _WIN32
    if (!QueryPerformanceCounter(&after) || after.QuadPart < before.QuadPart) abort();
    uint64_t ticks = (uint64_t)(after.QuadPart - before.QuadPart);
    uint64_t hz = (uint64_t)frequency.QuadPart;
    /* Split whole seconds before scaling to avoid overflowing ticks * 1e9. */
    uint64_t ns = (ticks / hz) * UINT64_C(1000000000) +
        (uint64_t)((double)(ticks % hz) * 1000000000.0 / (double)hz);
#else
    if (clock_gettime(CLOCK_MONOTONIC, &after)) abort();
    uint64_t ns = (uint64_t)((int64_t)(after.tv_sec - before.tv_sec) * INT64_C(1000000000)
                           + after.tv_nsec - before.tv_nsec);
#endif
    unsigned bucket = 0;
    for (uint64_t remaining = ns; remaining > 1; remaining >>= 1) bucket++;
    profile.pause_buckets[bucket]++;
    profile.attempts++;
    profile.failures += status != 0;
    profile.nanoseconds += ns;
    if (ns > profile.maximum_nanoseconds) profile.maximum_nanoseconds = ns;
#endif
#ifdef NEOCLR_DIAGNOSTICS
    neoclr_diagnostics_publish_v1();
#endif
    return status;
}
/* Opt-in generated-boundary pacing; private per-owner-thread state, not ABI.
 * Unexpected global collection epochs force a sweep instead of indefinitely
 * postponing cleanup across contexts. Rooted admission handles actual pressure.
 * The bounded 256-allocation / 1024-publication budget amortizes tracing on the
 * plain HTTP path; see benchmarks/http-server/primitive-comparison/pacing.
 * This is a publication budget, not a wall-clock finalization guarantee. */
static _Thread_local struct {
    const neoclr_aot_context *context;
    uint64_t allocations, collections, boundaries;
} pacing;
int32_t neoclr_gc_idle_v1(neoclr_aot_context *context, const neoclr_probe_frame *boundary) {
    int32_t status = neoclr_gc_collect_v1(context, boundary);
    if (!status) {
        pacing.context = context;
        pacing.allocations = statistics.allocations;
        pacing.collections = statistics.collections;
        pacing.boundaries = 0;
    }
    return status;
}
int32_t neoclr_gc_boundary_v1(neoclr_aot_context *context, const neoclr_probe_frame *boundary) {
    if (!context || resource_cleanup_active || boundary != neoclr_root_probe_head_v1() ||
        (boundary && boundary->context != context)) return 3;
    if (pacing.context == context && statistics.collections == pacing.collections &&
        statistics.allocations - pacing.allocations < 256 && ++pacing.boundaries < 1024) return 0;
    return neoclr_gc_idle_v1(context, boundary);
}
neoclr_gc_statistics neoclr_gc_statistics_v1(void) { return statistics; }

static int32_t string_intern(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
                               const neoclr_aot_text **output, int rooted) {
    if (!text || !output || !blocks_valid(arena)) return 3;
    uint64_t count = 0, bytes = 0;
    for (uint64_t at = 0; at < arena->used;) {
        const block *b = (const void *)(arena->data + at);
        if ((b->state & ALLOCATED) && b->kind == NEOCLR_GC_INTERN) {
            if (b->bytes != 8) return 3;
            const neoclr_aot_text *owner;
            memcpy(&owner, b + 1, 8);
            if (!owner || owner->length > UINT64_C(1048576) - bytes) return 3;
            if (owner->length == text->length && !memcmp(owner->bytes,text->bytes,(size_t)text->length)) {
                *output = owner;
                return 0;
            }
            bytes += owner->length; count++;
        }
        at += b->span;
    }
    if (count >= 4096 || text->length > UINT64_C(1048576) - bytes) return 10;
    void *entry;
    int32_t status;
    if (rooted) {
        const neoclr_probe_frame *head = neoclr_root_probe_head_v1();
        neoclr_aot_context *context = head ? (neoclr_aot_context *)head->context : NULL;
        if (!context || &context->text != arena) return 3;
        const void *owner = text;
        status = neoclr_gc_allocate_rooted_values_v1(context, head, &owner, 1, 8, NEOCLR_GC_INTERN, &entry);
    } else {
        status = neoclr_gc_allocate_v1(arena,8,NEOCLR_GC_INTERN,&entry);
    }
    if (status) return status;
    memcpy(entry,&text,8);
    *output = text;
    return 0;
}

int32_t neoclr_string_intern_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) { return string_intern(text, arena, output, 0); }
int32_t neoclr_string_intern_rooted_v1(const neoclr_aot_text *text, neoclr_aot_text_arena *arena,
    const neoclr_aot_text **output) { return string_intern(text, arena, output, 1); }

/* Managed GC.Collect is available independently of runtime suspension. */
int32_t neoclr_gc_collect_managed_v1(neoclr_aot_context *c, void *out) {
    (void)out;
    return neoclr_gc_collect_v1(c, neoclr_root_probe_head_v1());
}
