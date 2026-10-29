#include "socket-listener.h"
#include "../../../../tools/filewatch-provider/include/neoclr_filewatch.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "watch ownership failed at %d\n", __LINE__); return 1; } } while (0)
extern int32_t neoclr_file_watch_v1(int32_t, const neoclr_aot_text *, const neoclr_aot_text *, int64_t, int32_t, neoclr_aot_context *, void *);
static neoclr_aot_text *text(const char *value) {
    size_t length = strlen(value);
    neoclr_aot_text *result = malloc(sizeof(uint64_t) + length);
    if (!result) abort();
    result->length = length;
    memcpy(result->bytes, value, length);
    return result;
}
int main(int argc, char **argv) {
    CHECK(argc == 2);
    uint64_t storage[8192], other_storage[8192];
    neoclr_aot_context context = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    neoclr_aot_context other = {.text = {(unsigned char *)other_storage, sizeof(other_storage), 0}};
    neoclr_socket_scope scope, second;
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &context));
    neoclr_aot_text *path = text(argv[1]), *filter = text("*.json");
    uint64_t result[2] = {0};
    CHECK(!neoclr_file_watch_v1(0, path, filter, 0, 0, &context, result));
    CHECK(result[0] == 5 && result[1] > 0);
    int64_t handle = (int64_t)result[1];
    CHECK(!neoclr_socket_scope_enter_v1(&second, &other));
    CHECK(!neoclr_file_watch_v1(1, path, filter, handle, 0, &other, result));
    CHECK(result[0] == 2 && result[1] == 5);
    CHECK(!neoclr_socket_scope_leave_v1(&second));
    /* Same scope exit runs after success and guest panic: no explicit Close here. */
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    unsigned char buffer[32];
    CHECK(neoclr_filewatch_read_v1(handle, buffer, sizeof(buffer)) == -5);
    free(path); free(filter);
    puts("PASS invocation watcher ownership and teardown");
    return 0;
}
