# External I/O progress: host-side ownership probe

**2026-09-23 · M1 / S0, partial experimental evidence.** This is an isolated Rust
experiment, not a guest API, a socket backend or a change to neoCLR execution.
It tests the host half of [S0](../../http-poc-roadmap.md#small-executable-slices).
The [platform roadmap](../../platform-roadmap.md) remains authoritative.

## Question

Can an invocation retain an operation's destination, run unrelated ready work,
wait when its callback queue is empty and apply a delayed external completion on
its own thread, without giving the producer access to guest storage?

The current runtime drains the default TaskQueue on entry return in
[src/vm.rs](../../../src/vm.rs). Existing [workers](../../../src/workers.rs) exchange
owned text but JoinWorker waits synchronously. Neither path establishes the proposed
external-I/O lifecycle. Do not change public streams or Task contracts merely to fit
this experiment.

## Reduced implementation

[probe.rs](probe.rs) keeps a bounded pending-operation registry and an invocation-local
`Rc<RefCell<Vec<u8>>>` destination. Rc intentionally cannot cross OS threads. A fake
producer owns a resource stand-in and accepts a completion or cancellation command.
It releases that resource before sending an owned byte payload or cancellation
acknowledgement through a channel. Only the invocation loop writes the destination.

The loop polls a completion between ready callbacks. When no callback is ready but
operations remain, it waits for an event instead of declaring the invocation complete.
A cancellation request leaves the operation pending; the producer's acknowledgement
makes it terminal. The controlled backend processes its first command, so either
completion or cancellation may win. Late/duplicate events cannot write the destination.
Invocation teardown requests cancellation and joins producers before releasing retained
buffers. IDs are not reused during the invocation; the pending-operation cap is eight.

This models a **copy/owned-transfer boundary** rather than pinning or lending guest
memory to native code. A real backend would receive into host-owned buffers; applying
completion to a managed array may need another copy and byte-range validation. The
Vec replacement here is not that managed-array implementation.

## Comparison and provisional result

Reuse the [M1 primary-source comparisons](../../http-poc-roadmap.md#evidence-comparisons-and-costs)
(.NET 10 Socket.ReceiveAsync and Stream.ReadAsync; Tokio AsyncRead, checked 2026-09-23)
and [Task completion research](../task-contract/README.md#comparison-and-tradeoffs).
.NET's memory-based async receive and cancellation provide the application baseline;
this probe addresses host delivery and ownership, not API compatibility. Tokio provides
an alternative directional/polling model, not a selected dependency or scheduler.

| Alternative | Benefit | Cost / unanswered question |
| --- | --- | --- |
| Owned host payload, invocation-local delivery (this probe) | Producer cannot mutate guest objects; completion and cleanup have one application point | Extra buffers/copies; bounded payload sizes and actual GC roots still need implementation |
| Retain/pin managed memory for direct native writes | Potentially fewer copies | Requires proven GC stability, alias restrictions and cancellation quiescence; not tested here |
| Join an isolated worker from a guest callback | Reuses today's worker mechanism | Waiting blocks callback dispatch; existing workers accept text, not shared byte buffers |
| Readiness/completion backend with a wakeup channel | Can multiplex real I/O without a worker per operation | OS registration, cancellation and shutdown behavior need separate validation |

**Result:** the owned-payload boundary is viable in this reduced host model. Keep it
as a candidate for the first runtime bridge. No evidence here selects a production
backend, ambient versus explicit cancellation, public scheduler or buffer API. No
performance improvement is claimed.

## Reproduce

From the repository root, with Rust installed:

```sh
mkdir -p target/experiments/external-io-progress
rustc --edition 2024 --test docs/experiments/external-io-progress/probe.rs -o target/experiments/external-io-progress/probe
target/experiments/external-io-progress/probe
```

Validated locally with `rustc 1.95.0 (59807616e 2026-04-14)`: **7 passed, 0 failed**.
The seven cases check:

- No ready callbacks and an initially empty event queue, followed by external
  completion; dropping the caller's strong reference does not lose the destination.
- Unrelated ready work executes while a producer remains pending.
- Cancellation remains a request until acknowledged and does not write the buffer.
- Both deterministic completion/cancellation orderings produce one terminal result;
  a late duplicate cannot overwrite the destination.
- A later operation completes without waiting for an earlier pending operation.
- An already-available completion is handled before a ready callback backlog.
- Invocation teardown releases all eight pending resources and retained buffers.

These cases use channels rather than sleeps to control ordering. The five-second
receive timeout is a test watchdog, not an I/O deadline contract. The race cases
exercise both controlled orderings, not exhaustive concurrency model checking.

## Remaining S0 work

Rc retention is **not guest GC evidence**. Next connect a reduced host completion
source to the interpreter's invocation-owned registry, trace the Task/Promise and
managed destination through actual GC, and deliver completion through the existing
Task queue. Demonstrate one runnable Raven caller and direct-IL negative cases for
invalid buffers/handles and lifetime bypasses before calling S0 complete.

Also resolve producer failure/disconnection, operation and payload budgets, native
read errors, full queues and invocation cancellation/Fault teardown. The fake backend
always cooperates; its join is not proof of bounded OS cancellation. The standalone
probe uses one thread per producer and unbounded channels inside its eight-operation
fixture; neither is a production resource policy. It does not test actual sockets,
partial I/O, deadlines, byte-range writes, cross-platform backends or instruction
budget/fairness enforcement. Memory-copy stream cases (S1) remain a separate task.
