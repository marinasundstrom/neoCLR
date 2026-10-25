#include "native-gc.h"
#include <stdlib.h>
#include <stdio.h>
static unsigned expected_tag,expected_exclusive,calls;
static void boundary(neoclr_aot_context *c,const neoclr_probe_frame *f,unsigned exclusive) {
    if (!!(f->reserved & NEOCLR_GC_WORKER_READ_PHASE)!=expected_tag || exclusive!=expected_exclusive) abort();
    if (neoclr_gc_collect_v1(c,f)) abort();
    calls++;
}
int main(void) {
    uint64_t heap[128]={0},lane=0;
    neoclr_aot_context c={.text={(void *)heap,sizeof(heap),0}};
    neoclr_probe_frame frame;
    void *root=NULL;
    neoclr_probe_storage slot={&root,8,0};
    const neoclr_gc_root_map storage={NEOCLR_GC_STORAGE_MAP_MAGIC,1},map={NEOCLR_GC_ROOT_MAP_MAGIC,0};
    if(neoclr_gc_allocate_v1(&c.text,8,NEOCLR_GC_BYTES,&root))abort();
    neoclr_gc_enter_map_v1(&frame,&c,0,&slot,1,&storage,sizeof(storage));
    neoclr_gc_worker_boundary_hook=boundary;
    for(unsigned i=0;i<100;i++) for(expected_tag=0;expected_tag<2;expected_tag++)
        for(expected_exclusive=0;expected_exclusive<2;expected_exclusive++) {
            if(expected_tag)neoclr_gc_worker_read_phase_map_v1(&frame,0,&lane,0,&map,sizeof(map),expected_exclusive);
            else neoclr_gc_worker_stack_map_v1(&frame,0,&lane,0,&map,sizeof(map),expected_exclusive);
            uint64_t bytes;uint32_t kind;
            if(neoclr_gc_allocation_info_v1(&c,root,&bytes,&kind))abort();
        }
    neoclr_gc_worker_boundary_hook=NULL;
    neoclr_gc_leave_v1(&frame);root=NULL;
    if(neoclr_gc_collect_v1(&c,NULL) || c.text.used || calls!=400)abort();
    puts("PASS 400 old/new worker boundary tags, both lease kinds, retained roots and reclamation");
}
