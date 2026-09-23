# Concurrency and tracked threads — post-release direction

**Recorded 2026-09-23. Planned, not implemented.** The author directs renaming
`System.Threading` to `System.Concurrency` after the async/Tasks release to express
a broader area. The release keeps its current API. This does not select a move of
`System.Tasks` or change the Streams, Storage and Encoding priority before networking.

Thread support may be unavailable on some platforms. A future optional package
could be named `System.Concurrency.Threads`; that is a packaging possibility, not a
selected namespace. Package boundaries, capability detection and unsupported-target
behavior remain open.

## Two author-selected API scenarios

The author clarified the intended distinction with `Thread.Run`, construction,
`Start()` and a `Task` property. These are proposed shapes, not executable samples:

```raven
// Result-oriented execution: retain the operation, without managing a thread object.
let pending: Task<int> = Thread.Run(() => 42, ...)
let result: int = await pending
// Equivalently: let result = await Thread.Run(() => 42, ...)

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

`Run` describes executing work on a thread and observing its result. Whether it
always creates a dedicated thread or uses a pool must be explicit before adoption;
it must not silently imply both. The constructed object represents a particular
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
static result-oriented Start to Run and adding instance Start are separate API
changes to validate; the namespace rename alone does not implement them.

## Comparison and design questions

Primary .NET 10 library contracts reviewed 2026-09-23:

| Baseline | Relevance to neoCLR |
| --- | --- |
| [Task.Run](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.task.run?view=net-10.0) queues work to the thread pool and returns a task | Demonstrates result-oriented submission without owning a particular thread. The proposed Thread.Run name must specify its scheduling policy. |
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
  Compare dedicated and pooled execution and document the selected policy.
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
