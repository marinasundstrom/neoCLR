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
    CHECK(!neoclr_schedule_task_managed_v1(callback, &c, NULL));
    CHECK(admitted == 99);
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == 3);
    neoclr_probe_enter_v3(&child, &c, 8, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_submit_v1(&c, original, &entry) == 3);
    neoclr_probe_leave_v1(&child);
    CHECK(!neoclr_scheduler_submit_v1(&c, original, &entry));
    neoclr_probe_leave_v1(&entry);
    for (unsigned i = 2; i < NEOCLR_GC_HOST_ROOT_LIMIT - 1; i++) CHECK(!neoclr_scheduler_submit_v1(&c, original, NULL));
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT);
    CHECK(!neoclr_gc_host_root_read_v1(&c, original, &read) && read == callback);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    uint64_t delivery = 0;
    /* Keep the caller's root: first acquire must work at complete root saturation. */
    for (unsigned i = 0; i < NEOCLR_GC_HOST_ROOT_LIMIT - 1; i++) {
        CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_READY);
        CHECK(!neoclr_gc_callback_read_v1(&c, delivery, &read) && read == callback);
        CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == -3);
        CHECK(!neoclr_scheduler_complete_v1(&c, delivery, NULL));
    }
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_gc_host_root_release_v1(&c, original));
    CHECK(neoclr_scheduler_submit_v1(&c, original, NULL) == 3);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    /* A terminal fault abandons queued work without invoking it. */
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &original));
    for (unsigned i = 0; i < 128; i++) CHECK(!neoclr_scheduler_submit_v1(&c, original, NULL));
    CHECK(!neoclr_gc_host_root_release_v1(&c, original));
    CHECK(neoclr_scheduler_step_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_READY);
    CHECK(scope.runnable_delivery == delivery);
    c.fault.code = 4;
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == -3);
    CHECK(!neoclr_socket_scope_leave_v1(&scope) && c.fault.code == 4);
    c.fault.code = 0;
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    return 0;
}
static int destination_contract(void) {
    uint64_t buffer[128] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}}, other = {0};
    neoclr_socket_scope scope, foreign;
    int64_t destination = -9, again = -9, other_destination = -9;
    CHECK(neoclr_scheduler_capture_v1(&c, &destination, NULL) == 3 && destination == -9);
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    CHECK(neoclr_scheduler_capture_v1(&c, NULL, NULL) == 3);
    CHECK(!neoclr_scheduler_capture_v1(&c, &destination, NULL) && destination > 0);
    CHECK(!neoclr_scheduler_capture_v1(&c, &again, NULL) && again == destination);
    CHECK(!neoclr_socket_scope_enter_v1(&foreign, &other));
    CHECK(!neoclr_scheduler_capture_v1(&other, &other_destination, NULL));
    CHECK(other_destination != destination);
    void *callback = NULL, *read = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    uint64_t input = 0, delivery = 0;
    CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &input));
    CHECK(neoclr_scheduler_submit_to_v1(&other, destination, input, NULL) == 3);
    CHECK(neoclr_scheduler_submit_to_v1(&c, other_destination, input, NULL) == 3);
    CHECK(neoclr_scheduler_submit_to_v1(&c, 0, input, NULL) == 3);
    CHECK(neoclr_scheduler_step_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_socket_scope_leave_v1(&foreign));
    neoclr_probe_frame entry, child;
    neoclr_probe_storage empty = {0};
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    again = -9;
    CHECK(neoclr_scheduler_capture_v1(&c, &again, NULL) == 3 && again == -9);
    CHECK(!neoclr_scheduler_capture_managed_v1(&c, &again) && again == destination);
    CHECK(neoclr_scheduler_capture_managed_v1(&other, &again) == 3);
    neoclr_probe_enter_v3(&child, &c, 8, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_capture_v1(&c, &again, &entry) == 3);
    CHECK(neoclr_scheduler_submit_to_v1(&c, destination, input, &entry) == 3);
    neoclr_probe_leave_v1(&child);
    int32_t result = -9;
    CHECK(neoclr_scheduler_submit_to_managed_v1(other_destination, callback, &c, &result) == 3 && result == -9);
    CHECK(!neoclr_scheduler_submit_to_managed_v1(destination, callback, &c, &result) && result == 0);
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_gc_host_root_release_v1(&c, input));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_READY);
    CHECK(!neoclr_gc_host_root_read_v1(&c, delivery, &read) && read == callback);
    CHECK(!neoclr_scheduler_complete_v1(&c, delivery, NULL));
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    CHECK(!neoclr_scheduler_capture_v1(&c, &again, NULL) && again != destination);
    CHECK(neoclr_scheduler_submit_to_v1(&c, destination, input, NULL) == 3);
    CHECK(neoclr_scheduler_step_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    return 0;
}

