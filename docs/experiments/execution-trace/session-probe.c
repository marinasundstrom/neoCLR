#include "session.h"
#define main async_main
#include "async-probe.c"
#undef main
int main(int argc,char **argv) {
    size_t capacity=argc>1?(size_t)atoi(argv[1]):65536;
    neoclr_trace_session *session=NULL;
    if(capacity) {
        session=neoclr_trace_create(1,capacity);CHECK(session);
        CHECK(neoclr_trace_attach(session,0,1));
    }
    char *arguments[]={argv[0],"65536",argc>2?argv[2]:"8"};
    CHECK(!async_main(3,arguments));
#ifdef TRACE_LIVE
    /* Verify epoch cancellation on heap release without pretending it completed. */
    uint64_t storage[1024]={0};
    neoclr_aot_context c={.text={(unsigned char *)storage,sizeof(storage),0}};
    CHECK(!neoclr_gc_heap_acquire_v1(&c.text));
    CHECK(!neoclr_gc_idle_v1(&c,NULL));
    CHECK(!neoclr_gc_heap_release_v1(&c.text));
#endif
    if(!capacity) return 0;
    CHECK(neoclr_trace_detach());CHECK(neoclr_trace_close(session));
    neoclr_trace_view view;CHECK(neoclr_trace_view_lane(session,0,&view));
    printf("{\"schema\":\"session-1\",\"clock\":\"CLOCK_MONOTONIC\",\"session\":%llu,\"origin_ns\":%llu,\"lane_count\":1,\"lanes\":[{\"lane\":0,\"role\":%u,\"attempted\":%llu,\"dropped\":%llu,\"capacity\":%zu,\"events\":[",(unsigned long long)view.session,(unsigned long long)view.origin_ns,view.role,(unsigned long long)view.attempted,(unsigned long long)view.dropped,view.capacity);
    for(size_t i=0;i<view.count;i++) {
        const neoclr_trace_event *e=&view.events[i];
        printf("%s[%llu,%llu,%u,%u,%llu,%llu,%llu,%u]",i?",":"",(unsigned long long)e->ns,(unsigned long long)e->sequence,e->domain,e->kind,(unsigned long long)e->source,(unsigned long long)e->subject,(unsigned long long)e->related,e->detail);
    }
    puts("]}]}");CHECK(neoclr_trace_destroy(session));
}
