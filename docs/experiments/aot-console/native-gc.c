#include "native-gc.h"
#include <string.h>
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
                          b->kind < NEOCLR_GC_TEXT || b->kind > NEOCLR_GC_STRINGS))) return 0;
        at += b->span;
    }
    return 1;
}
int32_t neoclr_gc_allocate_v1(neoclr_aot_text_arena *a, uint64_t bytes,
                             uint32_t kind, void **output) {
    if (!output || !blocks_valid(a) || !bytes || kind < NEOCLR_GC_TEXT || kind > NEOCLR_GC_STRINGS)
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
static void mark(neoclr_aot_text_arena *a, uint64_t word) {
    uintptr_t base = (uintptr_t)a->data;
    if (word < base || word - base >= a->used) return;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at);
        uint64_t begin = at + sizeof(block);
        /* Includes interior addresses and low-bit tagged String object views. */
        if (b->state && word - base >= begin && word - base - begin < b->bytes) {
            b->state |= MARKED;
            return;
        }
        at += b->span;
    }
}
static void mark_slots(neoclr_aot_text_arena *a, const neoclr_probe_storage *slots, uint32_t count) {
    for (uint32_t i = 0; i < count; i++) {
        if (slots[i].read_bytes != 8) continue; /* 32-bit discriminators are not pointers. */
        uint64_t word; memcpy(&word, slots[i].address, 8); mark(a, word);
    }
}
int32_t neoclr_gc_collect_v1(neoclr_aot_context *context, const neoclr_probe_frame *head) {
    if (!context || !blocks_valid(&context->text)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    neoclr_aot_text_arena *a = &context->text;
    neoclr_probe_storage fault[65];
    int32_t count = neoclr_probe_fault_roots_v1(context, fault, 65);
    if (count < 0) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
    for (uint64_t at = 0; at < a->used;) {
        block *b = (void *)(a->data + at); b->state &= ALLOCATED; at += b->span;
    }
    mark_slots(a, fault, (uint32_t)count);
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        if (f->context != context) continue;
        mark_slots(a, f->storage, f->storage_count);
        mark_slots(a, f->transient, f->transient_count);
        for (uint32_t i = 0; i < f->lane_count; i++) mark(a, f->lanes[i]);
    }
    /* Bounded fixed-point traversal avoids recursion or an external mark stack.
     * Every pass processes at least one previously unscanned marked allocation. */
    int progress;
    do {
        progress = 0;
        for (uint64_t at = 0; at < a->used;) {
            block *b = (void *)(a->data + at);
            if (b->state == (ALLOCATED | MARKED)) {
                b->state |= SCANNED; progress = 1;
                const unsigned char *data = (const void *)(b + 1);
                if (b->kind == NEOCLR_GC_OBJECT) {
                    for (uint64_t offset = 8; offset + 8 <= b->bytes; offset += 8) {
                        uint64_t word; memcpy(&word, data + offset, 8); mark(a, word);
                    }
                } else if (b->kind == NEOCLR_GC_STRINGS) {
                    uint64_t kind, length;
                    if (b->bytes < 16) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
                    memcpy(&kind, data, 8); memcpy(&length, data + 8, 8);
                    int reserved = kind == UINT64_C(0x80000004);
                    if ((!reserved && kind != UINT64_C(0x80000003)) ||
                        length > (b->bytes - 16) / (reserved ? 9 : 8)) return NEOCLR_AOT_FAULT_RUNTIME_ERROR;
                    for (uint64_t i = 0; i < length; i++) {
                        if (reserved && !data[16 + 8 * length + i]) continue;
                        uint64_t word; memcpy(&word, data + 16 + 8 * i, 8); mark(a, word);
                    }
                }
            }
            at += b->span;
        }
    } while (progress);
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
}
neoclr_gc_statistics neoclr_gc_statistics_v1(void) { return statistics; }
