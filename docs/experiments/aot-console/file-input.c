#include "file-input.h"
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static int32_t io_status(int code) {
    switch (code) {
        case ENOENT: return 3;
        case EACCES: case EPERM: return 4;
        case EINVAL: return 2;
        default: return 6;
    }
}
static int32_t publish_error(void *output, uint64_t error) {
    uint32_t tag = 2;
    memcpy(output, &tag, sizeof(tag));
    memcpy((unsigned char *)output + 8, &error, sizeof(error));
    return 0;
}

int32_t neoclr_file_read_utf8_v1(const neoclr_aot_text *path, int32_t limit,
    neoclr_aot_text_arena *arena, void *output) {
    if (!path || !arena || !output) return 3;
    if (limit < 0) return publish_error(output, 1);
    if (path->length > SIZE_MAX - 1) return 5;
    if (!path->length || memchr(path->bytes, 0, (size_t)path->length))
        return publish_error(output, 2);
    char *name = malloc((size_t)path->length + 1);
    if (!name) return 5;
    memcpy(name, path->bytes, (size_t)path->length);
    name[path->length] = 0;
    int fd;
    do { fd = open(name, O_RDONLY); } while (fd < 0 && errno == EINTR);
    int saved_error = errno;
    free(name);
    if (fd < 0) return publish_error(output, (uint64_t)io_status(saved_error));
    struct stat info;
    int32_t error = 0, fault = 0;
    unsigned char *bytes = NULL;
    size_t length = 0, capacity = 0;
    if (fstat(fd, &info)) error = io_status(errno);
    else if (!S_ISREG(info.st_mode)) error = 5;
    else {
        unsigned char chunk[8192];
        for (;;) {
            size_t remaining = (size_t)limit - length;
            size_t requested = remaining < sizeof(chunk) ? remaining + 1 : sizeof(chunk);
            ssize_t count = read(fd, chunk, requested);
            if (count < 0 && errno == EINTR) continue;
            if (count < 0) { error = io_status(errno); break; }
            if (!count) break;
            if ((size_t)count > remaining) { error = 7; break; }
            size_t needed = length + (size_t)count;
            if (needed > capacity) {
                size_t grown = capacity ? capacity * 2 : sizeof(chunk);
                if (grown < needed) grown = needed;
                if (grown > (size_t)limit) grown = (size_t)limit;
                unsigned char *next = realloc(bytes, grown);
                if (!next) { fault = 5; break; }
                bytes = next;
                capacity = grown;
            }
            memcpy(bytes + length, chunk, (size_t)count);
            length = needed;
        }
    }
    (void)close(fd);
    if (fault || error) {
        free(bytes);
        return fault ? fault : publish_error(output, (uint64_t)error);
    }
    uint64_t result[2] = {0, 0};
    /* The file limit is checked before decoding, and the descriptor is already
     * closed before managed allocation can collect or exhaust the arena. */
    static const unsigned char empty = 0;
    fault = neoclr_text_decode_utf8_bytes_v1(bytes ? bytes : &empty, length, arena, result);
    free(bytes);
    if (fault) return fault;
    if (result[0] == 2) return publish_error(output, 8);
    memcpy(output, result, sizeof(result));
    return 0;
}
