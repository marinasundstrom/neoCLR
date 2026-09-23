# Cooperative host cancellation

Run and invocation APIs accept either Limits or ExecutionOptions. The latter adds an
optional CancellationToken shared with the host. This is an experimental Rust control
surface for the interpreter, separate from guest async, exceptions, and memory management.

```rust
let token = neoclr::CancellationToken::new();
let options = neoclr::ExecutionOptions {
    limits: neoclr::Limits::default(),
    cancellation: Some(token.clone()),
    ..neoclr::ExecutionOptions::default()
};
// Supply options to program.run or a resolved function's invoke method.
// Another host thread holding token can request cancellation:
token.cancel();
let result = program.run(options); // Fault: execution cancelled
```

## Observation and precedence

The interpreter checks for cancellation when execution begins and before each guest
instruction. Once observed, it returns a terminal Fault with message `execution cancelled`
and the current function/next instruction index. Guest code cannot catch it. Polling
adds no guest instructions or frames and does not consume the instruction budget.
A pre-cancelled run reports instruction zero without executing guest code.

Loading, resolution, entry checks, and input validation precede these polls and are not
cancellable in this slice. Invalid host arguments therefore still produce validation
faults. At execution entry, observed cancellation takes precedence over zero frame or
instruction limits. Later, the first observed failure or completion wins; cancellation
racing with a return, instruction fault, or exhausted budget may not win that race.

Cancellation is cooperative. It cannot interrupt a native call, library initializer,
destructor, blocking operation, or an instruction already in progress. The next poll
observes the request if execution continues. There is no wall-clock deadline guarantee,
forced thread termination, suspension, continuation, or rollback.

## Lifetime and isolation

Cloning a token shares its request flag. cancel is idempotent, does not wait for workers,
and cannot be reset. Reusing a cancelled token cancels subsequent executions too; create
a new token for independent work. Dropping the host's token does not request cancellation.
The flag is not a synchronization mechanism for other host data. Execution results
remain thread-local Rust values; handle or drop them on the executing thread rather
than transferring an Execution across threads.

Each run still owns fresh frames, allocations, and output. Cancellation drops this
execution state through the existing Fault path; it does not return a partial Execution.
External native side effects and already delivered [live console output](console-io.md)
are not rolled back. Console host calls cannot be interrupted by cancellation. The immutable program and resolved
handles remain reusable, and executions with independent tokens are unaffected.
Native-enabled APIs retain all existing unsafe ABI and lifetime requirements.

All entry helpers and loaded-program/static/instance invocation methods accept the same
options. Calls explicitly passing Limits remain supported. Since these methods now take
`impl Into<ExecutionOptions>`, callers using a bare `Default::default()` may need to write
`Limits::default()` or `ExecutionOptions::default()` to resolve the type. This is not a
stable native hosting ABI. Future JIT/AOT backends will need an explicit polling contract;
this slice does not establish their polling frequency or maximum response latency.

`cargo run --example cancellation` requests cancellation from another thread, joins the
worker, then invokes another function using the same loaded program.

## Worker completion boundaries — development

A worker may have completed while its caller is still waiting to consume the result.
Host cancellation controls the caller's invocation; worker completion does not
prevent that invocation from being cancelled. `JoinWorker` polls before waiting, after receiving a result, between
forwarded console lines and before returning the value. Observed cancellation ends
the invocation with `execution cancelled`, including when a success or producer Fault
was waiting to be consumed. It does not produce a guest `TaskOutcome.Cancelled`.

| Controlled ordering | Checked behavior |
| --- | --- |
| Result ready, cancellation observed before notification dispatch | The callback stays registered for teardown; no notification is delivered. |
| Notification dispatched, cancellation observed before join | Cached success or producer failure is discarded; no worker output is forwarded. |
| Cancellation requested by the first of two host writes | The first line remains visible; the second line and result delivery are skipped. |
| Cancellation requested by the final host write | The invocation still stops before returning the joined value. |
| Fresh invocation after cancellation | The same loaded program can deliver both lines and the result with independent options. |

The invocation registry requests producer cancellation and joins its threads before
releasing retained registrations/results. A controlled registry test checks that a
producer can acknowledge cancellation through its still-live result channel and has
stopped when teardown returns. These are selected orderings, not exhaustive concurrency
verification or a bounded shutdown guarantee for blocking native calls. There is still
a race after every poll; requesting cancellation does not synchronously stop delivery.
An observed host-write error remains an output Fault, even if the host also requested
cancellation inside that failing write. Previously delivered side effects remain.

Run the small [host sample](../examples/worker_cancellation.rs):

```sh
cargo run --locked --example worker_cancellation
```

Its [guest IL](../examples/worker_cancellation.neoil) declares its provisional worker
imports directly, so the same source can be assembled and verified independently.
This is a runtime embedding example; ordinary Raven applications use library APIs.
Expected output (also asserted by the executable):

```text
First worker line
Invocation: execution cancelled
Fresh invocation: both lines and result delivered
```

### Comparison and choice

[.NET cooperative cancellation](https://learn.microsoft.com/en-us/dotnet/standard/threading/cancellation-in-managed-threads)
(primary documentation checked 2026-09-23) leaves observing and responding to the
request to the operation. This is the relevant baseline; it does not impose a CLR
poll before every console line. neoCLR's existing host token stops an entire interpreter
invocation with a terminal Fault. Guest operation tokens and Task cancellation are
separate design work.

Keeping worker output delivery as one uninterrupted batch was simpler but postponed
observation across multiple host calls. Polling at line boundaries uses the existing
runtime control surface, with an extra flag read per line and explicitly partial
output on cancellation. No atomic delivery, rollback or performance improvement is
claimed. Operation cancellation/acknowledgement for native I/O remains provisional;
these host checks do not select that contract.
