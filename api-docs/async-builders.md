# Transitional async compiler protocol

These development APIs support generated Raven state machines while neoCLR builds
its application platform. They are compiler/runtime integration details and may be
deprecated or removed when runtime-owned suspension replaces generated machinery.
Use Task and await in application code; no public scheduler design is implied.

The reference includes IAsyncStateMachine, ITaskAwaiter and
AsyncTaskMethodBuilder<T> under System.Runtime.CompilerServices.

## State ownership

The builder remains a class with shared completion storage. The development
reference exposes `Start<TState>(ref TState)` and
`AwaitOnCompleted<TAwaiter,TState>(ref TAwaiter, ref TState)`. The importer specializes
these calls for admitted non-generic application state machines and target Task
awaiters; it does not execute declaration-only reference assembly bodies.

Start invokes MoveNext against the existing state storage. A struct is copied into
one box when its first await is pending; later continuations reuse that owner.
The builder's SetStateMachine, HasStateMachine and GetStateMachine methods support
that handoff. GetStateMachine faults if used before retention. SetResult and
SetCancelled clear the retained owner before publishing completion. No frame-backed
managed reference is retained in a callback or field.

Bootstrap library metadata retains its by-value Start/AwaitOnCompleted helpers;
application metadata exposes the validated ref projection. This is a bounded bridge
protocol, not general support for arbitrary custom builders or generic async methods.
The heap policy remains the default until wider validation justifies changing it.
A value state does not eliminate Promise, Task, dispatcher or continuation allocations.

## Continuation member

`ITaskAwaiter.OnCompleted(callback: Func<System.Void>)` registers a continuation on
the awaited operation's dispatcher. It returns no result. Task implements this
contract. The member is excluded from generated signatures because DocFX cannot
render the intrinsic Func<Void> shape; its type and the other builder members have
generated reference coverage. This omission follows the [callback reference](callbacks.md).

## Errors and limits

Cancellation completes the output task as cancelled. Task<Result<T,E>> carries
recoverable errors as values; terminal Faults remain terminal. The protocol does
not add CLR exceptions, ExecutionContext, thread affinity or runtime suspension.
By-reference value receivers and class field addresses are verified managed references:
null field addressing faults, access rules still apply, and interior references root
their heap owner for GC. Raw or frame-backed addresses cannot become heap fields.
