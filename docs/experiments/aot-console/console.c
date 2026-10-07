#include "console.h"
#include <stdio.h>
#include <stddef.h>

int32_t neoclr_console_read_byte_v1(void) {
    int value = fgetc(stdin);
    if (value != EOF) return (unsigned char)value;
    return ferror(stdin) ? -3 : -1;
}

int32_t neoclr_console_write_bytes_v1(int32_t error, const unsigned char *bytes,
    uint64_t length, int32_t offset, int32_t count) {
    if (offset < 0 || count < 0 || (uint64_t)offset > length ||
        (uint64_t)count > length - (uint64_t)offset) return -7;
    if (count > 65536) return -8;
    if (count == 0) return 0;
    FILE *stream = error ? stderr : stdout;
    const unsigned char *start = bytes + offset;
    size_t prefix = 0;
    /* Match StdioConsole's stdout line writer and unbuffered stderr without
     * changing process-wide buffering after a host may already have used stdio. */
    if (error) prefix = (size_t)count;
    else for (size_t i = 0; i < (size_t)count; i++) if (start[i] == '\n') prefix = i + 1;
    size_t written = prefix ? fwrite(start, 1, prefix, stream) : 0;
    if (ferror(stream) || (prefix && fflush(stream) != 0)) return -10;
    if (written != prefix) return (int32_t)written;
    if (prefix < (size_t)count) {
        written += fwrite(start + prefix, 1, (size_t)count - prefix, stream);
        if (ferror(stream)) return -10;
    }
    return (int32_t)written;
}

int32_t neoclr_console_flush_v1(int32_t error) {
    return fflush(error ? stderr : stdout) == 0 ? 0 : -10;
}
