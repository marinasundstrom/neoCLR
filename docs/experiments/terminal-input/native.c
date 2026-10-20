#include "console.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#if !defined(_WIN32)
#include <unistd.h>
#include <fcntl.h>
#endif

int main(int argc, char **argv) {
    assert(argc == 2);
    neoclr_terminal_input_v1 input = {0}, other = {0};
    unsigned char bytes[32] = {0};
    if (strcmp(argv[1], "bytes") == 0) {
        for (int n = 0; n < 256; n++) assert(neoclr_console_read_byte_v1() == n);
        assert(neoclr_console_read_byte_v1() == -1);
        puts("PASS all bytes");
        return 0;
    }
    assert(neoclr_terminal_open_v1(NULL, 0) == -6);
    assert(neoclr_terminal_close_v1(NULL) == -6);
    assert(neoclr_terminal_close_v1(&input) == 0);
    if (strcmp(argv[1], "redirected") == 0) {
        assert(neoclr_terminal_open_v1(&input, 1) == -3);
        assert(neoclr_console_read_byte_v1() == 97);
        assert(neoclr_console_read_byte_v1() == -1);
        puts("PASS redirected");
        return 0;
    }
    if (strcmp(argv[1], "switch") == 0) {
        assert(neoclr_console_read_byte_v1() == 97);
        assert(neoclr_terminal_open_v1(&input, 1) == 0);
        int total = 0;
        while (total < 4) {
            int count = neoclr_terminal_read_v1(&input, bytes + total, 4 - total, 2000);
            assert(count > 0);
            total += count;
        }
        assert(memcmp(bytes, "bcd\n", 4) == 0);
        assert(neoclr_terminal_close_v1(&input) == 0);
        puts("PASS no read-ahead");
        return 0;
    }
    assert(neoclr_terminal_open_v1(&input, 2) == -6);
    assert(neoclr_terminal_open_v1(&input, 1) == 0);
    assert(neoclr_terminal_open_v1(&input, 1) == -4);
    assert(neoclr_terminal_open_v1(&other, 1) == -4);
    assert(neoclr_console_read_byte_v1() == -3);
    assert(neoclr_terminal_read_v1(&input, bytes, 0, 0) == -6);
    assert(neoclr_terminal_read_v1(&input, bytes, 65537, 0) == -6);
    assert(neoclr_terminal_read_v1(&input, bytes, 32, -2) == -6);
    assert(neoclr_terminal_read_v1(&input, bytes, 32, 0) == -1);
    assert(neoclr_terminal_read_v1(&input, bytes, 32, 15) == -1);
#if !defined(_WIN32)
    struct termios mode;
    assert(tcgetattr(STDIN_FILENO, &mode) == 0);
    assert((mode.c_lflag & (ICANON | ECHO | ISIG)) == 0);
#endif
#if !defined(_WIN32)
    if (strcmp(argv[1], "replace") == 0) {
        int saved = dup(STDIN_FILENO);
        int null_input = open("/dev/null", O_RDONLY);
        assert(saved >= 0 && null_input >= 0);
        assert(dup2(null_input, STDIN_FILENO) >= 0);
        close(null_input);
        assert(neoclr_terminal_close_v1(&input) == 0);
        assert(neoclr_terminal_open_v1(&input, 1) == -3);
        assert(dup2(saved, STDIN_FILENO) >= 0);
        close(saved);
        puts("PASS replaced stdin");
        return 0;
    }
#endif
    puts("READY");
    fflush(stdout);
    if (strcmp(argv[1], "hangup") == 0) {
        assert(neoclr_terminal_read_v1(&input, bytes, 32, 2000) == -2);
        assert(neoclr_terminal_read_v1(&input, bytes, 32, 0) == -2);
        /* The terminal no longer exists; close may fail and remains retryable. */
        (void)neoclr_terminal_dispose_v1(&input);
        assert(input.active == 0);
        assert(neoclr_terminal_open_v1(&other, 1) != -4);
        (void)neoclr_terminal_dispose_v1(&other);
        puts("PASS hangup");
        return 0;
    }
    int total = 0;
    while (total < 8) {
        int count = neoclr_terminal_read_v1(&input, bytes + total, 8 - total, 2000);
        assert(count > 0);
        total += count;
    }
    assert(memcmp(bytes, "\x03\xc3\xa9\x1b[A\r\n", 8) == 0);
    assert(neoclr_terminal_close_v1(&input) == 0);
    assert(neoclr_terminal_close_v1(&input) == 0);
    assert(neoclr_terminal_read_v1(&input, bytes, 32, 0) == -5);
    assert(neoclr_terminal_open_v1(&input, 0) == 0);
#if !defined(_WIN32)
    assert(tcgetattr(STDIN_FILENO, &mode) == 0);
    assert((mode.c_lflag & ISIG) != 0);
#endif
    assert(neoclr_terminal_close_v1(&input) == 0);
    puts("PASS native terminal");
    return 0;
}
