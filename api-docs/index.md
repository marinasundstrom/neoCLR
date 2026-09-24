# neoCLR API reference

neoCLR is an **experimental application platform**. This reference describes the
development API following Preview 9, covering Tasks, explicit threads and the first file streams and host metadata lookup.
The new System.Concurrency APIs require matching development artifacts; Preview 9
downloads retain System.Threading. Names and contracts remain experimental;
this reference does not promise compatibility with future releases.

## Terminal failures

The [fault reference](faults.md) covers runtime-assigned FaultCode values, the Rust
host outcome, CLI/debugger diagnostics and System.Fault. Explicit guest faults use
UserFault; guest code cannot set a code.

## API overview

The feature guides explain current behavior and provide small Raven examples:

| Area | What to explore |
| --- | --- |
| [Arrays](/features/arrays/index.html) | Managed array storage, identity, bounds and collection capabilities |
| [Collections and queries](/features/collections/index.html) | Lists, maps, iteration and query operations |
| [Strings](/features/strings/index.html) | Text slices, Unicode scalars, graphemes and strict UTF-8 conversion |
| [Outcomes](/features/outcomes/index.html) | Option, Result, expected errors and propagation |
| [Tasks](/features/tasks/index.html) | Promise, completion, await, cancellation and isolated workers |
| [Files](/features/files/index.html) | The current bounded UTF-8 file-reading API |
| [Dates and clocks](/features/time/index.html) | Calendar values, instants and clock access |
| [Introspection](/features/introspection/index.html) | Inspecting assemblies, types and members |

The [file stream guide](streams.md) describes the first blocking byte APIs. Broader
Storage and Encoding APIs remain upcoming work; networking follows those foundations. Reference coverage is
being expanded beyond the initial Preview 9 async overview.

## Browse namespaces

The [namespace overview](namespaces.md) explains what each current namespace
contains and links to its available reference pages and feature guides. Start there
for collections, text, Tasks, storage, streams, introspection and runtime support.

Use the API reference navigation to expand namespaces with generated coverage and
select a type. Type pages list constructors, properties and methods. Reference
coverage for older areas is still being expanded; a feature guide does not replace
member documentation.

## Start with Tasks

- [Task&lt;T&gt;](xref:System.Tasks.Task`1) lets a consumer observe completion.
- [Promise&lt;T&gt;](xref:System.Tasks.Promise`1) lets a producer complete or cancel it.
- [TaskQueue](xref:System.Tasks.TaskQueue) dispatches callbacks within one invocation.
- [TaskState](xref:System.Tasks.TaskState) names the three observable states.

Read the [Tasks feature guide](/features/tasks/index.html) for tested Raven
examples, async/await, composition, workers and host cancellation. Those examples
are the intended application syntax. This initial reference does not yet cover the
rest of the library or composition extensions.

Three methods accepting `Func<System.Void>` are documented in the
[callback guide](callbacks.md). DocFX’s .NET metadata reader cannot currently
render this neoCLR signature, so these methods are omitted from generated type pages.

## Explicit threads in development

[Thread](xref:System.Concurrency.Thread) represents an explicit isolated host thread:
construct it, retain its pending Task, then call Start once. Thread.Run is a shortcut
for immediate submission. Completion includes thread termination. The current
callback still takes and returns strings and cannot capture guest objects.
[ThreadPool](xref:System.Concurrency.ThreadPool) retains the bounded worker-pool API.
Neither API implements the planned general Task.Run overload family.

## Storage exploration

The [Storage provider experiment](storage-experiment.md) documents application-owned
File and Directory descriptors used by the same tested Raven workflow with disk
and memory. It is a provisional contract, separate from the published System APIs.

## Reading generated declarations

Signatures come from the compiler reference assembly, with authored XML descriptions.
DocFX renders them in **C# metadata notation**; this does not imply a supported C#
frontend or execution on the .NET CLR. Raven uses spellings such as `Task<int>` and
`Func<System.Void>`. Some supporting union types and compiler interfaces appear in
signatures but do not yet have reference pages. Prefer union patterns in Raven over
metadata carrier accessors.

## How this differs from .NET

.NET places Task in `System.Threading.Tasks` and exposes producer completion through
`TaskCompletionSource<T>`. neoCLR uses `System.Tasks.Task<T>` and `Promise<T>`.
Completion and cancellation are terminal states; expected failures are values such
as `Result<T, E>`, rather than a faulted Task carrying an exception. This makes
expected outcomes visible in the type, but is not source or behavioral compatibility
with .NET Tasks.

The current queue is a single-invocation dispatcher, not a .NET thread pool or
SynchronizationContext. Registering a callback schedules it on the associated
queue, including after completion. These guest objects are not shared-memory,
thread-safe synchronization primitives. Worker execution uses isolated invocations;
its current join can block the parent queue. An async method alone does not provide
parallel execution or nonblocking I/O.

See the [platform overview](/about/index.html) for background and direction and
[proposals](/proposals/index.html) for open ideas. General API usage belongs here
and in the feature guides; repository implementation notes are supplementary.

The development [InputStream](xref:System.IO.InputStream) and
[OutputStream](xref:System.IO.OutputStream) interfaces let the same consumer
use a file stream or an application-defined memory stream. See the
[capability contracts](streams.md#capability-contracts) and the manually documented
[Flush methods](streams.md#flush).

The [Storage POC](storage-poc.md) demonstrates provider lookup, writing, text reading,
seekability and mixed-item enumeration using the integrated development APIs.

[Transitional async builders](async-builders.md) documents the compiler-facing state-machine protocol and its current limits. It may be replaced by runtime suspension.

[Type introspection](introspection.md) explains represented-type equality, hashing and display, with a browsable type/member reference.
