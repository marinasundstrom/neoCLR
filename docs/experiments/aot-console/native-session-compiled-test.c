#include "native-session.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
int main(void) {
    uint64_t buffer[257] = {0}; buffer[256] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 2048, 0}}, other = {0};
    for (int mode = 0; mode < 3; mode++) {
        uint64_t handle = 777;
        CHECK(!neoclr_bootstrap_callback_v1(mode, &handle, &c) && handle != 777);
        /* The export itself roots the callback: collect before ownership transfer. */
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t untouched = 777;
        uint64_t before = c.text.used;
        CHECK(neoclr_bootstrap_callback_v1(mode, &untouched, &c) == 3);
        CHECK(untouched == 777 && c.text.used == before && !c.fault.code);
        neoclr_native_session session = {0};
        CHECK(neoclr_session_adopt_v1(&session, &other, handle) == 3 && !session.owner);
        uint64_t foreign;
        CHECK(!neoclr_gc_host_root_create_v1(&c, NULL, &foreign));
        CHECK(neoclr_session_adopt_v1(&session, &c, handle) == 3 && !session.owner);
        CHECK(!neoclr_gc_host_root_release_v1(&c, foreign));
        CHECK(!neoclr_session_adopt_v1(&session, &c, handle));
        CHECK(neoclr_session_adopt_v1(&session, &c, handle) == 3);
        void *value = NULL;
        CHECK(!neoclr_gc_host_root_read_v1(&c, handle, &value));
        uint64_t *callback = value; /* Inspect only for the counter assertion. */
        CHECK(neoclr_invoke_void_callback_v1(handle, &other) == 3);
        neoclr_probe_frame frame;
        neoclr_probe_storage empty = {0};
        neoclr_probe_enter_v3(&frame, &c, 0, &empty, 0, "", 0);
        CHECK(neoclr_session_invoke_v1(&session, handle) == 3 && !c.fault.code);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t used = c.text.used;
        if (mode == 2) {
            CHECK(neoclr_session_invoke_v1(&session, handle) == 1 && c.fault.code == 1);
            CHECK(c.fault.frame_count == 1);
            CHECK(neoclr_bootstrap_callback_v1(0, &untouched, &c) == 3);
            CHECK(untouched == 777 && c.fault.code == 1 && c.fault.frame_count == 1);
            CHECK(!neoclr_aot_render_fault(stderr, &c.fault));
            CHECK(neoclr_session_invoke_v1(&session, handle) == 3 && c.fault.code == 1);
        } else {
            CHECK(!neoclr_session_invoke_v1(&session, handle));
            CHECK(!neoclr_gc_collect_v1(&c, NULL));
            CHECK(!neoclr_session_invoke_v1(&session, handle));
            if (!mode) CHECK(*(int32_t *)(uintptr_t)(callback[2] + 8) == 2);
        }
        CHECK(c.text.used == used && !neoclr_root_probe_depth_v1());
        CHECK(!neoclr_session_close_v1(&session));
        CHECK(neoclr_session_invoke_v1(&session, handle) == 3);
        CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    }
    uint64_t untouched = 777;
    CHECK(neoclr_bootstrap_callback_v1(0, NULL, &c) == 3 && !c.text.used);
    CHECK(neoclr_bootstrap_callback_v1(0, &untouched, NULL) == 3 && untouched == 777);
    CHECK(neoclr_bootstrap_callback_v1(3, &untouched, &c) == 1 && untouched == 777);
    CHECK(c.fault.code == 1 && !neoclr_gc_entry_check_v1(&c));
    /* Failed bootstrap transfers nothing; a new admitted bootstrap resets it. */
    CHECK(!neoclr_bootstrap_callback_v1(1, &untouched, &c));
    CHECK(!neoclr_gc_host_root_release_v1(&c, untouched));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used);
    neoclr_native_session stale = {0};
    CHECK(neoclr_session_adopt_v1(&stale, &c, untouched) == 3 && !stale.owner);
    /* Root quota exhaustion is atomic both at export and at adoption. */
    uint64_t occupied[256];
    for (unsigned i = 0; i < 256; i++)
        CHECK(!neoclr_gc_host_root_create_v1(&other, NULL, &occupied[i]));
    untouched = 777;
    CHECK(neoclr_bootstrap_callback_v1(1, &untouched, &c) == 5 && untouched == 777);
    CHECK(!c.fault.code && !neoclr_gc_entry_check_v1(&c));
    CHECK(!neoclr_gc_host_root_release_v1(&other, occupied[255]));
    CHECK(!neoclr_bootstrap_callback_v1(1, &untouched, &c));
    CHECK(neoclr_session_adopt_v1(&stale, &c, untouched) == 5 && !stale.owner);
    CHECK(!neoclr_gc_collect_v1(&c, NULL));
    for (unsigned i = 0; i < 255; i++)
        CHECK(!neoclr_gc_host_root_release_v1(&other, occupied[i]));
    CHECK(!neoclr_session_adopt_v1(&stale, &c, untouched));
    CHECK(!neoclr_session_invoke_v1(&stale, untouched));
    CHECK(!neoclr_session_close_v1(&stale));
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_v1(&c, NULL, &handle));
    CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    void *ordinary = NULL, *output = (void *)(uintptr_t)123;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 24, NEOCLR_GC_OBJECT, &ordinary));
    CHECK(!neoclr_gc_host_root_create_v1(&c, ordinary, &handle));
    CHECK(neoclr_gc_callback_read_v1(&c, handle, &output) == 3 && output == (void *)(uintptr_t)123);
    CHECK(neoclr_invoke_void_callback_v1(handle, &c) == 3);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    CHECK(buffer[256] == 1234567);
    return 0;
}
