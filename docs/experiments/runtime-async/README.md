# Interpreter retained-frame yield experiment

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
