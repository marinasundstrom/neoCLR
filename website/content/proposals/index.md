# Platform proposals and roadmap

These documents propose possible runtime and library contracts. The platform roadmap selects the current work; individual proposals may conflict, change or be rejected. Comparisons with .NET identify differences and their costs.

**Design snapshot · September 24, 2026.** This is not a release checklist or a promise to implement every proposal. The proposals may conflict and do not prescribe the final design. Names, contracts and priorities will be evaluated through small application experiments. Feature pages show the current implementation; the linked design records contain the fuller discussion.

<a id="http-poc"></a>

## First major milestone: HTTP applications

Preview 9's async and Tasks checkpoint has shipped. The author now selects the socket API as the first implementation goal towards a web app running on neoCLR. Keep the networking proposal as the direction and establish interfaces and behavior through small application cases.

The first major milestone is a Raven HTTP client and server running on neoCLR, exchanging UTF-8 text and a small JSON document through real sockets and shared streams. Existing memory, text/JSON, delayed-copy and storage cases supply the foundations. The next case is loopback TCP echo, followed by HTTP; test each HTTP side against an independent peer.

**Socket implementation started:** an isolated host transport probe checks nonblocking TCP lifecycle, short reads, EOF, half-close, errors and close behavior. Eight checks and a .NET baseline pass on macOS. This is not yet a guest Socket API; Task delivery, GC retention and operation cancellation remain integration work. [Socket contracts and next steps →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/socket-api-design.md)

**Current evidence:** memory byte-copy, strict UTF-8 chunk decoding and bounded JSON document experiments run. The delayed-copy checkpoint connects worker completion to the real VM, retains a suspended Raven consumer through garbage collection and resumes it through the default TaskQueue. Its adapter is experimental; ordinary worker APIs still use queued joins. Completion now progresses between returning default-queue callbacks even when they repost work. Successful worker text/output payloads now have a default 1 MiB limit per worker. Host cancellation now has controlled completion and output-delivery checks. Guest operation cancellation races, queue affinity and broader host-memory accounting remain open before a general I/O API.

The urgent investigations are external I/O progress, buffers that remain valid while suspended, cancellation and reliable resource cleanup. HttpClient and HttpServer/HttpListener-style APIs are the goal; their exact contracts remain provisional. The initial proposed scope is bounded loopback HTTP. TLS and broader networking follow later. These APIs are planned, not available in the current release.

The broader platform roadmap groups candidate follow-ups around working samples: a file catalog, a download queue, a time-aware report, an assembly explorer and a portable sample pack. These themes guide experiments; they are not commitments to every proposed API.

### Design background

[Detailed milestone plan and evidence →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/platform-roadmap.md) · [HTTP POC details and acceptance criteria →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/http-poc-roadmap.md)

<a id="text"></a>

## Text representation and indexing

String represents Unicode text; UTF-8 is the canonical internal representation. String already has strict UTF-8 conversion, byte-boundary helpers and UTF-8 ordinal ordering. Preview 8 has grapheme Char, String length and iteration, with explicit scalar access. Future questions include normalization, efficient traversal, text positions and a dedicated scalar type. This differs deliberately from .NET’s UTF-16 code unit and requires migration. Possible Utf8String and AsciiString types could provide encoding-specific guarantees alongside neutral String and Char. These types and broader encoding abstractions are deferred, not required next steps.

[Current String implementation →](../features/strings/) · [Text design and tradeoffs →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/text-model.md)

<a id="introspection"></a>

## Introspection and resolution contexts

TypeInfo, the sealed MemberInfo hierarchy and RuntimeContext provide basic discovery today. Future work may add dynamic assembly loading, offline metadata, invocation and emit. Compared with .NET’s reflection model, separating descriptions from execution could let tools reuse the model without loading code to run it. Resolution, lifetime and cross-context identity need explicit rules.

Dynamic loading belongs with RuntimeContext. Other runtime services might eventually live there too; an optional collector is an idea, not an implemented context API.

[Current Introspection implementation →](../features/introspection/) · [Introspection design →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/introspection-design.md)

<a id="collections"></a>

## Collection capabilities

Sequence, MutableSequence and List already distinguish read, replacement and growth. Preview 9 uses Filter/Map in place of Where/Select. Preview 8 retains its original names. Preview 9 includes Any, All, Count, Take, Skip, Concat, FlatMap and seeded Fold. Seedless reduction, ordering and grouping remain open extensions. Future immutable or frozen providers and variance rules need explicit contracts. Compared with .NET read-only interfaces, the same caution applies: a read-only view may observe another alias’s mutations. Additional guarantees require more than a different interface name.

[Current collections and queries →](../features/collections/) · [Collection design →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/collection-contracts.md)

<a id="time"></a>

## Time and globalization

