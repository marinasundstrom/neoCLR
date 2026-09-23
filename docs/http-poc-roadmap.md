# Roadmap: prove neoCLR with HTTP applications

Detailed M1 plan subordinate to the [authoritative platform roadmap](platform-roadmap.md).
Broader themes, candidate milestones and sample products are maintained there;
explicit author directions take precedence over either plan.

Planning direction **2026-09-23**. The author selects a minimal application POC
using HttpClient and an HttpServer/HttpListener-style API as the first major
milestone for justifying the runtime. Socket abstractions, streams, encoding and a
small JSON API belong to that milestone. Smaller executable cases should develop
each API gradually. This ordering supersedes older “immediate” sequences in the
[roadmap](roadmap.md) and [API plan](runtime-api-plan.md); it does not erase their
history or existing release requirements.

**Committed direction:** demonstrate useful applications on neoCLR. **Provisional
plan:** the scope, API names, implementation layers and slices below. No release
date, full .NET compatibility, production HTTP service or wholesale adoption of the
proposals is promised. All new slices below are planned, not implemented by this
roadmap update. Experiments may change the plan without committing to an API.

The author clarifies that the proposals are neither necessarily consistent nor
pictures of the final design. Treat them as evidence of needs, possible approaches
and questions. Do not resolve contradictions by automatically preferring the newest
text or combine every proposed abstraction into one platform. Start from the small
application cases, compare alternatives and retain, reshape or discard proposal ideas
according to the results. Even an internally consistent proposal is not an API spec.

## First major milestone: two applications

Build two ordinary Raven `.rvnproj` applications with a matching toolchain:

- A server binds a loopback endpoint, serves `GET /hello` as UTF-8 text, and accepts
  `POST /echo` with a small JSON document and returns a JSON response.
- A client uses the public HttpClient API for both routes, reads status/headers,
  decodes the text and inspects the JSON response. Include combining characters
  and emoji so byte lengths cannot accidentally be grapheme counts.
- Both execute on neoCLR through real sockets and the shared byte-stream layer.
  A .NET-hosted prototype can inform API design but cannot close this milestone.
- Exercise each against an independent HTTP peer, not only each other. Demonstrate
  malformed input, a refused connection, interrupted transfer, cancellation and
  bounded server shutdown. Repeated requests and start/stop cycles release resources.
- Supply checked source, expected output, build/run instructions, exact runtime and
  compiler revisions, and a documented platform/limitation matrix. Run the packaged
  cases outside the developer checkout before calling the POC reproducible.

Proposed first envelope: loopback TCP, cleartext HTTP/1.1, GET/POST, bounded headers
and bodies, one request per connection and explicit connection closure. This is a
controlled interoperability subset, not a claim of full HTTP/1.1 conformance.
Inventory required framing rules before implementation; explicitly reject unsupported
transfer coding and ambiguous framing. TLS, DNS and public-network use follow as a
separate slice. The proposal's broader portable server ambition remains open.

HttpServer is the networking proposal's term; the author's HttpServe/HttpListener
wording establishes the serving use case, not three required public types. Compare
a bounded accept/respond model with a handler model during the HTTP slice and choose
one. Keep backend details out of the public contract.

## Starting evidence and gaps

| Foundation | Current evidence | Gap relevant to this milestone |
| --- | --- | --- |
| Raven applications | [Application toolchain](raven-application-local-build.md), [MSBuild](raven-msbuild.md), tested library samples | Add matching networking projects and packaged execution; repair compiler/importer gaps only when a case exposes them |
| Task and Result | [Task contracts](task-contracts.md), including the September 23 default dispatch, cancellation propagation and MapResult additions | External I/O registration/wakeup, cancellation requests/tokens, safe cleanup and pending-operation lifetime are still needed; queued continuations alone do not establish socket progress |
| Text | [Current text contract](design/text-abstraction.md): grapheme Char, scalar access and strict UTF-8 conversions | Incremental decoding across byte chunks and an agreed minimal Encoding surface; no general Encoding hierarchy is established |
| Collections and types | [Generic arrays](generic-managed-arrays.md), [collection contracts](collection-contracts.md), ordinary nominal interfaces and Result/Option | Safe buffer ranges and any small header/JSON lookup needs; neither full variance nor new type-expression metadata is a prerequisite |
| I/O | [Bounded file API](raven-file-api.md), [stream design](stream-design.md) | Shared streams, guest networking APIs and guest JSON API are proposed work. Host JSON/module tooling is not evidence of a guest JSON library |

These links describe different dated slices. Later development additions take
precedence over historical limitations within them. No new runtime validation was
performed for this planning update.

## Urgent decisions, in dependency order

