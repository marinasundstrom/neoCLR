#include "gate.h"
#include <stdatomic.h>
#include <sched.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)
typedef struct {
    mutation_gate *gate;
    gate_writer token;
    uint64_t *payload, value;
    atomic_uint command, done;
    int result;
} writer;
static void *write_thread(void *opaque) {
    writer *w = opaque;
    unsigned prior = 0;
    for (;;) {
        unsigned command = atomic_load_explicit(&w->command, memory_order_acquire);
        if (command == prior) { sched_yield(); continue; }
        if (command == UINT32_MAX) break;
        uint64_t values[2] = {w->value,w->value};
        mutation_patch patch = {8,sizeof(values),values};
        /* Registration does not grant lifecycle control or the owner's fast path. */
        CHECK(gate_write(w->gate,w->payload,24,8,&patch,1)==GATE_INVALID);
        CHECK(gate_revoke_writer(w->gate,&w->token)==GATE_INVALID);
        gate_writer out;
        CHECK(gate_register_writer(w->gate,pthread_self(),&out)==GATE_INVALID);
        w->result=gate_write_registered(w->gate,&w->token,w->payload,24,8,&patch,1);
        prior=command;
        atomic_store_explicit(&w->done,command,memory_order_release);
    }
    return NULL;
}
static void start(writer *w, unsigned command, uint64_t value) {
    w->value=value;
    atomic_store_explicit(&w->command,command,memory_order_release);
}
static void finish(writer *w, unsigned command, int result) {
    while (atomic_load_explicit(&w->done,memory_order_acquire)!=command) sched_yield();
    CHECK(w->result==result);
}
typedef struct { mutation_gate *gate; uint64_t epoch,*payload; atomic_uint done; uint64_t reads; } reader;
static void *read_thread(void *opaque) {
    reader *r=opaque;
    while (!atomic_load(&r->done)) {
        uint64_t copy[3];
        int result=gate_read(r->gate,r->epoch,r->payload,24,0,copy,sizeof(copy));
        CHECK(result==GATE_OK || result==GATE_STALE || result==GATE_CANCELLED);
        if (!result) { CHECK(copy[0]==17 && copy[1]==copy[2]); r->reads++; }
    }
    return NULL;
}
static void capacity_and_generation(void) {
    mutation_gate g; uint64_t epoch;
    writer writers[17]={0};pthread_t threads[17];gate_writer tokens[16],out={0};
    CHECK(!gate_init(&g));CHECK(!gate_begin(&g,&epoch));
    for(unsigned i=0;i<17;i++) {
        atomic_init(&writers[i].command,0);atomic_init(&writers[i].done,0);
        CHECK(!pthread_create(&threads[i],NULL,write_thread,&writers[i]));
        if(i<16) CHECK(!gate_register_writer(&g,threads[i],&tokens[i]));
    }
    CHECK(gate_register_writer(&g,threads[16],&out)==GATE_INVALID);CHECK(!out.gate);
    CHECK(!gate_revoke_writer(&g,&tokens[0]));
    /* Artificial saturation must retire the slot, never wrap authority. */
    g.writers[0].generation=UINT64_MAX;
    CHECK(gate_register_writer(&g,threads[16],&out)==GATE_INVALID);CHECK(!out.gate);
    CHECK(!gate_stop(&g));CHECK(!gate_begin(&g,&epoch));
    CHECK(!gate_register_writer(&g,threads[16],&out));CHECK(out.slot==1);
    CHECK(!gate_stop(&g));
    for(unsigned i=0;i<17;i++) atomic_store(&writers[i].command,UINT32_MAX);
    for(unsigned i=0;i<17;i++) CHECK(!pthread_join(threads[i],NULL));
    CHECK(!gate_destroy(&g));
}
_Static_assert(GATE_WORDS >= 16, "Four-writer test requires sixteen log words");
int main(void) {
    capacity_and_generation();
    mutation_gate gate,other;
    uint64_t epoch,payload[3]={17,1,1},log[GATE_WORDS]; size_t count;
    CHECK(!gate_init(&gate)); CHECK(!gate_init(&other)); CHECK(!gate_begin(&gate,&epoch));
    writer writers[4]={0}; pthread_t threads[4],observer;
    for (unsigned i=0;i<4;i++) {
        writers[i].gate=&gate;writers[i].payload=payload;
        atomic_init(&writers[i].command,0);atomic_init(&writers[i].done,0);
        CHECK(!pthread_create(&threads[i],NULL,write_thread,&writers[i]));
        CHECK(!gate_register_writer(&gate,threads[i],&writers[i].token));
        gate_writer duplicate;
        CHECK(gate_register_writer(&gate,threads[i],&duplicate)==GATE_INVALID);
    }
    reader r={.gate=&gate,.epoch=epoch,.payload=payload};atomic_init(&r.done,0);
    CHECK(!pthread_create(&observer,NULL,read_thread,&r));
    for (unsigned round=1;round<=1000;round++) {
        for(unsigned i=0;i<4;i++) start(&writers[i],round,round*4+i);
        for(unsigned i=0;i<4;i++) finish(&writers[i],round,GATE_OK);
        CHECK(!gate_drain(&gate,epoch,log,GATE_WORDS,&count));CHECK(count==16);
        for(size_t i=0;i<count;i+=2) CHECK(log[i]==log[i+1]);
        /* Each successful transaction logs both old and new edges. */
        for(unsigned i=0;i<4;i++) {
            unsigned found=0;for(size_t j=2;j<count;j+=4) found+=log[j]==round*4+i;
            CHECK(found==1);
        }
    }
    gate_writer stale=writers[0].token;
    CHECK(!gate_revoke_writer(&gate,&stale));
    start(&writers[0],1001,999);finish(&writers[0],1001,GATE_STALE);
    CHECK(!gate_register_writer(&gate,threads[0],&writers[0].token));
    CHECK(writers[0].token.generation!=stale.generation);
    gate_writer valid=writers[0].token;
    writers[0].token=stale;start(&writers[0],1002,999);finish(&writers[0],1002,GATE_STALE);
    writers[0].token=writers[1].token;start(&writers[0],1003,999);finish(&writers[0],1003,GATE_INVALID);
    writers[0].token=valid;
    uint64_t value=99;mutation_patch patch={8,8,&value};
    CHECK(gate_write_registered(&other,&valid,payload,24,8,&patch,1)==GATE_INVALID);
    CHECK(gate_write_registered(&gate,&valid,payload,24,8,&patch,1)==GATE_INVALID);
    /* Stop races an attempted write. Either its complete transaction wins or it
     * is stale; after stop no registration can mutate or inspect payload bytes. */
    start(&writers[0],1004,4444);CHECK(!gate_stop(&gate));
    while(atomic_load(&writers[0].done)!=1004) sched_yield();
    CHECK(writers[0].result==GATE_OK || writers[0].result==GATE_STALE);
    uint64_t stopped=payload[1];start(&writers[0],1005,5555);finish(&writers[0],1005,GATE_STALE);CHECK(payload[1]==stopped);
    CHECK(!gate_begin(&gate,&epoch));
    start(&writers[0],1006,6666);finish(&writers[0],1006,GATE_STALE);
    CHECK(!gate_register_writer(&gate,threads[0],&writers[0].token));
    unsigned command=1007;
    for(unsigned i=0;i<GATE_WORDS/4;i++,command++) {start(&writers[0],command,command);finish(&writers[0],command,GATE_OK);}
    stopped=payload[1];start(&writers[0],command,7777);finish(&writers[0],command++,GATE_CANCELLED);CHECK(payload[1]==stopped);
    start(&writers[0],command,8888);finish(&writers[0],command,GATE_CANCELLED);CHECK(payload[1]==stopped);
    CHECK(gate_stop(&gate)==GATE_CANCELLED);
    for(unsigned i=0;i<4;i++) atomic_store(&writers[i].command,UINT32_MAX);
    atomic_store(&r.done,1);
    for(unsigned i=0;i<4;i++) CHECK(!pthread_join(threads[i],NULL));
    CHECK(!pthread_join(observer,NULL));CHECK(r.reads>0);
    CHECK(!gate_destroy(&gate));CHECK(!gate_destroy(&other));
    printf("PASS 4000 transactions, 16000 logged words, revocation, thread/epoch rejection, overflow; reads=%llu\n",(unsigned long long)r.reads);
}