Date, Time, Instant, Duration and clocks provide the initial foundation. Proposed additions include time zones, calendars and immutable culture descriptions. .NET’s DateOnly, TimeOnly, TimeProvider and CultureInfo provide comparison points. Explicit concepts can clarify APIs, but bring more types, data dependencies and conversion rules. Culture-sensitive parsing and collation are not established by these proposals.

### Design background

[Time design details →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/date-time-design.md) · [Globalization proposal →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/globalization-design.md)

<a id="io"></a>

## Files, streams and networking

A FileSystem could resolve paths in a host, memory or packaged namespace. Readable, writable and seekable stream interfaces could express supported operations, rather than .NET Stream’s capability flags. This could improve testing and make unsupported operations harder to express, at the cost of more contracts and adapters. Current whole-file helpers remain bounded and synchronous; streams and their waiting behavior still need implementation and agreement.

### Design background

[Filesystem proposal →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/filesystem-design.md) · [Stream design →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/stream-design.md)

<a id="async"></a>

## Completion, cancellation and failure
The proposed Task model describes whether an asynchronous computation produced a value. A task starts Pending and ends Completed with a value, or Cancelled. Its State describes that lifecycle; its Outcome is absent while pending and contains the terminal outcome afterward. Expected operation failures remain ordinary Result values. Faults remain terminal runtime failures outside Task outcomes.

The proposed CancellationToken represents a request; cancellation tokens are not implemented yet. The operation must observe it and choose to terminate before its task becomes Cancelled. The Map operator transforms a completed value; Then chains another task. Both propagate cancellation without invoking the skipped callback. They treat Result as an ordinary payload. The current await implementation propagates cancellation automatically, independently of Result propagation.

**What works in Preview 9:** the Raven-authored `System.Tasks.Task<T>` and `Promise<T>` PoC now exposes State and an optional terminal Outcome, with Complete/Cancel on the producer. Map and Then now compose those outcomes through queued callbacks, propagating cancellation without invoking skipped callbacks. The implementation supports ordinary values, unit and Result payloads, and named async functions. A pending await returns control. `Promise<T>()` and async calls use the active queue or `TaskQueue.Default`; the runtime automatically dispatches default-queue work before the invocation returns. Normal callers do not need to create or drain a queue. Explicit queues remain available for controlled dispatch. A separate worker PoC exposes `Thread.Start` and `ThreadPool.Queue`: isolated workers exchange text and return `Task<string>`. The caller’s queue may block waiting for a worker; guest objects are not shared. These APIs are included in Preview 9.

[Explore the working Task examples →](../features/tasks/)

**Next:** add cancellation tokens and extend host-event progress. A future scheduler abstraction will be evaluated alongside runtime suspension. Explicit producer cancellation, Outcome inspection and immediate/resumed cancelled awaits now work in Preview 9. Await inside for loops is diagnosed until iterator state and cleanup are suspension-aware; protected cleanup and async disposal remain outside the supported subset. Rebuild callers and library artifacts together; Promise replaces TaskCompletionSource, and Complete replaces TrySetResult. State machines are provisional machinery; Raven may need a separate lowering, and runtime suspension remains a later goal. Compared with .NET’s exception-based Task behavior, this model exposes outcomes explicitly but requires different library and compiler contracts. Scheduling, cleanup, logical context and UI affinity still need work; the aim is to avoid routine ConfigureAwait-style boilerplate. Networking and HTTP remain proposals built on this foundation.

### Design background

[Task model proposal →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/task-model.md) · [Implementation assessment →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/task-model-alignment.md) · [Networking proposal →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/network-api.md)

After the async release, System.Concurrency will be the namespace for concurrency, including threading. Thread remains an explicit thread API, with a proposed retained object, Start() and awaitable Task property; it may be unavailable on some platforms. Task will provide a general abstraction and API for work, including concurrent submission through a method such as Task.Run. The platform determines execution: a WebAssembly target might use Web Workers behind this API. An optional System.Concurrency.Threads package remains a possibility, distinct from the namespace. These are future directions; exact signatures, scheduling, isolation and lifecycle control still need design and implementation.

<a id="runtime"></a>

## Execution and language integration

The current backend is an interpreter. The architecture proposals explore native interoperation and language projections, while JIT or AOT execution remains future evaluation. .NET’s runtime, ABI and compiler layers are useful references. More execution paths could broaden use, but add ABI, verification and testing obligations; no replacement backend has been selected.

### Design background

[Architecture proposal →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/runtime-architecture.md) · [Execution direction →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/execution-architecture.md)

<a id="feedback"></a>

## Proposal discussion

Questions, alternative designs and counterexamples are welcome. A small application scenario helps evaluate a proposal against the existing API and .NET behavior. No implementation is required to join the discussion. See [contribution guidance](../#feedback) for code, sample and documentation work.

[Discuss an idea on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues) · [Original proposal index →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/README.md)
