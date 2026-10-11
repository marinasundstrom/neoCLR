# JIT timer completion layer

2026-10-11, private macOS ARM64 managed JIT. The invocation host now enters the existing
native operation scope and polls at the exact rooted async-entry boundary. Timer callback
roots survive delivery until the next poll or scope teardown. On return/fault the host
leaves the operation scope before releasing the task queue, collecting and unloading code.
This reuses the AOT timer/Task contracts; no public API or compiler bridge changes.

The existing timer suite passes in interpreter/AOT/JIT: one shot, stop, disposal, periodic
stop, callback rescheduling, async non-overlap/restart, disposal during async work, and
immediate/pending callback cancellation. A timer-fault case also leaves a second timer
pending; scope teardown cancels and unroots it before code release. No timer performance
claim is made. This is a polled host, not runtime suspension or a new scheduler.

[Timer qualification](validation.json), [abandoned-operation fault qualification](abandoned-validation.json),
[queued-async regression](async-validation.json) and [console regression](console-validation.json).
The latter retains 20 passing framework tests and identical AOT object bytes.

Reproduce with `docs/experiments/jit-async/verify.py --timers`, using the bundle,
console-qualification and fresh output arguments documented in the
[async probe](../jit-async/README.md). `--case timer-fault` selects the strengthened
fault guard alone. The subsequent [HTTP qualification](../jit-http/README.md) now
covers socket completions and client/server integration; execution defaults remain unchanged. No website build or API snapshot change is required for private hosting.
