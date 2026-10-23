#include "snapshot.h"
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static const char *names[] = {
    "collections", "allocations", "reclaimedAllocations", "reclaimedBytes",
    "rootPublications", "gcAttempts", "gcNanoseconds", "gcFailures",
    "allocatedBytes", "asyncFibers", "asyncResumes", "stackMappings", "stackUnmappings"
};
_Static_assert(sizeof(names) / sizeof(*names) == NEOCLR_METRIC_COUNT, "metric names");
static void write_snapshot(FILE *f, const neoclr_runtime_snapshot *s) {
    fputs("{", f);
    for (unsigned i = 0; i < NEOCLR_METRIC_COUNT; i++) {
        fprintf(f, "%s\"%s\":", i ? "," : "", names[i]);
        if (s->available & (UINT64_C(1) << i)) fprintf(f, "%" PRIu64, s->values[i]);
        else fputs("null", f);
    }
    fputs("}", f);
}
void neoclr_http_workers_snapshot_v1(const neoclr_runtime_snapshot *s, const int *valid, unsigned count) {
    const char *path = getenv("NEOCLR_HTTP_RUNTIME_SNAPSHOT");
    if (!path) return;
    /* The benchmark treats absent/incomplete output as a failed diagnostic run.
     * Diagnostic failure itself must not abort the application. */
    if (!count || count > 4) return;
    neoclr_runtime_snapshot total = s[0];
    for (unsigned i = 0; i < count; i++) {
        if (!valid[i]) return;
        if (i && !neoclr_runtime_snapshot_merge_v1(&total, &s[i], &total)) return;
    }
    FILE *f = fopen(path, "w");
    if (!f) return;
    fprintf(f, "{\"version\":1,\"scope\":\"completed isolated HTTP application invocations\",\"workers\":[");
    for (unsigned i = 0; i < count; i++) {
        if (i) fputs(",", f);
        write_snapshot(f, &s[i]);
    }
    fputs("],\"total\":", f);
    write_snapshot(f, &total);
    fputs("}\n", f);
    int failed = ferror(f);
    if (fclose(f)) failed = 1;
    if (failed) remove(path);
}
