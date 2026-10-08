#include "task-queue.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[129] = {0}; buffer[128] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 1024, 0}}, other = {0};
    neoclr_task_scope scope, nested;
    void *queue = NULL, *explicit_queue = NULL, *read = (void *)(uintptr_t)123;
    CHECK(neoclr_task_queue_default_v1(&c, &read) == 3 && read == (void *)(uintptr_t)123);
    CHECK(!neoclr_task_scope_enter_v1(&scope, &c));
    CHECK(neoclr_task_scope_enter_v1(&nested, &c) == 3);
    CHECK(!neoclr_task_queue_default_v1(&c, &read) && !read);
    CHECK(!neoclr_task_queue_host_read_v1(&c, &read) && !read);
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &queue));
    int32_t unit = -99;
    CHECK(neoclr_task_queue_register_v1(NULL, &c, &unit) == 3 && unit == -99);
    CHECK(!neoclr_task_queue_register_v1(queue, &c, &unit) && !unit);
    unit = -99;
    CHECK(neoclr_task_queue_register_v1(queue, &c, &unit) == 3 && unit == -99);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && c.text.used);
    CHECK(!neoclr_task_queue_current_v1(&c, -1, -1, &read) && read == queue);
    CHECK(!neoclr_gc_allocate_v1(&c.text, 16, NEOCLR_GC_OBJECT, &explicit_queue));
    neoclr_probe_frame frame;
    neoclr_probe_storage receiver = {&explicit_queue, 8, 0};
    neoclr_probe_enter_v3(&frame, &c, 42, &receiver, 1, "", 0);
    CHECK(!neoclr_gc_collect_v1(&c, neoclr_root_probe_head_v1()));
    CHECK(!neoclr_task_queue_current_v1(&c, 42, -1, &read) && read == explicit_queue);
    CHECK(!neoclr_task_queue_current_v1(&c, -1, 42, &read) && read == explicit_queue);
    CHECK(!neoclr_task_queue_default_v1(&c, &read) && read == queue);
    read = (void *)(uintptr_t)123;
    CHECK(neoclr_task_queue_host_read_v1(&c, &read) == 3 && read == (void *)(uintptr_t)123);
    CHECK(neoclr_task_scope_leave_v1(&scope) == 3);
    CHECK(!neoclr_task_scope_enter_v1(&nested, &other));
    CHECK(!neoclr_task_queue_current_v1(&other, 42, -1, &read) && !read);
    CHECK(neoclr_task_scope_leave_v1(&scope) == 3);
    CHECK(!neoclr_task_scope_leave_v1(&nested));
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_task_queue_current_v1(&c, 42, -1, &read) && read == queue);
    CHECK(!neoclr_task_queue_host_read_v1(&c, &read) && read == queue);
    c.fault.code = 9; read = (void *)(uintptr_t)123;
    CHECK(neoclr_task_queue_host_read_v1(&c, &read) == 3 && read == (void *)(uintptr_t)123 && c.fault.code == 9);
    c.fault.code = 0;
    CHECK(!neoclr_task_scope_leave_v1(&scope));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used && !neoclr_gc_entry_check_v1(&c));
    CHECK(buffer[128] == 1234567);
    return 0;
}
