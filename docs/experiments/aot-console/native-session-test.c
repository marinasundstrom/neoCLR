#include "native-session.h"
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <pthread.h>
#endif
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "session check failed: %d\n", __LINE__); return 1; } } while (0)
static neoclr_native_session session;
static uint64_t callback_handle;
/* Kernel stand-in for the generated dispatcher. The compiled fixture separately
 * exercises the same session API with actual generated guest callbacks. */
int32_t neoclr_invoke_void_callback_v1(uint64_t handle, neoclr_aot_context *c) {
    void *value = NULL;
    if (neoclr_gc_callback_read_v1(c, handle, &value)) return 3;
    uint64_t *callback = value;
    uint64_t *counter = (void *)(uintptr_t)callback[2];
    if (callback[1] == 2) {
        uint64_t unchanged = 777;
        CHECK(neoclr_session_close_v1(&session) == 3);
        CHECK(neoclr_session_collect_v1(&session) == 3);
        CHECK(neoclr_session_retain_v1(&session, counter, &unchanged) == 3 && unchanged == 777);
        CHECK(neoclr_session_invoke_v1(&session, handle) == 3);
    }
    counter[1]++;
    if (callback[1] == 1) {
        void *storage = NULL;
        CHECK(!neoclr_gc_allocate_v1(&c->text, 12, NEOCLR_GC_TEXT, &storage));
        neoclr_aot_text *message = storage;
        message->length = 4;
        memcpy(message->bytes, "kept", 4);
        c->fault.message = message;
        c->fault.code = 1;
        return 1;
    }
    return 0;
}
static void *allocate(neoclr_aot_context *c) {
    void *value = NULL;
    if (neoclr_gc_allocate_v1(&c->text, 24, NEOCLR_GC_OBJECT, &value)) return NULL;
    return value;
}
static int foreign_thread(void) {
    uint64_t unchanged = 777;
    CHECK(neoclr_session_invoke_v1(&session, callback_handle) == 3);
    CHECK(neoclr_session_collect_v1(&session) == 3);
    CHECK(neoclr_session_close_v1(&session) == 3);
    CHECK(neoclr_session_retain_v1(&session, NULL, &unchanged) == 3 && unchanged == 777);
    return 0;
}
#ifdef _WIN32
static DWORD WINAPI thread_entry(LPVOID unused) { (void)unused; return (DWORD)foreign_thread(); }
#else
static void *thread_entry(void *unused) { (void)unused; return (void *)(uintptr_t)foreign_thread(); }
#endif
int main(void) {
    uint64_t buffer[513] = {0}; buffer[512] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 4096, 0}}, other = {0};
    uint64_t *counter = allocate(&c), *callback = allocate(&c);
    CHECK(counter && callback);
    counter[0] = 0; counter[1] = 40; counter[2] = 0;
    callback[0] = UINT32_MAX; callback[1] = 0; callback[2] = (uintptr_t)counter;
    CHECK(!neoclr_session_open_v1(&session, &c));
    CHECK(neoclr_gc_entry_check_v1(&c) == 3);
    neoclr_native_session duplicate = {0};
    CHECK(neoclr_session_open_v1(&duplicate, &c) == 3 && !duplicate.owner);
    CHECK(!neoclr_session_retain_v1(&session, callback, &callback_handle));
    uint64_t saved = callback_handle;
    CHECK(!neoclr_session_invoke_v1(&session, callback_handle));
    CHECK(!neoclr_session_collect_v1(&session));
    CHECK(!neoclr_session_invoke_v1(&session, callback_handle) && counter[1] == 42);
    uint64_t unchanged = 777;
    CHECK(neoclr_session_retain_v1(&session, counter + 1, &unchanged) == 3 && unchanged == 777);
    neoclr_native_session copied = session;
    CHECK(neoclr_session_close_v1(&copied) == 3);
    CHECK(neoclr_session_invoke_v1(&copied, callback_handle) == 3);
