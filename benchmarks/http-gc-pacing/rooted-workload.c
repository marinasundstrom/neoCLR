/* Collection-only native GC workload. Construction is deliberately untimed. */
#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
static double seconds(void) {
    LARGE_INTEGER now, frequency;
    if (!QueryPerformanceFrequency(&frequency) || !QueryPerformanceCounter(&now)) abort();
    return (double)now.QuadPart / (double)frequency.QuadPart;
}
#else
#include <time.h>
static double seconds(void) {
    struct timespec now;
    if (clock_gettime(CLOCK_MONOTONIC, &now)) abort();
    return (double)now.tv_sec + (double)now.tv_nsec / 1e9;
}
#endif
#include "rooted-policy.h"
static const struct { uint64_t length; char bytes[4]; } word={3,"abc"};
int main(int argc,char **argv) {
    if (argc!=3) return 1;
    int stress=atoi(argv[1]), strings=!strcmp(argv[2],"strings");
    enum { COUNT=20000, ROOTS=64 };
    unsigned char *heap=malloc(65536);if (!heap) return 1;
    neoclr_aot_context c={.text={heap,65536,0}};
    void *roots[ROOTS]={0};neoclr_probe_storage slots[ROOTS];
    for (int i=0;i<ROOTS;i++) slots[i]=(neoclr_probe_storage){&roots[i],8,0};
    neoclr_probe_frame frame;neoclr_probe_enter_v3(&frame,&c,0,slots,ROOTS,"{}",2);
    rooted_pacing_state pacing={0};
    neoclr_gc_profile_reset_v1();
    double start=seconds();
    for (int i=0;i<COUNT;i++) {
        if (rooted_pacing_boundary(&c,&frame,&pacing,stress)) return 2;
        void *value=NULL;
        if (strings) {
            const neoclr_aot_text *text=NULL;
            if (neoclr_string_concat_rooted_v1((const void *)&word,(const void *)&word,&c.text,&text)) return 3;
            value=(void *)text;
        } else {
            if (neoclr_gc_allocate_rooted_v1(&c,&frame,32,NEOCLR_GC_OBJECT,&value)) return 3;
            ((uint64_t *)value)[1]=(uint64_t)i;
        }
        roots[i%ROOTS]=value;
    }
    double elapsed=seconds()-start;
    for (int i=0;i<ROOTS;i++) {
        if (strings) {
            const neoclr_aot_text *text=roots[i];
            if (text->length!=6 || memcmp(text->bytes,"abcabc",6)) return 4;
        } else if (((uint64_t *)roots[i])[1] < COUNT-ROOTS) return 4;
    }
    memset(roots,0,sizeof(roots));
    if (rooted_pacing_idle(&c,&frame,&pacing) || c.text.used) return 5;
    neoclr_probe_leave_v1(&frame);
    neoclr_gc_statistics stats=neoclr_gc_statistics_v1();
    neoclr_gc_profile profile=neoclr_gc_profile_v1();
    if (stats.allocations!=stats.reclaimed_allocations || neoclr_gc_entry_check_v1(&c)) return 6;
    printf("{\"seconds\":%.9f,\"collections\":%llu,\"allocations\":%llu,\"reclaimed\":%llu,\"peakArenaBytes\":%llu}\n",elapsed,
        (unsigned long long)stats.collections,(unsigned long long)stats.allocations,
        (unsigned long long)stats.reclaimed_allocations,(unsigned long long)profile.peak_arena_used);
    free(heap);return 0;
}
