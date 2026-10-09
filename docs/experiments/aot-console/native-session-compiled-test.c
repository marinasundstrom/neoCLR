#include "native-session.h"
#define CHECK(x) do { if (!(x)) return __LINE__; } while (0)
/* Fixture-only discovery of the single descriptor left in a guest local at return.
 * Real services receive and root callbacks during submission, never scan the heap. */
static uint64_t *descriptor(neoclr_aot_context *c) {
    for (uint64_t at = 0; at < c->text.used;) {
        uint64_t *header = (void *)(c->text.data + at);
        if (header[3] && header[1] == 24 && header[2] == NEOCLR_GC_OBJECT && header[4] == UINT32_MAX)
            return header + 4;
        at += header[0];
    }
    return NULL;
}
int main(void) {
    uint64_t buffer[257] = {0}; buffer[256] = 1234567;
    neoclr_aot_context c = {.text = {(unsigned char *)buffer, 2048, 0}}, other = {0};
    for (int mode = 0; mode < 4; mode++) {
        int32_t result = -1;
        CHECK(!neoclr_entry_v4(mode, &result, &c) && result == 42);
        uint64_t *callback = descriptor(&c);
        CHECK(callback);
        uint64_t handle = 0;
        neoclr_native_session session = {0};
        CHECK(!neoclr_session_open_v1(&session, &c));
        CHECK(!neoclr_session_retain_v1(&session, callback, &handle));
        CHECK(neoclr_invoke_void_callback_v1(handle, &other) == 3);
        neoclr_probe_frame frame;
        neoclr_probe_storage empty = {0};
        neoclr_probe_enter_v3(&frame, &c, 0, &empty, 0, "", 0);
        CHECK(neoclr_session_invoke_v1(&session, handle) == 3 && !c.fault.code);
        neoclr_probe_leave_v1(&frame);
        CHECK(!neoclr_gc_collect_v1(&c, NULL));
        uint64_t used = c.text.used;
        if (mode == 3) {
            CHECK(neoclr_session_invoke_v1(&session, handle) == 3 && !c.fault.code);
        } else if (mode == 2) {
            CHECK(neoclr_session_invoke_v1(&session, handle) == 1 && c.fault.code == 1);
            CHECK(c.fault.frame_count == 1);
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
