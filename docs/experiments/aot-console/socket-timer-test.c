#include "socket-listener.h"
#include "native-gc.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129] = {0}; buffer[128] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}};
    neoclr_socket_scope scope;
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    void *callback = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    int64_t id = 99;
    CHECK(neoclr_timer_start_v1(-1, callback, &c, &id) == 3 && id == 99);
    CHECK(neoclr_timer_start_v1(0, NULL, &c, &id) == 3 && id == 99);
    CHECK(!neoclr_timer_start_v1(INT32_MAX, callback, &c, &id) && id > 0);
    uint64_t root = 999;
    CHECK(!neoclr_socket_poll_v1(&c, &root) && root == 999);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    int32_t cancelled = 99;
    CHECK(!neoclr_timer_cancel_v1((uint64_t)id, &c, &cancelled) && cancelled);
    CHECK(!neoclr_timer_cancel_v1((uint64_t)id, &c, &cancelled) && !cancelled);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    int64_t first;
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &first));
    CHECK(neoclr_socket_poll_v1(&c, &root) == 1);
    CHECK(!neoclr_timer_cancel_v1((uint64_t)first, &c, &cancelled) && !cancelled);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    void *read = NULL;
    CHECK(!neoclr_gc_host_root_read_v1(&c, root, &read) && read == callback);
    /* Host has installed/invoked the callback before it asks for another. */
    CHECK(!neoclr_socket_poll_v1(&c, &root));
    CHECK(neoclr_gc_host_root_read_v1(&c, root, &read) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    for (unsigned i = 0; i < 64; i++) CHECK(!neoclr_timer_start_v1(INT32_MAX, callback, &c, &id));
    int64_t unchanged = 99;
    CHECK(neoclr_timer_start_v1(0, callback, &c, &unchanged) == 3 && unchanged == 99);
    CHECK(!neoclr_timer_cancel_v1((uint64_t)id, &c, &cancelled) && cancelled);
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &first) && first != id);
    CHECK(!neoclr_timer_cancel_v1((uint64_t)id, &c, &cancelled) && !cancelled);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(buffer[128] == 1234567);
    return 0;
}
