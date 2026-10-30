#include "native-gc.h"
#include "file-input.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#endif
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "file input check failed at %d\n", __LINE__); return 1; } } while (0)
static const char filename[] = "input-\xc3\xa9-\xf0\x9f\x8c\x8d.txt";
static neoclr_aot_text *text(const void *bytes, size_t length) {
    neoclr_aot_text *value = malloc(sizeof(uint64_t) + length);
    if (!value) abort();
    value->length = length;
    memcpy(value->bytes, bytes, length);
    return value;
}
static int save(const void *bytes, size_t length) {
#ifdef _WIN32
    FILE *file = NULL;
    CHECK(!_wfopen_s(&file, L"input-\u00e9-\U0001f30d.txt", L"wb"));
#else
    FILE *file = fopen(filename, "wb");
#endif
    CHECK(file);
    CHECK(fwrite(bytes, 1, length, file) == length);
    CHECK(!fclose(file));
    return 0;
}
static int check(const void *name, size_t name_length, int32_t limit,
    const void *expected, size_t length, uint64_t error) {
    neoclr_aot_text *path = text(name, name_length);
    for (int rooted = 0; rooted < 2; rooted++) {
        uint64_t storage[4096];
        neoclr_aot_context context = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
        neoclr_probe_storage empty = {0};
        neoclr_probe_frame frame;
        if (rooted) neoclr_probe_enter_v3(&frame, &context, 0, &empty, 0, "{}", 2);
        uint64_t result[2] = {0};
        int32_t status = (rooted ? neoclr_file_read_utf8_rooted_v1 : neoclr_file_read_utf8_v1)
            (path, limit, &context.text, result);
        CHECK(!status);
        CHECK(result[0] == (error ? 2u : 4u));
        if (error) CHECK(result[1] == error);
        else {
            const neoclr_aot_text *actual = (const neoclr_aot_text *)(uintptr_t)result[1];
            CHECK(actual->length == length);
            CHECK(!memcmp(actual->bytes, expected, length));
        }
        if (rooted) neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&context, NULL));
    }
    free(path);
    return 0;
}
int main(void) {
    const unsigned char bytes[] = {0xef, 0xbb, 0xbf, 'a', 0, 0xc3, 0xa9, '\r', '\n'};
    CHECK(!save(bytes, sizeof(bytes)));
    CHECK(!check(filename, strlen(filename), (int32_t)sizeof(bytes), bytes, sizeof(bytes), 0));
    CHECK(!check(filename, strlen(filename), (int32_t)sizeof(bytes) - 1, "", 0, 7));
    CHECK(!check(filename, strlen(filename), 0, "", 0, 7));
    CHECK(!check("", 0, -1, "", 0, 1));
    CHECK(!check("", 0, 0, "", 0, 2));
    CHECK(!check("bad\0path", 8, 0, "", 0, 2));
    CHECK(!check("missing-input", 13, 32, "", 0, 3));
    CHECK(!check(".", 1, 32, "", 0, 5));
    CHECK(!save("", 0));
    CHECK(!check(filename, strlen(filename), 0, "", 0, 0));
    const unsigned char invalid[] = {0xff, 'a'};
    CHECK(!save(invalid, sizeof(invalid)));
    CHECK(!check(filename, strlen(filename), 2, "", 0, 8));
    CHECK(!check(filename, strlen(filename), 1, "", 0, 7));
    unsigned char large[9000]; memset(large, 'x', sizeof(large));
    CHECK(!save(large, sizeof(large)));
    CHECK(!check(filename, strlen(filename), 9000, large, sizeof(large), 0));
    CHECK(!check(filename, strlen(filename), 8999, "", 0, 7));
#ifdef _WIN32
    CHECK(!check("\xff", 1, 32, "", 0, 2));
    CHECK(!check("NUL", 3, 32, "", 0, 5));
    HANDLE locked = CreateFileW(L"input-\u00e9-\U0001f30d.txt", GENERIC_READ, 0, NULL, OPEN_EXISTING, 0, NULL);
    CHECK(locked != INVALID_HANDLE_VALUE);
    CHECK(!check(filename, strlen(filename), 9000, "", 0, 4));
    CHECK(CloseHandle(locked));
    DWORD before = 0, after = 0;
    CHECK(GetProcessHandleCount(GetCurrentProcess(), &before));
#endif
    neoclr_aot_text *path = text(filename, strlen(filename));
    for (int i = 0; i < 128; i++) {
        neoclr_aot_context tiny = {0};
        uint64_t result[2] = {77, 88};
        CHECK(neoclr_file_read_utf8_v1(path, 9000, &tiny.text, result) == 5);
        CHECK(result[0] == 77 && result[1] == 88);
        CHECK(!check(filename, strlen(filename), 8999, "", 0, 7));
    }
    free(path);
#ifdef _WIN32
    CHECK(GetProcessHandleCount(GetCurrentProcess(), &after) && before == after);
    CHECK(DeleteFileW(L"input-\u00e9-\U0001f30d.txt"));
#else
    CHECK(!remove(filename));
#endif
    puts("PASS bounded native file input, UTF-8, error precedence and handle cleanup");
    return 0;
}
