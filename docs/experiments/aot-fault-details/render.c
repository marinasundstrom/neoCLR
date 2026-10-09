#include "fault-details.h"
#include <inttypes.h>
static void text(FILE *stream, const neoclr_aot_text *value) {
    if (value) fwrite(value->bytes, 1, (size_t)value->length, stream);
}
int neoclr_aot_render_fault(FILE *stream, const neoclr_aot_fault *fault) {
    if (!fault->code) return 0;
    const char *category = fault->code == 1 ? "DivideByZero" :
                           fault->code == 2 ? "ArithmeticOverflow" :
                           fault->code == 3 ? "RuntimeError" :
                           fault->code == 4 ? "UserFault" :
                           fault->code == 5 ? "NativeMemoryLimitExceeded" :
                           fault->code == 6 ? "NullReference" :
                           fault->code == 7 ? "ArrayLimitExceeded" :
                           fault->code == 8 ? "IndexOutOfRange" :
                           fault->code == 9 ? "StackOverflow" :
                           fault->code == 10 ? "InternPoolLimitExceeded" : "UnknownFault";
    fprintf(stream, "%s: ", category);
    text(stream, fault->message);
    fputc('\n', stream);
    for (uint32_t i = 0; i < fault->frame_count && i < 64; ++i) {
        fputs("   at ", stream);
        text(stream, fault->frames[i].function);
        fprintf(stream, " [instruction %" PRIu32 "]\n", fault->frames[i].instruction);
    }
    if (fault->truncated) fputs("   ... stack trace truncated\n", stream);
    return ferror(stream) ? -1 : 0;
}
