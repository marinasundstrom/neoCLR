# Concurrency and tracked threads — post-release direction

**Updated 2026-09-27. Explicit Thread implemented; shared-context Task.Run selected, implementation pending.** The author directs renaming
`System.Threading` to `System.Concurrency` after the async/Tasks release to express
a broader area. `System.Concurrency` is the namespace for concurrency, including
threading. Thread remains an explicit thread API and may be unavailable on some
platforms. Task is a general abstraction and API, independent of threads, and will
provide concurrent work submission using the target platform's execution mechanism.
These are author-selected design directions; exact signatures and implementation
remain open. The current release keeps its API, and Streams, Storage and Encoding
remain ahead of networking.

Thread support may be unavailable on some platforms. A future optional package
could be named `System.Concurrency.Threads`; that is a packaging possibility, not a
selected namespace. Package boundaries, capability detection and unsupported-target
behavior remain open.

## Task.Run with shared captures — author direction, 2026-09-27

`Task.Run` is selected as the canonical API for submitting new work with lexical
captures, including shared managed objects. The runtime selects how to execute
that work. This direction supersedes a noncapturing/transferable-values-only first
slice. It is **not implemented yet**. Tasks returned by I/O and Promise remain valid
ways to represent completion; they do not need an extra Task.Run wrapper.

“Context” here includes captured variables and managed object identity. It does not
yet select .NET ExecutionContext/AsyncLocal flow, thread-local inheritance, culture
flow or caller-thread affinity. Those are separate contracts. Keep Task/Promise in
System.Tasks. Keep completion-only work compatible with Task<unit>, alongside
value-producing work returning Task<T>. The subsequent author direction is to align with .NET Task.Run behavior. Include
async callback flattening in the intended overload family; exact Raven overload
resolution and completion-only representation still require executable evidence.

The author considers Thread a possible implementation primitive and leaves its
public future open. Do not remove or repurpose its existing isolated string-worker
API as part of that uncertainty. It cannot implement shared captures unchanged:
copying/serializing a captured object into an isolated worker loses shared identity.
A different backend must preserve the Task.Run sharing contract.

### Runtime prerequisites and implementation alternatives

The current runtime has an invocation-owned tracing heap. Managed slots use
`Rc<RefCell<Slot>>`, heap handles use weak references, and the root walk covers the
active invocation. The existing native workers start separate invocations. Merely
sending a delegate or substituting Arc for Rc is insufficient: slot access, GC,
Promise completion, task queues, cancellation and host resources also need a
consistent ownership/synchronization model. Do not add unsafe Send/Sync assertions
to conceal these constraints.

| Candidate backend | Benefit | Required work and cost |
| --- | --- | --- |
| Shared-heap native execution | Blocking work need not stop all runnable guest work; supports the proposed Thread implementation primitive | Coordinate root publication/collection across running and suspended executions, synchronize managed storage and task completion, define resource ownership and shutdown. Library collections do not automatically become safe for concurrent mutation. |
| Runtime-managed cooperative execution | Can retain the existing single-owner heap and preserve capture identity without cross-thread guest access | Add independently resumable executions, fair progress, GC roots and aggregate limits. Blocking host calls stall other tasks unless separately offloaded; simply posting arbitrary callbacks onto TaskQueue is insufficient. |

The author subsequently asked for a recommendation and said to align with .NET
Task.Run behavior. The assistant recommends native-thread execution for the first
implementation: blocking or CPU-bound submitted work must allow caller progress.
A single-thread cooperative backend that stalls all tasks on blocking host calls
would not meet that baseline. Dedicated threads versus a bounded pool remain
runtime policy; a task has no promised thread identity, start order or guaranteed
simultaneous execution. Preserve the ability to use another backend only when it
can meet the documented progress and sharing contract.

Implement the shared execution substrate before adding the public facade. Establish
synchronized storage and a shared heap/root registry, then concurrent Promise/queue
publication and bounded task ownership/teardown. Test those boundaries with native
contracts before exposing Raven Run overloads and refreshing the matching API
artifacts. This is a feature prerequisite, not an optimization project. Existing
Result/cancellation/Fault semantics remain neoCLR's; .NET-like scheduling does not
select exception-bearing Task outcomes.

### Focused acceptance before exposing the API

- Typed and completion-only callbacks, including instance-method delegates and
  captured locals that outlive their creating frame.
- Shared object/array identity and mutation visible after completion, including a
  reference returned from the callback; no serialization or deep-copy substitute.
- Pending/running/completed captures survive forced GC; references become
  collectible when no task, callback, continuation or result retains them.
- Submission makes work runnable without awaiting it. Exercise nested submissions,
  competing runnable work and the chosen backend's documented blocking behavior.
- Promise first-terminal-transition behavior, multiple observers and continuation
  publication remain correct under that backend. Specify cancellation and async
  callback behavior before claiming those overloads.
- Fault remains an invocation failure under the current TaskOutcome model, not an
  invented task error case. Invocation cancellation/fault/exit must stop and join
  owned execution before disposing its shared heap and host resources.
