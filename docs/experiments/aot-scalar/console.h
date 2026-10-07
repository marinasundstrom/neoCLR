#ifndef NEOCLR_AOT_CONSOLE_H
#define NEOCLR_AOT_CONSOLE_H
#include <stddef.h>
#include <stdint.h>

/* Borrow valid UTF-8 bytes for this synchronous call only; length excludes LF.
   Append LF and flush before success (zero). Any nonzero status means failure;
   partial output is possible. The callee must not retain or mutate the bytes. */
int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length);
#endif
