# NeoCLR Task Model

## Status

**Proposal**

This document defines the initial asynchronous Task model for NeoCLR, including task completion, cancellation, faults, `await` semantics, and promise-style composition.

The design assumes the broader NeoCLR error model:

- Expected operational errors are represented explicitly as values, typically through `Result<T, E>`.
- Cancellation is part of asynchronous computation and is represented by the Task abstraction.
- Faults represent unrecoverable runtime or program failures and cannot be caught or recovered from through normal application code.

These mechanisms are intentionally separate.

---

## Goals

The Task model should:

- provide a small language-neutral abstraction for asynchronous operations;
- support efficient `async`/`await`;
- provide first-class cancellation without exceptions;
- compose naturally with `Result<T, E>`;
- support promise-like functional composition;
- avoid encoding ordinary errors into Task itself;
- distinguish cancellation from operation failure;
- avoid exposing scheduler and runtime implementation details;
- support future runtime-level suspension without changing the public programming model.

The initial Raven implementation may use compiler-generated state machines. NeoCLR may later provide runtime suspension as an optimization or runtime capability.

---

# 1. Task

`Task<T>` represents an asynchronous computation that may eventually produce a value of type `T`.

Conceptually:

```raven
class Task<T> {
    State: TaskState
    Outcome: Option<TaskOutcome<T>>
}
```

A task begins in the `Pending` state and eventually reaches one terminal state:

```text
             ┌─── Completed(T)
Pending ─────┤
             └─── Cancelled
```

Once terminal, a Task cannot transition to another state.

Tasks are immutable from the perspective of their consumers.

---

# 2. Task State

The public state describes only the externally meaningful lifecycle of a task.

```raven
enum TaskState {
    Pending,
    Completed,
    Cancelled
}
```

The Task API intentionally does not expose runtime scheduling states such as:

```text
Created
Scheduled
Running
Suspended
Waiting
Completing
```

These are implementation details and may differ between runtimes, schedulers, or execution strategies.

`Pending` therefore means only:

> The Task has not yet produced a terminal outcome.

It does not imply whether the computation is currently executing.

---

# 3. Task Outcome

The outcome describes the terminal result of a Task.

```raven
enum TaskOutcome<T> {
    Completed(T),
    Cancelled
}
```

`Task.Outcome` is unavailable while the Task remains pending:

```raven
Outcome: Option<TaskOutcome<T>>
```

For example:

```raven
match task.Outcome {
    None =>
        // Task is still pending.

    Some(.Completed(value)) =>
        // Task completed with a value.

    Some(.Cancelled) =>
        // Task terminated due to cancellation.
}
```

`TaskOutcome<T>` contains the completion value directly. This preserves the invariant that a completed `Task<T>` always has a value.

There is therefore no independent nullable or optional `Task.Result` property.

---

# 4. Errors Are Values

Task does not have an error state.

An operation that can fail in an expected and recoverable manner expresses that failure through its result type:

```raven
func LoadUser(id: UserId) -> Task<Result<User, LoadError>>
```

The possible outcomes are therefore:

```text
Completed(Ok(User))
Completed(Err(LoadError))
Cancelled
```

The Task itself knows nothing about `LoadError` or `Result`.

This separation is fundamental to the model.

For example:

```raven
Task<int>
Task<Option<User>>
Task<Result<User, LoadError>>
Task<Result<Option<User>, QueryError>>
```

are all ordinary Tasks.

`Task<T>` does not impose an error protocol on `T`.

---

# 5. Faults

Faults are not Task outcomes.

A Fault represents an unrecoverable failure such as a violated runtime invariant or another condition from which normal program execution cannot safely recover.

Faults propagate according to NeoCLR fault semantics and cannot be caught through Task APIs.

Consequently, there is no:

```text
TaskState.Faulted
TaskOutcome.Faulted
Task.Error
Task.Exception
Catch(...)
RecoverFault(...)
```

A Task therefore has only two terminal outcomes:

```text
Completed(T)
Cancelled
```

This produces three distinct semantic channels across asynchronous application code:

```text
Value
    Successful computation

Result<T, E>
    Expected operation failure

Task cancellation
    Asynchronous computation did not complete

Fault
    Unrecoverable execution failure
```

These channels should not be collapsed into one another.

---

# 6. Cancellation

Cancellation is part of Task semantics rather than error semantics.

A cancellable operation accepts a `CancellationToken`:

```raven
async func LoadUser(
    id: UserId,
    cancellation: CancellationToken
) -> Task<Result<User, LoadError>>
```

A cancellation token represents a request for cancellation.

Cancellation of the token does not itself transition the Task into the `Cancelled` state. The operation must observe the request and terminate.