1. **External progress and ownership.** Register a pending operation, yield, wake the
   correct invocation, retain the Task/buffer/handle, and finish exactly once. Test
   work arriving when the continuation queue is empty. Compare a host event backend
   with a bounded worker-backed experiment; a Task return alone does not make a
   blocking operation nonblocking. Existing text-only isolated workers do not yet
   establish safe socket or buffer sharing. Retain current compiler state machines
   until runtime suspension demonstrates a concrete advantage.
2. **Cancellation and cleanup.** The Task proposal accepts explicit cancellation
   tokens, while the updated Streams proposal assumes a surrounding cancellation
   context and no token parameters. Compare both with the same pending-read case.
   Decide how requests reach host operations, when cancellation becomes terminal,
   how completion races resolve and who closes each resource. Use explicit
   tokens as one bounded experiment; compare ambient context without preselecting
   either as the final public contract.
   A cancelled Task must not leave native code writing into released memory.
3. **Buffers and stream semantics.** Begin with managed byte arrays and checked
   ranges if that avoids waiting for a general Memory/Span design. Compare retaining
   an owned buffer with copying into host-owned storage. Specify aliasing, mutation,
   GC rooting and release after completion. Defer zero-copy claims and reject escaped
   frame references. Define short reads/writes, empty buffers, EOF, no-progress writes
   and close behavior before sockets depend on them.
4. **Wire text and bounded parsing.** Protocol framing uses bytes and ASCII tokens;
   JSON/text bodies use strict UTF-8. Keep this independent of grapheme String.Length.
   Select byte/body limits, incremental decoder behavior and malformed-input outcomes.
   Grow a small JSON reader/writer from the application, without requiring reflective
   serialization, dynamic dispatch or new union metadata.

Cleanup is a milestone requirement, not a requirement to design every resource API
first. Compare library-owned operation scopes with compiler-protected cleanup on
return, Result propagation and cancelled await. Explicit close in success-only sample
code is insufficient. Record terminal Fault cleanup separately: the current runtime
does not promise guest recovery; the host must reclaim resources on invocation exit.

## Small executable slices

Every row should produce a runnable Raven case, focused negative tests and a short
contract/comparison note. Mark completion only with linked execution evidence.

| Slice / status | Small case and dependency | Exit evidence |
| --- | --- | --- |
| S0 — exploration | Fake delayed I/O producer plus a buffer and native-resource stand-in; reuse Task | Empty-queue wakeup, immediate/delayed completion, unrelated continuation progress, GC retention, cancellation/completion race and teardown; record backend choice and rejected alternatives |
| S1 — planned | In-memory input/output and copy case; can start alongside S0 | Directional contracts, partial transfers, empty-buffer rules, EOF only for nonempty reads, truncated ReadExactly, no-progress WriteAll, bounds errors, repeated cleanup and injected failures; copy helpers require no sockets |
| S2 — planned | Encode/decode a multilingual message split at every UTF-8 byte boundary; depends on S1 | Strict invalid/truncated input outcomes, carried decoder state, final-flush behavior and byte counts; reuse current whole-buffer conversions rather than change Char again |
| S3 — planned | Small JSON round trip in memory; depends on S2 | Read/write object, array, string, number, boolean and null; escaped strings and Unicode, malformed syntax, duplicate-key policy, numeric limits/precision and nesting/size bounds are explicit. Prefer explicit field access and construction; benchmark only if making performance claims |
| S4 — exploration then implementation | TCP listener and echo client; depends on S0/S1 and resolved ownership | Bind/listen/accept/connect/read/write/close, endpoint reporting, short transfers, peer EOF/reset, refused connection, pending accept/read cancellation and repeated shutdown. A stalled connection must not freeze unrelated work; bound admitted connections |
| S5 — planned | HTTP server returns text to an independent client; depends on S2/S4 | Split start lines/headers/body boundaries, methods/targets/status/headers, byte Content-Length, case-insensitive header names, bounded input and explicit rejection of unsupported/ambiguous framing. Choose accept/respond versus handler API using this case |
| S6 — planned | HttpClient calls an independent local server; depends on S2/S4 and shared HTTP framing work | Parse supported endpoint URLs, send GET/POST, inspect non-2xx responses as responses, consume bounded bodies, report transport/protocol errors distinctly and close resources after cancellation/error |
| M1 — planned | Two neoCLR apps exchange text and JSON; depends on S3/S5/S6 | All milestone cases above, independent-peer checks, shutdown/resource stress, recorded platform limits and packaged reproduction |

S2/S3 and S4 are independent after their prerequisites; neither a JSON implementation
nor file-provider redesign needs to block TCP. S5 and S6 should reuse framing tests
but have independent peers to avoid hiding matching client/server bugs. Seeking and
file-backed streams are useful later cases, not prerequisites for non-seekable TCP.

