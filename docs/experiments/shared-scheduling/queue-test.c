#include "ready-queue.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
static unsigned random_state=19;
static unsigned next(void) { random_state=random_state*1664525u+1013904223u; return random_state; }
int main(void) {
    unsigned sizes[]={0,1,2,16,256};
    for (unsigned n=0;n<5;n++) {
        neoclr_ready_item storage[256]={0}, model[256];
        neoclr_ready_queue queue; neoclr_ready_init(&queue,storage,sizes[n]);
        unsigned count=0,peak=0; uint64_t sequence=0,admitted=0,rejected=0,dispatched=0,removed=0;
        for (unsigned step=0;step<20000;step++) {
            unsigned operation=next()%5;
            if (operation<2) {
                sequence++;
                neoclr_ready_item item={sequence,sequence*3};
                int expected=count<sizes[n];
                CHECK(neoclr_ready_push(&queue,item)==expected);
                if (expected) { model[count++]=item; admitted++; if (count>peak) peak=count; }
                else rejected++;
            } else if (operation<4) {
                neoclr_ready_item item={UINT64_MAX,UINT64_MAX};
                int newest=operation==3;
                CHECK(neoclr_ready_pop(&queue,newest,&item)==(count!=0));
                if (count) {
                    unsigned at=newest?count-1:0;
                    CHECK(item.identity==model[at].identity && item.order==model[at].order);
                    memmove(model+at,model+at+1,(count-at-1)*sizeof(*model));count--;dispatched++;
                } else CHECK(item.identity==UINT64_MAX && item.order==UINT64_MAX);
            } else {
                unsigned at=count?next()%count:0;
                uint64_t identity=count?model[at].identity:UINT64_MAX;
                CHECK(neoclr_ready_remove(&queue,identity)==(count!=0));
                if (count) { memmove(model+at,model+at+1,(count-at-1)*sizeof(*model));count--;removed++; }
            }
            CHECK(queue.count==count);
            const neoclr_ready_item *first=neoclr_ready_first(&queue);
            CHECK((first!=NULL)==(count!=0));
            for (unsigned i=0;i<count;i++) {
                neoclr_ready_item actual=storage[(queue.head+i)%queue.capacity];
                CHECK(actual.identity==model[i].identity && actual.order==model[i].order);
            }
        }
#ifdef NEOCLR_SCHEDULER_DIAGNOSTICS
        neoclr_ready_statistics stats=neoclr_ready_stats(&queue);
        CHECK(stats.admitted==admitted && stats.rejected==rejected && stats.dispatched==dispatched && stats.removed==removed && stats.peak==peak);
#else
        (void)admitted;(void)rejected;(void)dispatched;(void)removed;(void)peak;
        CHECK(!neoclr_ready_stats(&queue).admitted);
#endif
        neoclr_ready_item item;
        while (neoclr_ready_pop(&queue,0,&item)) {}
        for (unsigned i=0;i<sizes[n];i++) CHECK(!storage[i].identity && !storage[i].order);
        CHECK(!neoclr_ready_push(&queue,(neoclr_ready_item){0,1}));
    }
    puts("PASS shared ready queue: FIFO/LIFO, wraparound, cancellation, admission, cleared slots and diagnostics");
}
