/* Real retained-stack lifetime contract; scheduler is idle in this focused host. */
#include "native-runtime.h"
#include "socket-listener.h"
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "check failed at %d\n", __LINE__); return 1; } } while (0)
static uint64_t signal_id;
static int entered, resumed;
static uint64_t *payload;
int32_t neoclr_scheduler_acquire_v1(neoclr_aot_context *c, uint64_t *out, const neoclr_probe_frame *f) {
    (void)c; (void)out; (void)f;
    return NEOCLR_SCHEDULER_IDLE;
}
int32_t neoclr_scheduler_complete_v1(neoclr_aot_context *c, uint64_t id, const neoclr_probe_frame *f) {
    (void)c; (void)id; (void)f;
    return 3;
}
static int32_t invoke(uint64_t handle, neoclr_aot_context *c) {
    void *callback = NULL;
    CHECK(!neoclr_gc_callback_read_v1(c, handle, &callback));
    uint64_t *local = payload;
    neoclr_probe_storage slot = {&local, 8, 0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame, c, 1, &slot, 1, "{}", 2);
    CHECK(!neoclr_gc_collect_v1(c, neoclr_root_probe_head_v1()));
    ++entered;
    int32_t result;
    CHECK(!neoclr_runtime_async_waitcompletion_v1(signal_id, c, &result));
    CHECK(result && local[1] == 42);
    CHECK(!neoclr_gc_collect_v1(c, neoclr_root_probe_head_v1()));
    ++resumed;
    neoclr_probe_leave_v1(&frame);
    return 0;
}
static int32_t drain(neoclr_aot_context *c) { (void)c; return 0; }
static void *allocate(neoclr_aot_context *c) {
    void *p = NULL;
    return neoclr_gc_allocate_v1(&c->text, 24, NEOCLR_GC_OBJECT, &p) ? NULL : p;
}
int main(void) {
    uint64_t storage[1024] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    for (int i = 0; i < 160; ++i) {
        void *owner = allocate(&c);
        uint64_t *callback = allocate(&c);
        payload = allocate(&c);
        CHECK(owner && callback && payload);
        callback[0] = UINT32_MAX; callback[1] = (uintptr_t)owner;
        payload[1] = 42; payload[2] = (uintptr_t)owner;
        uint64_t keep, body;
        CHECK(!neoclr_gc_host_root_create_v1(&c, owner, &keep));
        CHECK(!neoclr_runtime_async_create_v1(&c, &signal_id));
        CHECK(!neoclr_runtime_async_start_owned_v1(owner, callback, &c, &body, invoke));
        CHECK(entered == i + 1 && resumed == i / 2);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        CHECK(payload[1] == 42);
        int32_t result;
        if (!(i & 1)) {
            CHECK(!neoclr_gc_host_root_release_v1(&c, keep));
            CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
            if (!(i & 2)) CHECK(!neoclr_runtime_async_reap_v1(&c));
            /* Other iterations exercise automatic pump reclamation. */
            /* Later external completion must not revive the abandoned body. */
            CHECK(!neoclr_runtime_async_completeunit_v1(signal_id, &c, &result) && result);
            CHECK(!neoclr_runtime_async_pump_v1(&c, invoke, drain));
            CHECK(neoclr_runtime_async_iscompleted_v1(body, &c, &result) == 3);
            CHECK(resumed == i / 2);
        } else {
            CHECK(!neoclr_runtime_async_completeunit_v1(signal_id, &c, &result) && result);
            CHECK(!neoclr_runtime_async_pump_v1(&c, invoke, drain));
            CHECK(resumed == (i + 1) / 2);
            CHECK(!neoclr_runtime_async_iscompleted_v1(body, &c, &result) && result);
            CHECK(!neoclr_runtime_async_release_v1(body, &c, &result) && result);
            CHECK(!neoclr_gc_host_root_release_v1(&c, keep));
            CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
        }
        CHECK(!neoclr_runtime_async_release_v1(signal_id, &c, &result) && result);
        CHECK(!neoclr_gc_entry_check_v1(&c));
    }
    /* Exhaust strong activation capacity after dependent-handle allocation. */
    void *owner = allocate(&c);
    uint64_t *callback = allocate(&c);
    CHECK(owner && callback);
    callback[0] = UINT32_MAX;
    uint64_t handles[254], rejected = 777;
    for (unsigned i = 0; i < 254; ++i)
        CHECK(!neoclr_gc_host_root_create_v1(&c, NULL, &handles[i]));
    CHECK(neoclr_runtime_async_start_owned_v1(owner, callback, &c, &rejected, invoke) == 5);
    CHECK(rejected == 777 && entered == 160);
    for (unsigned i = 0; i < 254; ++i)
        CHECK(!neoclr_gc_host_root_release_v1(&c, handles[i]));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    CHECK(!neoclr_runtime_async_shutdown_v1(&c));
    neoclr_async_statistics stats = neoclr_runtime_async_statistics_v1();
    CHECK(!stats.reserved_bytes && stats.mappings == stats.unmappings);
    printf("{\"entered\":%d,\"resumed\":%d,\"abandoned\":%d,\"reservedBytesAfterShutdown\":%llu}\n",
        entered, resumed, entered - resumed, (unsigned long long)stats.reserved_bytes);
    return 0;
}
