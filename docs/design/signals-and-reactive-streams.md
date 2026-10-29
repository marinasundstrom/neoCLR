# Signals and reactive streams proposal

**Status: design proposal, 2026-10-29.** Requested by the author following the
[signal and Rx discussion](../development-timeline.md#2026-10-29--library-owned-signals-using-structural-functions).
This proposes ordinary library APIs over structural function types. It adds no
event metadata, nominal function type or multicast function representation.
Signature projection and argument representation are proposed compiler facilities;
boxed forwarding can reuse the existing runtime invocation operation.
API names and policies below are proposed contracts,
not implemented capabilities or a change to milestone priority.

Signals provide named subscription points on objects. An Rx-like layer represents
values over time and composes subscriptions through operators. Both share lifetime
machinery, but a callback list is only one possible stream producer.

All code blocks below are **API sketches in Raven-like notation**, including
signature-only declarations and application placeholders. They are not compiled
samples. Before adopting these APIs, qualify complete Raven implementations and
consumers against the matching native compiler bundle as described below.

## Existing foundation and .NET comparison

neoCLR already has structural function values with typed invocation and retained
receivers/captures. The integrated [function-kind constraint](../validation/function-kind-constraint/README.md)
admits structural function types through `where F: func`; it does not expose a
parameter list to a generic body. The [provisional dynamic invocation API](../validation/dynamic-function-invocation/README.md)
is available separately, but is unnecessary for the normal signal path.

.NET events expose subscription accessors and conventionally store nominal multicast
delegates. Its `IObservable<T>` and `IObserver<T>` contracts provide subscription
lifetime and value/error/completion notifications. These are the API comparison
points; none requires neoCLR to reproduce CLR event tables or delegate ancestry.
See [Microsoft events](https://learn.microsoft.com/dotnet/standard/events/) and
[IObservable<T>](https://learn.microsoft.com/dotnet/api/system.iobservable-1).

| Concern | Proposed neoCLR choice | Benefit and cost relative to .NET |
| --- | --- | --- |
| Callback identity | Structural function signature | Reuses native functions; no nominal handler declarations, but no nominal distinction between identical signatures. |
| Subscription boundary | Ordinary property returning a separate view | No event metadata; event-specific reflection/tooling needs an explicit convention. |
| Registration removal | Disposable handle for one registration | Unambiguous duplicate removal; differs from delegate-based `-=`. |
| Reactive failure | Generic error value `E` | Fits explicit errors without exceptions; adds a type parameter and requires error mapping when composing different domains. |
| Delivery | Immediate, owner-affine baseline | Fits synchronous event use; concurrent producers need an explicit scheduling/serialization adapter. |
| Composition | Familiar `Where`, `Select`, `Take` names | Familiar Rx operations; full Rx.NET compatibility is not implied. |

The [ReactiveX observable contract](https://reactivex.io/documentation/contract.html)
is the stream baseline: ordered, nonoverlapping notifications, optional terminal
completion or failure, and disposable subscriptions. It permits synchronous
notification during subscription and does not guarantee demand-based flow control.
The additional owner, queue and cancellation rules below are neoCLR proposals.

## Layers and ownership

```text
publisher owns SignalSource<F>
    └─ exposes Signal<F>                 subscription only
           └─ typed projection
                  └─ IObservable<T, E>
                         └─ operators
                                └─ consumer subscription

independent per-subscription producer ────┘
```

The source owns publishing authority. Its cached public view is a distinct object,
not the source cast to a narrower interface. Subscribers cannot cast that view back
to a source or enumerate handlers. Private references are protected by normal
access rules, not an event-specific runtime privilege.

Provisional packaging is a small signals library and an optional reactive library
depending on it, with `Neo.Signals` and `Neo.Reactive` as working module names.
This avoids committing all applications to a reactive operator framework. Final
assembly/module placement is open; nominal classes and interfaces are ordinary
containers and contracts, never nominal *function* types.

## Declaring a method with a function signature

**Subsequent author clarification:** this is a general function capability,
materialized as part of generics, not a signal-specific compiler feature. The
[generic function method investigation](generic-function-methods.md) now owns the
compiler/metadata design: an authoritative function-type reference on a method,
`MethodInfo.FunctionType`, partial substitution and ordinary closed method
reflection. The author permits experimental metadata redesign and compatibility
breaks. The unit restriction below applies only to the proposed signal policy;
general projected methods preserve arbitrary supported return contracts.

The author's central requirement is to express a publishing method using the
structural signature supplied as a generic argument:

```text
// Author-proposed syntax; not supported Raven syntax today.
class SignalSource<T: func> {
    func Signal: T {
        // Forward this invocation's arguments to the registered functions.
    }
}
```

For `SignalSource<(Widget, Change) -> unit>`, this should mean a method equivalent
in its call contract to `Signal(Widget, Change) -> unit`. It does not mean a
zero-argument method that returns a T. T describes the method signature; each
registered function remains a distinct bound value of that type.

This requires **signature projection**, beyond `where T: func`. The compiler must
substitute T's ordered parameters, modes and result into the method declaration,
type-check callers, and provide an argument representation to its implementation.
Parameter names cannot come from T: names are not part of structural function
identity. Generic metadata/imports must preserve the projected declaration until
T is known. Closed methods can use ordinary call metadata and ABI, but the open
projection may require new compiler/metadata representation or generated wrappers.
Avoid claiming this is an ordinary library-only addition merely because it needs
no event metadata.

### Representing the current arguments

The author suggested helpers to extract parameters from the materialized method
context, then refined the requirement to at least represent them. Separate three
things:

| Representation | Contents | Purpose |
| --- | --- | --- |
| Function type descriptor | Parameter types, modes and result type | Describe and validate the signature; no current argument values. |
| Invocation argument pack | Values supplied to this call, associated with T's parameter shape | Forward arguments without inventing parameter names or inspecting stack layout. |
| Boxed argument array | Object views/boxed values in parameter order | Adapt supported signatures to the existing reflection helper. |

Working notation for the compiler-provided pack is `Arguments<T>`. This denotes
the parameter portion of a function signature, not an ordinary unconstrained
library generic that already exists. The body needs an explicit binding to the
pack. The following spelling is illustrative; `arguments` must be compiler-bound,
not an unresolved implicit local:

```text
// Proposed signature projection and argument-pack binding.
func Signal: T with arguments {
    // arguments represents this call's Arguments<T>.
}
```

Prefer a compiler-known representation over a helper that discovers the current
frame by stack walking. Conceptually the pack can be a tuple of incoming values,
or a scoped view of the incoming argument slots; its public semantics should not
depend on JIT/AOT stack layout, optimization or inlining. Metadata describing T
alone cannot recover these values.

An initial reflection-based implementation can explicitly materialize an
`Object?[]` from that pack and call the existing API:

```text
FunctionReflection.Invoke(
    function: Object?,
    arguments: Object?[]
) -> Result<Object?, ReflectionError>
```

This is the actual development signature in
[FunctionReflection.rvn](../../runtime/System.Runtime/src/System.Runtime/System/Runtime/Reflection/FunctionReflection.rvn).
The conceptual signal body is then:

```text
// Pseudocode: pack conversion, snapshot and propagation policy are proposed.
func Signal: T with arguments {
    let boxed = arguments.ToObjectArray()
    for handler in SnapshotHandlers() {
        match FunctionReflection.Invoke(handler, boxed) {
            Ok(_) => { }
            Failure(error) => {
                PanicInvocationFailure(error)
            }
        }
    }
}
```

This version requires a **unit-returning signal signature**. T may otherwise
return any type, so `where T: func` alone is insufficient to validate the body.
Either add a result-shape constraint or enforce the unit-only source contract at
construction until the compiler can express it. That is an explicit admission
restriction, not a claim that all function types return unit. Returning a
`Result<unit, ReflectionError>` from this body would change T's signature; it
cannot be used as an implicit escape route for invocation errors.

`PanicInvocationFailure` is a placeholder policy for an invariant/unsupported
bridge failure, not a new API. Prefer rejecting unsupported signatures before
subscriptions/publication. The current helper has exact boxed scalar/reference
support, no coercion, no ref/out copyback and no arbitrary value marshalling.
Even a statically valid T can exceed that bridge profile. An empty subscriber
list must not bypass source signature validation. Reflection invocation failure
is distinct from the reactive domain error E.

The argument array could be built once per dispatch and reused privately for all
handlers. Conversion must preserve GC roots and captured receiver identity;
invocation must use each bound handler, never merely T's Invoke descriptor.
Boxing and array allocation are costs to measure, not an intrinsic requirement of
signal delivery. A later compiler-known `Invoke(handler, arguments)` pack operation
could forward values through ordinary typed calls without boxing or reflection.
It needs explicit compiler support; it is not currently supplied by `func`.

Initially restrict packs to by-value parameters and synchronous forwarding. A
scoped pack cannot escape the call, be retained by a handler or enter an async
queue; delayed delivery requires owned values with a separate lifetime contract.
Ref/out packs would need alias preservation and writeback rules, which copying
into an object array does not provide. Result projection/aggregation for non-unit
signals, pack indexing/destructuring and any public materialized invocation context
remain separate design questions.

The recommended experiment is therefore: project a concrete T onto a method,
represent its incoming arguments explicitly, then forward them using a supported
reflection profile. This is the desired call-site shape. The visitor below is an
implementation alternative using existing typing, not a substitute for the
author's signature-projection requirement.

## Signal API and typed visitor alternative

```text
public class Subscription : IDisposable {
    func Dispose() -> unit
}

public class Signal<F> where F: func {
    func Subscribe(handler: F) -> Subscription
}

public class SignalSource<F> where F: func {
    val Subscriptions: Signal<F>
    func Dispatch(invoke: (F) -> unit) -> unit
}
```

For example, a publisher exposes a change subscription point:

```text
public class Widget {
    private val changedSource = SignalSource<(Widget, Change) -> unit>()

    public val Changed: Signal<(Widget, Change) -> unit> => changedSource.Subscriptions

    private func PublishChange(change: Change) -> unit {
        changedSource.Dispatch(handler => {
            handler(self, change)
        })
    }
}

let subscription = widget.Changed.Subscribe((sender, change) => {
    RenderChange(sender, change)
})

// At the end of the consumer's lifetime:
subscription.Dispose()
```

The source view is called `Subscriptions` to leave `Signal` available as the
projected publishing method. The visitor alternative knows the signature
`(F) -> unit`, so it can call the visitor
with each stored F. At the publisher, F is `(Widget, Change) -> unit`, so the
visitor can call the handler with statically checked arguments. This preserves
exact structural typing and avoids object arrays, reflection and generic
parameter-pack assumptions. The prototype must qualify nested function signatures
and captures; category-constraint evidence alone does not prove this full API.

`Dispatch` calls the visitor once per registration. The visitor controls whether
and how often to invoke that handler and what to do with a result. Notification
APIs should normally use unit-returning handlers. Non-unit signatures do not
silently gain last-result, vote or aggregation semantics. By-reference signatures
and borrowing across delayed delivery are outside the initial public profile.

A payload-specific convenience source could store `(T) -> unit` and offer
`Emit(value: T)`. An arbitrary-signature `Emit(args...)` on `SignalSource<F>` would
require signature-specific wrappers or compiler support and is not required here.

### Registration and delivery rules

- Each `Subscribe` creates one registration. Equal functions can be registered
  twice and are called twice; each handle removes exactly its registration.
- `Dispose` is idempotent. A disposed handle releases its source/callback references.
  A retained undisposed handle may keep the source alive.
- Registration strongly retains the function, receiver and closure. Weak
  subscriptions are a possible separate API, not the default.
- `Dispatch` captures a stable ordered registration snapshot. Subscriptions added
  afterward are absent from that dispatch. Removal prevents inclusion in future
  snapshots, but does not retract callbacks from an already captured snapshot.
- Dispatch is synchronous on the owning execution context. Nested dispatch is
  allowed and depth-first, with a fresh snapshot. No worker scheduling is implied.
- Subscribe, dispose and dispatch must run on the same owning context initially.
  Sharing references across threads does not grant concurrent-use support.
- An empty source is a no-op. A signal has no completion/error channel; application
  event data may describe a failure, but it does not terminate the signal.

Use `use` when subscription lifetime is lexical. Long-lived components store and
dispose their subscriptions explicitly. GC manages memory; it does not substitute
for deterministic unsubscription. The initial source needs no `Dispose` or global
`Clear` operation; publisher shutdown with observable completion belongs below.

## Reactive API

Normalize event arguments into a value type T at the stream boundary. Multiple
arguments may become an existing domain object or a tuple; do not force a fresh
event-arguments wrapper when an existing value suffices.

```text
public interface IObservable<T, E> {
    func Subscribe(observer: Observer<T, E>) -> Subscription
}

public class Observer<T, E> {
    init(
        onNext: (T) -> unit,
        onError: (E) -> unit,
        onCompleted: () -> unit
    )
}

// Convenience overload supplied by the reactive library:
Subscribe<T, E>(
    source: IObservable<T, E>,
    onNext: (T) -> unit,
    onError: (E) -> unit,
    onCompleted: () -> unit
) -> Subscription
```

An observer is a container of three structural callbacks, not a callable nominal
type. The library invokes them through an internal guarded subscription endpoint;
the public container does not expose publishing authority. All three callbacks
are required for a fallible stream, avoiding an implicit ignored-error policy.
Here `onCompleted` denotes stream termination, not a Task continuation; ordinary
asynchronous application work continues to use `await`.

The ordinary `IObservable<T, E>` interface admits different producer
implementations. It is not defined as a subscriber list, nor as a structural
object contract. Generic variance is left out until the complete nested callback
profile is qualified; absence of variance is an initial API limitation.

### Completion and errors

Each subscription follows this protocol:

```text
Active ── OnNext(value) ──> Active
Active ── OnCompleted() ──> Completed
Active ── OnError(error) ─> Failed
Active ── Dispose() ─────> Cancelled
```

Completed, Failed and Cancelled are terminal. Completion or failure occurs at most
once; neither can follow the other. A stream may remain active indefinitely.
Cancellation silently detaches that subscriber; it does not call `onCompleted`,
fail other subscribers or terminate a shared producer. Terminal delivery
automatically releases that subscription's upstream resources and callbacks.

E is a meaningful domain error, such as `WatchError`, rather than a required
exception superclass. `onError(error)` terminates a subscription; a recoverable
per-item failure belongs in T, for example `IObservable<Result<Item, ItemError>, E>`.
A consumer can therefore distinguish a failed item from an unusable stream.

Panic is not converted into E, swallowed, or delivered to other observers as an
ordinary stream failure. Callback and cleanup panics retain the runtime's current
terminal behavior; this proposal does not depend on future panic unwinding.
Normal cleanup callbacks must not require recovery from panic to uphold the
protocol. A transformation that can fail uses an explicit Result-returning
operator, not implicit exception interception.

### Producer example

An explicit shared producer is analogous in purpose to an Rx Subject, but exposes
a separate subscription-only view:

```text
public union PublishError {
    case Stopped
    case PendingLimit
}

public class Subject<T, E> {
    init(pendingCapacity: int)
    val Observable: IObservable<T, E>
    func TryOnNext(value: T) -> Result<unit, PublishError>
    func TryOnCompleted() -> bool
    func TryOnError(error: E) -> bool
}

let changes = Subject<Change, WatchError>(pendingCapacity: 64)

let subscription = changes.Observable.Subscribe(
    change => {
        RenderChange(change)
    },
    error => {
        ShowWatchFailure(error)
    },
    () => {
        ShowWatchFinished()
    }
)

match changes.TryOnNext(change) {
    Ok(_) => { }
    Failure(error) => {
        HandlePublishRejection(error)
    }
}

let completed = changes.TryOnCompleted()
// On a failing run, use changes.TryOnError(watchError) instead.
subscription.Dispose() // Safe even after completion.
```

`TryOnNext` reports acceptance, not completion of all callbacks. `Stopped` means
a terminal notification has already been accepted. `PendingLimit` means that value
was not accepted and no stream failure was generated automatically; the producer
must decide whether to retry later, drop it explicitly, or terminate with a domain
error. Capacity must be positive; invalid configuration is a programmer error.
The spelling and capacity policy are provisional, but implementations must never
silently drop or accumulate an unbounded queue.

The first `TryOnCompleted` or `TryOnError` seals admission and returns true;
subsequent terminal attempts return false. Accepted values drain before the
terminal notification. The subject retains its terminal outcome so late
subscribers receive that outcome, without replaying values, and get an inert
subscription. Retaining an error may retain its object graph until the subject
becomes unreachable. Other observable producers need not share this late-subscribe
policy; a cold producer starts a new lifetime for every subscriber.

## Serialization and subscription races

Reactive delivery must not inherit raw signal reentrancy. A subject serializes
delivery on its owning context: a publication made within a callback is queued
and drained after the current notification finishes for its recipients. This
ensures one observer's callback has returned before the next notification enters
it. A pending terminal notification occupies a separate single slot, so a full
value queue cannot prevent completion or failure.

Each accepted value captures its recipients. A subscriber joining during draining
receives subsequent accepted values, not older queued ones. Each recipient has an
active flag checked immediately before delivery, unlike raw signal snapshots.
Disposing suppresses queued notifications for that subscriber; an executing
callback is allowed to finish. Clear detached callback references promptly even
when a queue still retains the registration entry.

A subscriber joining after terminal admission receives the terminal outcome in
the serialized drain, or immediately if no drain is running. Late registrations
must not interrupt an executing callback. Delivery order between independent
subscriptions is not a general observable guarantee; this subject preserves
registration order for each captured notification.

`Subscribe` can deliver values or a terminal outcome before it returns. Operators
must create their downstream state and a single-assignment upstream holder first.
If `Take(1)` completes before the upstream handle is returned, it marks the holder
disposed; assigning the handle later immediately disposes it. It must also close
its delivery gate immediately, so a synchronous upstream loop cannot forward more
values. A cooperative producer can inspect endpoint stop state to avoid wasted
work, but correctness must not depend on that optimization.

Owner affinity avoids concurrent races in this initial profile. A future
cross-thread adapter must marshal to an admitted owner context, define queue
capacity and ordering, and qualify GC/root/thread-entry contracts. It must not
invoke managed observers directly from an arbitrary native callback thread.

## Signal adapters and operators

The general adapter accepts a structural subscription factory. This lets the
publisher or caller choose a projection without teaching the reactive library
every possible F signature:

```text
Observable.FromSignal<T, E>(
    attach: ((T) -> unit) -> Subscription
) -> IObservable<T, E>

let changes = Observable.FromSignal<Change, WatchError>(next => {
    widget.Changed.Subscribe((sender, change) => {
        next(change)
    })
})

let subscription = changes
    .Where(change => change.IsVisible)
    .Select(change => change.Name)
    .Take(10)
    .Subscribe(
        name => { DisplayName(name) },
        error => { ShowWatchFailure(error) },
        () => { ShowBatchFinished() }
    )
```

`FromSignal` registers one handler per subscription and unregisters it on disposal.
Its guarded endpoint suppresses signal snapshot callbacks after cancellation.
The adapter itself never emits E or completion; E is selected for compatibility
with the surrounding pipeline. `Take` can complete this example. Publisher shutdown
requires an explicit terminal source or a separate adapter contract; unsubscription
must not be misreported as successful completion. A truly infallible stream type
or bottom error type is future design work, not a prerequisite.

Raw signals allow recursive dispatch. For this initial adapter, a nested
notification to the same subscription is a protocol violation detected by the
endpoint and reported as a programmer panic, rather than recursively invoking
the observer. A general signal-to-stream bridge will need an explicit bounded
serialization option and an overflow-to-E mapping before claiming arbitrary
reentrant signal support. This limitation is concrete: use the subject directly
when reentrant stream publication is required.

| Operator | Proposed contract |
| --- | --- |
| `Where((T) -> bool)` | Retain matching values; forward the same E and completion. |
| `Select<U>((T) -> U)` | Project values; forward the same E and completion. A panic remains terminal runtime failure. |
| `TrySelect<U>((T) -> Result<U, E>)` | First Failure terminates that operator subscription with E and detaches upstream. |
| `SelectError<E2>((E) -> E2)` | Map the terminal error domain explicitly; preserve values and completion. |
| `Take(count)` | Nonnegative count; zero completes without subscribing upstream, positive count completes after that many values and disposes upstream. |

Operators are lazy descriptions. Each subscription creates independent operator
state and upstream registration. They do not imply replay, caching, scheduling or
shared subscriptions. `Merge`, `Switch`, `Replay`, `Retry`, timers and time-based
operators require separate bounded work; combining differing error domains needs
explicit mapping or a standard union. No automatically generated error supertype
is proposed.

## Cold producers and asynchronous boundaries

A later `Observable.Create<T, E>` can accept a structural factory of shape
`(ObserverSink<T, E>) -> Subscription`, called separately for each subscriber.
Its sink is a guarded producer endpoint with stop state and the same notification
protocol. Returning the resource handle transfers cleanup responsibility to the
subscription holder, including when the factory completes synchronously before
returning. Resource-acquisition failures go through E; partially acquired resources
remain the factory's responsibility until transferred.

This supports a cold request, timer or watcher subscription without requiring a
global callback list. The initial subject and signal adapter should establish
lifetime semantics before exposing a general producer factory.

Callbacks return unit. An async lambda returning Task is not silently accepted or
discarded. Future Task/async-sequence adapters must define ordering, concurrency,
cancellation, failure mapping and producer resource release. Waiting for work uses
`await`; detaching notifications does not prove that an external operation has
stopped. Where shutdown needs acknowledgement, expose an awaitable shutdown
operation separately from synchronous subscription disposal.

Neither synchronous callbacks nor bounded subject queues provide demand-based
backpressure. Slow synchronous observers delay the producer; queued adapters need
explicit capacity and overflow behavior. Scheduling, GUI dispatch, time and
backpressure are separate policies, not event metadata or automatic Task.Run calls.

## Implementation and validation plan

Implement ordinary managed collections, source/view objects and registration
state. A copy-on-write registration array is a candidate for raw signals, where
dispatch retains a stable array and mutations replace it. Reactive entries also
need active gates; avoid cloning handler closures unnecessarily. Choose the
collection using relevant measurements if dispatch/mutation costs become a design
criterion. No throughput or allocation improvement is claimed here.

The visitor-based managed library needs no privileged runtime service or FFI
provider. Existing function and generic metadata describe its signatures; GC
traces captured values through ordinary containers. Signature projection and packs
require additional compiler design, and any materialized invocation context must
respect runtime roots and lifetimes. Do not add an event member table, special
add/remove opcodes or delegate inheritance. Use checked dynamic Invoke only for
the explicit boxed forwarding alternative, not as a hidden requirement of Rx.
The temporary Raven CLI bridge may retain nominal delegate transport internally;
that does not establish nominal native function identity. Qualify imports and
native output against the existing Runtime Contract instead of assuming ordinary
.NET execution proves neoCLR support. No new Runtime Contract setting is proposed.

Bound implementation in this order:

0. Prototype projected `func Signal: T` declarations and explicit incoming argument
   representation. Cover zero/multiple parameters, concrete and forwarded T,
   separate-provider imports, unit-result admission, unsupported reflection shapes,
   GC rooting and interpreter/JIT/AOT parity. Decide pack metadata and bridge
   encoding before claiming the syntax works. Compare typed forwarding with boxed
   forwarding if allocation/performance motivates choosing between them.
1. Compile `SignalSource<F>`, typed visitor dispatch and a separate-provider
   consumer. Verify captures and exact shape checking in interpreter, JIT and AOT.
2. Implement subscription handles and raw snapshot rules with focused behavioral
   tests, including duplicate functions, nested dispatch, removal during dispatch,
   retained receivers and collection after detachment.
3. Implement guarded observer endpoints and the subject. Test normal completion,
   failure, late subscribers, cancellation, queue limits, reentrancy, and terminal
   admission while values are pending. Verify rejected values produce no delivery.
4. Add the restricted signal adapter and `Where`, `Select`, `TrySelect`,
   `SelectError`, `Take`. Test cancellation through the whole chain, callback panic
   behavior, terminal-once delivery and synchronous completion before handle
   assignment. Verify two subscriptions maintain independent state.
5. Add a concrete executable consumer, then cold producers and scheduling only
   through separately scoped contracts and tests.

Before shipping public APIs, add XML/member documentation, RavenDoc type selection
and matching reference snapshots. Turn the sketches into compiled examples and
label website capabilities according to integration/release status. This proposal
alone changes no API snapshot and requires no website build. Any compiler-affecting
bridge changes must be documented and validated in both Raven and neoCLR.

## Decisions still open

- Signature-projected method syntax, argument-pack binding/representation,
  generic import encoding and unit-result constraints. The author's request is
  for these to be representable, not an approved spelling or implicit stack API.
- Final namespaces, assembly packaging, naming and variance admission.
- Whether bounded subject publication belongs in the first slice or a later
  reactive slice; raw signal delivery does not depend on it.
- A general bounded reentrant signal adapter and explicit overflow error mapping.
- An ergonomic infallible stream contract without inventing a nominal callback type.
- Concurrent producers, scheduling/clock contracts, async consumption and
  acknowledged resource shutdown.
- Tooling discovery of signal properties without event metadata. Optional ordinary
  attributes could support this later, without changing invocation or ownership.

The author has requested this proposal, not implementation of all these choices.
The [platform roadmap](../platform-roadmap.md) remains authoritative for sequencing.
