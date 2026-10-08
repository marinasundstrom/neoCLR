#include "native-gc.h"
#include <string.h>
#include <stdatomic.h>
/* Allocation descriptors live in the same bounded buffer as payloads. Free
 * blocks coalesce; payload addresses never move. Object words are conservative
 * candidates; text/byte payloads are atomic, String arrays have explicit slots. */
typedef struct { uint64_t span, bytes, kind, state; } block;
_Static_assert(sizeof(block) == 32, "private allocation descriptor");
enum { ALLOCATED = 1, MARKED = 2, SCANNED = 4 };
static _Thread_local neoclr_gc_statistics statistics;
static int valid(const neoclr_aot_text_arena *a) {
    return a && a->used <= a->capacity && !(a->used & 7) &&
        (!a->capacity || a->data) && !((uintptr_t)a->data & 7);
}
static int blocks_valid(const neoclr_aot_text_arena *a) {
    if (!valid(a)) return 0;
    for (uint64_t at = 0; at < a->used;) {
        if (a->used - at < sizeof(block)) return 0;
        const block *b = (const void *)(a->data + at);
        if (b->span < sizeof(block) + 8 || (b->span & 7) || b->span > a->used - at ||
            b->state > 7 || (b->state && !(b->state & ALLOCATED)) ||
            (b->state && (!b->bytes || b->bytes > b->span - sizeof(block) ||
                          b->kind < NEOCLR_GC_TEXT || b->kind > NEOCLR_GC_RECORDS))) return 0;
        at += b->span;
    }
    return 1;
}
typedef struct host_root {
    neoclr_aot_context *context;
    void *value;
    uint64_t handle;
    struct host_root *previous, *next;
} host_root;
static _Thread_local host_root host_roots[NEOCLR_GC_HOST_ROOT_LIMIT];
static _Thread_local host_root *host_head;
static _Atomic uint64_t next_host_handle = 1;
static int live_base(const neoclr_aot_text_arena *arena, const void *value) {
    if (!value) return 1;
    for (uint64_t at = 0; at < arena->used;) {
        const block *b = (const void *)(arena->data + at);
        if ((b->state & ALLOCATED) && value == (const void *)(b + 1)) return 1;
        at += b->span;
    }
    return 0;
}
static host_root *find_host_root(neoclr_aot_context *context, uint64_t handle) {
    if (!context || !handle) return NULL;
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context && root->handle == handle) return root;
    return NULL;
}
int32_t neoclr_gc_host_root_create_v1(neoclr_aot_context *context, void *value, uint64_t *output) {
    if (!context || !output || !blocks_valid(&context->text) || !live_base(&context->text, value))
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
    *available = (host_root){context, value, handle, NULL, host_head};
    if (host_head) host_head->previous = available;
    host_head = available;
    *output = handle;
    return 0;
}
int32_t neoclr_gc_host_root_replace_v1(neoclr_aot_context *context, uint64_t handle, void *value) {
    host_root *root = find_host_root(context, handle);
    if (!root || !blocks_valid(&context->text) || !live_base(&context->text, value)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    root->value = value;
    return 0;
}
int32_t neoclr_gc_host_root_read_v1(neoclr_aot_context *context, uint64_t handle, void **output) {
    host_root *root = find_host_root(context, handle);
    if (!root || !output) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
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
int32_t neoclr_gc_entry_check_v1(neoclr_aot_context *context) {
    if (!context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (const neoclr_probe_frame *frame = neoclr_root_probe_head_v1(); frame; frame = frame->previous)
        if (frame->context == context) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    return 0;
}
int32_t neoclr_gc_allocate_v1(neoclr_aot_text_arena *a, uint64_t bytes,
                             uint32_t kind, void **output) {
    if (!output || !blocks_valid(a) || !bytes || kind < NEOCLR_GC_TEXT || kind > NEOCLR_GC_RECORDS)
        return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    if (bytes > UINT64_MAX - sizeof(block) - 7) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
    uint64_t needed = sizeof(block) + ((bytes + 7) & ~UINT64_C(7));
    block *chosen = NULL;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        if (!b->state && b->span >= needed) { chosen = b; break; }
        at += b->span;
    }
    if (chosen) {
        if (chosen->span - needed >= sizeof(block) + 8) {
            block *rest = (void *)((unsigned char *)chosen + needed);
            *rest = (block){chosen->span - needed, 0, 0, 0};
            chosen->span = needed;
        }
    } else {
        if (needed > a->capacity - a->used) return NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT;
        chosen = (void *)(a->data + a->used);
        chosen->span = needed;
        a->used += needed;
    }
    chosen->bytes = bytes; chosen->kind = kind; chosen->state = ALLOCATED;
    memset(chosen + 1, 0, (size_t)(chosen->span - sizeof(block)));
    *output = chosen + 1;
    statistics.allocations++;
    return 0;
}
static void mark(neoclr_aot_text_arena *a, uint64_t word, block **pending) {
    uintptr_t base = (uintptr_t)a->data;
    if (word < base || word - base >= a->used) return;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        uint64_t begin = at + sizeof(block);
        /* Includes interior addresses and low-bit tagged String object views. */
        if (b->state && word - base >= begin && word - base - begin < b->bytes) {
            if (!(b->state & MARKED)) {
                /* Aligned header pointers share the state word with low-bit flags.
                 * No allocation, recursion or additional per-object storage. */
                b->state = (uint64_t)(uintptr_t)*pending | ALLOCATED | MARKED;
                *pending = b;
            }
            return;
        }
        at += b->span;
    }
}
static void mark_slots(neoclr_aot_text_arena *a, const neoclr_probe_storage *slots, uint32_t count, block **pending) {
    for (uint32_t i = 0; i < count; i++) {
        if (slots[i].read_bytes != 8) continue; /* 32-bit discriminators are not pointers. */
        uint64_t word; memcpy(&word, slots[i].address, 8); mark(a, word, pending);
    }
}
int32_t neoclr_gc_collect_v1(neoclr_aot_context *context, const neoclr_probe_frame *head) {
    if (!context || !blocks_valid(&context->text)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    neoclr_aot_text_arena *a = &context->text;
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context && !live_base(a, root->value)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    neoclr_probe_storage fault[65];
    int32_t count = neoclr_probe_fault_roots_v1(context, fault, 65);
    if (count < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at); b->state &= ALLOCATED; at += b->span;
    }
    block *pending = NULL;
    mark_slots(a, fault, (uint32_t)count, &pending);
    for (host_root *root = host_head; root; root = root->next)
        if (root->context == context) mark(a, (uintptr_t)root->value, &pending);
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        if (f->context != context) continue;
        mark_slots(a, f->storage, f->storage_count, &pending);
        mark_slots(a, f->transient, f->transient_count, &pending);
        for (uint32_t i = 0; i < f->lane_count; i++) mark(a, f->lanes[i], &pending);
    }
    /* Each newly marked allocation enters the intrusive worklist exactly once.
     * Address lookup remains linear; this removes repeated whole-heap scan passes. */
    while (pending) {
        block *b = pending;
        pending = (block *)(uintptr_t)(b->state & ~UINT64_C(7));
        b->state = ALLOCATED | MARKED | SCANNED;
        const unsigned char *data = (const void *)(b + 1);
        if (b->kind == NEOCLR_GC_OBJECT) {
            for (uint64_t offset = 8; offset + 8 <= b->bytes; offset += 8) {
                uint64_t word; memcpy(&word, data + offset, 8); mark(a, word, &pending);
            }
        } else if (b->kind == NEOCLR_GC_RECORDS) {
            uint64_t kind, length, lanes;
            if (b->bytes < 24) goto invalid_descriptor;
            memcpy(&kind, data, 8); memcpy(&length, data + 8, 8); memcpy(&lanes, data + 16, 8);
            if (kind != UINT64_C(0x80000005) || !lanes || lanes > 64 ||
                length > 65536 || length > (b->bytes - 24) / (lanes * 8 + 1)) goto invalid_descriptor;
            uint64_t markers = 24 + length * lanes * 8;
            for (uint64_t i = 0; i < length; i++) {
                if (!data[markers + i]) continue;
                for (uint64_t lane = 0; lane < lanes; lane++) {
                    uint64_t word; memcpy(&word, data + 24 + (i * lanes + lane) * 8, 8);
                    mark(a, word, &pending);
                }
            }
        } else if (b->kind == NEOCLR_GC_STRINGS) {
            uint64_t kind, length;
            if (b->bytes < 16) goto invalid_descriptor;
            memcpy(&kind, data, 8); memcpy(&length, data + 8, 8);
            int reserved = kind == UINT64_C(0x80000004);
            if ((!reserved && kind != UINT64_C(0x80000003)) ||
                length > (b->bytes - 16) / (reserved ? 9 : 8)) goto invalid_descriptor;
            for (uint64_t i = 0; i < length; i++) {
                if (reserved && !data[16 + 8 * length + i]) continue;
                uint64_t word; memcpy(&word, data + 16 + 8 * i, 8); mark(a, word, &pending);
            }
        }
    }
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        if (b->state && !(b->state & MARKED)) {
            statistics.reclaimed_allocations++; statistics.reclaimed_bytes += b->bytes;
            /* Poison reclaimed contents so stale reads fail functional stress tests. */
            memset(b + 1, 0xdd, (size_t)(b->span - sizeof(block)));
            b->state = 0; b->bytes = 0; b->kind = 0;
        } else b->state &= ALLOCATED;
        at += b->span;
    }
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        if (!b->state) {
            while (at + b->span < a->used) {
                block *next = (void *)(a->data + at + b->span);
                if (next->state) break;
                b->span += next->span;
            }
            if (at + b->span == a->used) { a->used = at; break; }
        }
        at += b->span;
    }
    statistics.collections++;
    return 0;
invalid_descriptor:
    /* Do not leave private queue links in public quiescent heap state on failure.
     * Nothing has been swept; a repaired descriptor can be collected again. */
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at); b->state &= ALLOCATED; at += b->span;
    }
    return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
}
neoclr_gc_statistics neoclr_gc_statistics_v1(void) { return statistics; }
