#include "native-gc.h"
#include <string.h>
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
static const struct { uint64_t length; char bytes[5]; } name = {5, "Inner"};
int main(void) {
    uint64_t arena[8192] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)arena, sizeof(arena), 0}};
    const neoclr_aot_text *names[] = {(const void *)&name};
    neoclr_probe_frame frames[66];
    neoclr_probe_storage unused = {0};
    for (unsigned i = 0; i < 66; i++) {
        neoclr_probe_enter_v3(&frames[i], &c, 0, &unused, 0, "{}", 2);
        frames[i].instruction = i;
    }
    const neoclr_aot_text *result = NULL;
    neoclr_aot_fault saved = c.fault;
    CHECK(!neoclr_stack_trace_capture_v1(0, &c, &result, names, 1));
    CHECK(!memcmp(&saved, &c.fault, sizeof(saved)));
    const char first[] = "   at Inner [instruction 65]\n";
    const char last[] = "   ... stack trace truncated\n";
    CHECK(result->length > sizeof(first) && !memcmp(result->bytes, first, sizeof(first) - 1));
    CHECK(!memcmp(result->bytes + result->length - (sizeof(last) - 1), last, sizeof(last) - 1));
    const neoclr_aot_text *unchanged = result;
    CHECK(neoclr_stack_trace_capture_v1(-1, &c, &result, names, 1) == 3 && result == unchanged);
    CHECK(neoclr_stack_trace_capture_v1(0, &c, &result, names, 0) == 3 && result == unchanged);
    const neoclr_aot_text *empty = NULL;
    CHECK(!neoclr_stack_trace_capture_v1(1000, &c, &empty, names, 1) && !empty->length);
    for (unsigned i = 66; i > 0; i--) neoclr_probe_leave_v1(&frames[i - 1]);
    neoclr_probe_storage slot = {&result, 8, 0};
    neoclr_probe_frame root = {.context = &c, .storage = &slot, .storage_count = 1};
    CHECK(!neoclr_gc_collect_v1(&c, &root));
    CHECK(!memcmp(result->bytes, first, sizeof(first) - 1));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    return 0;
}
