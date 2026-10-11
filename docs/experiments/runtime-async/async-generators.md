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

## Demand-driven producer prototype (2026-10-11)

[RuntimeAsyncGenerator.rvn](RuntimeAsyncGenerator.rvn) supplies a source-local,
Int32-specialized producer on the new managed interfaces. Its retained async body
awaits a `Yield(value)` resume gate. A pending MoveNext owns the next-item promise;
a yield publishes one item and waits for either another request or a stop request.
Dispose sends stop and awaits producer completion, so lexical resources leave their
scopes before disposal completes. An outcome callback transfers cancellation and
completion to the cursor; it is a deliberate adapter rather than ordinary async
application control flow.

The producer starts on first advancement. Acquisition alone does not run the body.
Separate iterators from one iterable have independent producer state. The fixture
checks a pending external input, two demand-driven yields, no run-ahead at yield,
normal exhaustion, early break, input cancellation, disposal before starting,
independent iterators, repeated execution and GC at live yield boundaries.
[Interpreter/JIT/AOT evidence](async-generator-prototype-validation.json) uses
retained-body lowering with no application AsyncStateMachine carrier. This is
functional prototype evidence, not a performance benchmark or a production API.

Comparison with .NET: the pull protocol, lazy iterator body, Current lifetime and
awaited disposal follow IAsyncEnumerable/IAsyncEnumerator semantics. The prototype
uses retained runtime bodies and managed Task/Promise handshakes rather than a
compiler-generated iterator state machine and reusable ValueTask machinery. That
makes the suspension behavior directly testable with existing runtime hooks, but
allocates promises per advancement/yield and retains a native stack per active body.
No speed or memory improvement is claimed.

Remaining work before native Raven `yield` support:

- Recognize selected async iterable return types and lower `yield` into a producer
  factory/cursor protocol, including generic element types. The source-local helper
  is not a new language intrinsic or public API.
- Integrate producer cleanup that itself awaits, plus yield delegation and an
  explicit acquisition cancellation contract. Do not infer these from the current
  synchronous producer-resource fixture.
- Resolve abandoned-yield lifetime: the current runtime roots suspended activations.
  Explicit disposal is qualified; dropping the last consumer reference while yielded
  is not yet proven collectible. GC during live iteration is a different guarantee.
- Qualify native generated-state-machine consumption if it remains a supported
  comparison path, and Windows retained suspension before claiming those targets.

## Native Raven yield checkpoint (2026-10-11)

[Native language consumer](RuntimeGeneratorLanguage.rvn) now uses ordinary `yield`
and `yield from` with the [source-included generic provider](RuntimeGeneratorBridge.rvn).
The explicit runtime-async compiler option enables the selected generator-body
capability. Compilation expands delegation and lexical cleanup before replacing yield
with a suspension/demand handshake. Iterable acquisition is lazy and independent;
direct AsyncIterator returns use the same cursor protocol.

[Interpreter/JIT/AOT evidence](async-generator-language-validation.json) passes all
three modes on this macOS ARM64 host. It covers integer/string items, input waits,
no producer run-ahead, independent iterators, normal exhaustion, unstarted disposal,
early consumer exits, delegated producer cleanup, input cancellation, repeated runs
and GC while yielded. No application AsyncStateMachine carrier is emitted.
The follow-up below qualifies pending producer cleanup; abandoned GC remains open.

Reproduce with the matching Raven native compiler and async-iteration library bundle:

```sh
python3 docs/experiments/runtime-async/validate_async_iteration.py \
  --recipe target/runtime-stack-cache-validation-v2/validation.json \
  --interpreter-recipe target/runtime-http-use-final/validation.json \
  --libraries target/async-iteration-libraries \
  --compiler /Users/robert/Projects/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --generator-language --output target/generator-language-validation-new
```

The report hashes source, compiler, libraries and native inputs. Shared binding is
Raven main `8d787feca`, deliberately carried into native `36a4fd711` on
`codex/source-object-metadata-resolution`. The retained backend
still rejects generic source methods and generic owners; closed element types work.
The compiler follow-up removes misleading RAV2706 warnings from selected generator
bodies; ordinary awaitless Task methods retain the warning.

