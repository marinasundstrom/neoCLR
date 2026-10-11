# Managed Scheduler projection experiment

2026-10-11, development prototype. This source-included **internal** Scheduler is not
an addition to the public class library. It exercises a managed facade over runtime
submission, while leaving Task/Promise and Raven-generated async state machines valid.

`Main.rvn` submits a callback, verifies that it did not run inline, submits a second
callback from it and completes a managed Promise. An ordinary async Main awaits that
Promise and an already-completed Task. `Fault.rvn` fails after admitting another
callback, checking terminal fault cleanup with pending work. The application never
constructs or drains a TaskQueue; its current async builder still uses the transitional
queue adapter internally.

## Service and ownership

The reserved prototype service is:

```text
neoCLR.Runtime.SchedulerSubmit(fn<Void>) -> Int32
```

It is an exact InternalCall contract. Successful admission returns zero, discarded by
the managed facade; failure is a runtime fault. This integer acknowledgment is a
private bridge representation, not a public Scheduler result type or a Task outcome.
The prototype does not yet admit the no-result callback shape. Those restrictions
must not become permanent native async semantics.

Interpreter admission retains a bounded private runnable collection, signals the
invocation wake latch, and transfers callbacks into traced frames at dispatch boundaries.
It does not need a TaskQueue. Completion-source callbacks retain their existing
adapter. Submitted work alternates with ready source delivery when both are present;
failed installation keeps roots. A callback's return cannot pollute the suspended
entry's evaluation stack. The invocation's budget and cancellation checks still apply.

AOT/JIT use explicit `--bind-scheduler` with `--compile-system` and `--native-gc`.
The compiled managed-call adapter retains the callable through the existing native
submit/acquire/complete protocol. The native host must provide a scope and progress
pump; the qualified consumer uses the generated async-entry pump. This is not a promise
that every synchronous host drains admitted work after entry return.

Interpreter submission has a limit of 64 pending callbacks; native submission shares
64 operation slots with I/O/timers. Both are bounded experimental implementations,
not a selected portable resource quota. No FIFO, cross-thread submission, scheduler
customization, parallel execution, cancellation ticket or stable foreign ABI is promised.
Runtime suspension is unimplemented. A future saved-activation adapter must preserve
these ownership obligations without requiring Task layout or generated state-machine
layout; callbacks remain a valid execution adapter.

## Evidence and reproduction

[Cross-mode evidence](validation.json): success and terminal-fault consumers pass in
interpreter, AOT and JIT (six executions), with input/binary hashes and output checks.
[Native contracts](native-contracts.json) include managed-call boundary/output validation
and the existing capacity, rooting, acquisition and cleanup checks. Qualification is
macOS ARM64; Windows has not run this revision. [Missing-binding rejection](missing-binding.json)
confirms the native service cannot execute without the explicit scheduler opt-in.

Focused Rust checks cover scheduler admission/rooting/capacity/source turns,
queue-free receiver retention and callback faults, exact service signatures, and
existing guest timer dispatch. The native backend's binding test includes malformed
SchedulerSubmit declarations alongside timer/deadline rejection cases.

```sh
python3 docs/experiments/jit-async/verify.py \
  --bundle target/data-cases-bundle \
  --console-qualification target/jit-startup-validation/validation.json \
  --scheduler --output target/scheduler-projection-fresh
cargo test --lib scheduler::tests
cargo test --lib task_callback_tests
cargo test --test scheduler_submission
cargo test --test timers
cargo test --release --manifest-path tools/aot-poc/Cargo.toml \
  deadlines_and_timers_require_exact_reserved_signatures_and_internal_bodies
```

Use matching compiler/runtime libraries and rebuilt runtime/AOT tools, plus the Xcode
SDK configuration described in the native scheduler experiment. The bundle/compiler
hashes identify this integration validation; Raven main alone is not evidence of native
execution support. No new Raven Runtime Contract option or lowering behavior is required.

Next: settle the managed public surface and reference documentation, migrate continuation
registration off producer queues, and extract native scheduler lifetime from the
experimental socket scope. This experiment does not remove public TaskQueue APIs.

Companion Raven shared-main bridge documentation: `f92f5ca43` (documentation only).
No shared compiler binary was changed by this slice.
