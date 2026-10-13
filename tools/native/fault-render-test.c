#include "fault-details.h"
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
int main(void) {
    FILE *stream = tmpfile();
    CHECK(stream != NULL);
    neoclr_aot_fault fault = {0};
    CHECK(!neoclr_aot_render_fault(stream, &fault) && ftell(stream) == 0);
    static const struct { uint64_t length; unsigned char bytes[4]; } message = {4, {'a',0,'b','c'}};
    static const struct { uint64_t length; unsigned char bytes[4]; } function = {4, {'M','a','i','n'}};
    fault.code = NEOCLR_AOT_FAULT_USER_FAULT;
    fault.message = (const void *)&message;
    fault.frame_count = 1;
    fault.frames[0].function = (const void *)&function;
    fault.frames[0].instruction = 42;
    fault.truncated = 1;
    neoclr_aot_fault saved = fault;
    CHECK(!neoclr_aot_render_fault(stream, &fault));
    CHECK(!memcmp(&saved, &fault, sizeof(fault)));
    CHECK(!fseek(stream, 0, SEEK_SET));
    char buffer[256] = {0};
    size_t count = fread(buffer, 1, sizeof(buffer), stream);
    const char expected[] = "UserFault: a\0bc\n   at Main [instruction 42]\n   ... stack trace truncated\n";
    CHECK(count == sizeof(expected)-1 && !memcmp(buffer, expected, count));
    CHECK(!fclose(stream));
    return 0;
}
