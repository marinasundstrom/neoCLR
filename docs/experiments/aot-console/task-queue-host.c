#include "native-gc.h"
#include "task-queue.h"
int main(void) {
    uint64_t buffer[8193];
    buffer[8192] = UINT64_C(0x1122334455667788);
    neoclr_aot_context context = {.text = {(unsigned char *)buffer, 65536, 0}};
    neoclr_task_scope tasks;
    if (neoclr_task_scope_enter_v1(&tasks, &context)) return 2;
    int32_t result = -99;
    int32_t status = neoclr_entry_v4(0, &result, &context);
    int32_t entry_status = status;
#ifdef NEOCLR_HOST_TASK_PUMP
    if (!status) status = neoclr_drain_default_queue_v1(&context);
#endif
    if (neoclr_root_probe_head_v1() || buffer[8192] != UINT64_C(0x1122334455667788)) return 2;
    if (neoclr_task_scope_leave_v1(&tasks)) return 2;
    if (status) {
        if ((entry_status && result != -99) || context.fault.code != (uint32_t)status) return 2;
        if (neoclr_aot_render_fault(stderr, &context.fault)) return 2;
        context.fault = (neoclr_aot_fault){0};
    }
    if (neoclr_gc_collect_v1(&context, NULL) || context.text.used) return 2;
    return status ? 1 : result;
}