This distinguishes:

```text
Cancellation requested
```

from:

```text
Computation cancelled
```

The latter is represented by:

```raven
TaskOutcome.Cancelled
```

---

# 7. Cancellation Token

The initial cancellation API may take the following general shape:

```raven
struct CancellationToken {
    IsCancellationRequested: bool

    func Register(
        callback: func()
    ) -> CancellationRegistration
}
```

Cancellation is initiated through a source:

```raven
class CancellationSource {
    Token: CancellationToken

    func Cancel()
}
```

A default non-cancellable token may be provided:

```raven
CancellationToken.None
```

Linked cancellation may be supported:

```raven
CancellationSource.Link(
    parent: CancellationToken,
    other: CancellationToken
)
```

The exact cancellation API is separable from the Task ABI and may be specified independently.

---

# 8. Await

`await` is the normal language-level mechanism for consuming a Task.

Given:

```raven
let value = await task;
```

where:

```raven
task: Task<T>
```

the expression evaluates to:

```raven
T
```

Conceptually, `await` behaves as:

```text
Pending
    suspend until an outcome exists

Completed(value)
    continue with value

Cancelled
    propagate cancellation
```

Cancellation propagation is automatic.

Application code therefore normally does not inspect `Task.State` or `Task.Outcome`.

For example:

```raven
async func LoadProfile(
    id: UserId,
    cancellation: CancellationToken
) -> Task<Result<Profile, ProfileError>>
{
    let user = await LoadUser(id, cancellation)?;
    let avatar = await LoadAvatar(user.AvatarId, cancellation)?;

    return Profile(user, avatar);
}
```

Two independent propagation mechanisms operate here:

```text
await
    Task<T> -> T
    propagates cancellation

?
    Result<T, E> -> T
    propagates expected errors
```

Therefore:

```raven
let user = await LoadUser(id, cancellation)?;
```

can be read as:

1. await the asynchronous operation;
2. propagate cancellation if the Task was cancelled;
3. obtain its `Result<User, LoadError>`;
4. propagate the error if the Result contains an error;
5. otherwise obtain the `User`.

Neither mechanism depends upon exceptions.

---

# 9. Explicit Outcome Inspection

Although normal `await` automatically propagates cancellation, some code needs to observe Task termination explicitly.

Examples include:

- task combinators;
- orchestration;
- races;
- schedulers;
- diagnostics;
- interoperability;
- fallback behavior.

`Task.Outcome` provides this lower-level view.

For example:

```raven
match task.Outcome {
    Some(.Completed(value)) =>
        Handle(value),

    Some(.Cancelled) =>
        HandleCancellation(),

    None =>
        HandlePending()
}
```

The distinction is intentional:

```text
await task
    Consume the computation.

task.Outcome
    Inspect the computation.
```

---

# 10. Promise-Style Composition

Task supports composition independently of `async`/`await`.

The fundamental operators are proposed as:

```raven
Task<T>.Map<U>(
    transform: func(T) -> U
) -> Task<U>
```

and:

```raven
Task<T>.Then<U>(
    continuation: func(T) -> Task<U>
) -> Task<U>
```

## Map

`Map` transforms a successfully completed value:

```raven
LoadUser(id)
    .Map(user => user.Name)
```

Semantics:

```text
Pending
    resulting Task remains pending

Completed(value)
    invoke transform(value)
    complete with transformed value

Cancelled
    resulting Task becomes cancelled
```

The transform is never invoked for a cancelled Task.

---

## Then

`Then` chains asynchronous computations:

```raven
LoadUser(id)
    .Then(user => LoadProfile(user.Id))
    .Then(profile => LoadAvatar(profile))
```

Its type transformation is:

```text
Task<T> × (T -> Task<U>)
    -> Task<U>
```

`Then` performs asynchronous flattening and therefore does not produce:

```text
Task<Task<U>>
```

Cancellation propagates through the chain automatically.

---

# 11. Composition Invariant

The fundamental Task combinators follow one rule:

> Task combinators operate on completed values. Cancellation propagates unless an API explicitly states otherwise.

Thus:

```text
Map / Then

Completed(value)
    -> invoke continuation

Cancelled
    -> Cancelled

Fault
    -> fault propagation
```

This makes promise-style composition consistent with `await`.

---

# 12. Task and Result Composition

Task combinators do not understand `Result`.

Given:

```raven
Task<Result<User, LoadError>>
```

calling:

```raven
task.Map(...)
```

maps:

```raven
Result<User, LoadError>
```

because that is the Task's value.

Task does not implicitly inspect `Ok` or `Err`.

Result-aware convenience APIs may be provided separately, for example:

