# Applying background traces in Serve (2026-10-18)

`apply.c` advances the [tracing-only experiment](README.md) to owner-side live
reclamation in the real asynchronous Serve host. This is an isolated, bounded AOT
experiment, not a production collector or a change to `--gc mark-sweep`.

## Safety and scheduling contract

The single owner captures the existing bounded private snapshot at a published
boundary. A single worker traces that copy. At a subsequent valid boundary, the
owner uses the already-qualified `reclaim.inc` coordinator: reject changed epochs
before dereferencing live identities, validate the complete arena and captured
block identities, import marks, retain allocations made since capture, and run the
existing resource cleanup/sweep on the owner. Worker completion is published under
its mutex; cancellation alone never transfers ownership. No live heap is traced
by the worker. Existing host-root and parked-root invalidation hooks remain enabled.

Boundary collection uses the existing 16-allocation/64-boundary pacing trigger;
idle calls can start a cycle immediately. While a current cycle is pending, these
collections defer without blocking. A stale pending cycle falls back to the original
synchronous boundary/idle policy. Capture failure also falls back synchronously.
Rooted allocation pressure is unchanged: it synchronously collects and invalidates
the outstanding snapshot. Successful application resets pacing. Shutdown cancels
and joins before async, socket, task, TLS and arena teardown. At most one snapshot
is outstanding; the inherited limits are 1 MiB, 4,096 root words, 1,024 conditional
pairs and 128 parked frames. Larger cases fall back instead of expanding these caps.

This preserves the .NET expectation that live async state survives suspension;
there is no new managed API or change in reference/value semantics. It differs
internally from the .NET concurrent collector: this POC copies a bounded arena and
conservatively discards on scheduler/root transitions rather than introducing a
production concurrent write barrier. It reuses the existing
[coordinator qualification](../concurrent-gc-walker/README.md) and research instead
of treating unchanged epochs alone as sufficient. Copying, validation, stale work,
and changed collection cadence all cost CPU; application is not evidence of a
throughput improvement. Broader interpreter/JIT, multi-owner/context reuse, TLS,
large heaps and terminal guest-fault integration remain unqualified.

## Qualification

The [sanitized build](evidence/apply/safety-build.json) includes a deterministic
adapter test with a gated worker. It proves live garbage reclamation, retention of
new allocations and a live value, invalid-boundary rejection, actual rooted
allocation pressure invalidating pending results, stale-result discard, forced
pending cancellation/join, an intact arena guard and full final reclamation.
The inherited coordinator tests cover conditional roots, resource cleanup,
same-address replacement and heap identity/shape validation.

The [real HTTP recovery test](evidence/apply/recovery.json), with a 96 KiB heap,
passes a malformed request, a partially received request held across other work,
16 concurrent clients, a recovery request, and explicit control shutdown. It
validates exact bodies/framing, 18 ordinary successes and one expected failure.
523 applied cycles reclaim 319,354 payload bytes; all 8,200 allocations are reclaimed
at exit, with no trace errors or rejected captures. The
[release build](evidence/apply/release-build.json) and
[release recovery test](evidence/apply/release-recovery.json) also pass.

A [10-second sanitizer load](evidence/apply/load.json) offers 2,000 arrivals at
200 req/s: 1,678 are admitted and validate, and 322 are unsent at the generator's
64-slot cap. It is an overload qualification, **not a performance win**. The
[GC report](evidence/apply/load-gc.json) covers warm-up/control too and verifies
actual application, reclamation and clean teardown. C adapters use UBSan/bounds;
retained guest code/static archives are not instrumented. An earlier precheck also
missed arrivals (271/2,000); neither sanitizer run establishes sustained capacity.

## Reproduce

```sh
python3 docs/experiments/concurrent-gc-http/build.py \
  --native-build target/http-concurrent-readiness/native \
  --walker-build target/gc-lifecycle-qualified-v2 \
  --mode apply --heap-bytes 98304 --output target/http-apply-new
python3 docs/experiments/concurrent-gc-http/validate.py \
  --build target/http-apply-new --output target/http-apply-recovery-new --apply
```

For timing, build a fresh directory with `--mode apply --release`, without the
small-heap override, and validate it before load. Use the same retained guest and
32 MiB arena as the synchronous baseline. Compare fixed offered rates and reverse
order, retaining every missed arrival, offered latency, generator lag, server CPU,
GC counters and exit/response validation. No HTTP speedup is claimed by this slice.

[Completed HTTP comparison](LOAD.md): the applying policy costs about 23–24%
more CPU per completion without a demonstrated throughput gain. Both initial and
confirmation trials are retained, including generator-lag failures.
