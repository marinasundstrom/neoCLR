#include "file-input.h"
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#include <limits.h>
typedef HANDLE input_handle;
#define INVALID_INPUT INVALID_HANDLE_VALUE
static int32_t io_status(DWORD code) {
    switch (code) {
        case ERROR_FILE_NOT_FOUND: case ERROR_PATH_NOT_FOUND: return 3;
        case ERROR_ACCESS_DENIED: case ERROR_SHARING_VIOLATION: return 4;
        case ERROR_INVALID_NAME: case ERROR_BAD_PATHNAME:
        case ERROR_FILENAME_EXCED_RANGE: case ERROR_INVALID_PARAMETER: return 2;
        default: return 6;
    }
}
static input_handle open_input(const neoclr_aot_text *path, int32_t *error, int32_t *fault) {
    if (path->length > (uint64_t)INT_MAX) { *error = 2; return INVALID_INPUT; }
    int count = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
        (const char *)path->bytes, (int)path->length, NULL, 0);
    if (!count) { *error = 2; return INVALID_INPUT; }
    wchar_t *name = malloc(((size_t)count + 1) * sizeof(wchar_t));
    if (!name) { *fault = 5; return INVALID_INPUT; }
    if (MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS,
        (const char *)path->bytes, (int)path->length, name, count) != count) {
        free(name); *error = 2; return INVALID_INPUT;
    }
    name[count] = 0;
    HANDLE file = CreateFileW(name, GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL,
        OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_SEQUENTIAL_SCAN, NULL);
    DWORD code = GetLastError();
    free(name);
    if (file == INVALID_INPUT) *error = io_status(code);
    return file;
}
static int32_t regular_input(input_handle file) {
    BY_HANDLE_FILE_INFORMATION info;
    SetLastError(NO_ERROR);
    DWORD type = GetFileType(file);
    if (type == FILE_TYPE_UNKNOWN && GetLastError() != NO_ERROR) return io_status(GetLastError());
    if (type != FILE_TYPE_DISK) return 5;
    if (!GetFileInformationByHandle(file, &info)) return io_status(GetLastError());
    return (info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) ? 5 : 0;
}
static int64_t read_input(input_handle file, unsigned char *bytes, size_t size, int32_t *error) {
    DWORD count = 0;
    if (!ReadFile(file, bytes, (DWORD)size, &count, NULL)) {
        *error = io_status(GetLastError()); return -1;
    }
    return (int64_t)count;
}
static void close_input(input_handle file) { (void)CloseHandle(file); }
#else
#include <errno.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>
typedef int input_handle;
#define INVALID_INPUT (-1)
static int32_t io_status(int code) {
    switch (code) {
        case ENOENT: return 3;
        case EACCES: case EPERM: return 4;
        case EINVAL: return 2;
        default: return 6;
    }
}
static input_handle open_input(const neoclr_aot_text *path, int32_t *error, int32_t *fault) {
    char *name = malloc((size_t)path->length + 1);
    if (!name) { *fault = 5; return INVALID_INPUT; }
    memcpy(name, path->bytes, (size_t)path->length);
    name[path->length] = 0;
    int file;
    do { file = open(name, O_RDONLY); } while (file < 0 && errno == EINTR);
    int code = errno;
    free(name);
    if (file < 0) *error = io_status(code);
    return file;
}
static int32_t regular_input(input_handle file) {
    struct stat info;
    if (fstat(file, &info)) return io_status(errno);
    return S_ISREG(info.st_mode) ? 0 : 5;
}
static int64_t read_input(input_handle file, unsigned char *bytes, size_t size, int32_t *error) {
    ssize_t count;
    do { count = read(file, bytes, size); } while (count < 0 && errno == EINTR);
    if (count < 0) *error = io_status(errno);
    return (int64_t)count;
}
static void close_input(input_handle file) { (void)close(file); }
#endif

static int32_t publish_error(void *output, uint64_t error) {
    uint32_t tag = 2;
    memcpy(output, &tag, sizeof(tag));
    memcpy((unsigned char *)output + 8, &error, sizeof(error));
    return 0;
}

static int32_t file_read_utf8(const neoclr_aot_text *path, int32_t limit,
    neoclr_aot_text_arena *arena, void *output, int rooted) {
    if (!path || !arena || !output) return 3;
    if (limit < 0) return publish_error(output, 1);
    if (path->length > SIZE_MAX - 1) return 5;
    if (!path->length || memchr(path->bytes, 0, (size_t)path->length))
        return publish_error(output, 2);
    int32_t error = 0, fault = 0;
    input_handle file = open_input(path, &error, &fault);
    if (file == INVALID_INPUT) return fault ? fault : publish_error(output, (uint64_t)error);
    unsigned char *bytes = NULL;
    size_t length = 0, capacity = 0;
    error = regular_input(file);
    if (!error) {
        unsigned char chunk[8192];
        for (;;) {
            size_t remaining = (size_t)limit - length;
            size_t requested = remaining < sizeof(chunk) ? remaining + 1 : sizeof(chunk);
            int64_t count = read_input(file, chunk, requested, &error);
            if (count < 0) break;
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
    close_input(file);
    if (fault || error) {
        free(bytes);
        return fault ? fault : publish_error(output, (uint64_t)error);
    }
    uint64_t result[2] = {0, 0};
    /* The file limit is checked before decoding, and the descriptor is already
     * closed before managed allocation can collect or exhaust the arena. */
    static const unsigned char empty = 0;
    fault = (rooted ? neoclr_text_decode_utf8_host_bytes_rooted_v1 : neoclr_text_decode_utf8_bytes_v1)(bytes ? bytes : &empty, length, arena, result);
    free(bytes);
    if (fault) return fault;
    if (result[0] == 2) return publish_error(output, 8);
    memcpy(output, result, sizeof(result));
    return 0;
}

int32_t neoclr_file_read_utf8_v1(const neoclr_aot_text *path, int32_t limit,
    neoclr_aot_text_arena *arena, void *output) {
    return file_read_utf8(path, limit, arena, output, 0);
}
int32_t neoclr_file_read_utf8_rooted_v1(const neoclr_aot_text *path, int32_t limit,
    neoclr_aot_text_arena *arena, void *output) {
    return file_read_utf8(path, limit, arena, output, 1);
}
