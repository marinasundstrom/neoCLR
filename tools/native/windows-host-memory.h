#ifndef NEOCLR_WINDOWS_HOST_MEMORY_H
#define NEOCLR_WINDOWS_HOST_MEMORY_H
#include <stddef.h>
/* Private Windows host experiment; not a guest allocator or public runtime ABI.
 * Zero-initialize before create. Single owner: do not copy, mutate or concurrently
 * access this record. Release only after all activations, callbacks and roots are
 * gone. No TLS ownership, GC, suspension or synchronization is supplied here. */
#define NEOCLR_WINDOWS_HEAP_LIMIT (1024u * 1024u)
typedef struct {
    void *reservation;
    unsigned char *data;
    size_t capacity;
    size_t committed;
    size_t page_size;
} neoclr_windows_heap;
/* Returns 0 on success, 1 on invalid input/OS failure. Failure preserves output.
 * Capacity is logical; the final committed page may contain inaccessible-to-guest
 * padding only by contract, not by hardware protection. Guard pages bound the
 * committed region, not individual allocations. */
int neoclr_windows_heap_create(neoclr_windows_heap *heap, size_t capacity);
/* Empty release succeeds. OS release failure retains ownership for diagnosis. */
int neoclr_windows_heap_release(neoclr_windows_heap *heap);
#endif
