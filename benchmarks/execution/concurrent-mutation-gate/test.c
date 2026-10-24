#include "gate.h"
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr,"line %d\n",__LINE__); abort(); } } while (0)

typedef struct { mutation_gate *gate; uint64_t epoch; uint64_t *payload; atomic_int ready, done; unsigned reads; } reader;
static void *observe(void *arg) {
    reader *r = arg;
    uint64_t copy[4];
    CHECK(gate_begin(r->gate, &copy[0]) == GATE_INVALID);
    CHECK(gate_write(r->gate, r->payload, 24, 8, NULL, 0) == GATE_INVALID);
    CHECK(gate_stop(r->gate) == GATE_INVALID);
    CHECK(gate_destroy(r->gate) == GATE_INVALID);
    while (!atomic_load(&r->done)) {
        CHECK(!gate_read(r->gate, r->epoch, r->payload, 25, 0, copy, 25));
        CHECK(copy[0] == 17 && copy[1] == copy[2]);
        CHECK(((unsigned char *)copy)[24] == (unsigned char)copy[1]);
        r->reads++;
        atomic_store(&r->ready, 1);
    }
    return NULL;
}
static void partial_lanes(void) {
    mutation_gate g;
    CHECK(!gate_init(&g));
    uint64_t epoch, words[GATE_WORDS]; size_t count;
    unsigned char data[35], expected[35], source[35];
    for (size_t offset = 0; offset <= sizeof(data); offset++) {
        for (size_t length = 0; length <= sizeof(data)-offset; length++) {
            memset(data, 0x11, sizeof(data));
            memset(source, 0x73, sizeof(source));
            memcpy(expected, data, sizeof(data));
            memcpy(expected + offset, source, length);
            CHECK(!gate_begin(&g, &epoch));
            mutation_patch patch = {offset,length,source};
            CHECK(!gate_write(&g, data, sizeof(data), 8, &patch, 1));
            CHECK(!memcmp(expected,data,sizeof(data)));
            CHECK(!gate_drain(&g, epoch, words, GATE_WORDS, &count));
            size_t at = 0;
            for (unsigned phase = 0; phase < 2; phase++) {
                for (size_t lane = 8; lane + 8 <= sizeof(data); lane += 8) {
                    if (!length || lane >= offset+length || lane+8 <= offset) continue;
                    uint64_t value = UINT64_C(0x1111111111111111);
                    if (phase) memcpy(&value, expected+lane, 8);
                    CHECK(at < count && words[at++] == value);
                }
            }
            CHECK(at == count);
            CHECK(!gate_stop(&g));
        }
    }
    CHECK(!gate_destroy(&g));
}
static void overflow_and_epochs(void) {
    mutation_gate g; uint64_t epoch, next, words[GATE_WORDS]; size_t count;
    uint64_t data[3] = {17, 1, 1}, value = 2, scratch[3];
    CHECK(!gate_init(&g)); CHECK(!gate_begin(&g, &epoch));
    CHECK(gate_begin(&g, &next) == GATE_INVALID);
    mutation_patch patch = {8,8,&value};
    for (unsigned i = 0; i < GATE_WORDS/2; i++) CHECK(!gate_write(&g,data,24,8,&patch,1));
    CHECK(!gate_drain(&g,epoch,NULL,0,&count) && count == 0);
    CHECK(!gate_drain(&g,epoch,words,1,&count) && count == 1 && words[0] == 1);
    value = 3;
    CHECK(gate_write(&g,data,24,8,&patch,1) == GATE_CANCELLED && data[1] == 2);
    CHECK(gate_write(&g,data,24,8,&patch,1) == GATE_CANCELLED && data[1] == 2);
    CHECK(gate_read(&g,epoch,data,24,0,scratch,24) == GATE_CANCELLED);
    CHECK(gate_drain(&g,epoch,words,GATE_WORDS,&count) == GATE_CANCELLED);
    CHECK(gate_stop(&g) == GATE_CANCELLED);
    CHECK(!gate_write(&g,data,24,8,&patch,1) && data[1] == 3);
    CHECK(!gate_begin(&g,&next) && next != epoch);
    CHECK(gate_read(&g,epoch,data,24,0,scratch,24) == GATE_STALE);
    CHECK(gate_drain(&g,epoch,words,GATE_WORDS,&count) == GATE_STALE);
    CHECK(!gate_drain(&g,next,words,GATE_WORDS,&count) && !count);
    patch.offset = SIZE_MAX;
    CHECK(gate_write(&g,data,24,8,&patch,1) == GATE_INVALID);
    CHECK(gate_read(&g,next,data,24,0,scratch,GATE_SCRATCH+1) == GATE_INVALID);
    CHECK(!gate_stop(&g));
    g.epoch = UINT64_MAX;
    CHECK(gate_begin(&g,&epoch) == GATE_INVALID);
    CHECK(!gate_destroy(&g));
}
static void transactions(void) {
    mutation_gate g; uint64_t epoch, words[GATE_WORDS]; size_t count;
    uint64_t data[4] = {17,0,0,0};
    CHECK(!gate_init(&g)); CHECK(!gate_begin(&g,&epoch));
    reader r = {.gate=&g,.epoch=epoch,.payload=data};
    pthread_t thread; CHECK(!pthread_create(&thread,NULL,observe,&r));
    while (!atomic_load(&r.ready)) { }
    for (uint64_t i = 1; i <= 100000; i++) {
        unsigned char marker = (unsigned char)i;
        mutation_patch patches[] = {{8,8,&i},{16,8,&i},{24,1,&marker}};
        CHECK(!gate_write(&g,data,25,8,patches,3));
        CHECK(!gate_drain(&g,epoch,words,GATE_WORDS,&count) && count == 4);
        CHECK(words[0] == i-1 && words[1] == i-1 && words[2] == i && words[3] == i);
    }
    atomic_store(&r.done,1);
    CHECK(!pthread_join(thread,NULL));
    CHECK(r.reads > 0);
    CHECK(!gate_stop(&g)); CHECK(!gate_destroy(&g));
    printf("100000 transactions, %u coherent concurrent snapshots\n",r.reads);
}
int main(void) {
    partial_lanes(); overflow_and_epochs(); transactions();
    puts("PASS partial lanes, transaction logging, overflow, epochs and thread ownership");
}
