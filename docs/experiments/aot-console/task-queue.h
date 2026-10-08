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
} neoclr_task_scope;
int32_t neoclr_task_scope_enter_v1(neoclr_task_scope *scope, neoclr_aot_context *context);
int32_t neoclr_task_scope_leave_v1(neoclr_task_scope *scope);
int32_t neoclr_task_queue_register_v1(void *queue, neoclr_aot_context *context, int32_t *output);
int32_t neoclr_task_queue_default_v1(neoclr_aot_context *context, void **output);
/* run/drain are verified image-local method indices, or -1 when absent. */
int32_t neoclr_task_queue_current_v1(neoclr_aot_context *context, int32_t run, int32_t drain, void **output);
#endif
