#include "native-gc.h"
#include <pthread.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
static void *allocate(neoclr_aot_context *c) {
    void *p = NULL;
    return neoclr_gc_allocate_v1(&c->text, 24, NEOCLR_GC_OBJECT, &p) ? NULL : p;
}
static neoclr_aot_context *foreign_context;
static uint64_t foreign_handle, thread_handle;
static void *check_thread(void *unused) {
    (void)unused;
    void *value = (void *)(uintptr_t)123;
    if (neoclr_gc_host_root_read_v1(foreign_context, foreign_handle, &value) != 3 ||
        value != (void *)(uintptr_t)123) return (void *)(uintptr_t)1;
    neoclr_aot_context c = {0};
    if (neoclr_gc_host_root_create_v1(&c, NULL, &thread_handle) ||
        thread_handle == foreign_handle || neoclr_gc_host_root_release_v1(&c, thread_handle))
        return (void *)(uintptr_t)2;
    return NULL;
}
int main(void) {
    uint64_t buffer[129] = {0}, other_buffer[129] = {0};
    buffer[128] = other_buffer[128] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}};
    neoclr_aot_context other = {.text = {(unsigned char *)other_buffer, 1024, 0}};
    uint64_t *child = allocate(&c), *parent = allocate(&c), *dead = allocate(&c);
    CHECK(child && parent && dead);
    parent[1] = (uintptr_t)child; child[2] = 42;
    uint64_t handle = 999;
    CHECK(neoclr_gc_host_root_create_v1(NULL, parent, &handle) == 3 && handle == 999);
    CHECK(neoclr_gc_host_root_create_v1(&c, parent + 1, &handle) == 3 && handle == 999);
    CHECK(neoclr_gc_host_root_create_v1(&other, parent, &handle) == 3 && handle == 999);
    CHECK(neoclr_gc_host_root_create_v1(&c, parent, NULL) == 3);
    CHECK(!neoclr_gc_host_root_create_v1(&c, parent, &handle));
    CHECK(neoclr_gc_entry_check_v1(&c) == 3 && !neoclr_gc_entry_check_v1(&other));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && child[2] == 42 && parent[1] == (uintptr_t)child);
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == 1);
    void *read = NULL;
    CHECK(!neoclr_gc_host_root_read_v1(&c, handle, &read) && read == parent);
    CHECK(neoclr_gc_host_root_replace_v1(&c, handle, dead) == 3);
    CHECK(neoclr_gc_host_root_replace_v1(&c, handle, (char *)parent + 1) == 3);
    CHECK(!neoclr_gc_host_root_read_v1(&c, handle, &read) && read == parent);
    CHECK(neoclr_gc_host_root_release_v1(&other, handle) == 3);
    foreign_context = &c; foreign_handle = handle;
    pthread_t thread; void *thread_result = NULL;
    CHECK(!pthread_create(&thread, NULL, check_thread, NULL));
    CHECK(!pthread_join(thread, &thread_result) && !thread_result);
    CHECK(neoclr_gc_host_root_read_v1(&c, thread_handle, &read) == 3 && read == parent);
    CHECK(!neoclr_gc_host_root_replace_v1(&c, handle, child));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && child[2] == 42);
    CHECK(neoclr_gc_statistics_v1().reclaimed_allocations == 2);
    CHECK(!neoclr_gc_host_root_replace_v1(&c, handle, NULL));
    CHECK(!neoclr_gc_host_root_read_v1(&c, handle, &read) && !read);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(neoclr_gc_entry_check_v1(&c) == 3);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle) && !neoclr_gc_entry_check_v1(&c));
    CHECK(neoclr_gc_host_root_release_v1(&c, handle) == 3);
    CHECK(neoclr_gc_host_root_read_v1(&c, handle, &read) == 3 && !read);
    uint64_t handles[NEOCLR_GC_HOST_ROOT_LIMIT];
    for (unsigned i = 0; i < NEOCLR_GC_HOST_ROOT_LIMIT; i++) {
        CHECK(!neoclr_gc_host_root_create_v1(i & 1 ? &c : &other, NULL, &handles[i]));
        CHECK(handles[i] > handle && handles[i] > thread_handle);
    }
    uint64_t unchanged = 777;
    CHECK(neoclr_gc_host_root_create_v1(&c, NULL, &unchanged) == 5 && unchanged == 777);
    for (unsigned parity = 0; parity < 2; parity++)
        for (unsigned i = parity; i < NEOCLR_GC_HOST_ROOT_LIMIT; i += 2)
            CHECK(!neoclr_gc_host_root_release_v1(i & 1 ? &c : &other, handles[i]));
    CHECK(!neoclr_gc_entry_check_v1(&c) && !neoclr_gc_entry_check_v1(&other));
    neoclr_probe_frame frame;
    neoclr_probe_storage unused = {0};
    neoclr_probe_enter_v3(&frame, &c, 0, &unused, 0, "", 0);
    CHECK(neoclr_gc_entry_check_v1(&c) == 3 && !neoclr_gc_entry_check_v1(&other));
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    CHECK(buffer[128] == 1234567 && other_buffer[128] == 1234567);
    return 0;
}