#ifdef _WIN32
    HANDLE thread = CreateThread(NULL, 0, thread_entry, NULL, 0, NULL);
    DWORD exit_code = 1;
    CHECK(thread && WaitForSingleObject(thread, INFINITE) == WAIT_OBJECT_0);
    CHECK(GetExitCodeThread(thread, &exit_code) && !exit_code && CloseHandle(thread));
#else
    pthread_t thread; void *thread_result = NULL;
    CHECK(!pthread_create(&thread, NULL, thread_entry, NULL));
    CHECK(!pthread_join(thread, &thread_result) && !thread_result);
#endif
    uint64_t foreign;
    CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &foreign));
    CHECK(neoclr_session_invoke_v1(&session, foreign) == 3);
    CHECK(neoclr_session_close_v1(&session) == 3 && counter[1] == 42);
    CHECK(!neoclr_gc_host_root_release_v1(&c, foreign));
    CHECK(neoclr_gc_owned_roots_check_v1(&c, NULL, 1) == 3);
    uint64_t repeated[] = {callback_handle, callback_handle};
    CHECK(neoclr_gc_owned_roots_check_v1(&c, repeated, 2) == 3);
    CHECK(neoclr_gc_owned_roots_check_v1(&other, session.roots, session.count) == 3);
    neoclr_probe_frame frame; neoclr_probe_storage empty = {0};
    neoclr_probe_enter_v3(&frame, &c, 0, &empty, 0, "", 0);
    CHECK(neoclr_session_close_v1(&session) == 3);
    CHECK(neoclr_session_collect_v1(&session) == 3);
    CHECK(neoclr_session_invoke_v1(&session, callback_handle) == 3);
    neoclr_probe_leave_v1(&frame);
    callback[1] = 2;
    CHECK(!neoclr_session_invoke_v1(&session, callback_handle) && counter[1] == 43);
    while (session.count < NEOCLR_SESSION_ROOT_LIMIT + 1)
        CHECK(!neoclr_session_retain_v1(&session, NULL, &unchanged));
    unchanged = 777;
    CHECK(neoclr_session_retain_v1(&session, NULL, &unchanged) == 5 && unchanged == 777);
    callback[1] = 1;
    CHECK(neoclr_session_invoke_v1(&session, callback_handle) == 1 && c.fault.code == 1 && counter[1] == 44);
    CHECK(!neoclr_session_collect_v1(&session) && counter[1] == 44);
    CHECK(c.fault.message->length == 4 && !memcmp(c.fault.message->bytes, "kept", 4));
    CHECK(!neoclr_gc_host_root_create_v1(&c, NULL, &foreign));
    CHECK(neoclr_session_close_v1(&session) == 3 && c.fault.code == 1);
    CHECK(!neoclr_gc_host_root_release_v1(&c, foreign));
    /* Clearing diagnostics externally must not make mutated state dispatchable. */
    c.fault.code = 0;
    CHECK(neoclr_session_invoke_v1(&session, callback_handle) == 3 && counter[1] == 44);
    c.fault.code = 1;
    CHECK(!neoclr_session_close_v1(&session) && !c.text.used && !c.fault.code);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    CHECK(neoclr_session_close_v1(&session) == 3);
    CHECK(neoclr_session_invoke_v1(&session, saved) == 3);
    void *value = (void *)(uintptr_t)777;
    CHECK(neoclr_gc_host_root_read_v1(&c, saved, &value) == 3 && value == (void *)(uintptr_t)777);
    CHECK(!neoclr_session_open_v1(&session, &c));
    const struct { uint64_t length; unsigned char bytes[4]; } literal = {4, {'t','e','s','t'}};
    const neoclr_aot_text *interned = NULL;
    CHECK(!neoclr_string_intern_v1((const neoclr_aot_text *)&literal, &c.text, &interned));
    CHECK(!neoclr_session_collect_v1(&session) && c.text.used);
    CHECK(!neoclr_session_close_v1(&session) && !c.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&c) && buffer[512] == 1234567);
    return 0;
}
