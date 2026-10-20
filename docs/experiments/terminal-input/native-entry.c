/* Exercise the real console host cleanup ordering with an instrumented guest entry.
 * The collector/renderer are stubs here; managed execution is checked separately. */
#include "native-gc.h"
#include "console.h"
#include <assert.h>
#include <stdlib.h>
#include <unistd.h>

static struct termios original;
#ifndef NEOCLR_TEST_REAL_HOST
static void restored(void) {
    struct termios current;
    assert(tcgetattr(STDIN_FILENO, &current) == 0);
    assert((current.c_lflag & (ICANON | ECHO | ISIG)) ==
           (original.c_lflag & (ICANON | ECHO | ISIG)));
}
#endif
int32_t neoclr_entry_v4(int32_t value, int32_t *result, neoclr_aot_context *context) {
    (void)value;
    assert(tcgetattr(STDIN_FILENO, &original) == 0);
    assert(neoclr_terminal_scope_open_v1(context, 1) > 0);
    struct termios current;
    assert(tcgetattr(STDIN_FILENO, &current) == 0);
    assert((current.c_lflag & (ICANON | ECHO | ISIG)) == 0);
    *result = 0;
    if (getenv("NEOCLR_TEST_TERMINAL_FAULT")) {
        context->fault.code = NEOCLR_AOT_FAULT_USER_FAULT;
        return NEOCLR_AOT_FAULT_USER_FAULT;
    }
    return 0;
}
#ifndef NEOCLR_TEST_REAL_HOST
const neoclr_probe_frame *neoclr_root_probe_head_v1(void) { return NULL; }
int neoclr_aot_render_fault(FILE *stream, const neoclr_aot_fault *fault) {
    (void)stream;
    assert(fault->code == NEOCLR_AOT_FAULT_USER_FAULT);
    restored();
    return 0;
}
int32_t neoclr_gc_collect_v1(neoclr_aot_context *context, const neoclr_probe_frame *head) {
    assert(!head);
    assert(neoclr_terminal_scope_open_v1(context, 1) == -5);
    restored();
    return 0;
}

#endif
#ifdef NEOCLR_TEST_JIT
#include <string.h>
extern int32_t neoclr_jit_run_v1(const void *, int32_t, int32_t, const char **);
int main(int argc, char **argv) {
    int32_t (*entry)(int32_t, int32_t *, neoclr_aot_context *) = neoclr_entry_v4;
    const void *pointer;
    _Static_assert(sizeof(pointer) == sizeof(entry), "entry pointer size");
    memcpy(&pointer, &entry, sizeof(pointer));
    int32_t first = neoclr_jit_run_v1(pointer, 0, argc, (const char **)argv);
    int32_t second = neoclr_jit_run_v1(pointer, 0, argc, (const char **)argv);
    assert(first == second);
    return second;
}
#endif
