#ifndef NEOCLR_AOT_CONSOLE_H
#define NEOCLR_AOT_CONSOLE_H
#include <stdint.h>
/* Experimental linked service ABI, independent of private System.Value layout.
 * 0..255: byte, -1: EOF, -2: unavailable, -3: read failed.
 * Any other result is an invalid service status, diagnosed by managed Console.
 * Hosts without input may supply a replacement implementation returning -2.
 */
int32_t neoclr_console_read_byte_v1(void);
/* Raw output: nonnegative byte count; -7 invalid range, -8 limit, -10 I/O failure.
 * Flush: 0 success or -10 failure. error selects stderr; zero selects stdout.
 * Stdout flushes through the last LF; stderr is flushed on each nonempty write.
 * Buffers are borrowed only for the call. Neither operation closes the channel.
 */
int32_t neoclr_console_write_bytes_v1(int32_t error, const unsigned char *bytes,
    uint64_t length, int32_t offset, int32_t count);
int32_t neoclr_console_flush_v1(int32_t error);
#endif