```raven
Task<Result<T, E>>.MapResult(...)
Task<Result<T, E>>.ThenResult(...)
```

but these would be composition helpers rather than fundamental Task behavior.

This maintains the abstraction boundary:

```text
Task
    asynchronous completion and cancellation

Result
    expected success and failure
```

---

# 13. Async/Await and Promise Composition

`async`/`await` and promise-style Task composition are not separate asynchronous models.

They are two ways of composing the same abstraction.

Imperative style:

```raven
let user = await LoadUser(id, cancellation)?;
let profile = await LoadProfile(user, cancellation)?;

return profile;
```

Compositional style:

```raven
return LoadUser(id, cancellation)
    .Then(...);
```

The language may lower `async` methods into compiler-generated state machines initially.

A future NeoCLR runtime may implement suspension directly.

Neither implementation strategy changes the semantics of `Task<T>`.

---

# 14. Task Completion

The public `Task<T>` abstraction should not itself expose arbitrary mutation.

Task producers require an internal or separately exposed completion mechanism analogous to a promise/completion source:

```raven
class TaskCompletionSource<T> {
    Task: Task<T>

    func Complete(value: T) -> bool
    func Cancel() -> bool
}
```

Only one terminal transition succeeds:

```text
Pending -> Completed(T)
Pending -> Cancelled
```

Attempts to complete an already terminal Task have no effect and report failure.

This mechanism supports integration with callbacks, operating-system APIs, event loops, and foreign runtimes without making Task itself mutable.

The exact naming and visibility of the completion-source abstraction should be specified separately.

---

# 15. Concurrency Combinators

The model is intended to support higher-level operations such as:

```raven
Task.WhenAll(...)
Task.WhenAny(...)
Task.Race(...)
Task.Delay(...)
```

Their precise cancellation semantics require separate specification.

In particular, APIs involving multiple Tasks must explicitly define:

- whether cancellation of one Task affects sibling Tasks;
- whether parent cancellation propagates to child operations;
- what outcome results from mixtures of completion and cancellation;
- ownership of cancellation sources;
- whether losing Tasks in a race continue executing.

These questions should not complicate the core `Task<T>` abstraction.

They belong to the concurrency/composition layer.

---

# 16. Structured Concurrency

The initial Task model does not require structured concurrency, but it should not prevent it.

A future structured concurrency API may define scopes that:

- own child Tasks;
- propagate cancellation;
- wait for children before leaving the scope;
- define lifetime relationships between asynchronous operations.

Such APIs can be layered on top of the same Task cancellation model.

---

# 17. Runtime Contract

The minimal conceptual runtime contract for a Task is therefore:

```raven
Task<T> {
    State: TaskState
    Outcome: Option<TaskOutcome<T>>
}
```

with:

```raven
enum TaskState {
    Pending,
    Completed,
    Cancelled
}

enum TaskOutcome<T> {
    Completed(T),
    Cancelled
}
```

and the following invariants:

```text
1. A Task begins Pending.

2. A Task may transition exactly once from Pending
   to Completed(T) or Cancelled.

3. A terminal Task never changes outcome.

4. Completed always contains a T.

5. Cancellation is not an error.

6. Expected errors are represented by T itself,
   commonly Result<T, E>.

7. Faults are not Task outcomes.

8. await propagates cancellation automatically.

9. Map and Then operate only on completed values
   and propagate cancellation.

10. Scheduler states are not part of the public Task model.
```

---

# 18. Design Summary

NeoCLR deliberately keeps asynchronous completion, expected errors, cancellation, and faults as separate concepts.

```text
                    ┌─────────────────────────────┐
                    │ Fault                       │
                    │ unrecoverable               │
                    │ runtime propagation         │
                    └─────────────────────────────┘

                    ┌─────────────────────────────┐
Task<T>             │ Pending                     │
                    │          │                  │
                    │     ┌────┴────┐             │
                    │     ▼         ▼             │
                    │ Completed   Cancelled        │
                    │    T                        │
                    └─────────────────────────────┘
                           │
                           ▼
                 T may itself be:

                 Result<T, E>
                 Option<T>
                 or any other value
```

This allows ordinary asynchronous application code to remain concise:

```raven
let user = await LoadUser(id, cancellation)?;
```

while the underlying Task remains explicit enough for runtimes, libraries, orchestration, and interoperability:

```raven
task.State
task.Outcome

task.Map(...)
task.Then(...)
```

The resulting model has one central rule:

> **Task describes whether an asynchronous computation produced a value. Result describes whether an operation represented by that value succeeded. Cancellation terminates the Task without producing a value. Faults exist outside the recoverable Task model.**