For S3, compare a forward-only reader/writer with a tiny bounded document tree. A tree
is simpler for field access but allocates more; a token API exposes parsing state and
buffer lifetime. Prototype both on the echo document if the choice is unclear. The
JSON number grammar must not silently become Int32-only: either retain number text
with checked conversions or document and reject unsupported values. Streaming JSON
can follow bounded body buffering; incremental UTF-8 decoding still gets its own case.

## Proposal triage

Priority is based on the application dependency, not the size or apparent finality
of a proposal. “Later” means retained exploration, not rejection. This inventory
covers the proposal files present on 2026-09-23, including the working Streams revision.

| Proposal | Priority and .NET/CLR comparison | Bounded disposition |
| --- | --- | --- |
| [Networking](proposals/network-api.md) | Critical: Socket/TcpListener, HttpClient and HttpListener roles | TCP and both HTTP application roles first; UDP, TLS, DNS, pooling, proxies, HTTP/2/3 and web frameworks follow evidence |
| [Streams](proposals/streams-api.md) | Critical: .NET Stream versus directional contracts | Compare directional Task/Result streams with the familiar Stream alternative; resolve buffers, cancellation and lifecycle. Seeking, transforms and structural intersections can wait |
| [Task](proposals/task-model.md) | Critical gap: .NET Task completion, cancellation tokens and I/O integration | Extend current implementation with external progress and bounded cancellation/cleanup; no requirement for a new scheduler API, structured concurrency framework or runtime stack suspension first |
| [String/encoding](proposals/string-api.md) | Critical subset: .NET Encoding/Decoder versus neoCLR grapheme text | UTF-8 boundary and incremental decoding only. Original scalar Char proposal is superseded by the current text contract; specialized strings and other encodings remain open |
| JSON — author-requested addition | Critical subset: System.Text.Json reader/writer/document versus serializer | New maintained contract from S3; no standalone original JSON proposal exists in this inventory. Avoid making reflection or dynamic a dependency |
| [Collections](proposals/collections-api.md) | Supporting: .NET collections/read-only interfaces | Reuse arrays/sequences; add only lookup/growth exposed by headers or JSON. Preserve duplicate headers; do not assume a single-value dictionary suffices. Full hierarchy/variance/frozen collections remain later |
| [Date/time](proposals/datetime-api.md) | Supporting: TimeProvider and monotonic elapsed time versus civil dates | Reuse clocks; investigate deterministic deadlines for I/O. Calendar/time-zone expansion is not required |
| [Globalization](proposals/globalization-api.md) | Later: CultureInfo and culture-sensitive operations | HTTP tokens and JSON numbers need culture-independent behavior. No ambient culture redesign for M1 |
| [Storage](proposals/storage-api.md), [async extensions](proposals/storage-api-extensions.md) | Follow-up: .NET File/Directory/FileStream versus provider capabilities | Reuse stream contracts later for file copy; provider resolution, traversal and async enumeration are not networking prerequisites |
| [Environment](proposals/environment-api.md) | Supporting then exploration: .NET Environment versus contextual capabilities | Use existing arguments/configuration for endpoint selection; ShellEnvironment, desktop integration and extension-property discovery remain open |
| [Introspection/reflection](proposals/introspection-and-reflection-api.md), [capability model](proposals/introspection-model.md) | Later: Type/TypeInfo and reflection execution | Preserve current discovery; no descriptor hierarchy rewrite, metadata contexts, invocation or Emit required for explicit JSON |
| [Generic relationships](proposals/generic-type-relationships.md) | Later except exposed defects: CLI generics/constraints and library numerics | Reuse current Task/Result/nominal interfaces; self types, generic math and general variance need independent evidence |
| [Runtime unions/intersections](proposals/runtime-unions-and-intersection-types.md) | Exploration: CLI nominal interfaces and tagged library carriers versus native type expressions | Stream capability composition is a useful later test, not a blocker. A concrete connection can implement both interfaces without new signature forms |
| [Metadata format](proposals/metadata-format.md) | Exploration: ECMA-335 tables/signatures with extensions | Preserve current compiler/importer boundary for M1. Separate metadata-reader compatibility probes from networking implementation |
| [Dynamic dispatch](proposals/dynamic-dispatch.md) | Later: .NET dynamic binding versus new runtime operations | JSON values can use ordinary explicit cases; dynamic binders, caches and cross-language dispatch require another motivating application |
| [Runtime architecture](proposals/runtime-architecture.md) | Supporting boundary, broader exploration: CLR host/library/runtime layers | Host owns OS I/O, runtime owns execution/rooting, library owns codecs and HTTP contracts. New backends, JIT/AOT, self-hosting and constrained profiles do not gate M1 |

