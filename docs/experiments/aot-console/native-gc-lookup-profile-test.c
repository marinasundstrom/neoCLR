/* Lookup counters describe work without changing validation, roots or caches. */
#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while (0)
static int exercise(void) {
    uint64_t heap[128] = {0};
    neoclr_aot_context context = {.text = {(unsigned char *)heap, sizeof(heap), 0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&context.text));
    void *object, *child;
    CHECK(!neoclr_gc_allocate_v1(&context.text, 32, NEOCLR_GC_OBJECT, &object));
    CHECK(!neoclr_gc_allocate_v1(&context.text, 8, NEOCLR_GC_BYTES, &child));
    uint64_t *words = object;
    words[0] = 1; words[1] = (uintptr_t)child; words[2] = 42; words[3] = (uintptr_t)child;
    uint64_t handle, bytes;
    uint32_t kind;
    CHECK(!neoclr_gc_host_root_create_v1(&context, object, &handle));
    neoclr_gc_lookup_profile_reset_v1();
    CHECK(!neoclr_gc_allocation_info_v1(&context, object, &bytes, &kind));
    CHECK(bytes == 32 && kind == NEOCLR_GC_OBJECT);
    CHECK(!neoclr_gc_collect_v1(&context, NULL));
    neoclr_gc_lookup_profile measured = neoclr_gc_lookup_profile_v1();
#ifdef NEOCLR_GC_PROFILE
    CHECK(measured.base_calls == 1 && measured.base_cache_hits == 1 && !measured.base_walks);
    CHECK(measured.index_build_spans == 2 && measured.object_words == 3);
    CHECK(measured.newly_marked == 2 && measured.payload_matches >= 3);
    CHECK(measured.mark_candidates >= 5 && measured.index_outside >= 1);
    CHECK(measured.index_probes >= measured.index_queries - measured.index_outside);
#endif
    uint64_t used = context.text.used;
    neoclr_gc_statistics before = neoclr_gc_statistics_v1();
    neoclr_gc_lookup_profile_reset_v1();
    CHECK(context.text.used == used && neoclr_gc_statistics_v1().collections == before.collections);
    CHECK(!neoclr_gc_allocation_info_v1(&context, object, &bytes, &kind));
    CHECK(!neoclr_gc_allocation_info_v1(&context, object, &bytes, &kind));
    bytes = 777; kind = 777;
    CHECK(neoclr_gc_allocation_info_v1(&context, (char *)object + 1, &bytes, &kind) == 3);
    CHECK(bytes == 777 && kind == 777);
    measured = neoclr_gc_lookup_profile_v1();
#ifdef NEOCLR_GC_PROFILE
    CHECK(measured.base_calls == 3 && measured.base_cache_hits == 2);
    CHECK(measured.base_walks == 1 && measured.base_spans == 2);
    CHECK(!measured.mark_candidates && !measured.object_words);
#else
    neoclr_gc_lookup_profile zero = {0};
    CHECK(!memcmp(&measured, &zero, sizeof(zero)));
#endif
    CHECK(!neoclr_gc_allocation_info_v1(&context, child, &bytes, &kind));
    CHECK(!neoclr_gc_host_root_release_v1(&context, handle));
    CHECK(!neoclr_gc_collect_v1(&context, NULL) && !context.text.used);
    CHECK(neoclr_gc_allocation_info_v1(&context, object, &bytes, &kind) == 3);
    CHECK(neoclr_gc_allocation_info_v1(&context, child, &bytes, &kind) == 3);
    CHECK(!neoclr_gc_heap_release_v1(&context.text));
    return 0;
}

#ifdef NEOCLR_TEST_THREADS
#include <pthread.h>
static void *threaded(void *unused) {
    (void)unused;
    neoclr_gc_lookup_profile before = neoclr_gc_lookup_profile_v1(), zero = {0};
    if (memcmp(&before, &zero, sizeof(zero))) return (void *)(uintptr_t)1;
    return (void *)(uintptr_t)exercise();
}
#endif
int main(void) {
    int status = exercise();
    if (status) return status;
#ifdef NEOCLR_TEST_THREADS
    neoclr_gc_lookup_profile before = neoclr_gc_lookup_profile_v1();
    pthread_t thread;
    CHECK(!pthread_create(&thread, NULL, threaded, NULL));
    void *result;
    CHECK(!pthread_join(thread, &result) && !result);
    neoclr_gc_lookup_profile after = neoclr_gc_lookup_profile_v1();
    CHECK(!memcmp(&before, &after, sizeof(before)));
#endif
    return 0;
}
