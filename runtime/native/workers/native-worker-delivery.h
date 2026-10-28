#ifndef NEOCLR_NATIVE_WORKER_DELIVERY_H
#define NEOCLR_NATIVE_WORKER_DELIVERY_H
#include "native-workers.h"
/* Private POSIX NEOCLR_WORKER_DELIVERY experiment. Owner-only lazy registration.
 * Returns a borrowed nonblocking close-on-exec descriptor, or -1 on failure.
 * Poll for readability only: never read, write, duplicate or close it.
 * Readability means at least one terminal, not-yet-dispatched ticket exists;
 * dispatch/take refresh it under the pool gate. Notification is coalesced,
 * carries no managed pointers and does not consume captures/results.
 * Remove from every external wait set before destroy. Remains valid across close.
 * Existing dispatch budget, exactly-once notification and explicit take apply.
 * No general managed Task.Run or foreign-heap capture admission is implied. */
int neoclr_workers_completion_descriptor(neoclr_workers *);
#endif
