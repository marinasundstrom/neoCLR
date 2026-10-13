/* Admission must validate the entire heap before publishing a matching base. */
#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "check failed at %d\n", __LINE__); return 1; } } while (0)
static void cleanup(void *state) { (*(int *)state)++; }
int main(void) {
    uint64_t memory[128] = {0}, foreign_memory[16] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)memory, sizeof(memory), 0}};
    neoclr_aot_context foreign = {.text = {(unsigned char *)foreign_memory, sizeof(foreign_memory), 0}};
    void *first = NULL, *middle = NULL, *last = NULL, *alien = NULL;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 24, NEOCLR_GC_OBJECT, &first));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 24, NEOCLR_GC_OBJECT, &middle));
    CHECK(!neoclr_gc_allocate_v1(&c.text, 24, NEOCLR_GC_OBJECT, &last));
    CHECK(!neoclr_gc_allocate_v1(&foreign.text, 24, NEOCLR_GC_OBJECT, &alien));
    *(uint64_t *)first = UINT32_MAX; /* callback object discriminator */
    uint64_t root = 0, tail_root = 0, handle = 123;
    CHECK(!neoclr_gc_host_root_create_v1(&c, first, &root));
    CHECK(!neoclr_gc_host_root_create_v1(&c, last, &tail_root));
    uint64_t bytes = 456;
    uint32_t kind = 789;
    void *read = alien;
    neoclr_gc_resource resource = {0}, empty_resource = {0};
    neoclr_parked_roots parked = {0}, empty_parked = {0};
    int cleaned = 0;
    /* Private descriptor corruption: first base is valid, trailing span is not. */
    uint64_t *tail_header = (uint64_t *)last - 4;
    uint64_t span = tail_header[0];
    tail_header[0] = 0;
    CHECK(neoclr_gc_allocation_info_v1(&c, first, &bytes, &kind) == 3);
    CHECK(bytes == 456 && kind == 789);
    CHECK(neoclr_gc_host_root_create_v1(&c, first, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_create_v1(&c, NULL, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_create_owned_v1(&c, first, first, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_replace_v1(&c, root, middle) == 3);
    CHECK(!neoclr_gc_host_root_read_v1(&c, root, &read) && read == first);
    read = alien;
    CHECK(neoclr_gc_callback_read_v1(&c, root, &read) == 3 && read == alien);
    CHECK(neoclr_gc_resource_attach_v1(&c, first, &resource, &cleaned, cleanup) == 3);
    CHECK(!memcmp(&resource, &empty_resource, sizeof(resource)) && !cleaned);
    CHECK(neoclr_gc_roots_park_owned_v1(&c, first, &parked) == 3);
    CHECK(!memcmp(&parked, &empty_parked, sizeof(parked)));
    tail_header[0] = span;
    CHECK(!neoclr_gc_allocation_info_v1(&c, first, &bytes, &kind));
    CHECK(bytes == 24 && kind == NEOCLR_GC_OBJECT);
    CHECK(!neoclr_gc_callback_read_v1(&c, root, &read) && read == first);
    CHECK(neoclr_gc_host_root_create_v1(&c, (char *)first + 8, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_create_v1(&c, alien, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_create_owned_v1(&c, NULL, first, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_create_owned_v1(&c, alien, first, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_replace_v1(&foreign, root, alien) == 3);
    CHECK(!neoclr_gc_host_root_create_v1(&c, NULL, &handle));
    CHECK(!neoclr_gc_host_root_read_v1(&c, handle, &read) && !read);
    CHECK(!neoclr_gc_host_root_release_v1(&c, handle));
    CHECK(!neoclr_gc_collect_v1(&c, NULL)); /* middle becomes a free hole */
    handle = 123;
    CHECK(neoclr_gc_host_root_create_v1(&c, middle, &handle) == 3 && handle == 123);
    CHECK(neoclr_gc_host_root_replace_v1(&c, root, middle) == 3);
    CHECK(!neoclr_gc_host_root_read_v1(&c, root, &read) && read == first);
    CHECK(!neoclr_gc_resource_attach_v1(&c, first, &resource, &cleaned, cleanup));
    CHECK(!neoclr_gc_host_root_release_v1(&c, root));
    CHECK(!neoclr_gc_host_root_release_v1(&c, tail_root));
    CHECK(!neoclr_gc_collect_v1(&c, NULL) && !c.text.used && cleaned == 1);
    CHECK(!neoclr_gc_entry_check_v1(&c));
    CHECK(!neoclr_gc_collect_v1(&foreign, NULL) && !foreign.text.used);
    puts("PASS exact-base admission, trailing corruption, atomic failure and reclamation");
    return 0;
}
