#include "session.h"
#define main scheduler_contract_main
#include "../aot-console/socket-scheduler-test.c"
#undef main
#include <stdio.h>
int main(void) {
    neoclr_trace_session *session = neoclr_trace_create(1, 65536);
    CHECK(session && neoclr_trace_attach(session, 0, 1));
    int result = scheduler_contract_main();
    CHECK(!result);
    CHECK(neoclr_trace_detach() && neoclr_trace_close(session));
    neoclr_trace_view view;
    CHECK(neoclr_trace_view_lane(session, 0, &view) && !view.dropped);
    unsigned admitted=0, dequeued=0, abandoned=0;
    uint64_t previous=view.origin_ns;
    for (size_t i=0; i<view.count; i++) {
        const neoclr_trace_event *e=&view.events[i];
        CHECK(e->ns>=previous && e->sequence==i+1);
        previous=e->ns;
        if (e->domain!=NEOCLR_TRACE_SCHEDULER) continue;
        CHECK(e->source && e->subject);
        admitted += e->kind==NEOCLR_TRACE_SCHEDULE_ADMIT;
        dequeued += e->kind==NEOCLR_TRACE_SCHEDULE_DEQUEUE;
        abandoned += e->kind==NEOCLR_TRACE_SCHEDULE_ABANDON;
    }
    CHECK(admitted && dequeued && abandoned && admitted==dequeued+abandoned);
    CHECK(neoclr_trace_destroy(session));
    printf("PASS trace admission=%u dequeue=%u abandon=%u\n", admitted, dequeued, abandoned);
    return 0;
}
