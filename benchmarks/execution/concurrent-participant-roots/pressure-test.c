#include "native-workers.h"
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do {if(!(x)){fprintf(stderr,"line %d\n",__LINE__);abort();}} while(0)
static int32_t bootstrap(int32_t argument,uint64_t *handle,neoclr_aot_context *c) {
    (void)argument;void *capture=NULL;
    int32_t status=neoclr_gc_allocate_v1(&c->text,24,NEOCLR_GC_OBJECT,&capture);
    if(!status)*(uint64_t *)capture=UINT32_MAX;
    return status?status:neoclr_gc_host_root_create_v1(c,capture,handle);
}
static _Thread_local neoclr_gc_worker_collector underlying;
static _Thread_local unsigned direct_requests;
static int32_t observe(neoclr_aot_context *c,const neoclr_probe_frame *frame) {
    direct_requests++;return underlying(c,frame);
}
static int32_t invoke(uint64_t handle,neoclr_aot_context *c) {
    void *capture=NULL,*value=NULL;
    CHECK(!neoclr_gc_callback_read_v1(c,handle,&capture));
    neoclr_probe_storage slots[]={{&capture,8,0},{&value,8,0}};
    neoclr_gc_root_map storage={NEOCLR_GC_STORAGE_MAP_MAGIC,2},map={NEOCLR_GC_ROOT_MAP_MAGIC,0};
    neoclr_probe_frame frame;uint64_t empty=0;
    neoclr_gc_enter_map_v1(&frame,c,0,slots,2,&storage,sizeof(storage));
    neoclr_gc_worker_read_phase_map_v1(&frame,0,&empty,0,&map,sizeof(map),1);
    underlying=neoclr_gc_worker_collect_hook;direct_requests=0;
    neoclr_gc_worker_collect_hook=observe;
    /* No budget is due. Explicit collection must still reach the service. */
    CHECK(!neoclr_gc_collect_v1(c,&frame));
    CHECK(!neoclr_gc_allocate_v1(&c->text,1500,NEOCLR_GC_BYTES,&value));
    value=NULL; /* Reclaimable at the rooted allocation's pressure rendezvous. */
    CHECK(!neoclr_gc_allocate_rooted_v1(c,&frame,1500,NEOCLR_GC_BYTES,&value));
    uint64_t bytes;uint32_t kind;
    CHECK(!neoclr_gc_allocation_info_v1(c,value,&bytes,&kind) && bytes==1500);
    CHECK(direct_requests==2);
    neoclr_gc_worker_collect_hook=underlying;underlying=NULL;
    neoclr_gc_leave_v1(&frame);return 0;
}
static int work(neoclr_worker_job *job,void *state) {
    (void)state;CHECK(!neoclr_worker_invoke_capture_parallel(job));return 0;
}
int main(void) {
    neoclr_workers *p=neoclr_workers_create(1,1,2048);neoclr_worker_handle capture;
    CHECK(p && !neoclr_workers_bootstrap(p,bootstrap,invoke,0,&capture));
    for(unsigned i=0;i<32;i++) {
        neoclr_worker_handle ticket,root;neoclr_worker_value value;int outcome;
        CHECK(neoclr_workers_submit(p,capture,work,NULL,&ticket));
        CHECK(neoclr_workers_wait(p,ticket,30000)==1);
        CHECK(neoclr_workers_take(p,ticket,&outcome,&value,&root)==1 && outcome==NEOCLR_WORKER_OK && !root.pool);
    }
    uint64_t collections,requests;unsigned peak;
    CHECK(neoclr_workers_compiled_statistics(p,&peak,&collections) && collections>=64 && collections<=96);
    CHECK(neoclr_workers_collection_requests(p,&requests) && requests==collections);
    CHECK(neoclr_workers_allocation_requests(p,&requests) && requests==96);
    CHECK(neoclr_workers_root_release(p,capture));
    CHECK(neoclr_workers_close(p) && neoclr_workers_destroy(p));
    puts("PASS 32 explicit and 32 pressure collections despite pacing; 96 allocator attempts and cleanup");
}
