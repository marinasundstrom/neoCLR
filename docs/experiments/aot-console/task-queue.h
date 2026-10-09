#ifndef NEOCLR_AOT_TASK_QUEUE_H
#define NEOCLR_AOT_TASK_QUEUE_H
#include "native-gc.h"
/* Private invocation adapter, not a public scheduler or suspension representation.
 * Enter/leave around guest entry AND subsequent completion dispatch. Thread-affine,
 * LIFO distinct contexts. Teardown releases the registered default queue root. */
typedef struct neoclr_task_scope {
    neoclr_aot_context *context;
    struct neoclr_task_scope *previous;
    uint64_t default_root;
    int entry_draining;
    const neoclr_probe_frame *entry_boundary;
    /* Installed by the native host before entry; 0 idle, 1 callback, 2 pending, -3 fault. */
    int32_t (*poll)(neoclr_aot_context *, uint64_t *, const neoclr_probe_frame *, void *);
    void *poll_state;
} neoclr_task_scope;
int32_t neoclr_task_scope_enter_v1(neoclr_task_scope *scope, neoclr_aot_context *context);
int32_t neoclr_task_scope_leave_v1(neoclr_task_scope *scope);
int32_t neoclr_task_queue_register_v1(void *queue, neoclr_aot_context *context, int32_t *output);
int32_t neoclr_task_queue_default_v1(neoclr_aot_context *context, void **output);
/* run/drain are verified image-local method indices, or -1 when absent. */
int32_t neoclr_task_queue_current_v1(neoclr_aot_context *context, int32_t run, int32_t drain, void **output);
/* Quiescent, fault-free host reader; no default queue is a successful null result.
 * Generated drain entry retains heap/fault state and invokes the ordinary CIL body.
 * Misuse returns 3 without replacing an existing guest fault. */
int32_t neoclr_task_queue_host_read_v1(neoclr_aot_context *context, void **output);
int32_t neoclr_drain_default_queue_v1(neoclr_aot_context *context);
/* Entry lifecycle: startup stays on the published native frame stack.
 * Begin rejects nesting, queue callbacks and calls outside the selected entry.
 * End must run on both success and guest fault before returning from the wrapper. */
int32_t neoclr_entry_tasks_begin_v1(neoclr_aot_context *context, int32_t entry,
    int32_t run, int32_t drain, void **output);
void neoclr_entry_tasks_end_v1(neoclr_aot_context *context);
int32_t neoclr_entry_tasks_poll_v1(neoclr_aot_context *, int32_t required, uint64_t *);
int32_t neoclr_entry_callback_read_v1(neoclr_aot_context *, uint64_t, void **);
int32_t neoclr_invoke_entry_callback_v1(uint64_t, neoclr_aot_context *);
#endif
