/* Diagnostic-only attribution to published managed frames, not machine stacks.
 * Inclusive rows overlap; recursion contributes once per function/allocation. */
#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <inttypes.h>
typedef struct { uint64_t blocks, bytes, text_blocks, text_bytes; } site_count;
static site_count exclusive[NEOCLR_NATIVE_FUNCTION_LIMIT + 1];
static site_count inclusive[NEOCLR_NATIVE_FUNCTION_LIMIT + 1];
static uint64_t seen[NEOCLR_NATIVE_FUNCTION_LIMIT], epoch;
static int measuring;
static void add(site_count *count, uint32_t kind, uint64_t bytes) {
    count->blocks++; count->bytes += bytes;
    if (kind == NEOCLR_GC_TEXT) { count->text_blocks++; count->text_bytes += bytes; }
}
void neoclr_allocation_sites_allocated(uint32_t kind, uint64_t bytes) {
    if (!measuring) return;
    if (!++epoch) { memset(seen, 0, sizeof(seen)); epoch = 1; }
    const neoclr_probe_frame *head = neoclr_root_probe_head_v1();
    unsigned id = head ? head->function : NEOCLR_NATIVE_FUNCTION_LIMIT;
    if (head && id >= NEOCLR_NATIVE_FUNCTION_LIMIT) abort();
    add(&exclusive[id], kind, bytes);
    if (!head) add(&inclusive[id], kind, bytes);
    for (const neoclr_probe_frame *f = head; f; f = f->previous) {
        id = f->function;
        if (id >= NEOCLR_NATIVE_FUNCTION_LIMIT) abort();
        if (seen[id] == epoch) continue;
        seen[id] = epoch;
        add(&inclusive[id], kind, bytes);
    }
}
void neoclr_allocation_sites_request(uint64_t requests) {
    if (requests == 8) measuring = 1;
    if (requests == 32) measuring = 0;
}
static void write_count(FILE *file, site_count count) {
    fprintf(file, "{\"blocks\":%" PRIu64 ",\"bytes\":%" PRIu64
        ",\"textBlocks\":%" PRIu64 ",\"textBytes\":%" PRIu64 "}",
        count.blocks, count.bytes, count.text_blocks, count.text_bytes);
}
void neoclr_allocation_sites_report(FILE *file) {
    fputs("{\"sites\":[", file);
    int comma = 0;
    for (unsigned id = 0; id <= NEOCLR_NATIVE_FUNCTION_LIMIT; id++) {
        if (!inclusive[id].blocks) continue;
        if (comma++) fputc(',', file);
        fprintf(file, "{\"function\":%d,\"exclusive\":", id == NEOCLR_NATIVE_FUNCTION_LIMIT ? -1 : (int)id);
        write_count(file, exclusive[id]);
        fputs(",\"inclusive\":", file); write_count(file, inclusive[id]); fputc('}', file);
    }
    fputs("]}\n", file);
}
