#ifndef NEOCLR_AOT_FAULT_DETAILS_H
#define NEOCLR_AOT_FAULT_DETAILS_H
#include <stdint.h>
#include <stddef.h>
#include <stdio.h>

/* Experimental ARM64 ABI v3. Text belongs to the loaded native image.
   These codes preserve the earlier AOT status mapping; not Rust enum ordinals. */
#define NEOCLR_AOT_FAULT_NONE 0
#define NEOCLR_AOT_FAULT_DIVIDE_BY_ZERO 1
#define NEOCLR_AOT_FAULT_ARITHMETIC_OVERFLOW 2
#define NEOCLR_AOT_FAULT_RUNTIME_ERROR 3
#define NEOCLR_AOT_FAULT_USER_FAULT 4
#define NEOCLR_AOT_FAULT_STACK_OVERFLOW 9
#define NEOCLR_AOT_FAULT_INVALID_CAST 11
/* MSVC reports C4200 for this C flexible array even in /std:c11 mode.
 * Preserve the length-plus-bytes ABI; suppress only this declaration. */
#ifdef _MSC_VER
#pragma warning(push)
#pragma warning(disable: 4200)
#endif
typedef struct { uint64_t length; unsigned char bytes[]; } neoclr_aot_text;
#ifdef _MSC_VER
#pragma warning(pop)
#endif
_Static_assert(sizeof(neoclr_aot_text) == 8, "text header size");
_Static_assert(offsetof(neoclr_aot_text, bytes) == 8, "text bytes offset");
typedef struct {
    const neoclr_aot_text *function;
    uint32_t instruction;
    uint32_t reserved;
} neoclr_aot_frame;
typedef struct {
    uint32_t code;
    uint32_t frame_count;
    const neoclr_aot_text *message;
    uint32_t truncated;
    uint32_t reserved;
    neoclr_aot_frame frames[64];
} neoclr_aot_fault;
_Static_assert(sizeof(void *) == 8, "ARM64 pointer size");
_Static_assert(sizeof(neoclr_aot_frame) == 16, "frame layout");
_Static_assert(offsetof(neoclr_aot_fault, message) == 8, "message layout");
_Static_assert(offsetof(neoclr_aot_fault, frames) == 24, "frame layout");
_Static_assert(sizeof(neoclr_aot_fault) == 1048, "context layout");

/* Both pointers must be nonnull, aligned, writable and nonoverlapping.
   Only success writes result. Each call resets the diagnostic header.
   Frame order is failure site to root. Instruction is a neoIL index, not a byte offset.
   One caller-owned context per concurrent invocation; no global/TLS state. */
int32_t neoclr_entry_v3(int32_t value, int32_t *result, neoclr_aot_fault *fault);
/* Returns 0 on success, -1 on output failure. Does not exit the host process. */
int neoclr_aot_render_fault(FILE *stream, const neoclr_aot_fault *fault);
#endif
