# Async-generator investigation — 2026-10-11

Status: implementation underway following the author's explicit direction to add
managed async iteration and await-for consumption before yield/resume. The managed
contract and retained-body await-for consumption are implemented; native generator
production remains open.

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

The runtime library now also exposes `AsyncIterable<T>`, `AsyncIterator<T>` and
`AsyncDisposable`. Raven main's optional `RuntimeIterationContract.Async` selects
asynchronous acquisition/advancement/current/disposal members. The neoCLR retained-body POC explicitly
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

## Managed contract checkpoint (2026-10-11)

`AsyncIterable<T>.GetAsyncIterator()` returns `AsyncIterator<T>`, whose
`MoveNextAsync()` returns `Task<bool>` and `Current` returns `T`. Its inherited
`AsyncDisposable.DisposeAsync()` returns `Task<()>`. The consumer serializes
advancement and disposal; Current is valid only between a successful advancement
and the next operation. Acquisition does not take a cancellation token in this
first contract: source construction owns cancellation policy for now. Automatic
token forwarding is a gap, not a permanent divergence.

This follows .NET's pull-based IAsyncEnumerable/IAsyncEnumerator/IAsyncDisposable
structure. neoCLR keeps its existing Iterable/Iterator names and Task result
contract rather than introducing ValueTask solely for enumeration. This simplifies
initial integration but may allocate more on repeated synchronous advancement;
performance comparison is still needed. No overlapping operations are promised.

[Executable contract consumer](RuntimeAsyncIteration.rvn) forces pending advancement
and disposal, checks Current timing and exactly-once cleanup, and collects GC at
both suspension points. It passes interpreter, macOS ARM64 JIT and AOT with both
state-machine and retained-body lowering: [six-run evidence](async-iteration-contract-validation.json).
This consumer uses explicit protocol calls to isolate the library contract; it is
not evidence for native await-for lowering or yield production. Raven main
`da0596cf7` separately qualifies selected-contract await-for with pending disposal
on completion/break/return/cancellation in .NET 11 (61 focused tests; final contract
recheck eight tests). Native await-for cleanup is the next integration slice.

## Retained-body await-for checkpoint (2026-10-11)

Raven native `e7a5e6cf8` consumes the managed contract through `await for` with the
explicit runtime-async POC provider. The ownership manifest now selects AsyncIterable
and AsyncIterator. Compiler-owned cleanup routes normal completion, return, break,
continue and cancellation through the appropriate disposal scopes. Cleanup itself
can suspend or cancel; outer cleanup still runs. Synchronous `use` resources share
that lexical ordering, including value-producing blocks. Terminal Fault semantics
remain nonrecoverable without an unwinding promise.

[Executable lifecycle fixture](RuntimeAwaitFor.rvn) covers eight advancement/body/
cleanup outcomes, nested disposal with cancellation, outward exits, labelled
break/continue, mixed synchronous resources and GC while cleanup is pending.
[Interpreter/JIT/AOT evidence](async-await-for-validation.json) uses macOS ARM64 for
native execution. [Existing synchronous-loop regression](async-iteration-sync-validation.json)
also passes all three modes with the same compiler. The general labelled await-for
fix was developed on Raven main `4972d9def` (61 focused .NET 11 tests) before carrying
it to the native line.

This is retained-body consumption only. The native generated-state-machine path
still lacks asynchronous cleanup integration. Generator return-type recognition,
`yield` synthesis and abandoned-generator lifetime remain open. Windows retained
runtime support is not established by these results.
