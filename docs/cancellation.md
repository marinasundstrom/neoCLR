# Cooperative host cancellation

Run and invocation APIs accept either Limits or ExecutionOptions. The latter adds an
optional CancellationToken shared with the host. This is an experimental Rust control
surface for the interpreter, separate from guest async, exceptions, and memory management.

```rust
let token = neoclr::CancellationToken::new();
let options = neoclr::ExecutionOptions {
    limits: neoclr::Limits::default(),
    cancellation: Some(token.clone()),
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
External native side effects are not rolled back. The immutable program and resolved
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
