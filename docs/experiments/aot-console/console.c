#include "console.h"
#include <stdio.h>
#include <stddef.h>
#include <stdatomic.h>
#include <errno.h>
#if !defined(_WIN32)
#include <fcntl.h>
#include <poll.h>
#include <time.h>
#include <unistd.h>
#endif

static atomic_flag input_owned = ATOMIC_FLAG_INIT;
static int acquire_input(void) {
    return !atomic_flag_test_and_set_explicit(&input_owned, memory_order_acquire);
}
static void release_input(void) {
    atomic_flag_clear_explicit(&input_owned, memory_order_release);
}

int32_t neoclr_console_read_byte_v1(void) {
    if (!acquire_input()) return -3;
#if !defined(_WIN32)
    unsigned char byte;
    ssize_t count;
    do { count = read(STDIN_FILENO, &byte, 1); } while (count < 0 && errno == EINTR);
    int32_t result = count == 1 ? byte : count == 0 ? -1 : -3;
#else
    int value = fgetc(stdin);
    int32_t result = value != EOF ? (unsigned char)value : ferror(stdin) ? -3 : -1;
#endif
    release_input();
    return result;
}

int32_t neoclr_terminal_open_v1(neoclr_terminal_input_v1 *input, int32_t capture_interrupts) {
    if (!input || (capture_interrupts != 0 && capture_interrupts != 1)) return -6;
    if (input->active) return -4;
#if !defined(_WIN32)
    if (!acquire_input()) return -4;
    if (isatty(STDIN_FILENO) != 1) { release_input(); return -3; }
    int fd = fcntl(STDIN_FILENO, F_DUPFD_CLOEXEC, 0);
    if (fd < 0) { release_input(); return -7; }
    struct termios original;
    if (tcgetattr(fd, &original) != 0) { close(fd); release_input(); return -7; }
    struct termios mode = original;
    mode.c_lflag &= ~(ICANON | ECHO | ECHONL | IEXTEN);
    if (capture_interrupts) mode.c_lflag &= ~ISIG;
    else mode.c_lflag |= ISIG;
    mode.c_iflag &= ~(ISTRIP | INLCR | IGNCR | ICRNL | IXON | PARMRK);
    mode.c_cc[VMIN] = 0;
    mode.c_cc[VTIME] = 0;
    input->descriptor = fd;
    input->original = original;
    input->ended = 0;
    input->active = 1;
    if (tcsetattr(fd, TCSANOW, &mode) != 0) {
        /* Preserve ownership if restoration fails; caller must retry close even
         * after an unsuccessful open when active remains set. */
        (void)neoclr_terminal_close_v1(input);
        return -7;
    }
    return 0;
#else
    return -3;
#endif
}

#if !defined(_WIN32)
static int monotonic_ns(uint64_t *value) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now) != 0) return 0;
    *value = (uint64_t)now.tv_sec * UINT64_C(1000000000) + (uint64_t)now.tv_nsec;
    return 1;
}
#endif

int32_t neoclr_terminal_read_v1(neoclr_terminal_input_v1 *input, unsigned char *bytes,
    uint64_t length, int32_t timeout_ms) {
    if (!input || !bytes || length == 0 || length > 65536 || timeout_ms < -1) return -6;
#if !defined(_WIN32)
    if (!input->active) return -5;
    if (input->ended) return -2;
    uint64_t deadline = 0;
    if (timeout_ms > 0) {
        if (!monotonic_ns(&deadline)) return -7;
        uint64_t delta = (uint64_t)timeout_ms * UINT64_C(1000000);
        if (UINT64_MAX - deadline < delta) return -6;
        deadline += delta;
    }
    for (;;) {
        int wait = timeout_ms;
        if (timeout_ms > 0) {
            uint64_t now;
            if (!monotonic_ns(&now)) return -7;
            uint64_t remaining = now < deadline ? deadline - now : 0;
            wait = (int)((remaining + 999999) / 1000000);
        }
        struct pollfd ready = { .fd = input->descriptor, .events = POLLIN, .revents = 0 };
        int status = poll(&ready, 1, wait);
        if (status < 0 && errno != EINTR) return -7;
        if (status > 0) {
            if (ready.revents & POLLNVAL) return -5;
            ssize_t count = read(input->descriptor, bytes, (size_t)length);
            if (count > 0) return (int32_t)count;
            if (count < 0 && errno != EINTR && errno != EAGAIN && errno != EWOULDBLOCK &&
                !((ready.revents & POLLHUP) && errno == EIO)) return -7;
            if (count == 0 || (count < 0 && errno == EIO)) {
                if (ready.revents & POLLHUP) { input->ended = 1; return -2; }
                if (ready.revents & POLLERR) return -7;
            }
        }
        if (timeout_ms == 0) return -1;
        if (timeout_ms > 0) {
            uint64_t now;
            if (!monotonic_ns(&now)) return -7;
            if (now >= deadline) return -1;
        }
    }
#else
    return -3;
#endif
}

int32_t neoclr_terminal_close_v1(neoclr_terminal_input_v1 *input) {
    if (!input) return -6;
    if (!input->active) return 0;
#if !defined(_WIN32)
    if (tcsetattr(input->descriptor, TCSANOW, &input->original) != 0) return -7;
    close(input->descriptor);
    input->descriptor = -1;
    input->active = 0;
    input->ended = 0;
    release_input();
    return 0;
#else
    return -3;
#endif
}

int32_t neoclr_terminal_dispose_v1(neoclr_terminal_input_v1 *input) {
    int32_t status = neoclr_terminal_close_v1(input);
#if !defined(_WIN32)
    if (input && input->active) {
        close(input->descriptor);
        input->descriptor = -1;
        input->active = 0;
        input->ended = 0;
        release_input();
    }
#endif
    return status;
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
