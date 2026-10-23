#ifndef NEOCLR_HTTP_WORKERS_H
#define NEOCLR_HTTP_WORKERS_H
/* Opt-in POSIX application owners. Each worker runs a complete invocation with
 * a private heap, statics, Task queue, socket scope and pinned async stacks.
 * Only the first listener is shared (identical IPv4/port/backlog required).
 * Descriptor duplication shares the kernel accept queue, never managed state.
 * Callback/state outlive the joined run. No nested run or resource migration. */
#include <stdint.h>
int neoclr_http_workers_run(unsigned count, int (*entry)(void *), void *state);
/* Called by the socket adapter after ordinary argument validation.
 * 0: ordinary listener; 1: output owns a duplicate; -1: errno describes failure. */
int neoclr_http_workers_listener(uint32_t address, int port, int backlog, int *output);
int neoclr_http_workers_stopping(void);
#endif
