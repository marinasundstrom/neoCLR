# HttpClient lifetime and timeouts — development, 2026-10-10

The author requested configurable request timeouts and Disposable support immediately
after the encoding migration. Lifetime support is implemented in this slice; configurable timeout remains next.

## Current gaps

HttpClient has no Timeout member. HttpExchange creates a single 15000 ms
NetworkDeadline covering DNS, connection and transfers. Request cancellation is
supported; built-in exchanges close connections and owned upload streams on completion.
Custom handlers receive a cancellation token. CancellationTokenSource has no timer-based
CancelAfter operation. NetworkDeadline currently accepts only 1–60000 milliseconds.
The native HTTP correctness host also has a 15-second watchdog independent of requests.

## .NET baseline and proposed sequence

Primary contracts checked 2026-10-10:

- [.NET HttpClient.Timeout](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.timeout?view=net-10.0)
  uses TimeSpan, defaults to 100 seconds, accepts the infinite sentinel, and cannot be
  changed after requests start or disposal. Request cancellation may shorten its budget.
- [.NET HttpClient constructors](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.-ctor?view=net-10.0)
  own the supplied handler by default; the handler-plus-Boolean overload allows a
  borrowed handler. Clients are intended to be reused.
- [.NET HttpClient.Dispose](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.dispose?view=net-10.0)
  establishes the familiar IDisposable lifetime surface. Examine the linked runtime
  implementation before defining pending-request cleanup and cancellation races.

First make client/handler ownership explicit, implement Disposable and exercise use,
repeated disposal, borrowed handlers, use after disposal and pending requests. Avoid
adding an inert Dispose method that leaves handler ownership or cancellation undefined.

Then add a Duration-shaped Timeout following the existing neoCLR time model, with a
single monotonic budget per request rather than renewing it between network phases.
Determine cancellation/timeout result distinctions and custom-handler behavior before
implementing the property. The existing 60-second network bound and host watchdog are
constraints to resolve, not reasons to claim a permanent divergence from .NET. A
transport-only setting must not be presented as timing out arbitrary handler pipelines.

## Required qualification

Use deterministic fake handlers for ownership and terminal-state cases, followed by a
controlled local peer for DNS/connect/transfer cancellation and deadline cleanup.
Verify request-timeout, caller-cancellation and client-disposal races finish once,
release registrations/connections and respect borrowed ownership. Run the relevant
interpreted/native consumers on macOS and in Windows CI; record unexecuted platforms
explicitly. Add API reference entries and update showcase authoring only after the
contracts are implemented and tested.

## Lifetime implementation

HttpClient and the abstract HttpHandler base implement Disposable. Default and handler constructors
own their handler; `(handler, disposeHandler: false)` borrows it. Dispose is idempotent,
marks the owner closed before callbacks, requests cancellation of its sends and disposes
an owned handler once. HttpSocketHandler independently cancels its pending exchanges.
Disposed sends fault through the current platform terminal-fault mechanism, including
precancelled sends. Configuration getters remain readable; configuration freezing is
not added in this lifetime slice.

Each send links caller and owner cancellation privately. A completion adapter releases
registrations before publishing its outcome and preserves response/request association.
It waits for handler acknowledgement rather than reporting cancellation before transport
cleanup. Custom handlers which ignore cancellation can remain pending. Completed
responses are caller-owned and survive client disposal. DelegatingHandler supplies forwarding and idempotent inner-handler ownership.
Its nullable InnerHandler property permits construction before assignment, but rejects
null assignments and freezes on first send or disposal. Middleware overrides Send
and normally awaits base.Send. HttpHandler is now an abstract class; existing custom
handlers migrate from interface implementation to override Send. Stateless handlers
inherit Dispose. This experimental API change requires recompilation.

This follows the ownership and cooperative cancellation structure in
[HttpClient.cs](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Http/src/System/Net/Http/HttpClient.cs)
and [HttpMessageInvoker.cs](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Http/src/System/Net/Http/HttpMessageInvoker.cs).
neoCLR currently signals use after disposal with a terminal runtime fault rather than
ObjectDisposedException. Cancellation remains invocation-local, not thread-safe. An
explicit continuation observes cancelled tasks because await cannot currently recover
that outcome. No inert disposal or runtime friend grant is introduced.

