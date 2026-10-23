#define _POSIX_C_SOURCE 200809L
#include "http-workers.h"
#include "native-workers.h"
#include "socket-os.h"
#include <pthread.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <errno.h>
typedef struct {
    pthread_mutex_t gate;
    atomic_int stopping;
    int listener, port, backlog;
    uint32_t address;
    int (*entry)(void *);
    void *state;
    unsigned completed;
    int failed;
} group;
static _Thread_local group *current;
static _Thread_local int primary_pending;
int neoclr_http_workers_stopping(void) {
    return current && atomic_load(&current->stopping);
}
int neoclr_http_workers_listener(uint32_t address, int port, int backlog, int *output) {
    group *g = current;
    if (!g || !primary_pending) return 0;
    primary_pending = 0;
    if (pthread_mutex_lock(&g->gate)) abort();
    int fd = -1;
    if (atomic_load(&g->stopping)) errno = ECANCELED;
    else if (g->listener >= 0 && (g->address != address || g->port != port || g->backlog != backlog)) errno = EINVAL;
    else {
        if (g->listener < 0) {
            int listener = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
            struct sockaddr_in endpoint = {.sin_family = AF_INET, .sin_port = htons((uint16_t)port)};
            endpoint.sin_addr.s_addr = htonl(address);
            if (listener >= 0) {
                if (neoclr_os_configure(listener) || bind(listener, (struct sockaddr *)&endpoint, sizeof(endpoint)) || listen(listener, backlog)) {
                    int error = errno; close(listener); errno = error;
                } else {
                    g->listener = listener; g->address = address; g->port = port; g->backlog = backlog;
                }
            }
        }
        if (g->listener >= 0) {
            fd = dup(g->listener);
            if (fd >= 0 && neoclr_os_configure(fd)) {
                int error = errno; close(fd); fd = -1; errno = error;
            }
        }
    }
    int error = errno;
    if (fd < 0) atomic_store(&g->stopping, 1);
    if (pthread_mutex_unlock(&g->gate)) abort();
    if (fd < 0) { errno = error; return -1; }
    *output = fd;
    return 1;
}
static int application(neoclr_worker_job *job, void *state) {
    (void)job;
    group *g = state;
    if (current) abort();
    current = g; primary_pending = 1;
    int status = g->entry(g->state);
    if (status) atomic_store(&g->stopping, 1);
    current = NULL; primary_pending = 0;
    return status;
}
static void completed(neoclr_workers *pool, neoclr_worker_handle ticket, void *state) {
    group *g = state;
    int outcome; neoclr_worker_value value; neoclr_worker_handle root;
    if (neoclr_workers_take(pool, ticket, &outcome, &value, &root) != 1 || root.pool) abort();
    ++g->completed;
    if (outcome != NEOCLR_WORKER_OK) g->failed = 1;
}
int neoclr_http_workers_run(unsigned count, int (*entry)(void *), void *state) {
    if (!count || count > 4 || !entry || current) return 2;
    group g = {.listener = -1, .entry = entry, .state = state};
    if (pthread_mutex_init(&g.gate, NULL)) return 2;
    neoclr_workers *pool = neoclr_workers_create(count, count, 8192);
    if (!pool) { pthread_mutex_destroy(&g.gate); return 2; }
    neoclr_worker_handle root, tickets[4]; unsigned submitted = 0;
    if (!neoclr_workers_root_new(pool, 0, &root)) abort();
    for (; submitted < count; submitted++) {
        if (!neoclr_workers_submit(pool, root, application, &g, &tickets[submitted])) {
            atomic_store(&g.stopping, 1); g.failed = 1; break;
        }
    }
    if (!neoclr_workers_root_release(pool, root)) abort();
    for (unsigned i = 0; i < submitted; i++) {
        int status;
        do { status = neoclr_workers_wait(pool, tickets[i], 1000); } while (!status);
        if (status < 0) abort();
    }
    if (neoclr_workers_dispatch(pool, submitted, completed, &g) != (int)submitted) abort();
    if (!neoclr_workers_close(pool) || !neoclr_workers_destroy(pool)) abort();
    if (g.listener >= 0 && close(g.listener)) g.failed = 1;
    if (pthread_mutex_destroy(&g.gate)) abort();
    return g.failed || g.completed != count ? 2 : 0;
}
