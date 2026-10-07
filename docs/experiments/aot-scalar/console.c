#define _POSIX_C_SOURCE 200809L
#include "console.h"
#include <stdio.h>

int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length) {
    int32_t failed = 0;
    flockfile(stdout);
    size_t offset = 0;
    while (offset < length) {
        size_t written = fwrite(bytes + offset, 1, length - offset, stdout);
        if (written == 0) {
            failed = 1;
            break;
        }
        offset += written;
    }
    if (!failed && fputc('\n', stdout) == EOF) failed = 1;
    if (fflush(stdout) != 0) failed = 1;
    if (ferror(stdout)) failed = 1;
    funlockfile(stdout);
    return failed;
}
