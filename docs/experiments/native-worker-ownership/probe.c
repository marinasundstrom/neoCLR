/* Diagnostic of current ownership, not a shared-heap worker implementation.
 * Parent is quiescent until join; the worker never dereferences parent payloads. */
#include "native-gc.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "check failed at %d\n", __LINE__); abort(); } } while (0)
typedef struct {
    neoclr_aot_context *parent;
    uint64_t parent_handle;
    int rejected_foreign_read;
    int rejected_foreign_release;
    int own_root_survived;
} job;
static void *worker(void *argument) {
    job *work = argument;
    void *foreign = NULL;
    work->rejected_foreign_read =
        neoclr_gc_host_root_read_v1(work->parent, work->parent_handle, &foreign) != 0 && !foreign;
    work->rejected_foreign_release =
        neoclr_gc_host_root_release_v1(work->parent, work->parent_handle) != 0;
    uint64_t storage[128] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    void *value = NULL, *read = NULL;
    uint64_t handle = 0;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 8, NEOCLR_GC_BYTES, &value));
    *(uint64_t *)value = 42;
    CHECK(!neoclr_gc_host_root_create_v1(&context, value, &handle));
    CHECK(!neoclr_gc_collect_conditional_v1(&context, NULL, NULL, 0));
    CHECK(!neoclr_gc_host_root_read_v1(&context, handle, &read));
    work->own_root_survived = read == value && *(uint64_t *)read == 42;
    CHECK(!neoclr_gc_host_root_release_v1(&context, handle));
    CHECK(!neoclr_gc_collect_conditional_v1(&context, NULL, NULL, 0));
    CHECK(!context.text.used);
    return NULL;
}
int main(void) {
    uint64_t storage[128] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)storage, sizeof(storage), 0}};
    void *value = NULL, *read = NULL;
    uint64_t handle = 0;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 8, NEOCLR_GC_BYTES, &value));
    *(uint64_t *)value = 17;
    CHECK(!neoclr_gc_host_root_create_v1(&context, value, &handle));
    job work = {.parent = &context, .parent_handle = handle};
    pthread_t thread;
    CHECK(!pthread_create(&thread, NULL, worker, &work));
    CHECK(!pthread_join(thread, NULL));
    CHECK(work.rejected_foreign_read && work.rejected_foreign_release && work.own_root_survived);
    CHECK(!neoclr_gc_collect_conditional_v1(&context, NULL, NULL, 0));
    CHECK(!neoclr_gc_host_root_read_v1(&context, handle, &read));
    CHECK(read == value && *(uint64_t *)read == 17);
    CHECK(!neoclr_gc_host_root_release_v1(&context, handle));
    CHECK(!neoclr_gc_collect_conditional_v1(&context, NULL, NULL, 0));
    CHECK(!context.text.used);
    puts("PASS foreign handle rejection, independent worker roots, parent retention, final reclamation");
    return 0;
}
