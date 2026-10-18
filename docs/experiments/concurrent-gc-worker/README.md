# Concurrent GC worker infrastructure proof of concept

2026-10-18. Isolated POSIX worker experiment, not a selectable collector. The author
asked to validate concurrent infrastructure before integrating parts of the existing
GC, keeping the solution practical. The [phase measurements](../../../benchmarks/http-server/serve/gc-phases-20261018/README.md)
identify tracing as the largest sampled collector cost; this slice validates the
handoff independently of graph tracing or reclamation.

## Small contract

One owner, one persistent worker, at most one outstanding job; no queue, provider
framework or live-heap access. Job input must remain immutable to the owner while the
worker uses it. Worker scratch/output belongs exclusively to the worker until the
owner consumes the completed result or destroys/joins the worker. The caller owns
allocation and cleanup of that storage. Submission refuses both running and completed
but unconsumed jobs, preventing accidental replacement or reuse.

A mutex/condition variable publishes the job and completed result, including ordinary
scratch writes. Only cancellation is atomic. A cancellation request does **not** grant
permission to free scratch; the owner must wait for completion. Destroy cancels, waits
for callback return, stops and joins the worker, then releases synchronization state.
The callback must check cancellation at bounded work intervals; an uncooperative
callback can block shutdown. The implementation cannot forcibly terminate arbitrary C
code. APIs are owner-thread-only and must not be called from the callback.

No managed cleanup runs on the worker. No managed heap, roots, native descriptors or
runtime TLS are shared by this harness. This avoids implying that a correct pthread
handoff alone makes concurrent heap scanning safe.

## Validation and limits

`run.py --output NEW_DIRECTORY` uses the repository Xcode SDK environment and builds
with UBSan/bounds by default. It retains input hashes, commands, outcomes and timeouts.
The final test passes:

- 1,000 repeated submissions on one worker, alternating normal completion and
  cooperative cancellation; owner progress is required after worker entry and before
  normal completion. A second submission is refused while the first is outstanding.
- Worker writes published output; the owner checks it and frees the payload only
  after consuming the result. A consumed result cannot be taken twice.
- 100 create/submit/shutdown races, including immediate shutdown and an already
  executing callback; payloads remain valid until join.
- Deterministic completed-result pickup, callback error propagation, refusal to
  overwrite an unconsumed completed result, completed-job shutdown and idle shutdown.

This establishes functional overlap and exercises lifecycle transitions; it is not
proof of parallel CPU speedup or exhaustive interleaving coverage. Allocation/thread
creation failure branches are handled but not fault-injected. Windows and interpreter
heap integration are not qualified.

The initial ASan/UBSan attempt timed out after 60 seconds. Its captured sample shows
sanitizer initialization before `main`; the initial runner omitted timed-out commands,
so a supplemental record documents the observed timeout. ThreadSanitizer again exits
with SIGSEGV before output, matching the earlier [empty-control failure](../concurrent-gc-snapshot/tsan-control.json).
These attempts used the earlier test before the final completed/error cases; their
hashes identify it. **Neither sanitizer attempt is a pass.** Final UBSan/bounds passes,
but dynamic race checking remains unavailable on this host.

## Next integration boundary

Reuse existing tracing logic on immutable, owner-captured input and worker-private
scratch first; compare marked/retained objects with synchronous collection. Keep the
[conditional-owner counterexample](../concurrent-gc-snapshot/README.md) as a regression.
Root capture, conditional-root fixed points, host-handle access during a cycle and
publication/reclamation remain separate unresolved collector problems. Do not attach
the current rejected snapshot algorithm to this worker and call it safe.

The [existing collector research](../../design/concurrent-gc-evaluation.md) remains the
basis for .NET/Go comparison. Managed GC semantics and production runtime code are
unchanged; no throughput or pause improvement is claimed by worker tests.
