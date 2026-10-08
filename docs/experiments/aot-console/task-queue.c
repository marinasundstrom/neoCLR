#include "task-queue.h"
#include <string.h>
static _Thread_local neoclr_task_scope *head;
static neoclr_task_scope *find(neoclr_aot_context *context) {
    for (neoclr_task_scope *scope = head; scope; scope = scope->previous)
        if (scope->context == context) return scope;
    return NULL;
}
int32_t neoclr_task_scope_enter_v1(neoclr_task_scope *scope, neoclr_aot_context *context) {
    if (!scope || !context || find(context)) return 3;
    for (neoclr_task_scope *s = head; s; s = s->previous) if (s == scope) return 3;
    *scope = (neoclr_task_scope){context, head, 0}; head = scope;
    return 0;
}
int32_t neoclr_task_scope_leave_v1(neoclr_task_scope *scope) {
    if (!scope || head != scope) return 3;
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context == scope->context) return 3;
    if (scope->default_root && neoclr_gc_host_root_release_v1(scope->context, scope->default_root)) return 3;
    head = scope->previous; *scope = (neoclr_task_scope){0};
    return 0;
}
int32_t neoclr_task_queue_register_v1(void *queue, neoclr_aot_context *context, int32_t *output) {
    neoclr_task_scope *scope = find(context);
    if (!scope || !queue || !output || scope->default_root) return 3;
    int32_t status = neoclr_gc_host_root_create_v1(context, queue, &scope->default_root);
    if (status) return status;
    *output = 0; /* inhabited Void */
    return 0;
}
int32_t neoclr_task_queue_default_v1(neoclr_aot_context *context, void **output) {
    neoclr_task_scope *scope = find(context);
    if (!scope || !output) return 3;
    if (scope->default_root) return neoclr_gc_host_root_read_v1(context, scope->default_root, output);
    *output = NULL; return 0;
}
int32_t neoclr_task_queue_current_v1(neoclr_aot_context *context, int32_t run, int32_t drain, void **output) {
    if (!find(context) || !output) return 3;
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous) {
        if (f->context != context || !((run >= 0 && f->function == (uint32_t)run) ||
            (drain >= 0 && f->function == (uint32_t)drain))) continue;
        /* Compiler publication orders arguments first; TaskQueue receiver is slot zero. */
        if (!f->storage_count || f->storage[0].read_bytes != 8 || !f->storage[0].address) return 3;
        void *queue; memcpy(&queue, f->storage[0].address, sizeof(queue));
        if (!queue) return 3;
        *output = queue; return 0;
    }
    return neoclr_task_queue_default_v1(context, output);
}
int32_t neoclr_task_queue_host_read_v1(neoclr_aot_context *context, void **output) {
    if (!context || context->fault.code || !output || !find(context)) return 3;
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context == context) return 3;
    return neoclr_task_queue_default_v1(context, output);
}
