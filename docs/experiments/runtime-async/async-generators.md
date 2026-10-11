# Async-generator investigation — 2026-10-11

Status: investigation requested by the author while fixing `await` inside ordinary
`for` loops. This does not select a new public API or claim native generator support.

## What works in Raven

Raven main `a23705567` passes 31 tests selected by `AsyncIterator|AwaitFor`, plus
36 tests selected by `YieldFromCodeGenTests|ForExpressionTests`, on .NET 11.
[Recorded commands and results](async-generators-validation.json).
The selections overlap; the latter also includes synchronous iteration. Existing tests cover
pending `MoveNextAsync` without blocking, cancellation-token forwarding/combination,
asynchronous delegation, early disposal that itself awaits, cleanup exceptions,
and resuming after delegation. The selections include executable lifecycle tests alongside diagnostic/emission checks.

The language already has `async func ... -> IAsyncEnumerable<T>`, `yield value`,
`yield from source`, and `await for`. Ordinary `for { await ... }` waits inside a
synchronous iteration body; `await for` waits for the next item itself. Those are
independent capabilities.

Compiler implementation evidence (Raven repository):

- `IteratorLowerer.CreateMoveNextBody` lowers yield/delegation and uses async
  lowering for an asynchronous advancement body.
- `IteratorLowerer.DetermineIteratorSignature` recognizes .NET iterator shapes.
- `AwaitForLowerer` creates asynchronous advancement and protected async cleanup.
- `AsyncIteratorCancellationCodeGenTests` and `YieldFromCodeGenTests` qualify
  lifecycle behavior. Runtime async for ordinary methods does not eliminate the
  iterator state-machine protocol.

## What neoCLR lacks

The runtime library currently exposes synchronous `Iterable<T>`/`Iterator<T>`.
Raven's alternative-runtime `RuntimeIterationContract` selects only synchronous
acquisition/advancement/current members. The neoCLR retained-body POC explicitly
rejects iterator methods and requires a nongeneric `Task<T>` result. The .NET
iterator recognizer and generated helpers depend on .NET async iterator types.
Consequently, removing the ordinary-loop diagnostic does **not** enable native
async generators in interpreter, JIT or AOT. No native generator execution is
claimed by the .NET tests above.

## Recommended bounded follow-up

Keep a pull-based managed contract: acquire an iterator, asynchronously request
one next item, read `Current`, and asynchronously dispose it. This follows the
[shipped .NET async-stream model](https://learn.microsoft.com/en-us/dotnet/csharp/asynchronous-programming/generate-consume-asynchronous-stream)
and [IAsyncEnumerator contract](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.iasyncenumerator-1?view=net-10.0).
.NET uses `ValueTask<bool>` for advancement and `ValueTask` for disposal. A neoCLR
projection using existing `Task<bool>` and `Task<()>` would avoid introducing a
second operation type solely for this feature, but might allocate more per item;
that is a provisional tradeoff requiring measurement, not a settled improvement.

First qualify a hand-authored asynchronous iterator and consuming `await for`
through an explicit target contract. Then add producer `yield` lowering. Keeping
these separate distinguishes protocol/cleanup bugs from runtime suspension bugs.
State-machine generators can provide a reference implementation while retained
runtime generators are developed; no permanent state-machine dependency is implied.

A runtime generator must suspend at both an asynchronous wait and a yielded item.
A yielded item is not terminal task completion: the activation remains owned by
the iterator until the consumer advances or disposes it. A one-shot Task completion
bridge cannot simply be reused as a generator-yield operation. Pull-based demand
provides backpressure without an implicit queue or eager buffering. Retained stacks
also have an idle-memory cost that must be measured against state-machine storage.

Required qualification includes laziness, repeated and empty enumeration, one
outstanding advancement, pending advancement cancellation, early disposal after
yield, cleanup that awaits, disposal failure, nested delegation, and retained
references under GC. Terminal neoCLR Fault stays non-recoverable; this work must
not imply CLR-style unwinding on Fault. Clarify cancellation versus asynchronous
cleanup before implementing a retained generator, so cancellation cannot skip
resource cleanup by cancelling the cleanup operation itself. GC must also distinguish
an actively awaited operation from an abandoned generator suspended at a yield:
a permanently rooted dormant stack could keep the iterator and its captured object
graph alive indefinitely. Reclaiming abandoned state must not be presented as a
replacement for deterministic asynchronous disposal.

As an alternative, ECMAScript async generators retain an execution context and
queue resume requests, resolving each request with an iterator result. Its
[suspension/resume and yield algorithms](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-asyncgenerator-objects)
are useful runtime-design references, but adopting request queuing would add a
concurrency contract and storage cost beyond the first managed pull protocol.
Keep that choice open; do not accidentally promise concurrent advancement because
the scheduler itself accepts multiple operations. Sources consulted 2026-10-11:
Microsoft Learn's shipped .NET 10 API contract and TC39's living specification
(currently headed ECMAScript 2027), not a claim about a particular JS engine's
stack representation or performance.

This follow-up belongs to managed runtime-async qualification. It does not require
finalizing foreign awaitable interfaces or the native interop ABI first.