The author explicitly selected .NET's delegating-handler architecture during this
implementation. Compared with [DelegatingHandler.cs](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Http/src/System/Net/Http/DelegatingHandler.cs),
the same owned chain and configuration freeze apply. Existing neoCLR names retain
HttpHandler and asynchronous Send; .NET names the base HttpMessageHandler and its
asynchronous method SendAsync. Dispose is directly virtual here because neoCLR has
no finalizer path requiring a disposing Boolean. This simplifies current resource
ownership but is not the complete .NET finalization pattern. Thread-safe disposal
and recoverable object-disposed exceptions remain gaps, not claimed improvements.

## Qualified foundation (2026-10-11)

The ten lifetime/middleware cases and eight guard cases pass in the interpreter and
macOS ARM64 AOT (`target/http-delegating-validation-v8/report.json` and
`target/http-delegating-faults-v3/validation.json`). The pipeline exposed and now uses
[external storage and class dispatch support](native-class-dispatch.md). General
async-base and overload-lookup fixes are on Raven main, with native-specific metadata
and emission on the integration branch. Windows CI is configured but unexecuted.
Timeout remains unimplemented; these results do not qualify a Timeout property.

## Private timer foundation (2026-10-11)

The interpreter now supplies exact InternalCall services `TimerStart(Int32, fn<Void>)
-> Int64` and `TimerCancel(Int64) -> Boolean`, classified as TaskDispatch. These are
private runtime mechanisms, not public Raven APIs. Native AOT binding and the HTTP
library adapter are still required before Timeout can be exposed consistently.

