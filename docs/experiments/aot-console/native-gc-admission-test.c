#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"check failed at %d\n",__LINE__); return 1; } } while (0)
int main(void) {
    uint64_t memory[32] = {0};
    neoclr_aot_context c = {.text = {(unsigned char *)memory, sizeof(memory), 0}};
    void *live = NULL, *dead = NULL, *out = (void *)(uintptr_t)123;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 64, NEOCLR_GC_BYTES, &live));
    *(uint64_t *)live = 42;
    CHECK(!neoclr_gc_allocate_v1(&c.text, 80, NEOCLR_GC_BYTES, &dead));
    neoclr_probe_storage slot = {&live,8,0};
    neoclr_probe_frame frame, stale = {0};
    neoclr_probe_enter_v3(&frame,&c,0,&slot,1,"{}",2);
    uint64_t collections = neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_gc_allocate_rooted_v1(&c,&stale,8,NEOCLR_GC_BYTES,&out)==3);
    CHECK(neoclr_gc_allocate_rooted_v1(&c,NULL,8,NEOCLR_GC_BYTES,&out)==3);
    neoclr_aot_context other = {0};
    CHECK(neoclr_gc_allocate_rooted_v1(&other,&frame,8,NEOCLR_GC_BYTES,&out)==3);
    CHECK(out==(void *)(uintptr_t)123 && collections==neoclr_gc_statistics_v1().collections);
    /* Noncollecting API still fails; opt-in reclaims only the unreachable block. */
    CHECK(neoclr_gc_allocate_v1(&c.text,64,NEOCLR_GC_BYTES,&out)==5);
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,&frame,64,NEOCLR_GC_BYTES,&out));
    CHECK(out && *(uint64_t *)live==42 && neoclr_gc_statistics_v1().collections==collections+1);
    out=(void *)(uintptr_t)123;
    collections=neoclr_gc_statistics_v1().collections;
    CHECK(neoclr_gc_allocate_rooted_v1(&c,&frame,256,NEOCLR_GC_BYTES,&out)==5);
    CHECK(neoclr_gc_allocate_rooted_v1(&c,&frame,UINT64_MAX,NEOCLR_GC_BYTES,&out)==5);
    CHECK(neoclr_gc_allocate_rooted_v1(&c,&frame,1,99,&out)==3);
    CHECK(out==(void *)(uintptr_t)123 && neoclr_gc_statistics_v1().collections==collections);
    CHECK(neoclr_gc_allocate_rooted_v1(&c,&frame,144,NEOCLR_GC_BYTES,&out)==5);
    CHECK(out==(void *)(uintptr_t)123 && *(uint64_t *)live==42);
    CHECK(neoclr_gc_statistics_v1().collections==collections+1);
    neoclr_probe_leave_v1(&frame);
    CHECK(!neoclr_gc_collect_v1(&c,NULL) && !c.text.used);
    /* Quiescent host admission: the former 4 MiB counterexample now succeeds. */
    unsigned char *large=calloc(1,4*1024*1024);CHECK(large);
    neoclr_aot_context big={.text={large,4*1024*1024,0}};
    CHECK(!neoclr_gc_allocate_v1(&big.text,1024*1024,NEOCLR_GC_BYTES,&dead));
    CHECK(!neoclr_gc_allocate_rooted_v1(&big,NULL,7*512*1024,NEOCLR_GC_BYTES,&out));
    CHECK(!neoclr_gc_collect_v1(&big,NULL) && !big.text.used);
    CHECK(!neoclr_gc_entry_check_v1(&big));free(large);
    puts("PASS rooted allocation retry, live temporary, rejection and exhaustion contracts");
    return 0;
}
