#ifndef NEOCLR_NATIVE_MEMORY_VIEWS_H
#define NEOCLR_NATIVE_MEMORY_VIEWS_H
#include "text-arena.h"
/* Private checked byte-window kernel, not the final guest ABI. The caller supplies
 * a live context and allocator-produced mutable byte-array base. The future typed
 * binder must distinguish array references from immutable value snapshots and
 * supply kind from the exact source/result contract, never a guest integer.
 * Operations do not allocate,
 * collect or register roots. Publish owner at collecting boundaries, including
 * for empty windows. Publish only owner as a stack root; the other fields are numeric.
 * GC builds validate the exact live allocation/context before dereferencing it.
 * Arena builds require the trusted allocation base (no allocation registry).
 * These values do not extend context lifetime or authorize borrowed suspension.
 */
enum { NEOCLR_VIEW_SLICE = 0, NEOCLR_VIEW_READONLY_SLICE = 1,
       NEOCLR_VIEW_MEMORY = 2, NEOCLR_VIEW_READONLY_MEMORY = 3 };
typedef struct {
    void *owner;
    uint32_t offset, length;
    uint64_t kind;
} neoclr_byte_view;
_Static_assert(sizeof(neoclr_byte_view) == 24, "private byte-window size");
_Static_assert(offsetof(neoclr_byte_view, owner) == 0, "window owner root");
/* Null arrays admit only the empty window. Bounds failures return IndexOutOfRange;
 * invalid storage, readonly writes and uninitialized reads return RuntimeError.
 * All output parameters and backing storage are unchanged on failure. Outputs
 * may alias the input descriptor; callers must not overlap descriptors with data.
 */
int32_t neoclr_byte_view_create_v1(neoclr_aot_context *, void *, int32_t, int32_t,
    uint64_t kind, neoclr_byte_view *);
int32_t neoclr_byte_view_create_all_v1(neoclr_aot_context *, void *, uint64_t kind,
    neoclr_byte_view *);
int32_t neoclr_byte_view_slice_v1(neoclr_aot_context *, const neoclr_byte_view *,
    int32_t, int32_t, neoclr_byte_view *);
/* Only access restriction and persistent-to-borrowed conversions are permitted. */
int32_t neoclr_byte_view_convert_v1(neoclr_aot_context *, const neoclr_byte_view *,
    uint64_t kind, neoclr_byte_view *);
int32_t neoclr_byte_view_get_v1(neoclr_aot_context *, const neoclr_byte_view *,
    int32_t, uint8_t *);
int32_t neoclr_byte_view_set_v1(neoclr_aot_context *, const neoclr_byte_view *,
    int32_t, uint8_t);
/* Copies the entire source into the destination prefix, including overlapping
 * windows. Validates both descriptors, writable destination, capacity and every
 * source initialization bit before changing bytes or initialization state.
 * The destination suffix and both descriptors are preserved. A short destination
 * returns IndexOutOfRange; readonly destinations/uninitialized sources return
 * RuntimeError. Even empty copies validate storage and destination capability.
 */
int32_t neoclr_byte_view_copy_v1(neoclr_aot_context *, const neoclr_byte_view *,
    const neoclr_byte_view *);
#endif
