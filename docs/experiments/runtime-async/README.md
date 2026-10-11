# Interpreter runtime-async experiments

Current development status: [async-body invocation](#async-body-invocation-ownership--2026-10-11)
passes the extended Raven consumer and 30 focused checks. Detached activation
checks from the preceding slice remain recorded below. Public Task/compiler await integration
and default switching remain open. Earlier sections record the preceding POC slices.

Development experiment, 2026-10-11. `Yield.rvn` calls a private runtime boundary
from ordinary Raven functions; no async state machine is generated for these calls.
`RuntimeAsyncYield` temporarily retains the current VM frame chain, dispatches at
most one eligible queued callback, and resumes after the call. An empty yield
returns immediately. It neither waits for an operation nor drains all ready sources.

The retained frames preserve locals, evaluation-stack values, heap roots, method
identity and resource charges. Callback faults terminate the invocation. Nested
yields resume their own callers. Task mutation and worker execution reject this
boundary. This is cooperative reentrant execution within one invocation, not a
detached activation, green thread, cross-thread resume or public Task await.

The exact private InternalCall `neoCLR.Runtime.RuntimeAsyncYield()` accepts a
neoCLR unit result or CLI no-result encoding. It requires TaskDispatch capability.
CLI no-result resumes without pushing a unit value. The Raven consumer declares
this service locally; there is no Runtime Contract option or automatic await
mapping. JIT/AOT support is absent and must not be inferred from compiler support.

## Validation

`cargo test --test runtime_async_yield --test scheduler_submission --test timers`
passes 13 focused tests covering retained state and GC roots, callback faults, nested empty yield, one callback
per boundary, frame/evaluation-stack limits, and exact service signatures.
[Recorded consumer commands](validation.json) compile the source with the existing
Raven bundle and execute it with the modified interpreter. Binary/source hashes
identify the tested artifacts. The compiled Raven consumer passes on macOS ARM64. No speedup is asserted and this is not yet the
state-machine versus runtime-suspension benchmark gate.

## Design boundary and next gate

Raven's existing CLR runtime-async lowering targets AsyncHelpers and CLR metadata;
it is not a target-neutral neoCLR insertion contract. Keep that .NET path intact.
A future explicit neoCLR target contract should carry method/await intent and
source information; the runtime should own activation retention, scheduling and
resume/result/fault delivery. Generated state machines remain supported.

Compared with compiler state machines, this probe retains ordinary runtime frames
without synthesized managed carriers. Its costs are retaining the full call chain,
reentrant execution and frame-budget pressure. It does not establish a native stack
strategy or complete logical async stack traces. Next, design operation-triggered
parking with one-shot resumption, atomic registration/completion, cancellation,
lifetime cleanup and logical await links before connecting Raven await lowering.
Detached interpreter activations and JIT/AOT frame/liveness contracts remain open.

The public Task/Promise implementation, API reference and website capability claims
remain unchanged; no public API is introduced by this experiment.

## Operation-triggered suspension POC — 2026-10-11

`Await.rvn` now demonstrates an ordinary nested function waiting for a producer.
The producer itself waits on a 20 ms timer, forces GC, updates its receiver and
completes the original operation with an integer result. Both ordinary call chains
resume with their state intact. A second case consumes an already completed result.
There are no async methods or generated state machines in this consumer. Explicit
private calls are intentional here; user-facing asynchronous code should still use
`await` with the supported managed Task implementation.

The private services are:

| Service | Result | POC behavior |
| --- | --- | --- |
| `RuntimeAsyncCreate()` | Int64 | Create a pending integer-result operation. |
| `RuntimeAsyncDelay(Int32 milliseconds)` | Int64 | Create a nonnegative monotonic timer; completion yields zero. |
| `RuntimeAsyncComplete(Int64 id, Int32 value)` | Boolean | Complete a pending manual operation once; false for invalid/terminal/timer handles. |
| `RuntimeAsyncCancel(Int64 id)` | Boolean | Cancel a pending manual/timer operation once. |
| `RuntimeAsyncAwait(Int64 id)` | Int32 | Retain frames until ready, consume once, or fault on cancellation. |

These are context-owned tokens, not Task replacements. In particular, public Task
multiple-await behavior must remain supported when projections are integrated.
Completed-before-wait results use the same boundary without parking. Pending checks,
callback completion and boundary installation serialize on the dispatcher; there is
no asynchronous host producer ABI yet. Unknown, consumed and recursively awaited
handles fault. Completion cannot overwrite cancellation. Cancelled operations fault
the invocation with ExecutionCancelled; no recoverable managed cancellation outcome
is introduced. Timer readiness is observed lazily by the wait; cancellation can win
before that observation. Timer handles reject explicit producer completion.

Pending execution returns `InstructionProgress::Waiting`, publishes roots and releases
the managed graph guard before host parking. The durable wake latch covers newly
submitted callbacks; waits use the smaller of the timer deadline and the existing
10 ms cancellation polling bound. Tests directly step the VM to prove that repeated
pending checks preserve frames without spending guest instruction budget.

### Validation

The compiled Raven consumer passes on macOS ARM64, alongside 25 focused checks:
22 integration tests and three library checks for host return/resumption, context
ownership, quotas and timer terminal states. See [operation-wait evidence](await-validation.json).
This qualifies the interpreter POC only; no Windows/native result is asserted.

### Reproduce

```sh
CARGO_TARGET_DIR=target/scheduler-wait-validation cargo build --bin neoclr
python3 docs/experiments/runtime-async/validate.py \
  --bundle target/data-cases-bundle \
  --runtime target/scheduler-wait-validation/debug/neoclr \
  --output target/runtime-async-await
```

The script requires a previously verified matching Raven bundle. It records commands,
source/runtime/compiler/reference hashes and output in its validation report. Focused
checks are `cargo test --test runtime_async_await --test runtime_async_yield
--test scheduler_submission --test timers`, plus library filters
`runtime_operation_await_returns_to_host` and `runtime_async::tests`.

### Boundaries and next step

This is an interpreter POC with integer results and at most 64 unconsumed operations
per context. Abandoned operations live until context teardown; consumed slots are
reusable. Worker execution and task mutation reject these services. A parked wait
runs submitted callbacks, but does not pump the legacy I/O/default-TaskQueue adapter.
It does not yet support real HTTP operation awaits or public Task projection.

Frames are retained in LIFO order. If a nested callback parks, its parent cannot
resume independently even when the parent's operation completes. This head-of-line
blocking is a concrete reason to investigate detached activations next. The probe
proves runtime-owned parking/resumption, not scalable green threads. It also does
not supply complete logical async traces, native/JIT frame maps or cross-thread
completion rules. No throughput/latency improvement is claimed. Comparative
state-machine benchmarks belong with an equivalent workload once these paths exist.

The bundled Raven compiler recursed during binding when a property initializer
called the source-local attributed intrinsic (an explicit property type did not
fix it). Passing the operation into a constructor avoids that form. The compiler
issue is tracked in both bridge documents for a separate minimized regression.
Public API reference and website capabilities remain unchanged.

The author subsequently clarified that green threads are the intended direction.
Independent activation scheduling is therefore the next POC gate; native stack
representation and preemption remain design questions, not established contracts.

## Detached interpreter activations — 2026-10-11

The private operation-wait POC now detaches a callback's frames when it parks and
can resume a ready parent independently. When no chain is active, ready parked activations are selected in
parking order at dispatch boundaries. Their frame permits, GC roots and array
payload accounting remain live while detached. Entry results are retained while
remaining activations finish; callback completion cannot overwrite the entry result.
The host wait considers deadlines across parked activations. Existing state-machine
and public Task behavior are unchanged.

The new handshake has a callback complete its parent's operation, then await a reply
from that parent. The parent resumes, sends the reply and awaits callback completion.
This cannot finish under the preceding LIFO-only implementation. The compiled Raven
consumer covers it, and focused tests additionally collect while the callback is the
only owner of its receiver and check detached array budgets and entry-result retention.

This is cooperative, interpreter-only scheduling of private integer-result operations.
No native stack strategy, preemption, migration across workers, stable foreign ABI,
public Task projection or compiler `await` lowering is selected by this implementation.
Uninterrupted guest code still runs until an existing execution boundary; fairness
and scalable work selection require measurement. Selection currently scans a bounded
operation/activation set; this is not a throughput improvement claim.

The preceding LIFO limitation is historical for the earlier POC revision. See
[detached-activation consumer evidence](detached-validation.json). Run the same
`validate.py` command with a fresh dedicated output directory to reproduce the
extended consumer. Earlier evidence files remain tied to their recorded source hashes.

The author's immediate goal is Raven runtime await by default, backed by stability
and improvement evidence. Next is the explicit compiler/managed-await integration
contract and equivalent comparison cases. This POC does not change Raven defaults.

## Async-body invocation ownership — 2026-10-11

A private interpreter `RuntimeAsyncStart(fn<Int32>) -> Int64` now runs a body
eagerly until it returns or first parks on a pending operation. Its caller receives
an operation handle at that point. Returning from the body completes that operation
with the integer result; completion belongs to the runtime, so manual Complete cannot
overwrite it. A cancellation outcome wins over a subsequent body return. Already
completed bodies and nested starts use the same mechanism. Frame budget admission
precedes operation publication; receiver roots travel with the detached body.

This mirrors the synchronous-prefix/returned-operation responsibility of .NET async
invocation. It is separate from suspension alone: Raven's existing CLR lowering
relies on the CLR to supply this responsibility through async method metadata.
neoCLR cannot safely enable that lowering just by omitting state-machine generation.
The POC uses a checked source-local delegate and private integer handle, not CLR
AsyncHelpers or the CLR async method flag. The next compiler contract must explicitly
connect body invocation, returned managed Task and awaited outcomes.

This remains an interpreter mechanics experiment. The source body is an ordinary
function, not a compiler-lowered Raven async method. Int32-only single-consumption
handles do not redefine Task semantics. Body faults remain terminal invocation faults,
not captured Task outcomes, and cancellation does not implement public Task semantics.
Managed result rooting, repeated Task awaits, fault/cancellation projection and
JIT/AOT support are still gates before ordinary await or default switching.
No performance improvement is claimed by this slice.

`Await.rvn` includes an eager body whose Phase is 1 when Start returns, then 2
when its awaited result becomes available. [Invocation evidence](start-validation.json)
records the compiled consumer. Focused tests additionally cover already completed
bodies, nested starts, frame limits, producer authority and late return after cancel.
The reproduction script is unchanged; use a dedicated output directory.
