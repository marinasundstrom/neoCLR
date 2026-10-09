#include "native-session.h"
#include <string.h>
enum { READY = 1, RUNNING = 2, FAULTED = 3 };
static int owned(neoclr_native_session *s) {
    void *anchor = NULL;
    return s && s->owner == s && s->count && s->count <= NEOCLR_SESSION_ROOT_LIMIT + 1 &&
        !neoclr_gc_host_root_read_v1(s->context, s->roots[0], &anchor) && !anchor;
}
static int quiescent(neoclr_native_session *s) {
    if (!owned(s) || s->phase == RUNNING) return 0;
    for (const neoclr_probe_frame *f = neoclr_root_probe_head_v1(); f; f = f->previous)
        if (f->context == s->context) return 0;
    return 1;
}
int32_t neoclr_session_open_v1(neoclr_native_session *s, neoclr_aot_context *c) {
    if (!s || s->owner || s->phase || !c || c->fault.code || neoclr_gc_entry_check_v1(c)) return 3;
    uint64_t anchor = 0;
    int32_t status = neoclr_gc_host_root_create_v1(c, NULL, &anchor);
    if (status) return status;
    *s = (neoclr_native_session){.owner = s, .context = c, .roots = {anchor}, .count = 1, .phase = READY};
    return 0;
}
int32_t neoclr_session_adopt_v1(neoclr_native_session *s, neoclr_aot_context *c, uint64_t handle) {
    void *callback = NULL;
    if (!s || s->owner || s->phase || !c ||
        neoclr_gc_callback_read_v1(c, handle, &callback) ||
        neoclr_gc_owned_roots_check_v1(c, &handle, 1)) return 3;
    uint64_t anchor = 0;
    int32_t status = neoclr_gc_host_root_create_v1(c, NULL, &anchor);
    if (status) return status;
    *s = (neoclr_native_session){.owner = s, .context = c,
        .roots = {anchor, handle}, .count = 2, .phase = READY};
    return 0;
}
int32_t neoclr_session_retain_v1(neoclr_native_session *s, void *value, uint64_t *output) {
    if (!quiescent(s) || s->phase != READY || s->context->fault.code || !output) return 3;
    if (s->count == NEOCLR_SESSION_ROOT_LIMIT + 1) return 5;
    uint64_t handle;
    int32_t status = neoclr_gc_host_root_create_v1(s->context, value, &handle);
    if (status) return status;
    s->roots[s->count++] = handle;
    *output = handle;
    return 0;
}
int32_t neoclr_session_invoke_v1(neoclr_native_session *s, uint64_t handle) {
    if (!quiescent(s) || s->phase != READY || s->context->fault.code) return 3;
    uint32_t i = 1;
    for (; i < s->count && s->roots[i] != handle; i++) { }
    if (i == s->count) return 3;
    s->phase = RUNNING;
    int32_t status = neoclr_invoke_void_callback_v1(handle, s->context);
    s->phase = s->context->fault.code ? FAULTED : READY;
    return status;
}
int32_t neoclr_session_collect_v1(neoclr_native_session *s) {
    if (!quiescent(s)) return 3;
    return neoclr_gc_collect_v1(s->context, NULL);
}
int32_t neoclr_session_close_v1(neoclr_native_session *s) {
    if (!quiescent(s) || neoclr_gc_owned_roots_check_v1(s->context, s->roots, s->count)) return 3;
    /* Validate heap/fault roots before changing ownership. Diagnostic roots stay
     * live until this explicit discard boundary. Services must have shut down. */
    int32_t status = neoclr_gc_collect_v1(s->context, NULL);
    if (status) return status;
    for (uint32_t i = s->count; i; i--)
        if (neoclr_gc_host_root_release_v1(s->context, s->roots[i - 1])) return 3;
    memset(&s->context->fault, 0, sizeof(s->context->fault));
    /* Exclusive ownership permits whole-heap reset, including the per-entry
     * intern pool. This does not execute guest finalizers or close OS resources. */
    s->context->text.used = 0;
    *s = (neoclr_native_session){0};
    return 0;
}