The .NET comparison remains the async pull/disposal protocol, implemented there with
compiler-generated state and reusable ValueTask machinery. This POC instead retains
runtime stacks and allocates Task/Promise handshakes per item. It makes suspension
runtime-owned but is not inherently faster or smaller. Existing cached runtime-await
measurements still take 1.31–1.91 times the native state-machine time; they are not
measurements of these generators. The next gates are generic source generators,
abandoned lifetime, followed by matched JIT/AOT throughput,
allocation, retained-memory and latency measurements. Keep state-machine controls.
Do not change defaults on the strength of functional evidence alone.

### Pending producer cleanup qualification (2026-10-11)

The native language fixture now nests two explicitly gated async iterators inside a
generator, with a synchronous `use` outside both. Four cases cover exhaustion,
early consumer break, cancellation before yielding, and cancellation of inner
disposal after early stop. Each asserts that the consumer remains incomplete while
inner disposal is pending, outer disposal starts only after inner completion or
cancellation, and the synchronous resource is disposed exactly once after outer
disposal finishes. Both suspension points include a GC collection. Successful
consumers retain the yielded value; cancelled cases publish cancellation only after
cleanup completes.

[Refreshed three-mode evidence](async-generator-language-validation.json) passes
interpreter, macOS ARM64 JIT and AOT using the unchanged native compiler `36a4fd711`.
The existing lowering already satisfies these cases; no compiler/runtime behavior
change was necessary. This validates awaited disposal ordering analogous to .NET
async iteration, while preserving neoCLR's nonrecoverable terminal Fault contract.
It is not an abandoned-generator reclamation test or a performance measurement.

### Generic producer prerequisite and diagnostic follow-up (2026-10-11)

A minimal generic producer `async func Echo<T>(value: T) -> AsyncIterable<T>` with
`yield value`, called with both integer and string arguments, reaches the existing
nongeneric retained-body guard. Temporarily removing that guard exposes the portable
emitter's rejection of unconstrained generic closure captures. The retained lowering
captures source parameters in its producer callback; `T` is neither a known reference
nor one of the supported primitive capture shapes. The guard was restored, and the
probe is not claimed as working generic support.

The prerequisite is generic closure emission: preserve enclosing method/type generic
context in synthesized callbacks and their bindings, define capture storage and GC
representation for instantiated value/reference arguments, and qualify both generic
methods and generic owners. This should be a shared compiler capability where
applicable, with explicit native-backend validation, rather than a generator-specific
boxing workaround. This is an implementation plan, not a changed language contract.

A separate diagnostic correction suppresses RAV2706 for selected iterator bodies,
since yield suspends even without explicit await. It does not mark the source as
containing an await or alter emitted metadata. Ordinary awaitless Task methods still
warn; default .NET behavior is unchanged. All 37 runtime-iteration compiler contract
tests pass on .NET 11. Refreshed native evidence above passes interpreter/JIT/AOT
without the spurious warning, including pending producer cleanup.

The diagnostic correction is Raven main `f5abf1dd0`, deliberately carried as native
`273e3e3e1`. The executable report identifies the tested compiler binaries by hash.

### Typed closure representation qualification (2026-10-11)

`CheckGenericClosure` in the executable fixture uses an explicit `Capture<T>` class
and returns its bound `Read` method from a generic factory. It checks integer, string
and newly allocated object captures after GC, then checks two independently captured
objects after another GC. The caller retains the callable, not the holder itself.
The refreshed three-mode report passes interpreter, macOS ARM64 JIT and AOT with
unchanged compiler `273e3e3e1`. This establishes typed generic storage, constructed
method binding and captured-reference reachability for the intended representation.
It is deliberately not a claim of synthesized generic closures or generic generators.

The compiler follow-up must preserve scoped generic identity throughout synthesis:

- Emit a generic closure owner with typed capture fields, analogous to a .NET
  compiler's generic display class. Avoid an object-boxing workaround for T.
- Remap enclosing method parameters into that owner's parameter scope in callback
  signatures, local types, function shapes, field references and constructed calls.
- Resolve callable references in that scope. The current emission table caches by
  method symbol alone; a callback remapping cannot reuse a handle resolved in its
  enclosing method's distinct generic scope.
- Keep definition import independent of caller-scope substitution, and retain the
  generic-owner guard until receiver captures and owner/method parameter collisions
  have their own executable coverage.

These are compiler implementation steps, not new runtime contracts. The portable
emission planner involved here currently exists only on the native compiler line;
this is not a reason to copy that experimental subsystem wholesale into Raven main.
Any independently applicable general compiler fix still belongs on main first.
