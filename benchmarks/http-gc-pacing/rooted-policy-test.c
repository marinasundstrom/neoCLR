#include "rooted-policy.h"
#include <stdlib.h>
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"failed line %d\n",__LINE__); return 1; } } while (0)
static void clean(void *state) { ++*(int *)state; }
int main(void) {
    unsigned char *heap=malloc(4*1024*1024); CHECK(heap);
    neoclr_aot_context c={.text={heap,4*1024*1024,0}};
    void *root=NULL, *dead=NULL, *result=NULL;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame;
    neoclr_probe_enter_v3(&frame,&c,0,&slot,1,"{}",2);
    rooted_pacing_state pacing={0};
    CHECK(!rooted_pacing_boundary(&c,&frame,&pacing,0));
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,&frame,1024*1024,NEOCLR_GC_BYTES,&dead));
    CHECK(!rooted_pacing_boundary(&c,&frame,&pacing,0));
    uint64_t before=neoclr_gc_statistics_v1().collections;
    /* The former admission counterexample: no reserve heuristic is sufficient. */
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,&frame,7*512*1024,NEOCLR_GC_BYTES,&root));
    CHECK(neoclr_gc_statistics_v1().collections==before+1);
    void *saved=root;
    CHECK(neoclr_gc_allocate_rooted_v1(&c,&frame,1024*1024,NEOCLR_GC_BYTES,&result)==5 && !result);
    CHECK(root==saved);
    root=NULL;
    CHECK(!rooted_pacing_idle(&c,&frame,&pacing) && !c.text.used);
    /* Reachability changes without allocation still trigger bounded cleanup. */
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,&frame,16,NEOCLR_GC_OBJECT,&root));
    neoclr_gc_resource resource={0};int cleaned=0;
    CHECK(!neoclr_gc_resource_attach_v1(&c,root,&resource,&cleaned,clean));
    CHECK(!rooted_pacing_idle(&c,&frame,&pacing) && !cleaned);
    root=NULL;
    for (int i=0;i<63;i++) CHECK(!rooted_pacing_boundary(&c,&frame,&pacing,0) && !cleaned);
    CHECK(!rooted_pacing_boundary(&c,&frame,&pacing,0) && cleaned==1 && !c.text.used);
    /* No further guest boundaries while idle: cleanup must be explicit. */
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,&frame,16,NEOCLR_GC_OBJECT,&root));
    CHECK(!neoclr_gc_resource_attach_v1(&c,root,&resource,&cleaned,clean));
    root=NULL;
    CHECK(!rooted_pacing_idle(&c,&frame,&pacing) && cleaned==2 && !c.text.used);
    CHECK(!rooted_pacing_idle(&c,&frame,&pacing) && cleaned==2);
    neoclr_probe_leave_v1(&frame);
    /* An owned suspended stack survives while its owner has a strong handle;
     * once abandoned, an explicit idle collection expires it and releases both. */
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,NULL,16,NEOCLR_GC_OBJECT,&root));
    uint64_t handle=0;
    CHECK(!neoclr_gc_host_root_create_v1(&c,root,&handle));
    void *value=NULL;
    CHECK(!neoclr_gc_allocate_rooted_v1(&c,NULL,16,NEOCLR_GC_BYTES,&value));
    *(uint64_t *)value=42;
    neoclr_probe_storage suspended_slot={&value,8,0};
    neoclr_probe_frame suspended_frame;
    neoclr_probe_enter_v3(&suspended_frame,&c,0,&suspended_slot,1,"{}",2);
    neoclr_parked_roots parked={0};
    CHECK(!neoclr_gc_roots_park_owned_v1(&c,root,&parked));
    CHECK(!rooted_pacing_idle(&c,NULL,&pacing) && !parked.expired && *(uint64_t *)value==42);
    CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
    CHECK(!rooted_pacing_idle(&c,NULL,&pacing) && parked.expired && !c.text.used);
    neoclr_roots_discard_v1(&parked);
    CHECK(!neoclr_gc_entry_check_v1(&c));free(heap);return 0;
}