- Bound queued/running work and account for all live frames, captures, heap objects
  and instruction consumption. A task must not reset invocation resource budgets.

### Comparison and sources

.NET Task.Run uses the thread pool and provides completion-only, typed and
async-unwrapping overloads. neoCLR keeps the familiar submission/result role while
making backend choice a runtime policy; the cost is defining portable progress and
sharing explicitly. See [Task.Run](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.run?view=net-10.0).
.NET's ambient [ExecutionContext](https://learn.microsoft.com/en-us/dotnet/api/system.threading.executioncontext?view=net-10.0)
is separate from lexical closure capture; adopting the latter does not implement
the former. Rust documents [Rc](https://doc.rust-lang.org/std/rc/index.html) as
single-threaded; safe native sharing needs more than moving today's heap handles.
Primary sources reviewed 2026-09-27; the alternatives above are neoCLR design
analysis, not claims of shipped behavior.

## Earlier clarification: Task.Run overloads — 2026-09-23

The author clarified that Task.Run should have overloads and return values, as in
.NET. The intended shape includes completion-only callbacks and value-producing
callbacks (`Task.Run(() => 42)` yielding `Task<int>`). This is not a selection of
the existing isolated string-to-string worker API as the general Task.Run contract.
Task and Promise remain in **System.Tasks**; explicit Thread moves to
**System.Concurrency**.

The author expects to revisit Task.Run once suspension and scheduling have a clear
model. Before implementing its public surface, settle callback/capture ownership,
result transport and supported result types, progress and scheduling, and overloads
for async callbacks (including whether their returned tasks are flattened, as in
.NET). The completion-only return shape must also fit neoCLR's existing
`Task<unit>` convention; a non-generic Task is not implemented merely by this plan.

The assistant removed an uncommitted Task.Run facade forwarding only
`Func<string, string>` plus an input string to ThreadPool.Queue. That adapter would
have prematurely presented the worker restriction as the general API. Explicit
Thread and the existing ThreadPool.Queue remain useful bounded implementation
steps; the general Task.Run overload family is pending this design work.

## Platform abstractions before execution primitives

The author clarified that the broader namespace should accommodate forms of
concurrency beyond threads. Applications should primarily use abstractions; expose
lower-level execution primitives only where appropriate for the target. Thread is
one such platform capability, not the definition of concurrency or a requirement
for every neoCLR application platform.

Keep these roles distinct while designing the public surface:

- **Task:** a general abstraction for work, completion and results, with an API for
  submitting work concurrently. The platform supplies the execution mechanism;
  neither the abstraction nor submission requires a public Thread API.
- **Worker:** a possible abstraction for independently executing work, with an
  explicit input/output and isolation contract. Its backend and scheduling policy
  may vary by platform; no common Worker API has been selected yet.
- **Thread:** an explicit lower-level execution resource with identity and lifecycle,
  available only where the platform chooses to expose that capability.

For a WebAssembly target, Task's concurrent execution might use Web Workers behind
its general API. This does not require applications to use a separate Worker API or
make Thread available on that target. This is a platform-policy example,
not a claim that every WebAssembly environment lacks thread support. Determine
contracts for each actual target, including unsupported capabilities, rather than
silently emulating a Thread with different identity or parallelism guarantees.

The author's subsequent suggestion places general work submission on Task through
an operation similar to `Task.Run`, with the platform determining how execution is
scheduled concurrently. The author subsequently confirmed that Task will provide a way to run work
concurrently, with the mechanism depending on the platform. This selects the role
of the API; Task.Run remains the illustrative method shape. A Task remains useful for
any work, including operations initiated elsewhere; not every Task requires Run.
Explicit worker or Thread APIs remain useful when their particular guarantees matter.

A portable Run contract must separate stable application guarantees from backend
policy. Define admission, progress, callback/data transfer, completion, cancellation
and resource ownership consistently; document whether a target uses workers, threads
or cooperative scheduling. Concurrency need not guarantee simultaneous parallel
execution. Merely posting arbitrary blocking work to an event loop does not establish
useful concurrent progress. Determine unsupported-work behavior rather than silently
changing isolation or shared-state semantics across targets. Structured lifetimes and
the exact scheduling contract remain open.

.NET's Task.Run/Thread separation below is a starting comparison, not a reason to
make thread-pool execution the universal backend. Portable abstractions can reduce
application dependence on host primitives; they also require honest capability,
isolation and progress contracts. Keeping explicit Thread access is useful when
thread identity is the actual requirement. Backend selection must not imply shared
objects, preemption or parallelism where the target cannot provide those guarantees.

## Two author-selected API scenarios

The author first illustrated result submission with `Thread.Run`, then suggested
Task.Run-style submission with platform-selected execution. The retained Thread
scenario remains separate. These are proposed shapes, not executable samples:

```raven
// Result-oriented execution: retain the operation, without managing a thread object.
let pending: Task<int> = Task.Run(() => 42, ...)
let result: int = await pending
// Equivalently: let result = await Task.Run(() => 42, ...)

// Explicit thread lifecycle: retain and control the actual thread.
let thread = Thread(() => {}, ...)
thread.Start()
let completion = thread.Task
await completion
```

The original example annotated the awaited result as `Task<int>`. If `Run` returns
`Task<int>`, awaiting it produces `int`; the forms above distinguish the task from
its result. Ellipses stand for undecided options, not a selected signature. The
empty callback illustrates completion without a value; its exact Task type is open.

`Task.Run` would submit general work and return its completion/result, without
requiring the caller to select a thread. An explicit `Thread.Run`, if retained,
would need a separate thread-specific contract. The constructed object represents a particular
thread, with identity and lifecycle control, and exposes awaitable termination
through `Task`. Exact control operations, repeated Start behavior, and access to
Task before Start still need contracts. Awaiting termination should mean that the
thread has finished, not merely that its callback published a value.

## Current implementation and migration

The development library moves explicit workers from `System.Threading` to
`System.Concurrency`. `Thread(callback, input)` creates an unstarted thread object;
`Task` is stable and pending before start, `IsStarted` becomes true after successful
submission, and instance `Start()` rejects repeated calls. `Thread.Run(callback,
input)` constructs and starts a thread and returns its Task. A never-started
object owns no native thread and its Task stays pending.

The completion queue is selected at construction, using the current queue or its
default. Starting in another queue does not change the Task's dispatcher. A
successful Task completion includes joining the dedicated native thread, including
host teardown. Invocation exit retains responsibility for cancelling and joining
outstanding workers even if the guest Thread object becomes unreachable.

This still uses static, uncaptured `Func<string, string>` callbacks and isolated
guest heaps. It does **not** implement arbitrary result types, shared objects,
per-thread cancellation, naming, priorities or a general Task.Run. Queued joins
can block the dispatcher. Callback faults remain invocation faults.
`ThreadPool.Queue` remains the existing bounded worker API.

Preview 9 retains `System.Threading.Thread.Start(callback, input)`. Development
callers must change the namespace and use `Thread.Run`, or construct a Thread and
call its instance Start. Rebuild applications with matching core metadata, library
and bridge; no old namespace aliases or mixed-artifact compatibility are provided.
Task/Promise remain in System.Tasks.

See the [worker sample](experiments/raven-target/samples/library-workers.rvn) and
[contract checks](experiments/task-contract/verify_workers.py). The general
Task.Run model means **spawn work concurrently**: submission starts the operation;
await observes its eventual completion, rather than starting it.

## Comparison and design questions

Primary .NET 10 library contracts reviewed 2026-09-23:

| Baseline | Relevance to neoCLR |
| --- | --- |
| [Task.Run](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.run?view=net-10.0) queues work to the thread pool and returns a task | Demonstrates result-oriented submission without owning a particular thread. neoCLR could retain the familiar submission/result shape while allowing target-specific execution. That divergence requires a clear progress and portability contract. |
| [Thread.Join](https://learn.microsoft.com/en-us/dotnet/api/system.threading.thread.join?view=net-10.0) blocks the calling thread until the represented thread terminates | Supplies the lifecycle baseline. A Task property could compose termination with ordinary await, but needs completion delivery that does not block unrelated work. |

These are library/host-runtime contracts, not requirements to add a language keyword
or new CLI instruction. Reuse existing Task/await mechanisms where possible. The
[worker comparison](isolated-workers.md#comparison-decision-and-open-design) also
covers Rust thread creation and the costs of isolated heaps. A retained thread
object does not by itself authorize shared guest objects, captured closures or raw
native handles.

Keeping the current result-only surface is simpler, but cannot provide the requested
thread lifecycle. Copying .NET's blocking Join is familiar but does not provide the
requested awaitable completion. A retained object with Task composition is the
working direction; it adds ownership, GC roots, completion races and teardown work.
No performance benefit is claimed. Broader API-review experience and alternative
.NET library comparisons remain research tasks before settling implementation.

Control should be explored through explicit scenarios. Cooperative stop requests
are a candidate, not a selected API. A request to stop, actual termination, and
cancelling a wait must remain distinct. Forced abort, suspension, priorities and
restart are not implied by the author's request. Define who owns the live resource
when the object becomes unreachable, what happens on invocation exit, and how
faults and start failures are represented without inventing a faulted Task state.

## Bounded samples and validation to develop after release

- **Result Worker:** compute a value through Run; retain its Task or await directly.
  Compare target execution policies and document progress, isolation and unsupported-work behavior.
- **Tracked Worker:** construct, start, observe identity/state and await `thread.Task`.
  Add one useful lifecycle control only after defining its completion semantics.
- Exercise completion before/after awaiting, multiple observers, access before Start,
  repeated Start, self-wait, failed creation, callback faults and invocation teardown.
  Verify GC retention while pending and cleanup when the object is dropped.
- Check unsupported platforms at the chosen package/build/runtime boundary and prove
  that unrelated concurrency APIs remain usable without thread support.

These are proposed acceptance cases, not test results. Implemented examples must be
compiled on the target before being presented as website samples. This design work
is not an additional gate for the current release.
