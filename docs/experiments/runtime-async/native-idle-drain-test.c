/* Empty native polls must not manufacture drain fibers. Real callbacks must
 * still trigger drains, including when an earlier drain suspended. */
#include "native-runtime.h"
#include "socket-listener.h"
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "check failed at %d\n", __LINE__); return 1; } } while (0)
static int polls, drains, queued, consumed, suspended;
static uint64_t signal_id;
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c, uint64_t *out, const neoclr_probe_frame *f) {
    (void)f;
    if (polls++ < 3) return NEOCLR_SCHEDULER_PENDING;
    if (polls > 4) return NEOCLR_SCHEDULER_IDLE;
    void *callback = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c->text, 24, NEOCLR_GC_OBJECT, &callback));
    CHECK(!neoclr_gc_host_root_create_v1(c, callback, out));
    return NEOCLR_SCHEDULER_READY;
}
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *c, uint64_t id, const neoclr_probe_frame *f) {
    (void)f;
    return neoclr_gc_host_root_release_v1(c, id);
}
static int32_t invoke(uint64_t handle, neoclr_aot_context *c) {
    (void)handle;
    queued++;
    if (suspended) {
        int32_t result;
        CHECK(!neoclr_runtime_async_completeunit_v1(signal_id, c, &result) && result);
    }
    return 0;
}
static int32_t drain(neoclr_aot_context *c) {
    drains++;
    if (suspended && drains == 1) {
        int32_t result;
        CHECK(!neoclr_runtime_async_create_v1(c, &signal_id));
        CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id, c, &result) && result);
        CHECK(!neoclr_runtime_async_release_v1(signal_id, c, &result) && result);
    }
    consumed += queued;
    queued = 0;
    return 0;
}
int main(void) {
    uint64_t storage[1024] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    for (suspended = 0; suspended < 2; suspended++) {
        polls = drains = queued = consumed = 0;
        CHECK(!neoclr_runtime_async_pump_v1(&c, invoke, drain));
        CHECK(polls == 5 && drains == 2 && consumed == 1 && !queued);
        CHECK(!neoclr_runtime_async_shutdown_v1(&c));
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        CHECK(!neoclr_gc_entry_check_v1(&c));
    }
    puts("PASS empty polls, callback-triggered drains and suspended drain resumption");
    return 0;
}
