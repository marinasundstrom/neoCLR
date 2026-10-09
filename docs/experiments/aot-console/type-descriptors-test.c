/* Standalone contract host; stdout is the test's UTF-8 byte sink. */
#include "type-tokens-test.c"
int32_t neoclr_console_write_line_utf8_v1(const uint8_t *bytes, size_t length) {
    if (fwrite(bytes, 1, length, stdout) != length || fputc('\n', stdout) == EOF) return 1;
    return fflush(stdout) == 0 ? 0 : 1;
}
