/* Experimental synchronous Windows console host; private collector ABI. */
#include "windows-gc-host.h"
#include "../../docs/experiments/aot-scalar/console.h"
#include <fcntl.h>
#include <io.h>
#include <stdio.h>

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
#ifndef NEOCLR_HTTP_HOST
int main(void) {
    /* Byte contracts include embedded NUL and LF; console display policy remains
     * the terminal's responsibility. No CRT CRLF or Ctrl-Z translation. */
    if (_setmode(_fileno(stdin), _O_BINARY) == -1 ||
        _setmode(_fileno(stdout), _O_BINARY) == -1 ||
        _setmode(_fileno(stderr), _O_BINARY) == -1) return 2;
    neoclr_windows_gc_host host = {0};
    if (neoclr_windows_gc_host_open(&host, 1024 * 1024)) return 2;
    int32_t result = 0;
    int32_t status = neoclr_entry_v4(0, &result, &host.context);
    int host_error = neoclr_root_probe_head_v1() != NULL;
    if (status && neoclr_aot_render_fault(stderr, &host.context.fault)) host_error = 1;
    if (!host_error && neoclr_gc_collect_v1(&host.context, NULL)) host_error = 1;
    if (fflush(stdout) || ferror(stdout)) host_error = 1;
    if (neoclr_windows_gc_host_close(&host)) host_error = 1;
    if (host_error) fputs("neoCLR native console cleanup failed\n", stderr);
    return host_error ? 2 : status ? 1 : result;
}

#endif
