#include "participant-roots.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do {if(!(x)){fprintf(stderr,"line %d\n",__LINE__);abort();}} while(0)
typedef struct {
    pthread_mutex_t mutex;
    pthread_cond_t condition;
    unsigned ready,resume;
    neoclr_gc_participant participant;
    neoclr_aot_context *owner;
    void **objects;
} handoff;
static void erase(volatile unsigned char *bytes, size_t count) {
    while(count--) *bytes++=0xa5;
}
static void *publish(void *opaque) {
    handoff *h=opaque;
    neoclr_aot_context context={.text=h->owner->text};
    uint64_t stored=(uintptr_t)h->objects[0],transient=(uintptr_t)h->objects[1];
    uint64_t lane=(uintptr_t)h->objects[2],pointee=(uintptr_t)h->objects[5],borrow=(uintptr_t)&pointee;
    uint32_t tag=42;
    neoclr_probe_storage storage[]={{&stored,8,0},{&tag,4,1},{&borrow,8,2}};
    neoclr_probe_storage temporary={&transient,8,0};
    neoclr_probe_frame frame={.context=&context,.storage=storage,.storage_count=3,
        .transient=&temporary,.transient_count=1,.lanes=&lane,.lane_count=1};
    context.fault.code=4;context.fault.message=h->objects[3];
    context.fault.frame_count=1;context.fault.frames[0].function=h->objects[6];
    context.static_storage=h->objects[4];
    CHECK(!pthread_mutex_lock(&h->mutex));
    h->participant=(neoclr_gc_participant){&context,&frame};h->ready=1;
    CHECK(!pthread_cond_signal(&h->condition));
    while(!h->resume) CHECK(!pthread_cond_wait(&h->condition,&h->mutex));
    CHECK(!pthread_mutex_unlock(&h->mutex));
    /* Invalidate everything the snapshot must not retain. */
    erase((volatile unsigned char *)&context,sizeof(context));
    erase((volatile unsigned char *)storage,sizeof(storage));
    erase((volatile unsigned char *)&frame,sizeof(frame));
    stored=transient=lane=pointee=borrow=0;
    return NULL;
}
static void invalid_inputs(neoclr_aot_context *owner) {
    uint64_t word=123,out[8];for(unsigned i=0;i<8;i++)out[i]=777;
    size_t written=999;
    neoclr_probe_storage slot={&word,8,0};
    neoclr_probe_frame frame={.context=owner,.storage=&slot,.storage_count=1};
    neoclr_gc_participant part={owner,&frame};
    CHECK(neoclr_gc_participant_root_capture_v1(owner,&part,1,out,0,&written)==3 && written==999);
    CHECK(out[0]==777);
    frame.previous=&frame;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);frame.previous=NULL;
    slot.address=NULL;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);slot.address=&word;
    slot.read_bytes=2;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);slot.read_bytes=8;
    slot.flags=3;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);slot.flags=0;
    frame.context=NULL;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);frame.context=owner;
    frame.lane_count=1;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);frame.lane_count=0;
    neoclr_aot_context foreign={0};part.context=&foreign;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);part.context=owner;
    owner->fault.code=4;owner->fault.frame_count=65;CHECK(neoclr_gc_participant_root_count_v1(owner,&part,1,&written)==3);memset(&owner->fault,0,sizeof(owner->fault));
    CHECK(neoclr_gc_participant_root_count_v1(owner,&part,18,&written)==3);
    CHECK(written==999 && out[0]==777);
}
static void reusable_buffer(void) {
    uint64_t heap[8]={0},lanes[300],expected[301];
    neoclr_aot_context owner={.text={(unsigned char *)heap,sizeof(heap),0}};
    neoclr_probe_frame frame={.context=&owner,.lanes=lanes};
    neoclr_gc_participant part={&owner,&frame};
    neoclr_gc_root_buffer buffer={0};size_t count=999,written;
    for (unsigned pass=0;pass<100;pass++) {
        frame.lane_count=pass<2?pass+1:300-pass;
        for(unsigned i=0;i<frame.lane_count;i++) lanes[i]=1000*pass+i;
        CHECK(!neoclr_gc_participant_root_capture_v1(&owner,&part,1,expected,301,&written));
        CHECK(!neoclr_gc_participant_root_snapshot_v1(&owner,&part,1,&buffer,&count));
        CHECK(count==written && !memcmp(expected,buffer.values,count*sizeof(uint64_t)));
    }
    CHECK(buffer.captures==100 && buffer.growths==2 && buffer.capacity==512);
    uint64_t *saved=buffer.values;size_t capacity=buffer.capacity;
    frame.previous=&frame;count=999;
    CHECK(neoclr_gc_participant_root_snapshot_v1(&owner,&part,1,&buffer,&count)==3);
    CHECK(count==999 && buffer.values==saved && buffer.capacity==capacity && buffer.captures==100);
    frame.previous=NULL;frame.lane_count=0;
    CHECK(!neoclr_gc_participant_root_snapshot_v1(&owner,&part,1,&buffer,&count) && count==1);
    CHECK(buffer.growths==2 && buffer.values==saved);
    neoclr_gc_root_buffer_dispose_v1(&buffer);
    CHECK(!buffer.values && !buffer.capacity && !buffer.captures && !buffer.growths);
    neoclr_gc_root_buffer_dispose_v1(&buffer);
    puts("PASS reusable roots: changing values, growth, shrink, malformed preservation and disposal");
}
int main(void) {
    reusable_buffer();
    for(unsigned cycle=0;cycle<100;cycle++) {
        uint64_t heap[1024]={0},snapshot[32];
        neoclr_aot_context owner={.text={(unsigned char *)heap,sizeof(heap),0}};
        void *objects[4][8]={{0}};
        for(unsigned w=0;w<4;w++) for(unsigned i=0;i<8;i++)
            CHECK(!neoclr_gc_allocate_v1(&owner.text,i?8:16,i?NEOCLR_GC_BYTES:NEOCLR_GC_OBJECT,&objects[w][i]));
        for(unsigned w=0;w<4;w++) ((uint64_t *)objects[w][0])[1]=(uintptr_t)objects[(w+1)%4][7];
        handoff h[4]={0};pthread_t workers[4];neoclr_gc_participant parts[4];
        for(unsigned w=0;w<4;w++) {
            h[w].owner=&owner;h[w].objects=objects[w];
            CHECK(!pthread_mutex_init(&h[w].mutex,NULL));CHECK(!pthread_cond_init(&h[w].condition,NULL));
            CHECK(!pthread_create(&workers[w],NULL,publish,&h[w]));
        }
        for(unsigned w=0;w<4;w++) {
            CHECK(!pthread_mutex_lock(&h[w].mutex));
            while(!h[w].ready) CHECK(!pthread_cond_wait(&h[w].condition,&h[w].mutex));
            parts[w]=h[w].participant;
        }
        size_t required=0,written=0;
        CHECK(!neoclr_gc_participant_root_count_v1(&owner,parts,4,&required));CHECK(required==28);
        CHECK(!neoclr_gc_participant_root_capture_v1(&owner,parts,4,snapshot,32,&written));CHECK(written==required);
        neoclr_gc_root_buffer buffer={0};size_t reused;
        CHECK(!neoclr_gc_participant_root_snapshot_v1(&owner,parts,4,&buffer,&reused));
        CHECK(reused==written && !memcmp(buffer.values,snapshot,written*sizeof(uint64_t)));
        memcpy(snapshot,buffer.values,written*sizeof(uint64_t));
        neoclr_gc_root_buffer_dispose_v1(&buffer);
        for(unsigned w=0;w<4;w++) {
            h[w].resume=1;CHECK(!pthread_cond_signal(&h[w].condition));CHECK(!pthread_mutex_unlock(&h[w].mutex));
            CHECK(!pthread_join(workers[w],NULL));
        }
        /* Actual collector receives copied values after the entire source stack
         * has returned. Host C locals are deliberately not automatic GC roots. */
        neoclr_probe_frame roots={.context=&owner,.lanes=snapshot,.lane_count=(uint32_t)written};
        CHECK(!neoclr_gc_collect_v1(&owner,&roots));
        for(unsigned w=0;w<4;w++) for(unsigned i=0;i<8;i++) {
            uint64_t bytes;uint32_t kind;
            CHECK(neoclr_gc_allocation_info_v1(&owner,objects[w][i],&bytes,&kind)==(i==5?3:0));
        }
        roots.lane_count=0;CHECK(!neoclr_gc_collect_v1(&owner,&roots));
        for(unsigned w=0;w<4;w++) for(unsigned i=0;i<8;i++) {uint64_t bytes;uint32_t kind;CHECK(neoclr_gc_allocation_info_v1(&owner,objects[w][i],&bytes,&kind)==3);}
        invalid_inputs(&owner);
        for(unsigned w=0;w<4;w++) {CHECK(!pthread_cond_destroy(&h[w].condition));CHECK(!pthread_mutex_destroy(&h[w].mutex));}
    }
    puts("PASS 400 worker-stack captures; 6400 live/dead comparisons; malformed inputs and capacity rejected");
}
