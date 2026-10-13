#include "native-gc.h"
#include <string.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "failed line %d\n", __LINE__); return 1; } } while (0)
static const struct { uint64_t length; char bytes[5]; } name = {5, "Inner"};
int main(void) {
    uint64_t memory[64] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)memory, sizeof(memory), 0}};
    const neoclr_aot_text *names[] = {(const void *)&name};
    const neoclr_aot_text *trace = NULL;
    void *live = NULL, *dead = NULL, *filler = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 64, NEOCLR_GC_BYTES, &live));
    *(uint64_t *)live = 42;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 320, NEOCLR_GC_BYTES, &dead));
    neoclr_probe_storage slots[] = {{&live, 8, 0}, {&trace, 8, 0}, {&filler, 8, 0}};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, &c, 0, slots, 3, "{}", 2);
    frame.instruction = 7;
    neoclr_aot_fault saved = c.fault;
    uint64_t collections = neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_stack_trace_capture_v1(0, &c, &trace, names, 1));
    CHECK(neoclr_gc_statistics_v1().collections == collections + 1);
    const char expected[] = "   at Inner [instruction 7]\n";
    CHECK(trace->length == sizeof(expected) - 1 && !memcmp(trace->bytes, expected, sizeof(expected) - 1));
    CHECK(*(uint64_t *)live == 42 && !memcmp(&saved, &c.fault, sizeof(saved)));
    const neoclr_aot_text *unchanged = trace;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 248, NEOCLR_GC_BYTES, &filler));
    *(uint64_t *)filler = 123;
    collections = neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_stack_trace_capture_v1(0, &c, &trace, names, 1) == 5);
    CHECK(trace == unchanged && neoclr_gc_statistics_v1().collections == collections + 1);
    CHECK(*(uint64_t *)live == 42 && *(uint64_t *)filler == 123);
    CHECK(!memcmp(trace->bytes, expected, sizeof(expected) - 1));
    collections = neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_stack_trace_capture_v1(-1, &c, &trace, names, 1) == 3);
    CHECK(neoclr_stack_trace_capture_v1(0, &c, &trace, names, 0) == 3);
    CHECK(trace == unchanged && neoclr_gc_statistics_v1().collections == collections);
    /* Foreign top frames preserve the old noncollecting capture behavior. */
    neoclr_aot_context foreign = {0};
    neoclr_probe_frame other;
    neoclr_probe_enter_v3(&other, &foreign, 0, slots, 0, "{}", 2);
    CHECK(neoclr_stack_trace_capture_v1(0, &c, &trace, names, 1) == 5);
    CHECK(trace == unchanged && neoclr_gc_statistics_v1().collections == collections);
    neoclr_probe_leave_v1(&other);
    /* An impossible snapshot does not collect or publish a partial result. */
    neoclr_probe_frame many[64];
    for (unsigned i = 0; i < 64; i++) neoclr_probe_enter_v3(&many[i], &c, 0, slots, 0, "{}", 2);
    CHECK(neoclr_stack_trace_capture_v1(0, &c, &trace, names, 1) == 5);
    CHECK(trace == unchanged && neoclr_gc_statistics_v1().collections == collections);
    for (unsigned i = 64; i > 0; i--) neoclr_probe_leave_v1(&many[i-1]);
    CHECK(neoclr_root_probe_head_v1() == &frame && !memcmp(&saved, &c.fault, sizeof(saved)));
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    /* Quiescent capture also reclaims dead storage when the empty text cannot fit. */
    CHECK(!neoclr_gc_allocate_v1(&c.text, 472, NEOCLR_GC_BYTES, &dead));
    collections = neoclr_gc_statistics_v1().collections;
    CHECK(!neoclr_stack_trace_capture_v1(0, &c, &trace, names, 1) && !trace->length);
    CHECK(neoclr_gc_statistics_v1().collections == collections + 1);
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used && !neoclr_gc_entry_check_v1(&c));
    puts("PASS collecting helper retry, live roots, exhaustion, foreign context and quiescent capture");
    return 0;
}
