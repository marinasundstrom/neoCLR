#define _POSIX_C_SOURCE 200809L
#include "session.h"
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <time.h>
struct lane { size_t count; uint64_t attempted, dropped; unsigned claimed, role; };
struct neoclr_trace_session {
    pthread_t owner;
    pthread_mutex_t gate;
    unsigned lanes, active, closed;
    size_t capacity;
    uint64_t identity, origin;
    neoclr_trace_event *events;
    struct lane lane[16];
};
static _Atomic uint64_t identities=1;
static _Thread_local neoclr_trace_session *current;
static _Thread_local unsigned current_lane;
static uint64_t now(void) {
    struct timespec t;
    if (clock_gettime(CLOCK_MONOTONIC,&t)) return 0;
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
neoclr_trace_session *neoclr_trace_create(unsigned lanes,size_t capacity) {
    if (!lanes || lanes>16 || !capacity || capacity>65536) return NULL;
    neoclr_trace_session *s=calloc(1,sizeof(*s));
    if (!s) return NULL;
    s->events=calloc(lanes*capacity,sizeof(*s->events));
    s->origin=now();
    if (!s->events || !s->origin) { free(s->events);free(s);return NULL; }
    if (pthread_mutex_init(&s->gate,NULL)) { free(s->events);free(s);return NULL; }
    s->owner=pthread_self();s->lanes=lanes;s->capacity=capacity;
    s->identity=atomic_fetch_add(&identities,1);
    if (!s->identity || s->identity==UINT64_MAX) abort();
    return s;
}
int neoclr_trace_attach(neoclr_trace_session *s,unsigned lane,unsigned role) {
    if (!s || current || lane>=s->lanes) return 0;
    if (pthread_mutex_lock(&s->gate)) abort();
    int ok=!s->closed && !s->lane[lane].claimed;
    if (ok) { s->lane[lane].claimed=1;s->lane[lane].role=role;s->active++;current=s;current_lane=lane; }
    if (pthread_mutex_unlock(&s->gate)) abort();
    return ok;
}
int neoclr_trace_detach(void) {
    neoclr_trace_session *s=current;
    if (!s) return 0;
    if (pthread_mutex_lock(&s->gate)) abort();
    s->active--;current=NULL;
    if (pthread_mutex_unlock(&s->gate)) abort();
    return 1;
}
int neoclr_trace_close(neoclr_trace_session *s) {
    if (!s || !pthread_equal(s->owner,pthread_self())) return 0;
    if (pthread_mutex_lock(&s->gate)) abort();
    int ok=!s->active;
    if (ok) s->closed=1;
    if (pthread_mutex_unlock(&s->gate)) abort();
    return ok;
}
int neoclr_trace_view_lane(neoclr_trace_session *s,unsigned lane,neoclr_trace_view *out) {
    if (!s || !out || !pthread_equal(s->owner,pthread_self()) || !s->closed || lane>=s->lanes) return 0;
    struct lane *b=&s->lane[lane];
    *out=(neoclr_trace_view){s->events+lane*s->capacity,b->count,s->capacity,
        s->identity,s->origin,b->attempted,b->dropped,b->role};
    return 1;
}
int neoclr_trace_destroy(neoclr_trace_session *s) {
    if (!s || !pthread_equal(s->owner,pthread_self()) || !s->closed) return 0;
    if (pthread_mutex_destroy(&s->gate)) abort();
    free(s->events);free(s);return 1;
}
void neoclr_trace_record(unsigned domain,unsigned kind,uint64_t source,uint64_t subject,uint64_t related,unsigned detail) {
    neoclr_trace_session *s=current;
    if (!s) return;
    struct lane *b=&s->lane[current_lane];uint64_t seq=++b->attempted;
    if (b->count==s->capacity) { b->dropped++;return; }
    uint64_t ns=now();if (!ns) { b->dropped++;return; }
    s->events[current_lane*s->capacity+b->count++]=(neoclr_trace_event){ns,seq,source,subject,related,domain,kind,detail};
}
