/* Windows-only scalar/literal-console acceptance host, not a managed runtime host. */
#include <fcntl.h>
#include <io.h>
#include <stdio.h>
#include "../../docs/experiments/aot-scalar/console.h"

int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length) {
    int32_t failed = 0;
    _lock_file(stdout);
    size_t offset = 0;
    while (offset < length) {
        size_t written = fwrite(bytes + offset, 1, length - offset, stdout);
        if (!written) { failed = 1; break; }
        offset += written;
    }
    if (!failed && fputc('\n', stdout) == EOF) failed = 1;
    if (fflush(stdout) != 0 || ferror(stdout)) failed = 1;
    _unlock_file(stdout);
    return failed;
}

#define main scalar_main
#include "../../docs/experiments/aot-scalar/host.c"
#undef main

int main(int argc, char **argv) {
    /* Preserve UTF-8 bytes, embedded NUL and LF in redirected acceptance output.
       Interactive Windows console rendering/code-page policy is separate work. */
    if (_setmode(_fileno(stdout), _O_BINARY) == -1 ||
        _setmode(_fileno(stderr), _O_BINARY) == -1) return 2;
    return scalar_main(argc, argv);
}