The .NET baseline is scheduled cancellation: [CancelAfter](https://learn.microsoft.com/en-us/dotnet/api/system.threading.cancellationtokensource.cancelafter?view=net-10.0)
starts its countdown at registration and allows later reset (checked 2026-10-11).
This slice implements only the one-shot mechanism underneath such library policy.
It does not add CancelAfter, reset policy, infinite-delay semantics or Timeout itself.

Each invocation owns at most 64 pending timers with monotonic deadlines and
process-unique positive handles. Start accepts 0 through Int32.MaxValue milliseconds;
zero queues work for a later poll rather than invoking inline. Negative delays, invalid
callbacks, exhausted handles and capacity overflow fault before retaining work.
Cancellation removes pending work and releases its managed roots; unknown, foreign,
cancelled or already-delivered handles return false. A callback already transferred
to the scheduler ready slot cannot be retracted. The future request adapter must
check its terminal state, as cancellation cannot undo a callback already dispatched.

Timers share existing fair completion arbitration and the traced ready-slot handoff.
The interpreter parks between polls; it does not allocate a worker thread or post
a self-repeating callback per timer. Due callbacks are ordered by deadline, with
registration order breaking ties. Pending callbacks remain rooted, failed ready-slot
installation preserves roots, and successful installation transfers ownership to
the active frame. Invocation teardown drops remaining timers.

A bounded linear scan reuses the current scheduler and keeps this foundation small;
it costs O(n) polling for up to 64 timers and inherits the current approximately 10 ms
park interval. This is not a throughput or latency claim. A heap/timer wheel and
nearest-deadline parking may be evaluated if measurements justify them. As with the
existing invocation scheduler, callbacks run at safe owner-thread boundaries, not
as preemptive thread-pool callbacks. Full .NET timer concurrency is still a gap.

Focused tests cover deadline boundaries without sleeping, maximum delay, capacity
recovery, stale/foreign handles, GC handoff, exact metadata contracts and executable
guest entry-pump delivery/cancellation. Windows and native AOT qualification remain
required before the HTTP timeout milestone is complete.

Reproduce the interpreter foundation checks with:

```sh
cargo test --lib timer
cargo test --lib scheduler::tests
cargo test --test timers
```

Website review: existing pages continue to label configurable timeout as pending;
this private interpreter mechanism adds no public API reference entry or API snapshot.

## Native timer delivery and public layering (2026-10-11)

The AOT prototype now has an explicit `--bind-timers` contract for the same private
TimerStart/TimerCancel signatures. It requires managed system compilation and native
GC. The existing polled host scope admits one-shot timers alongside network operations,
with a shared 64-operation limit. This shared native capacity differs from the
interpreter's separate 64-timer bound; neither is a public Timer capacity guarantee.
The poller uses monotonic deadlines, preserves timer deadline/registration order, and
never runs guest code inline. Cancellation releases pending roots and returns false
for stale or delivered handles. A delivered timer's root remains valid until the next
owner-thread poll or scope teardown; the host must install/invoke the returned callback
before polling again. AOT entry pumping recognizes timers as host completion work.

Five sanitized native host cases (timer plus existing socket/DNS paths) pass on macOS,
and a compiled AOT consumer retains its callback receiver across collection, invokes
it once, and releases roots after delivery or cancellation. Windows runs the same
kernel validator in CI but has not been executed locally. Public Timer and HTTP
adapters remain pending; these are private mechanisms with explicit host contracts.

The author clarified that the public API should make sense for Raven rather than
necessarily copy .NET. The comparison remains [.NET Timer](https://learn.microsoft.com/en-us/dotnet/api/system.threading.timer?view=net-10.0)
(checked 2026-10-11): its callbacks use thread-pool scheduling. Proposed Raven layering
uses a duration-based Timer with captured callback state and explicit start/stop/dispose,
then timed cancellation, then HttpClient.Timeout. Exact periodic cadence, rescheduling
and disposal behavior must be qualified before exposing those APIs. Invocation-local
callbacks are not a claim of .NET timer thread-pool or concurrent callback support.

## Public Timer and asynchronous callbacks (2026-10-11)

`System.Concurrency.Timer` now accepts either `() -> ()` or `() -> Task<()>`.
Construction leaves it stopped. `Start(delay)` replaces the schedule with one run;
`Start(delay, interval)` repeats, measuring each interval **after callback completion**.
Durations use ticks, round positive fractional milliseconds upward, and are bounded
by Int32.MaxValue milliseconds. Initial zero delay is deferred; intervals must be positive.
`Stop` suppresses unstarted callbacks; `Dispose` additionally forbids restarting.
Neither cancels a callback's own work. `IsRunning` includes awaiting completion until
stopped; a cancelled callback task stops its schedule. Runtime Faults remain terminal.

Restarting while a callback awaits replaces its future schedule but cannot overlap
that callback, even if the new deadline arrives first. Old completion cannot resurrect
an old schedule. An outcome-observation adapter uses OnCompleted specifically to
clean up cancelled callbacks; consumer async functions use await.

Compared with System.Threading.Timer, the deliberately different cadence prevents
async callback overlap and dropped task outcomes. Captured state and Duration avoid
object-state casts and numeric disabled-delay sentinels. Costs: no thread-pool execution,
no fixed-rate cadence, and progress depends on the invocation scheduler; synchronous
callbacks must return promptly. Disposal is nonblocking and does not join callback work.
The current Task API has success/cancellation outcomes, not recoverable exception tasks.

The [consumer](../experiments/timers/Main.rvn) covers ten sync/async scenarios and
[guards](../experiments/timers/Faults.rvn) cover five invalid operations. The validator
runs both interpreter and AOT; Windows CI uses the same fixtures. The test framework
currently accepts synchronous tests, so this asynchronous consumer remains separate.
Native `use` in async functions and state machines beyond the current sixteen-field
AOT value profile remain compiler/backend gaps; small test functions use explicit disposal.
A combined comparison/OR parser failure seen in validation also reproduces on ordinary
Raven CLI. Separate validation statements avoid it; a shared compiler regression/fix
remains deferred rather than defining a native language restriction.

[Local qualification](../experiments/timers/validation.json): all ten callback scenarios
and five guards pass in interpreter and macOS ARM64 AOT. Windows remains unexecuted.

## Timed cancellation (2026-10-11)

`CancellationTokenSource.CancelAfter(Duration)` schedules or replaces its countdown.
`CancelAfter(Option<Duration>)` accepts Some with identical behavior or None to disable
scheduling. Zero is deferred; positive fractions round upward; bounds match Timer.
Cancel and Dispose remove scheduling. An already requested source never resets its
request flag; invalid durations and disposed use still fault. This matches .NET's
reset-before-cancellation and cooperative request model. The deliberate API difference
is explicit Option disabling instead of a negative infinite sentinel: clearer duration
validation at the cost of an additional overload and Some/None at optional call sites.
No linked-source public factory is added in this slice. Scheduling remains invocation-local.

[Timed-cancellation qualification](../experiments/cancel-after/validation.json): four
behavior scenarios and five fault guards pass in the interpreter and macOS ARM64
native executable. Matching API snapshots pass their checks. Windows CI is configured,
not executed. The source remains cooperative; no timeout preemption is claimed.
