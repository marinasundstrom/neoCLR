#include "native-gc.h"
#include <stdio.h>
#include <stdlib.h>
extern int32_t experiment_gc_begin(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern int32_t experiment_gc_finish(neoclr_aot_context *, const neoclr_probe_frame *, neoclr_gc_conditional_root *, uint32_t);
extern void experiment_gc_shadow_gate(int);
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while(0)
static uint64_t *object(neoclr_aot_context *c, uint64_t bytes) {
    uint64_t *p;
    CHECK(!neoclr_gc_allocate_v1(&c->text,bytes,NEOCLR_GC_OBJECT,(void **)&p));
    p[0]=17;
    return p;
}
static void future_word(int hole, int offset) {
    uint64_t heap[256]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24), *source=object(&c,24);
    root[1]=(uintptr_t)source;
    void *free_slot=NULL;
    if(hole) {
        free_slot=object(&c,24);
        source[2]=(uintptr_t)object(&c,16);
    }
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    CHECK(!neoclr_gc_collect_v1(&c,&frame));
    /* The private descriptor is 32 bytes. The allocator result below checks the
     * anticipated address, rather than assuming it succeeded at that location. */
    uintptr_t anticipated=hole?(uintptr_t)free_slot:(uintptr_t)c.text.data+c.text.used+32;
    source[1]=anticipated+offset;
    experiment_gc_shadow_gate(1);
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    uint64_t *fresh=object(&c,16), *child=object(&c,16);
    CHECK((uintptr_t)fresh==anticipated);
    fresh[1]=(uintptr_t)child;
    experiment_gc_shadow_gate(0);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    if(offset>=0 && offset<16) CHECK(*fresh==17 && *child==17);
    else CHECK(*fresh==UINT64_C(0xdddddddddddddddd) && *child==UINT64_C(0xdddddddddddddddd));
    root=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static void closures(void) {
    uint64_t heap[512]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *root=object(&c,24), *old=object(&c,24), *white=object(&c,24), *dead=object(&c,24);
    root[1]=(uintptr_t)old;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    experiment_gc_shadow_gate(1);
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    uint64_t *fresh=object(&c,24), *garbage=object(&c,24);
    old[1]=(uintptr_t)fresh; fresh[1]=(uintptr_t)white;
    white[1]=(uintptr_t)old;
    dead[1]=123; garbage[1]=(uintptr_t)dead;
    experiment_gc_shadow_gate(0);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    CHECK(*root==17 && *old==17 && *fresh==17 && *white==17);
    CHECK(*dead==UINT64_C(0xdddddddddddddddd) && *garbage==UINT64_C(0xdddddddddddddddd));
    root=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
/* Conditional state exposed after capture becomes a current strong root. */
static void exposure(void) {
    uint64_t heap[128]={0};
    neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    uint64_t *owner=object(&c,16), *value=object(&c,16), *child=object(&c,16);
    value[1]=(uintptr_t)child;
    uint64_t handle;
    CHECK(!neoclr_gc_host_root_create_owned_v1(&c,owner,value,&handle));
    void *root=NULL;
    neoclr_probe_storage slot={&root,8,0};
    neoclr_probe_frame frame={.context=&c,.storage=&slot,.storage_count=1};
    experiment_gc_shadow_gate(1);
    CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
    CHECK(!neoclr_gc_host_root_read_v1(&c,handle,&root));
    experiment_gc_shadow_gate(0);
    CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
    CHECK(*owner==UINT64_C(0xdddddddddddddddd) && *value==17 && *child==17);
    CHECK(!neoclr_gc_host_root_release_v1(&c,handle));
    root=NULL;
    CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
}
static unsigned random_state=1234567;
static unsigned random_word(void) {
    random_state ^= random_state << 13;
    random_state ^= random_state >> 17;
    random_state ^= random_state << 5;
    return random_state;
}
static void close_graph(unsigned char *live, int edges[200][2], unsigned count) {
    int changed;
    do {
        changed=0;
        for(unsigned i=0;i<count;i++) if(live[i])
            for(unsigned j=0;j<2;j++) if(edges[i][j]>=0 && !live[edges[i][j]]) {
                live[edges[i][j]]=1; changed=1;
            }
    } while(changed);
}
static void randomized(void) {
    for(unsigned round=0;round<100;round++) {
        uint64_t heap[2048]={0};
        neoclr_aot_context c={.text={(unsigned char *)heap,sizeof(heap),0}};
        CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
        uint64_t *nodes[200]; int edges[200][2];
        for(unsigned i=0;i<80;i++) nodes[i]=object(&c,24);
        for(unsigned i=0;i<80;i++) for(unsigned j=0;j<2;j++) {
            unsigned value=random_word()%120;
            edges[i][j]=value<80?(int)value:-1;
            nodes[i][j+1]=value<80?(uintptr_t)nodes[value]:0;
        }
        unsigned char expected[200]={0}; expected[0]=1;
        close_graph(expected,edges,80);
        uint64_t root=(uintptr_t)nodes[0];
        neoclr_probe_frame frame={.context=&c,.lanes=&root,.lane_count=1};
        experiment_gc_shadow_gate(1);
        CHECK(!experiment_gc_begin(&c,&frame,NULL,0));
        for(unsigned i=80;i<200;i++) nodes[i]=object(&c,24);
        for(unsigned i=0;i<200;i++) for(unsigned j=0;j<2;j++) {
            if(i<80 && random_word()%3) continue;
            unsigned value=random_word()%300;
            edges[i][j]=value<200?(int)value:-1;
            nodes[i][j+1]=value<200?(uintptr_t)nodes[value]:0;
        }
        unsigned next=random_word()%200;
        root=(uintptr_t)nodes[next]; expected[next]=1;
        close_graph(expected,edges,200);
        experiment_gc_shadow_gate(0);
        CHECK(!experiment_gc_finish(&c,&frame,NULL,0));
        for(unsigned i=0;i<200;i++) {
            uint64_t bytes; uint32_t kind;
            int retained=neoclr_gc_allocation_info_v1(&c,nodes[i],&bytes,&kind)==0;
            CHECK(retained==expected[i]);
            if(retained) CHECK(bytes==24 && kind==NEOCLR_GC_OBJECT && nodes[i][0]==17);
        }
        root=0;
        CHECK(!neoclr_gc_collect_v1(&c,&frame) && !c.text.used);
        CHECK(!neoclr_gc_heap_release_v1(&c.text));
    }
}
int main(void) {
    int offsets[]={-1,0,1,7,8,15,16,23};
    for(int hole=0;hole<2;hole++)
        for(unsigned i=0;i<sizeof(offsets)/sizeof(offsets[0]);i++) future_word(hole,offsets[i]);
    for(unsigned i=0;i<100;i++) closures();
    exposure();
    randomized();
    puts("PASS 16 future-address cases, 100 closure cycles, conditional exposure and 20000 differential block checks");
    return 0;
}
