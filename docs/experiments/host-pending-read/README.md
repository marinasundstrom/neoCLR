# Host-backed pending read: deferred cancellation

Development experiment, 2026-09-23. This extends the queued
[pending-read model](../pending-read/README.md) through the existing experimental
[worker notification adapter](../delayed-copy/README.md). A real isolated host worker
returns owned text; the invocation receives its notification, joins the producer,
and resumes ordinary Raven awaits. The producer never holds a guest reference.
No runtime service or public Storage API is added.

## Reproduce

From a checkout with matching development artifacts:

```sh
python3 docs/experiments/delayed-copy/verify.py --toolchain-root /path/to/development/bundle --consumer-root docs/experiments/host-pending-read
```

The shared verifier compiles the existing experimental Workers adapter and substitutes
its generated fragments only in a temporary System library. Normal installed Thread
APIs still use queued joins. Building this consumer with an ordinary SDK alone does
not reproduce the notification experiment. The verifier checks the first two output
lines in order, then all five outcomes exactly once in any producer completion order.
It requires actual collections and zero final live guest objects.

## Product and policy

Each operation privately owns `[9,9,9,9]` and can receive at most two UTF-8 bytes
into positions 1–2. Its Task<Result<int,string>> retains the existing distinction
between expected payload failure and cancellation. Five independent jobs exercise:

- A request before delivery: Task stays pending, then cancels after the joined
  producer's payload is discarded; bytes remain unchanged.
- Successful `Hi` delivery: a later cancellation request cannot replace completion.
- An oversized payload: expected error, no destination mutation.
- An empty payload: zero bytes copied.
- A short `H` payload: only the first requested byte changes.

Main returns after allocation pressure. An unrelated ready callback runs before
consumer outcomes. Registered worker callbacks, Promise continuations and generated
state machines retain the pending graph through actual GC. This uses the existing
VM's root transfer into TaskQueue.Post, not a second model queue or collector.
Dedicated-worker Join completes before the adapter publishes its string result;
that existing boundary establishes host-thread acknowledgement for this sample.

**Cancellation is deferred, not producer interruption.** RequestCancel records
intent on the invocation thread. It never cancels the worker and never completes
the consumer early. On delivery, a recorded request discards the result and cancels
the Promise. If delivery already happened, RequestCancel returns false. A host job
may physically finish before the request; only invocation-side delivery order is
the policy's decision point. No sleep attempts to force OS-thread timing.

This is narrower than the earlier model, which allowed an explicit producer event
to win even after a request. Both are useful contract alternatives; this experiment
selects request-before-delivery precedence only for its adapter. It is not a permanent
Storage cancellation policy. Close, multiple concurrent operations, linked tokens,
explicit queue affinity and cancellation of a blocked native call remain open.
Producer Faults retain the adapter's existing invocation-Fault behavior; only an
oversized *successful* payload becomes an expected read error in this sample.

## Comparison, costs and limits

Reuse the [.NET cooperative cancellation and ReadAsync comparison](../pending-read/README.md#net-comparison-and-alternatives).
The existing .NET baseline distinguishes a request from an operation's response;
neither immediate interruption nor a universal race winner follows from a token.
This sample demonstrates one response policy on neoCLR's current state machines,
without changing metadata, task representation or scheduling contracts.

Immediate Promise cancellation would let a caller treat the operation as terminal
before host acknowledgement. Deferred cancellation avoids that early signal but
cannot improve producer completion latency and may wait indefinitely. Producer-side
cooperative cancellation is an alternative requiring a per-operation channel and
acknowledgement; the current worker API only supplies invocation teardown cancellation.
The owned-text boundary incurs encoding and copying and is not a byte-oriented I/O
backend or a performance claim. Payload budgets and host cancellation still follow
the existing worker implementation.

An instance async draft was rejected by the bridge's private application field
access rules. AwaitDelivery therefore awaits outside the owner and calls Finish,
which mutates its private state synchronously. This sample-only method is not a
secured producer endpoint or selected public API. A reduced general compiler/bridge
access test remains a follow-up; no compiler change is made here.

## Next evidence

Use this host-delivery boundary to investigate a per-operation producer cancellation
channel and bounded acknowledgement, distinguishing it from invocation cancellation.
Also decide queue affinity and byte payload accounting before adding a filesystem
producer. Do not generalize a no-op String worker into a guarantee for noncooperative
OS calls, async Storage, safe buffer borrowing or bounded shutdown.
