/* Experimental standalone profiler transport. Not a stable runtime ABI. */
#include "attach.h"
#include "../aot-console/native-gc.h"
#ifndef NEOCLR_GC_PROFILE
#error diagnostic attachment requires GC profile instrumentation
#endif
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <poll.h>
#include <pthread.h>
#include <stdatomic.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/random.h>
#include <sys/un.h>
#include <time.h>
#include <unistd.h>

static atomic_int enabled, requested, stopping;
static pthread_t owner, worker;
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
static int listener = -1;
static char endpoint[sizeof(((struct sockaddr_un *)0)->sun_path)], instance[33];
static struct {
    uint64_t sequence, timestamp;
    neoclr_gc_statistics counters;
    neoclr_gc_profile profile;
} snapshot;

static uint64_t now(void) {
    struct timespec ts;
    if (clock_gettime(CLOCK_MONOTONIC, &ts)) abort();
    return (uint64_t)ts.tv_sec * UINT64_C(1000000000) + (uint64_t)ts.tv_nsec;
}
void neoclr_diagnostics_publish_v1(void) {
    if (!atomic_load_explicit(&enabled, memory_order_relaxed) ||
        !atomic_load_explicit(&requested, memory_order_relaxed) ||
        !pthread_equal(owner, pthread_self())) return;
    /* A slow reader never blocks the runtime owner. Retry at a later boundary. */
    if (pthread_mutex_trylock(&lock)) return;
    snapshot.counters = neoclr_gc_statistics_v1();
    snapshot.profile = neoclr_gc_profile_v1();
    snapshot.timestamp = now();
    snapshot.sequence++;
    atomic_store(&requested, 0);
    pthread_mutex_unlock(&lock);
}
static int ready(int fd, short events, uint64_t deadline) {
    while (!atomic_load(&stopping)) {
        uint64_t time = now();
        if (time >= deadline) return 0;
        struct pollfd p = {fd, events, 0};
        int result = poll(&p, 1, (int)((deadline - time) / 1000000 + 1));
        if (result > 0) return (p.revents & events) != 0;
        if (result < 0 && errno != EINTR) return 0;
    }
    return 0;
}
static void respond(int fd) {
    char request[64], response[4096];
    size_t count = 0;
    uint64_t deadline = now() + UINT64_C(250000000);
    while (count < sizeof(request) - 1 && ready(fd, POLLIN, deadline)) {
        ssize_t n = recv(fd, request + count, 1, 0);
        if (n != 1) break;
        if (request[count++] == '\n') break;
    }
    request[count] = 0;
    if (strcmp(request, "NEOCLR/1 SNAPSHOT\n")) {
        snprintf(response, sizeof(response), "{\"version\":1,\"error\":\"unsupported-request\"}\n");
    } else {
        pthread_mutex_lock(&lock);
        uint64_t previous = snapshot.sequence;
        atomic_store(&requested, 1);
        pthread_mutex_unlock(&lock);
        uint64_t until = now() + UINT64_C(100000000);
        while (atomic_load(&requested) && now() < until && !atomic_load(&stopping)) {
            struct timespec delay = {0, 1000000};
            nanosleep(&delay, NULL);
        }
        /* Copy under the lock, format and write after releasing it. */
        pthread_mutex_lock(&lock);
        neoclr_gc_statistics c = snapshot.counters;
        neoclr_gc_profile p = snapshot.profile;
        uint64_t seq = snapshot.sequence, stamp = snapshot.timestamp;
        pthread_mutex_unlock(&lock);
        int used = snprintf(response, sizeof(response),
            "{\"version\":1,\"pid\":%ld,\"instance\":\"%s\","
            "\"capabilities\":[\"gc-snapshot\"],\"scope\":\"runtime-owner-thread\","
            "\"available\":%s,\"fresh\":%s,\"sequence\":%" PRIu64
            ",\"timestampNanoseconds\":%" PRIu64 ",\"ageNanoseconds\":%" PRIu64
            ",\"collections\":%" PRIu64 ",\"allocations\":%" PRIu64
            ",\"reclaimedAllocations\":%" PRIu64 ",\"reclaimedPayloadBytes\":%" PRIu64
            ",\"gcAttempts\":%" PRIu64 ",\"gcFailures\":%" PRIu64
            ",\"gcNanoseconds\":%" PRIu64 ",\"gcMaximumNanoseconds\":%" PRIu64
            ",\"allocatedPayloadBytes\":%" PRIu64 ",\"livePayloadBytesAfterCollection\":%" PRIu64
            ",\"liveBlocksAfterCollection\":%" PRIu64 ",\"peakLivePayloadBytesAfterCollection\":%" PRIu64
            ",\"peakArenaUsedBytes\":%" PRIu64 ",\"gcPauseBuckets\":[",
            (long)getpid(), instance, seq ? "true" : "false", seq > previous ? "true" : "false",
            seq, stamp, seq ? now() - stamp : 0, c.collections, c.allocations,
            c.reclaimed_allocations, c.reclaimed_bytes, p.attempts, p.failures,
            p.nanoseconds, p.maximum_nanoseconds, p.allocated_bytes, p.live_bytes,
            p.live_blocks, p.peak_live_bytes, p.peak_arena_used);
        for (unsigned i = 0; i < NEOCLR_GC_PAUSE_BUCKETS; i++) {
            if (used < 0 || (size_t)used >= sizeof(response)) abort();
            used += snprintf(response + used, sizeof(response) - (size_t)used,
                             "%s%" PRIu64, i ? "," : "", p.pause_buckets[i]);
        }
        if (used < 0 || (size_t)used + 4 > sizeof(response)) abort();
        memcpy(response + used, "]}\n", 4);
    }
    size_t sent = 0, length = strlen(response);
    deadline = now() + UINT64_C(250000000);
    while (sent < length && ready(fd, POLLOUT, deadline)) {
        #ifdef MSG_NOSIGNAL
        ssize_t n = send(fd, response + sent, length - sent, MSG_NOSIGNAL);
#else
        ssize_t n = send(fd, response + sent, length - sent, 0);
#endif
        if (n <= 0) break;
        sent += (size_t)n;
    }
}
static void *serve(void *unused) {
    (void)unused;
    while (!atomic_load(&stopping)) {
        if (!ready(listener, POLLIN, now() + UINT64_C(50000000))) continue;
        int fd = accept(listener, NULL, NULL);
        if (fd < 0) continue;
        fcntl(fd, F_SETFD, FD_CLOEXEC);
        fcntl(fd, F_SETFL, O_NONBLOCK);
#ifdef SO_NOSIGPIPE
        int one = 1;
        setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, &one, sizeof(one));
#endif
        respond(fd);
        close(fd);
    }
    return NULL;
}
__attribute__((destructor)) static void stop(void) {
    if (!atomic_load(&enabled)) return;
    atomic_store(&enabled, 0);
    atomic_store(&stopping, 1);
    pthread_join(worker, NULL);
    close(listener);
    unlink(endpoint);
}
__attribute__((constructor)) static void start(void) {
    const char *directory = getenv("NEOCLR_DIAGNOSTICS_DIRECTORY");
    if (!directory || !*directory) return;
    struct stat info;
    /* Explicit caller-owned private directory; never replace an existing socket. */
    if (lstat(directory, &info) || !S_ISDIR(info.st_mode) ||
        info.st_uid != geteuid() || (info.st_mode & 0777) != 0700) goto invalid;
    int length = snprintf(endpoint, sizeof(endpoint), "%s/neoclr-%ld.sock", directory, (long)getpid());
    if (length < 0 || (size_t)length >= sizeof(endpoint)) goto invalid;
    unsigned char random[16];
    if (getentropy(random, sizeof(random))) goto invalid;
    for (unsigned i = 0; i < sizeof(random); i++) snprintf(instance + i * 2, 3, "%02x", random[i]);
    listener = socket(AF_UNIX, SOCK_STREAM, 0);
    if (listener < 0) goto invalid;
    fcntl(listener, F_SETFD, FD_CLOEXEC);
    fcntl(listener, F_SETFL, O_NONBLOCK);
    struct sockaddr_un address = {0};
    address.sun_family = AF_UNIX;
    memcpy(address.sun_path, endpoint, (size_t)length + 1);
    if (bind(listener, (struct sockaddr *)&address, sizeof(address))) goto close_invalid;
    if (chmod(endpoint, 0600) || listen(listener, 4)) goto unlink_invalid;
    owner = pthread_self();
    atomic_store(&requested, 1);
    atomic_store(&enabled, 1);
    if (pthread_create(&worker, NULL, serve, NULL)) {
        atomic_store(&enabled, 0);
        goto unlink_invalid;
    }
    return;
unlink_invalid:
    unlink(endpoint);
close_invalid:
    close(listener);
invalid:
    fputs("neoCLR: diagnostics endpoint unavailable\n", stderr);
    /* Explicit diagnostics configuration errors must not look like successful attach. */
    exit(78);
}
