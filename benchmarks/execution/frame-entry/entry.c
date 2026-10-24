#include "native-gc.h"
#include <stdio.h>
#include <string.h>
#include <time.h>
int main(void) {
    neoclr_aot_context context = {0};
    uint64_t value = 0;
    neoclr_probe_storage slots[256];
    for (unsigned i=0;i<256;i++) slots[i]=(neoclr_probe_storage){&value,8,i%3};
    unsigned counts[]={0,1,4,16,64,256};
    for (unsigned c=0;c<6;c++) {
        unsigned count=counts[c];
        neoclr_gc_root_map map={NEOCLR_GC_STORAGE_MAP_MAGIC,count};
        neoclr_probe_frame frame;
        memset(&frame,0xa5,sizeof(frame));
        neoclr_gc_enter_map_v1(&frame,&context,1,slots,count,&map,sizeof(map));
        if (frame.previous || frame.context!=&context || frame.function!=1 ||
            frame.lanes || frame.plan || frame.length || frame.lane_count || frame.instruction ||
            frame.storage!=slots || frame.storage_count!=count || frame.storage_plan!=(const char *)&map ||
            frame.storage_length!=sizeof(map) || frame.transient || frame.transient_plan ||
            frame.transient_count || frame.transient_length || frame.transient_phase ||
            frame.reserved!=(NEOCLR_GC_OWNED_STORAGE|NEOCLR_GC_STORAGE_MAP|NEOCLR_GC_OWNED_CHAIN)) return 1;
        neoclr_gc_leave_v1(&frame);
        clock_t begin=clock();
        for(unsigned i=0;i<1000000;i++) {
            neoclr_gc_enter_map_v1(&frame,&context,1,slots,count,&map,sizeof(map));
            neoclr_gc_leave_v1(&frame);
        }
        printf("%u %.9f\n",count,(double)(clock()-begin)/CLOCKS_PER_SEC);
    }
    return 0;
}
