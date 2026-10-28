#include "heap-adoption.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#define CHECK(x) do { if (!(x)) abort(); } while (0)
static double now(void) {
    struct timespec t; CHECK(!clock_gettime(CLOCK_MONOTONIC,&t));
    return t.tv_sec+t.tv_nsec/1e9;
}
int main(void) {
    unsigned sizes[]={0,128,1024};
    for (unsigned n=0;n<3;n++) {
        uint64_t storage[16384]={0};
        neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
        for (unsigned i=0;i<sizes[n];i++) {
            void *value; CHECK(!neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&value));
            *(uint64_t *)value=i;
        }
        for (unsigned i=0;i<100;i++) {
            CHECK(!neoclr_gc_heap_adopt_exclusive_v1(&c.text));
            CHECK(!neoclr_gc_heap_release_v1(&c.text));
        }
        clock_t cpu=clock(); double start=now();
        for (unsigned i=0;i<2000;i++) {
            CHECK(!neoclr_gc_heap_adopt_exclusive_v1(&c.text));
            CHECK(!neoclr_gc_heap_release_v1(&c.text));
        }
        double elapsed=now()-start, seconds=(double)(clock()-cpu)/CLOCKS_PER_SEC;
        printf("{\"blocks\":%u,\"leases\":2000,\"seconds\":%.9f,\"cpuSeconds\":%.9f}\n",sizes[n],elapsed,seconds);
    }
}
