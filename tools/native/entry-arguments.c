/* Private process argument adapter. The rooted adapter establishes a
 * temporary construction frame before the guest entry frame exists. */
#include "entry-arguments.h"
#include <string.h>
#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdlib.h>
#endif
static int32_t process_arguments(int32_t argc, neoclr_process_char **argv,
    neoclr_aot_context *context, void **output, void **array, int rooted) {
    if (argc < 1 || !argv || !context || !output) return 3;
    int32_t status = rooted ? neoclr_allocate_strings_rooted_v1(&context->text, argc - 1, 0, array)
                            : neoclr_allocate_strings_v1(&context->text, argc - 1, 0, array);
    if (status) return status;
    for (int32_t i = 1; i < argc; i++) {
        if (!argv[i]) return 3;
        uint64_t decoded[2] = {0};
#ifdef _WIN32
        /* Use the wide CRT entry, never the active ANSI code page. Invalid UTF-16
         * fails explicitly, consistent with strict UTF-8 at the Unix boundary. */
        int bytes = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, argv[i], -1, NULL, 0, NULL, NULL);
        if (!bytes) return 3;
        if ((uint64_t)bytes > context->text.capacity) return 5;
        char *utf8 = malloc((size_t)bytes);
        if (!utf8) return 5;
        if (WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, argv[i], -1, utf8, bytes, NULL, NULL) != bytes) {
            free(utf8); return 3;
        }
        status = (rooted ? neoclr_text_decode_utf8_host_bytes_rooted_v1 : neoclr_text_decode_utf8_bytes_v1)((const unsigned char *)utf8,
            (uint64_t)bytes - 1, &context->text, decoded);
        free(utf8);
#else
        status = (rooted ? neoclr_text_decode_utf8_host_bytes_rooted_v1 : neoclr_text_decode_utf8_bytes_v1)((const unsigned char *)argv[i],
            (uint64_t)strlen(argv[i]), &context->text, decoded);
#endif
        if (status) return status;
        if (decoded[0] != 4) return 3;
        ((uint64_t *)*array)[1 + i] = decoded[1];
    }
    *output = *array;
    return 0;
}

int32_t neoclr_process_arguments_v1(int32_t argc, neoclr_process_char **argv,
    neoclr_aot_context *context, void **output) {
    void *array = NULL;
    return process_arguments(argc, argv, context, output, &array, 0);
}
int32_t neoclr_process_arguments_rooted_v1(int32_t argc, neoclr_process_char **argv,
    neoclr_aot_context *context, void **output) {
    if (argc < 1 || !argv || !context || !output) return 3;
    void *array = NULL;
#ifdef NEOCLR_NATIVE_GC
    neoclr_probe_frame frame;
    neoclr_probe_storage root = {&array, sizeof(array), 0};
    neoclr_probe_enter_v3(&frame, context, 0, &root, 1, "{}", 2);
#endif
    int32_t status = process_arguments(argc, argv, context, output, &array, 1);
#ifdef NEOCLR_NATIVE_GC
    neoclr_probe_leave_v1(&frame);
#endif
    return status;
}