static int registration_contract(void) {
    uint64_t buffer[128] = {0}, delivery = 0;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}};
    neoclr_socket_scope scope;
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    int64_t destination = 0, registrations[64], rejected = -9;
    CHECK(!neoclr_scheduler_capture_v1(&c, &destination, NULL));
    void *callback = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(neoclr_scheduler_register_managed_v1(destination, callback, &c, &rejected) == 3 && rejected == -9);
    neoclr_probe_frame entry;
    neoclr_probe_storage empty = {0};
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(neoclr_scheduler_register_managed_v1(0, callback, &c, &rejected) == 3 && rejected == -9);
    CHECK(neoclr_scheduler_register_managed_v1(destination, callback, &c, NULL) == 3);
    for (unsigned i = 0; i < 64; i++)
        CHECK(!neoclr_scheduler_register_managed_v1(destination, callback, &c, &registrations[i]));
    CHECK(neoclr_scheduler_register_managed_v1(destination, callback, &c, &rejected) == NEOCLR_AOT_FAULT_NATIVE_MEMORY_LIMIT && rejected == -9);
    int32_t changed = -9;
    CHECK(!neoclr_scheduler_submit_managed_v1(callback, &c, &changed) && changed == 0);
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, &entry) == NEOCLR_SCHEDULER_READY);
    CHECK(!neoclr_scheduler_complete_v1(&c, delivery, &entry));
    CHECK(!neoclr_gc_collect_v1(&c, &entry) && c.text.used);
    CHECK(neoclr_scheduler_step_v1(&c, &delivery, &entry) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_scheduler_activate_managed_v1(registrations[0], &c, &changed) && changed);
    CHECK(!neoclr_scheduler_activate_managed_v1(registrations[0], &c, &changed) && !changed);
    CHECK(!neoclr_scheduler_cancel_managed_v1(registrations[0], &c, &changed) && !changed);
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, &entry) == NEOCLR_SCHEDULER_READY);
    CHECK(!neoclr_scheduler_complete_v1(&c, delivery, &entry));
    for (unsigned i = 1; i < 64; i++) {
        CHECK(!neoclr_scheduler_cancel_managed_v1(registrations[i], &c, &changed) && changed);
        CHECK(!neoclr_scheduler_cancel_managed_v1(registrations[i], &c, &changed) && !changed);
        CHECK(!neoclr_scheduler_activate_managed_v1(registrations[i], &c, &changed) && !changed);
    }
    CHECK(neoclr_scheduler_step_v1(&c, &delivery, &entry) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!neoclr_gc_collect_v1(&c, &entry) && !c.text.used);
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    CHECK(!neoclr_scheduler_register_managed_v1(destination, callback, &c, &rejected));
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    neoclr_probe_enter_v3(&entry, &c, 7, &empty, 0, "", 0);
    CHECK(!neoclr_scheduler_activate_managed_v1(rejected, &c, &changed) && !changed);
    CHECK(!neoclr_scheduler_cancel_managed_v1(rejected, &c, &changed) && !changed);
    neoclr_probe_leave_v1(&entry);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    return 0;
}

static int backlog_and_sources_contract(void) {
    uint64_t buffer[128] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, sizeof(buffer), 0}};
    neoclr_socket_scope scope;
    CHECK(!neoclr_socket_scope_enter_v1(&scope, &c));
    void *callback = NULL;
    CHECK(!neoclr_allocate_object_v1(&c.text, UINT32_MAX, 24, &callback));
    ((uint64_t *)callback)[1] = 1;
    uint64_t original = 0, delivery = 0;
    CHECK(!neoclr_gc_host_root_create_v1(&c, callback, &original));
    int64_t timer;
    for (unsigned i = 0; i < 64; i++) CHECK(!neoclr_timer_start_v1(0, callback, &c, &timer));
    /* All I/O slots occupied, but runnable admission is independent. */
    for (unsigned i = 0; i < 128; i++) CHECK(!neoclr_scheduler_submit_v1(&c, original, NULL));
    CHECK(!neoclr_gc_host_root_release_v1(&c, original));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    for (unsigned i = 0; i < 192; i++) {
        neoclr_runnable *before = scope.runnable_head;
        CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_READY);
        if (i < 128) CHECK((scope.runnable_head != before) == (i % 2 == 1));
        void *read = NULL;
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
        CHECK(!neoclr_gc_callback_read_v1(&c, delivery, &read) && read == callback);
        CHECK(!neoclr_scheduler_complete_v1(&c, delivery, NULL));
        CHECK(neoclr_scheduler_complete_v1(&c, delivery, NULL) == 3);
    }
    CHECK(neoclr_scheduler_acquire_v1(&c, &delivery, NULL) == NEOCLR_SCHEDULER_IDLE);
    CHECK(!scope.runnable_head && !scope.runnable_tail && !scope.runnable_delivery);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_socket_scope_leave_v1(&scope));
    return 0;
}

int main(void) {
    CHECK(!backlog_and_sources_contract());
    CHECK(!registration_contract());
    CHECK(!destination_contract());
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
