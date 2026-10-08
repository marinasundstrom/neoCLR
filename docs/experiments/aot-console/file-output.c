#include "file-output.h"
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
        case EISDIR: return 5;
        default: return 6;
    }
}

int32_t neoclr_file_write_utf8_v1(const neoclr_aot_text *path,
    const neoclr_aot_text *text, int32_t limit, int32_t *output) {
    if (!path || !text || !output) return 3;
    if (limit < 0) { *output = 1; return 0; }
    if (path->length > SIZE_MAX - 1) return 5;
    if (!path->length || memchr(path->bytes, 0, (size_t)path->length)) {
        *output = 2; return 0;
    }
    if (text->length > (uint64_t)limit) { *output = 7; return 0; }
    char *name = malloc((size_t)path->length + 1);
    if (!name) return 5;
    memcpy(name, path->bytes, (size_t)path->length);
    name[path->length] = 0;
    int fd;
    do { fd = open(name, O_WRONLY | O_CREAT, 0666); } while (fd < 0 && errno == EINTR);
    int saved_error = errno;
    free(name);
    if (fd < 0) { *output = io_status(saved_error); return 0; }
    struct stat info;
    int32_t status = 0;
    if (fstat(fd, &info)) status = io_status(errno);
    else if (!S_ISREG(info.st_mode)) status = 5;
    else if (ftruncate(fd, 0)) status = io_status(errno);
    else {
        size_t offset = 0;
        while (offset < (size_t)text->length) {
            ssize_t count = write(fd, text->bytes + offset, (size_t)text->length - offset);
            if (count < 0 && errno == EINTR) continue;
            if (count <= 0) { status = count < 0 ? io_status(errno) : 6; break; }
            offset += (size_t)count;
        }
    }
    /* Like std::fs::File's Write::flush, no fsync/durability guarantee is added.
     * A failed close is not retried: the descriptor may already have been released. */
    (void)close(fd);
    *output = status;
    return 0;
}
