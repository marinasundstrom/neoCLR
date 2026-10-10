/* Private process argument adapter. All managed allocations use the invocation
 * heap and cannot collect here; no guest frame exists until the entry call. */
#include "entry-arguments.h"
#include <string.h>
#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <stdlib.h>
#endif
int32_t neoclr_process_arguments_v1(int32_t argc, neoclr_process_char **argv,
    neoclr_aot_context *context, void **output) {
    if (argc < 1 || !argv || !context || !output) return 3;
    void *array;
    int32_t status = neoclr_allocate_strings_v1(&context->text, argc - 1, 0, &array);
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
        status = neoclr_text_decode_utf8_bytes_v1((const unsigned char *)utf8,
            (uint64_t)bytes - 1, &context->text, decoded);
        free(utf8);
#else
        status = neoclr_text_decode_utf8_bytes_v1((const unsigned char *)argv[i],
            (uint64_t)strlen(argv[i]), &context->text, decoded);
#endif
        if (status) return status;
        if (decoded[0] != 4) return 3;
        ((uint64_t *)array)[1 + i] = decoded[1];
    }
    *output = array;
    return 0;
}
