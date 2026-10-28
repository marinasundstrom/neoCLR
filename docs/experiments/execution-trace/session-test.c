#include "session.h"
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#define CHECK(x) do { if(!(x)) abort(); } while(0)
static neoclr_trace_session *session;
static void *produce(void *arg) {
    unsigned lane=(unsigned)(uintptr_t)arg;
    CHECK(neoclr_trace_attach(session,lane,42));
    CHECK(!neoclr_trace_attach(session,lane,42));
    CHECK(!neoclr_trace_close(session)); /* non-owner */
    for(unsigned n=0;n<1000;n++) neoclr_trace_record(99,1,lane,n,0,0);
    CHECK(neoclr_trace_detach());CHECK(!neoclr_trace_detach());
    return NULL;
}
int main(void) {
    CHECK(!neoclr_trace_create(0,1));CHECK(!neoclr_trace_create(17,1));
    CHECK(!neoclr_trace_create(1,65537));
    for(unsigned round=0;round<2;round++) {
        session=neoclr_trace_create(3,round?1000:1);CHECK(session);
        CHECK(neoclr_trace_attach(session,0,1));
        CHECK(!neoclr_trace_close(session));CHECK(!neoclr_trace_destroy(session));
        neoclr_trace_view view;CHECK(!neoclr_trace_view_lane(session,0,&view));
        pthread_t a,b;CHECK(!pthread_create(&a,NULL,produce,(void *)(uintptr_t)1));
        CHECK(!pthread_create(&b,NULL,produce,(void *)(uintptr_t)2));
        neoclr_trace_record(99,1,0,0,0,0);
        CHECK(!pthread_join(a,NULL));CHECK(!pthread_join(b,NULL));
        CHECK(neoclr_trace_detach());
        CHECK(!neoclr_trace_attach(session,1,42)); /* detached lanes cannot be reused */
        CHECK(neoclr_trace_close(session));CHECK(neoclr_trace_close(session));
        CHECK(!neoclr_trace_attach(session,0,1));
        uint64_t id=0;
        for(unsigned lane=0;lane<3;lane++) {
            CHECK(neoclr_trace_view_lane(session,lane,&view));
            if(!lane) id=view.session; else CHECK(id==view.session);
            CHECK(view.attempted==(lane?1000:1));
            CHECK(view.count+view.dropped==view.attempted);
            if(lane) CHECK(view.dropped==(round?0:999));
            for(size_t n=0;n<view.count;n++) {
                CHECK(view.events[n].sequence==n+1 && view.events[n].source==lane);
                CHECK(view.events[n].ns>=view.origin_ns);
            }
        }
        CHECK(neoclr_trace_destroy(session));
    }
    puts("PASS session ownership, simultaneous producers, loss, close and no lane reuse");
}
