#ifndef NEOCLR_NATIVE_STACK_H
#define NEOCLR_NATIVE_STACK_H
#include <stdint.h>
/* Private macOS ARM64 guest helper and Windows x64 host probe.
 * Windows managed-code admission is separately gated.
 * Not a green-thread or public callable ABI.
 * The generated caller must cap every final machine frame at 64 KiB and check
 * before crossing a host entry and after publishing each guest frame. A passing
 * check leaves 256 KiB: one maximum guest frame plus 192 KiB for the matched
 * non-recursive C adapters and fault unwind. This is not a guard for arbitrary
 * foreign code, signal handlers, manually switched stacks or host reentry.
 * No allocation, collection, guest invocation or mutation. 0 = room, 9 = limit.
 * This helper alone does not authorize recursive CIL admission. */
#define NEOCLR_NATIVE_FRAME_LIMIT (64u * 1024u)
#define NEOCLR_NATIVE_STACK_RESERVE (256u * 1024u)
int32_t neoclr_native_stack_check_v1(void);
#endif
