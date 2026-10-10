#ifndef NEOCLR_ENTRY_ARGUMENTS_H
#define NEOCLR_ENTRY_ARGUMENTS_H
#include "native-gc.h"
#ifdef _WIN32
#include <wchar.h>
#define NEOCLR_PROCESS_MAIN wmain
typedef wchar_t neoclr_process_char;
#else
#define NEOCLR_PROCESS_MAIN main
typedef char neoclr_process_char;
#endif
/* Private process-entry ABI, not a general embedding API. argc/argv include the
 * executable; the generated adapter copies argv[1..] after admission/reset.
 * Windows argv is UTF-16, other hosts supply strict UTF-8. Never retained. */
int32_t neoclr_entry_args_v1(int32_t argc, int32_t *result,
    neoclr_aot_context *context, neoclr_process_char **argv);
int32_t neoclr_process_arguments_v1(int32_t argc, neoclr_process_char **argv,
    neoclr_aot_context *context, void **output);
static inline int32_t neoclr_process_entry(int argc, neoclr_process_char **argv,
    int32_t *result, neoclr_aot_context *context) {
#ifdef NEOCLR_ENTRY_ARGUMENTS
    return neoclr_entry_args_v1(argc, result, context, argv);
#else
    (void)argc; (void)argv;
    return neoclr_entry_v4(0, result, context);
#endif
}
#endif
