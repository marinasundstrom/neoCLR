#include "socket-listener.h"
#include "native-gc.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129] = {0}; buffer[128] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}}, other = {0};
    neoclr_socket_scope scope;
    uint64_t root = 999;
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == -3 && root == 999);
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    CHECK(neoclr_scheduler_step_v1(&c, NULL, NULL) == -3);
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_IDLE && root == 999);
    void *callback = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    int64_t late, immediate;
    CHECK(!neoclr_timer_start_v1(INT32_MAX, callback, &c, &late));
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_PENDING && root == 999);
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &immediate));
    neoclr_probe_frame entry, child;
    neoclr_probe_storage empty = {0};
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == -3 && root == 999);
    CHECK(neoclr_scheduler_step_v1(&other, &root, &entry) == -3 && root == 999);
    neoclr_probe_enter_v3(&child, &c, 8, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_step_v1(&c, &root, &entry) == -3 && root == 999);
    neoclr_probe_leave_v1(&child);
    CHECK(neoclr_scheduler_step_v1(&c, &root, &entry) == NEOCLR_SCHEDULER_READY);
    CHECK(!neoclr_gc_collect_v1(&c, &entry) && c.text.used);
    void *read = NULL;
    CHECK(!neoclr_gc_callback_read_suspended_v1(&c, root, &read, &entry) && read == callback);
    int32_t cancelled = 99;
    CHECK(!neoclr_timer_cancel_v1((uint64_t)immediate, &c, &cancelled) && !cancelled);
    CHECK(!neoclr_timer_cancel_v1((uint64_t)late, &c, &cancelled) && cancelled);
    /* The callback has been consumed. Advancing to idle releases its delivery root. */
    uint64_t delivered = root;
    CHECK(neoclr_scheduler_step_v1(&c, &root, &entry) == NEOCLR_SCHEDULER_IDLE && root == delivered);
    CHECK(neoclr_gc_host_root_read_v1(&c, delivered, &read) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, &entry) && !c.text.used);
    neoclr_probe_leave_v1(&entry);
    /* Delivery is not consumption: a completed socket outcome is still owned. */
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    const struct { uint64_t length; char bytes[16]; } address = {9, "127.0.0.1"};
    uint64_t result[2] = {99, 99};
    CHECK(!neoclr_socket_listen_v1((const void *)&address, 0, 4, &c, result) && result[0] == 5);
    uint64_t listener = result[1];
    CHECK(!neoclr_socket_accept_v1(listener, callback, &c, result) && result[0] == 5);
    uint64_t operation = result[1];
    CHECK(!neoclr_socket_cancel_v1(operation, &c, &cancelled) && cancelled);
    CHECK(neoclr_scheduler_acquire_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_READY);
    delivered = root;
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == -3 && root == delivered);
    CHECK(neoclr_socket_poll_v1(&c, &root) == -3 && root == delivered);
    CHECK(neoclr_scheduler_acquire_v1(&c, &root, NULL) == -3 && root == delivered);
    CHECK(neoclr_scheduler_complete_v1(&other, delivered, NULL) == 3);
    CHECK(neoclr_scheduler_complete_v1(&c, delivered + 1, NULL) == 3);
    CHECK(!neoclr_socket_cancel_v1(operation, &c, &cancelled) && !cancelled);
    CHECK(!neoclr_socket_connect_result_v1(operation, &c, result) && result[0] == 2 && result[1] == 5);
    /* Consuming the source result must not release active dispatch ownership. */
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    CHECK(!neoclr_gc_host_root_read_v1(&c, delivered, &read) && read == callback);
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_complete_v1(&c, delivered, NULL) == 3);
    CHECK(neoclr_scheduler_complete_v1(&c, delivered, &entry) == 3);
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_scheduler_complete_v1(&c, delivered, NULL));
    CHECK(neoclr_scheduler_complete_v1(&c, delivered, NULL) == 3);
    CHECK(neoclr_gc_host_root_read_v1(&c, delivered, &read) == 3);
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    /* Dispatch completion does not consume an outstanding source outcome. */
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_socket_accept_v1(listener, callback, &c, result) && result[0] == 5);
    operation = result[1];
    CHECK(!neoclr_socket_cancel_v1(operation, &c, &cancelled) && cancelled);
    CHECK(neoclr_scheduler_acquire_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_READY);
    CHECK(!neoclr_scheduler_complete_v1(&c, root, NULL));
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_PENDING);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    CHECK(!neoclr_socket_connect_result_v1(operation, &c, result));
    CHECK(neoclr_scheduler_step_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    /* Complete at the exact entry boundary, even after a guest fault. */
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &immediate));
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_acquire_v1(&c, &root, &entry) == NEOCLR_SCHEDULER_READY);
    CHECK(root != delivered);
    CHECK(neoclr_scheduler_complete_v1(&c, delivered, &entry) == 3);
    neoclr_probe_enter_v3(&child, &c, 8, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_complete_v1(&c, root, &entry) == 3);
    neoclr_probe_leave_v1(&child);
    c.fault.code = 3;
    CHECK(!neoclr_scheduler_complete_v1(&c, root, &entry) && c.fault.code == 3);
    c.fault.code = 0;
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &immediate));
    CHECK(neoclr_scheduler_acquire_v1(&c, &root, NULL) == NEOCLR_SCHEDULER_READY);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(neoclr_gc_host_root_read_v1(&c, root, &read) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(buffer[128] == 1234567);
    return 0;
}
