#define _POSIX_C_SOURCE 200809L
#include "native-gc.h"
#include <assert.h>
#include <stdio.h>
#include <time.h>
extern int experiment_gc_store(neoclr_aot_context *,void *,uint64_t,uint32_t);
#ifdef NEOCLR_LIVE_STORE_HANDLE
extern void *experiment_gc_store_state_borrow(void);
extern int experiment_gc_store_borrowed(void *,neoclr_aot_context *,void *,uint64_t,uint32_t);
#endif
static uint64_t clock_ns(void) {
    struct timespec t;assert(!clock_gettime(CLOCK_THREAD_CPUTIME_ID,&t));
    return (uint64_t)t.tv_sec*1000000000+t.tv_nsec;
}
int main(void) {
    uint64_t heap[128]={0},outside=0;
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    assert(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *object;assert(!neoclr_gc_allocate_v1(&c.text,16,NEOCLR_GC_OBJECT,(void **)&object));
#ifdef NEOCLR_LIVE_STORE_HANDLE
    void *handle=experiment_gc_store_state_borrow();assert(handle);
#endif
    for(unsigned mode=0;mode<2;mode++) {
        uint64_t *destination=mode?object+1:&outside;
        uint64_t start=clock_ns();
        for(unsigned i=0;i<1000000;i++) {
#ifdef NEOCLR_LIVE_STORE_HANDLE
            assert(!experiment_gc_store_borrowed(handle,&c,destination,i,8));
#else
            assert(!experiment_gc_store(&c,destination,i,8));
#endif
        }
        uint64_t elapsed=clock_ns()-start;assert(*destination==999999);
        printf("%u %llu\n",mode,(unsigned long long)elapsed);
    }
    neoclr_probe_frame frame={.context=&c};
    assert(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    assert(!neoclr_gc_heap_release_v1(&c.text));return 0;
}