## Reconcile proposals before stabilizing APIs

- The earlier [stream design](stream-design.md) uses System.IO, separate sync/async
  interfaces and a possible Cancelled I/O error. The supplied revision uses
  System.Streams, InputStream/OutputStream, Task-based I/O and Task cancellation.
  Treat both as alternatives to test against the same small cases; recency does not
  select the final namespace, names or sync/async shape. Record migration only when
  executable evidence selects a contract.
- `Memory<Byte>`, `Size`, intersection signatures and resource syntax in proposal
  snippets are not proof that the current Raven target supports them. Compile the
  minimal surface; use existing nominal interfaces and managed arrays where adequate.
- Task cancellation is distinct from Result.Error and terminal Fault. Do not
  silently adopt ambient cancellation from Streams or duplicate cancellation as
  a StreamError while the Task contract is being resolved.
- The networking proposal permits early .NET-backed experiments. Keep these
  comparisons separate from native neoCLR milestone evidence. A host HTTP client
  wrapper alone would not demonstrate the requested socket/stream foundations.

## Evidence, comparisons and costs

Primary API/specification sources checked **2026-09-23**, using .NET 10 API views:

| Baseline | What informs the plan | Tradeoff / validation still needed |
| --- | --- | --- |
| [.NET Socket.ReceiveAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.receiveasync?view=net-10.0) | Memory-based receive and cancellation; reads may be short, and a zero-length request needs separate interpretation | Managed arrays first reduce API scope but require a rooting/copying policy; compare completion and cancellation with a recorded .NET probe |
| [.NET Stream.ReadAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.stream.readasync?view=net-10.0) | Async byte reads and EOF, distinct from text | Directional interfaces narrow required capabilities but cost adapters and differ from Stream's common base; preserve short-read behavior |
| [.NET Decoder.Convert](https://learn.microsoft.com/en-us/dotnet/api/system.text.decoder.convert?view=net-10.0) | Stateful conversion and final flushing | Strict UTF-8-only scope is smaller but intentionally excludes broader encoding/fallback compatibility; compare chunk boundaries |
| [.NET HttpClient](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient?view=net-10.0), [HttpListener](https://learn.microsoft.com/en-us/dotnet/api/system.net.httplistener?view=net-10.0) | Familiar client and request/response server roles | Closing each connection reduces initial lifecycle complexity but loses pooling throughput. Backend-independent naming does not establish portability or performance |
| [.NET Utf8JsonReader](https://learn.microsoft.com/en-us/dotnet/api/system.text.json.utf8jsonreader?view=net-10.0) | Forward-only UTF-8 parsing is separable from object serialization | A small explicit JSON API avoids reflection but needs deliberate numeric, duplicate-key and ownership contracts |
| [RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html), especially §§2, 5–7 and 9 | HTTP/1.1 parsing, body length, transfer coding and connection management | A restricted POC must reject unsupported messages predictably; internal echo success does not establish protocol conformance |
| [Tokio AsyncRead](https://docs.rs/tokio/latest/tokio/io/trait.AsyncRead.html) | Other-platform evidence for directional async byte I/O and pending wakeups | Its polling and Rust ownership contracts do not transfer automatically to managed Task/GC; pin a version for any implementation experiment |
| [Json.NET reader/writer](https://www.newtonsoft.com/json/help/html/ReadingWritingJSON.htm) | Independent .NET library evidence that explicit JSON reading/writing can be separate from automatic serialization | This is an API comparison, not a choice of dependency or a performance result |

Reuse [stream comparisons](stream-design.md), [text evidence](design/text-abstraction.md)
and [Task design evidence](task-contracts.md) for the rest. No benchmark, .NET comparison
program or backend prototype was run for this roadmap. Specific API-review discussions,
version-pinned backend source analysis and cross-platform probes remain research tasks
for the selected slices; do not infer a .NET defect from our smaller requirements.

## After M1 and progress recording

Prioritize HTTPS/TLS validation, DNS, broader framing/interoperability and connection
reuse when moving beyond loopback; add file streams as a second backend for the same
contracts. Revisit concurrency limits/backpressure with multiple clients and measured
resource use. Then reassess the deferred proposal families against actual application
friction. These are follow-up candidates, not another committed release scope.

For each slice, append evidence/status here and update its maintained design note,
changelog, samples and relevant website page. Keep original proposals intact. General
Raven fixes must be isolated, independently tested and integrated separately from
neoCLR target policy; compiler-affecting integrations need both repositories' docs
and changelogs. Existing Raven debugging and release gates remain required for a
release even though they are not dependencies of every local POC case.
