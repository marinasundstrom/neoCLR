# Concurrency and tracked threads — post-release direction

**Recorded 2026-09-23. Planned, not implemented.** The author directs renaming
`System.Threading` to `System.Concurrency` after the async/Tasks release to express
a broader area. The release keeps its current API. This does not select a move of
`System.Tasks` or change the Streams, Storage and Encoding priority before networking.

Thread support may be unavailable on some platforms. A future optional package
could be named `System.Concurrency.Threads`; that is a packaging possibility, not a
selected namespace. Package boundaries, capability detection and unsupported-target
behavior remain open.

## Platform abstractions before execution primitives

The author clarified that the broader namespace should accommodate forms of
concurrency beyond threads. Applications should primarily use abstractions; expose
lower-level execution primitives only where appropriate for the target. Thread is
one such platform capability, not the definition of concurrency or a requirement
for every neoCLR application platform.

Keep these roles distinct while designing the public surface:

- **Task:** represents an operation's completion and possible result. A Task alone
  does not promise parallel execution or create an execution resource.
- **Worker:** a possible abstraction for independently executing work, with an
  explicit input/output and isolation contract. Its backend and scheduling policy
  may vary by platform; no common Worker API has been selected yet.
- **Thread:** an explicit lower-level execution resource with identity and lifecycle,
  available only where the platform chooses to expose that capability.

The author suggested WebAssembly as a possible target exposing workers or
Task-oriented concurrency instead of Thread. This is a platform-policy example,
not a claim that every WebAssembly environment lacks thread support. Determine
contracts for each actual target, including unsupported capabilities, rather than
silently emulating a Thread with different identity or parallelism guarantees.

The author's subsequent suggestion places general work submission on Task through
an operation similar to `Task.Run`, with the platform determining how execution is
scheduled concurrently. This is the preferred direction to explore, superseding the
earlier open placement question for portable submission. A Task remains useful for
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

[Current isolated workers](isolated-workers.md) expose static `Thread.Start` and
`ThreadPool.Queue`, returning `Task<string>`. They accept static, uncaptured
string-to-string callbacks, isolate guest heaps and deliver results through queued
blocking joins. There is no public retained Thread instance or lifecycle Task.
The proposed integer and lambda examples therefore express a future API goal,
not existing generic worker or closure support.

After release, inventory namespace/type references in the library, runtime bindings,
compiler integration, metadata, samples, API docs and website. Decide source and
artifact compatibility, aliases if any, and rebuild requirements together. Renaming
the result-oriented entry point to Task.Run and adding instance Thread.Start are
separate API changes to evaluate; the namespace rename alone does not implement them.

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
