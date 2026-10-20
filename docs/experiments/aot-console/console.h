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
/* Private native terminal transport, not a supported embedding ABI.
 * Caller zero-initializes storage, never copies an active session, and serializes
 * calls on it. Retain storage until successful close or final dispose.
 * open/close: 0 success. read: positive count, -1 pending, -2 end.
 * Errors: -3 unsupported, -4 busy, -5 closed, -6 invalid argument, -7 I/O.
 * timeout_ms: -1 indefinite, 0 immediate, otherwise a total wait bound.
 * No decoding, echo, allocation, signal handler or invocation registration.
 */
#if !defined(_WIN32)
#include <termios.h>
typedef struct {
    int descriptor;
    int active;
    int ended;
    struct termios original;
} neoclr_terminal_input_v1;
#else
typedef struct { int active; } neoclr_terminal_input_v1;
#endif
int32_t neoclr_terminal_open_v1(neoclr_terminal_input_v1 *input, int32_t capture_interrupts);
int32_t neoclr_terminal_read_v1(neoclr_terminal_input_v1 *input, unsigned char *bytes,
    uint64_t length, int32_t timeout_ms);
int32_t neoclr_terminal_close_v1(neoclr_terminal_input_v1 *input);
/* Final best-effort restoration: always releases ownership, even on I/O failure. */
int32_t neoclr_terminal_dispose_v1(neoclr_terminal_input_v1 *input);
#endif
