# NeoCLR Task and Async Model Proposal

## 1. Goals

NeoCLR should provide a modern asynchronous execution model that cleanly separates three concerns:

```text
Task<T>         asynchronous completion
Result<T, E>    recoverable failure
async / await   language-level suspension
```

The runtime should provide the primitives necessary for efficient suspension and resumption, while languages remain responsible for their own async syntax and lowering strategies.

Raven will initially implement `async` using compiler-generated state machines. NeoCLR's runtime suspension facilities allow this implementation to evolve later without changing the public task model.

---

## 2. Task as the asynchronous abstraction

`Task<T>` represents an asynchronous operation that eventually produces a value of type `T`.

```raven
func ReadAsync() -> Task<Data>
```

`Task<T>` represents **completion**, not failure.

Errors that are part of the expected contract of an operation should therefore not be encoded as a faulted task. They are represented explicitly using `Result<T, E>`:

```raven
func LoadUser(id: UserId)
    -> Task<Result<User, LoadError>>
```

This gives NeoCLR orthogonal composition:

```text
Task<T>                 asynchronous value
Result<T, E>            fallible value
Task<Result<T, E>>      asynchronous fallible value
```

---

## 3. Explicit async signatures in Raven

Raven does not implicitly transform the declared return type of an `async` function.

The complete return type remains visible:

```raven
async func LoadUser(id: UserId) -> Task<Result<User, LoadError>>
{
    let data = await database.Load(id)?;
    return User.Parse(data);
}
```

Here:

```text
async
    ↓
The implementation is allowed to suspend.

Task<...>
    ↓
The function's asynchronous API contract.

Result<User, LoadError>
    ↓
The operation's success/failure contract.
```

This avoids making `async` part of the type transformation rules.

A caller sees the same `Task<Result<User, LoadError>>` contract regardless of whether the implementation uses `async`, manually constructs a task, or is implemented by another NeoCLR language.

---

## 4. `await`

`await` operates exclusively on the asynchronous layer.

Given:

```raven
let result = await database.Load(id);
```

and:

```text
database.Load(id)
    : Task<Result<Data, LoadError>>
```

the expression:

```raven
await database.Load(id)
```

has type:

```text
Result<Data, LoadError>
```

Conceptually:

```text
Task<T>
   │
 await
   ↓
   T
```

`await` does not perform error propagation.

---

## 5. `Result` and `?`

Recoverable errors are represented explicitly:

```text
Result<T, E>
```

Raven's `?` operator handles propagation through the `Result` layer:

```raven
let data = await database.Load(id)?;
```

This composes naturally:

```text
Task<Result<Data, LoadError>>
          │
        await
          ↓
 Result<Data, LoadError>
          │
          ?
          ↓
        Data
```

The two operators therefore have independent meanings:

```text
await    suspend until the asynchronous value is available
?        propagate a recoverable error
```

This distinction should remain fundamental to Raven and NeoCLR.

---

## 6. Runtime suspension

NeoCLR should provide runtime primitives for suspending and resuming asynchronous execution.

The runtime's responsibility is the underlying mechanism rather than prescribing how every language must expose asynchronous programming.

Conceptually:

```text
Language
   │
   │ async/await semantics
   ↓
Task model
   │
   │ suspension/resumption
   ↓
NeoCLR runtime
```

This allows Raven and other NeoCLR languages to share the runtime infrastructure while presenting different language abstractions.

---

## 7. Initial Raven implementation

Initially, Raven will compile `async` functions into generated state machines.

For:

```raven
async func LoadUser(id: UserId)
    -> Task<Result<User, LoadError>>
{
    let data = await database.Load(id)?;
    return User.Parse(data);
}
```

the compiler can conceptually produce:

```text
LoadUser(...)
      │
      ↓
generated async state machine
      │
      ├── execute
      │
      ├── suspend at await
      │
      ├── register continuation
      │
      └── resume
              │
              ↓
     Task<Result<User, LoadError>>
```

The exact generated representation is an implementation detail and should not leak into the public type system.

---

## 8. Evolution toward runtime-managed async

Because NeoCLR itself understands suspension, Raven does not need to remain dependent on generated state machines forever.

The implementation can evolve:

```text
Raven v1

async func
    ↓
compiler-generated state machine
    ↓
NeoCLR suspension primitives
    ↓
Task<T>
```

toward:

```text
Future Raven

async func
    ↓
NeoCLR runtime suspension
    ↓
Task<T>
```

without changing:

```raven
Task<T>
await
async
Result<T, E>
?
```

or existing library signatures.

This makes runtime async an implementation evolution rather than a new application programming model.

---

## 9. Task completion states

The normal semantic model should remain deliberately small:

```text
Task<T>

Pending
   ↓
Completed(T)
```

For an operation with expected failure:

```text
Task<Result<T, E>>

Pending
   ↓
Completed
   ├── Ok(T)
   └── Error(E)
```

This is importantly different from making failure another ordinary `Task` completion channel:

```text
Pending
 ├── Success(T)
 └── Fault(Exception)
```

NeoCLR libraries should use `Result` for failures that callers are expected to handle.

Runtime catastrophes, cancellation, and other non-domain termination conditions should be designed separately rather than prematurely forcing them into either `Result` or a .NET-style exception/fault model.

---

## 10. `Void` and non-value-producing tasks

Because NeoCLR/Raven treats `Void` as a real type, a task that produces no meaningful value does not require a separate non-generic `Task` abstraction:

```raven
async func Delay(duration: Duration) -> Task<Void>
```

A fallible operation without a produced value becomes naturally:

```raven
async func Save(user: User)
    -> Task<Result<Void, SaveError>>
```

This keeps the task model uniform:

```text
Task<T>
```

rather than requiring both `Task` and `Task<T>`.

---

## 11. Core design principle

The important separation is:

```text
                 ┌─────────────────────┐
                 │       async         │
                 │ implementation mode │
                 └──────────┬──────────┘
                            │
                            ↓
                    ┌───────────────┐
                    │    Task<T>    │
                    │ async contract│
                    └───────┬───────┘
                            │ await
                            ↓
                            T

When T = Result<V, E>:

                  Result<V, E>
                       │
                       │ ?
                       ↓
                       V
```

**Task answers *when*. Result answers *whether*.**

Raven's `async`/`await` provides the language ergonomics, `Task<T>` provides the stable asynchronous API contract, `Result<T, E>` provides explicit recoverable failure, and NeoCLR provides the underlying suspension/resumption mechanism.

That gives NeoCLR a task model that can start with conventional compiler-generated async state machines without baking that implementation strategy permanently into the platform.
