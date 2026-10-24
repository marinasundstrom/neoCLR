#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
void *experiment_state(void);
void experiment_stack(void *,neoclr_probe_frame *,uint32_t,const uint64_t *,uint32_t,const void *,uint32_t);
void experiment_transient(void *,neoclr_probe_frame *,uint32_t,const neoclr_probe_storage *,uint32_t,const void *,uint32_t);
void experiment_leave(void *,neoclr_probe_frame *);
int main(int argc,char **argv) {
    if(argc!=2)return 2;
    int cached=atoi(argv[1]);
    neoclr_aot_context c={0}; uint64_t value=0;
    neoclr_probe_storage slot={&value,8,0};
    neoclr_gc_root_map storage={NEOCLR_GC_STORAGE_MAP_MAGIC,1}, transient={NEOCLR_GC_TRANSIENT_MAP_MAGIC,1}, root={NEOCLR_GC_ROOT_MAP_MAGIC,0};
    unsigned counts[]={1,4,16};
    for(unsigned k=0;k<3;k++) {
        unsigned n=counts[k]; clock_t begin=clock();
        for(unsigned i=0;i<20000000;i++) {
            neoclr_probe_frame frame;
            neoclr_gc_enter_map_v1(&frame,&c,0,&slot,1,&storage,sizeof(storage));
            /* Include capture cost once per activation, not once per process. */
            void *state=cached?experiment_state():NULL;
            for(unsigned j=0;j<n;j++) {
                if(cached) {
                    experiment_stack(state,&frame,j,&value,0,&root,sizeof(root));
                    experiment_transient(state,&frame,2,&slot,1,&transient,sizeof(transient));
                } else {
                    neoclr_gc_stack_map_v1(&frame,j,&value,0,&root,sizeof(root));
                    neoclr_gc_transient_map_v1(&frame,2,&slot,1,&transient,sizeof(transient));
                }
            }
            if(cached)experiment_leave(state,&frame);else neoclr_gc_leave_v1(&frame);
        }
        printf("%u %.9f\n",n,(double)(clock()-begin)/CLOCKS_PER_SEC);
    }
    return neoclr_root_probe_depth_v1()!=0;
}
