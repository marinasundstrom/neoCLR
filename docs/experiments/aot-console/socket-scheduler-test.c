#include "socket-listener.h"
#include "native-gc.h"
#include "task-queue.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
static int32_t acquire_entry(neoclr_aot_context *c, uint64_t *ticket,
    const neoclr_probe_frame *boundary, void *state) {
    (void)state;
    return neoclr_scheduler_acquire_v1(c, ticket, boundary);
}
static int32_t reject_completion(neoclr_aot_context *c, uint64_t ticket,
    const neoclr_probe_frame *boundary) {
    (void)c; (void)ticket; (void)boundary;
    return 3;
}
static int entry_delivery_contract(void) {
    uint64_t buffer[128] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}};
    neoclr_socket_scope sockets;
    neoclr_task_scope tasks;
    CHECK(!neoclr_socket_scope_enter_v1(&sockets, &c));
    CHECK(!neoclr_task_scope_enter_v1(&tasks, &c));
    tasks.poll = acquire_entry;
    tasks.complete = neoclr_scheduler_complete_v1;
    void *callback = NULL, *read = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    int64_t timer;
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &timer));
    neoclr_probe_frame entry, child;
    neoclr_probe_storage empty = {0};
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(!neoclr_entry_tasks_begin_v1(&c, 7, -1, -1, &read));
    uint64_t ticket = 0;
    CHECK(neoclr_entry_tasks_poll_v1(&c, 1, &ticket) == 1);
    uint64_t first = ticket;
    CHECK(neoclr_entry_tasks_poll_v1(&c, 1, &ticket) == -3 && ticket == first);
    CHECK(neoclr_entry_callback_read_v1(&c, first + 1, &read) == 3);
    CHECK(!neoclr_entry_callback_read_v1(&c, first, &read) && read == callback);
    CHECK(neoclr_entry_callback_read_v1(&c, first, &read) == 3);
    neoclr_probe_enter_v3(&child, &c, 8, &empty, 0, "", 0);
    CHECK(neoclr_entry_tasks_poll_v1(&c, 1, &ticket) == -3);
    CHECK(!neoclr_gc_collect_v1(&c, &child) && c.text.used);
    neoclr_probe_leave_v1(&child);
    CHECK(!neoclr_entry_tasks_poll_v1(&c, 0, &ticket));
    CHECK(neoclr_gc_host_root_read_v1(&c, first, &read) == 3);
    CHECK(!tasks.entry_delivery && !sockets.scheduler_delivery);
    /* End, rather than another poll, acknowledges the final faulting callback. */
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &timer));
    CHECK(neoclr_entry_tasks_poll_v1(&c, 1, &ticket) == 1);
    CHECK(!neoclr_entry_callback_read_v1(&c, ticket, &read));
    c.fault.code = 4;
    neoclr_entry_tasks_end_v1(&c);
    CHECK(!tasks.entry_delivery && !sockets.scheduler_delivery && c.fault.code == 4);
    CHECK(neoclr_gc_host_root_read_v1(&c, ticket, &read) == 3);
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_socket_scope_leave_v1(&sockets));
    CHECK(!neoclr_task_scope_leave_v1(&tasks));
    c.fault.code = 0;
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    /* A failing completion hook is terminal and reported by scope leave. */
    CHECK(!neoclr_socket_scope_enter_v1(&sockets, &c));
    CHECK(!neoclr_task_scope_enter_v1(&tasks, &c));
    tasks.poll = acquire_entry;
    tasks.complete = reject_completion;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_timer_start_v1(0, callback, &c, &timer));
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(!neoclr_entry_tasks_begin_v1(&c, 7, -1, -1, &read));
    CHECK(neoclr_entry_tasks_poll_v1(&c, 1, &ticket) == 1);
    c.fault.code = 4;
    neoclr_entry_tasks_end_v1(&c);
    CHECK(tasks.completion_error && c.fault.code == 4);
    c.fault.code = 0;
    CHECK(neoclr_entry_tasks_begin_v1(&c, 7, -1, -1, &read) == 3);
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_socket_scope_leave_v1(&sockets));
    CHECK(neoclr_task_scope_leave_v1(&tasks) == 3);
    CHECK(neoclr_gc_host_root_read_v1(&c, ticket, &read) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    return 0;
}
static int submitted_work_contract(void) {
    uint64_t buffer[128] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}}, other = {0};
    neoclr_socket_scope scope;
    CHECK(neoclr_scheduler_submit_v1(&c, 1, NULL) == 3);
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    void *callback = NULL, *read = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    uint64_t original;
    CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &original));
    neoclr_socket_scope foreign;
    CHECK(!neoclr_socket_scope_enter_v1(&foreign, &other));
    CHECK(neoclr_scheduler_submit_v1(&other, original, NULL) == 3);
    CHECK(!neoclr_socket_scope_leave_v1(&foreign));
    CHECK(neoclr_scheduler_submit_v1(&c, 0, NULL) == 3);
    c.fault.code = 4;
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == 3 && c.fault.code == 4);
    c.fault.code = 0;
    neoclr_probe_frame entry, child;
    neoclr_probe_storage empty = {0};
    int32_t admitted = 99;
    CHECK(neoclr_scheduler_submit_managed_v1(callback, &c, &admitted) == 3 && admitted == 99);
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_submit_managed_v1(callback, &c, NULL) == 3);
    CHECK(!neoclr_scheduler_submit_managed_v1(callback, &c, &admitted) && admitted == 0);
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == 3);
    neoclr_probe_enter_v3(&child, &c, 8, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_submit_v1(&c, original, &entry) == 3);
    neoclr_probe_leave_v1(&child);
    CHECK(!neoclr_scheduler_submit_v1(&c, original, &entry));
    neoclr_probe_leave_v1(&entry);
    for (unsigned i = 2; i < 64; i++) CHECK(!neoclr_scheduler_submit_v1(&c, original, NULL));
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
    CHECK(!neoclr_gc_host_root_read_v1(&c, original, &read) && read == callback);
    CHECK(!neoclr_gc_host_root_release_v1(&c, original));
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    uint64_t delivery = 0;
    for (unsigned i = 0; i < 64; i++) {
        CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_READY);
        CHECK(!neoclr_gc_callback_read_v1(&c, delivery, &read) && read == callback);
        CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == -3);
        CHECK(!neoclr_scheduler_complete_v1(&c, delivery, NULL));
    }
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    /* A terminal fault abandons queued work without invoking it. */
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &original));
    CHECK(!neoclr_scheduler_submit_v1(&c, original, NULL));
    CHECK(!neoclr_gc_host_root_release_v1(&c, original));
    c.fault.code = 4;
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == -3);
    CHECK(!neoclr_socket_scope_leave_v1(&scope) && c.fault.code == 4);
    c.fault.code = 0;
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    return 0;
}
int main(void) {
    CHECK(!submitted_work_contract());
    CHECK(!entry_delivery_contract());
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
