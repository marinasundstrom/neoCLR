# neoCLR platform roadmap

**Updated 2026-09-25 · Product-led planning, not a release schedule.**

Build a platform that can justify itself through useful programs. Each milestone
has a theme, a concrete sample product and smaller cases that make the underlying
APIs testable before they are combined. The first major milestone is the
author-selected HTTP application POC. Later milestones and sample names below are
proposed directions; they may be reordered, reduced or replaced after experience.

The [original proposals](proposals/README.md) are inputs about needs and possible
solutions. They may conflict and do not prescribe what the platform must eventually
look like. A milestone selects an outcome to investigate, not every abstraction in
its source proposals. Keep, adapt or discard ideas based on working samples and
[comparison with .NET/CLR](design-research.md).

**Author-directed website work, 2026-09-24:** migrate the product and API documentation
to one RavenDoc site, with Markdown guides, an HTML landing page, front-matter layout
controls and a pinned portable CI build. Documentation may precede a runtime release;
show that status explicitly. This bounded publishing change does not reorder the
application milestones. See [website maintenance](design/feature-pages.md#ravendoc-site-and-landing-page--2026-09-24).

**Author direction, 2026-09-24:** after finishing the semantics slices, implement
APIs and behavior as concrete application cases need them. Examine each case before
choosing its contract; keep readily changeable choices provisional. The discussed
Iterable<char> constructor/count overload remains undecided, not a planned API.

## Active direction — sockets for a web application, 2026-09-24

The author directs us to start implementing the APIs needed to build a web app
running on neoCLR, establishing interfaces and behavior as we go. **The first goal
is the socket API; keep the [networking proposal](proposals/network-api.md) as the
direction.** Its async I/O, Result failures, portable sockets and shared-stream
layering guide the work; exact signatures develop through executable cases.

This direction supersedes earlier “networking remains later” priorities and the
requirement to finish a general semantics review first. Preserve the unfinished
Object/String work and its evidence; handle further foundations when the socket or
web-app case needs them. It does not select all later proposal features at once.

Start with bounded loopback TCP echo. The [socket design](socket-api-design.md)
records candidate contracts and the next integration gates. The initial
[transport implementation](experiments/socket-api/README.md) passes eight host-side
checks plus a .NET comparison baseline. It is not a guest Socket API: Task delivery,
GC roots, operation cancellation and the runnable Raven/neoCLR echo remain open.
Continue with that bridge, then TCP streams and HTTP. S4 has started, not completed.

## HTTP client pipeline checkpoint — 2026-09-24

The author asks for a provisional client following the proposal's request/response
and content model, with handlers attaching behavior to the request/response pipeline.
The [HTTP client sample](experiments/http-client/README.md) separates client,
forwarding/fake handlers and socket transport. Keep those boundaries while testing a
bounded GET/200 exchange and UTF-8 content; the provisional APIs now live under System.Web.Http with reference coverage.
The [HTTP design](http-client-design.md) records .NET comparisons and remaining gates.
The [HttpServer sample](experiments/http-server/README.md) now exchanges UTF-8 text
with the neoCLR client and an independent .NET client. Its bounded ServeOne callback
keeps the hosting loop provisional. The [JSON report application](experiments/http-json/README.md)
now composes the existing JSON consumer with this exchange, including independent
Python client/server checks. JSON remains application-local pending public API design. Keep transfer/request lifetime explicit. M1 remains incomplete; broad HTTP, TLS,
retry policy and runtime suspension are not prerequisites for this controlled POC.

## Networking and web release target — discussion, 2026-09-25

**Author direction:** work toward a release containing networking and web that feels
coherent and reasonably complete, without requiring a finished platform. HttpClient
and cancellation tokens are explicitly included. The author subsequently adds
IPAddress, initially as a standard union. The author subsequently selects a closed
class hierarchy with IPv4Address and IPv6Address implementations instead. The additional scope below is the
assistant's recommendation for review, not approval of every proposed API or a
release date. Continue the current typed HTTP/error/base-address integration first.

**Author follow-up:** requests completion of the slices one by one. The sequence is
1. typed HTTP errors; 2. IPAddress; 3. cancellation foundations; 4. HttpClient/base
address; 5. methods/content/statuses; 6. framing; 7. server lifecycle; 8. application;
9. release stabilization. The author proposes an explicit asynchronous
Accept/AcceptRequest returning a Result<HttpContext, HttpError> alongside callbacks;
[evaluate ownership and response completion in slice 7](http-server-design.md#explicit-asynchronous-acceptance--exploration-2026-09-25).
The author clarifies HttpContext as a foundational per-exchange concept for HTTP web
applications, closed/disposed when handling is done. That is the architectural direction;
accept naming, response completion and disposal details remain design work. No context
API is implemented yet, and the current slice order is unchanged.
The [address hierarchy checkpoint](experiments/ip-address-hierarchy/README.md)
now links the [implemented address slice](ip-address-design.md): public parsing/formatting,
value semantics, typed DNS results and Socket overloads. The [cancellation foundations](cancellation-design.md)
now provide invocation-local source/token/registration contracts and keep a request
separate from acknowledged completion. [BaseUri and string/Uri address overloads](http-client-design.md#baseuri-and-address-overloads--implemented-2026-09-25)
are now integrated and tested through the handler pipeline. Slice 4 is complete for
the bounded GET POC. Slice 5 now supports final response statuses 200–599,
IsSuccessStatusCode and GetString status errors, with bodyless 204/205/304 handling
and [independent status evidence](experiments/http-status/README.md). HttpStatusCode now provides common names and preserves unnamed values; response/error
signatures carry it, with a retained numeric constructor. Optional property patterns
are verified separately from proposed positional deconstruction. Slice 5 now includes a [buffered POST echo checkpoint](experiments/http-post/README.md):
string/Uri and token overloads, byte/UTF-8 content, bounded server reads and independent
peer checks. Request/response GetHeaderValues now supplies ASCII case-insensitive
lookup preserving repeated fields; [header evidence](experiments/http-headers/README.md).
WithHeader now constructs replacement application fields on a new request and the
socket provider validates/serializes them; [request header evidence](experiments/http-request-headers/README.md).
Put/Patch/Delete helpers and request factories now share the pipeline, with matching
server parsing and [focused verb evidence](experiments/http-verbs/README.md). This completes
the bounded slice 5 checkpoint. Slice 6 now implements HEAD and bounded fixed-length,
chunked and close-delimited reception; see [framing behavior and limits](http-client-design.md#bounded-response-framing-and-head--2026-09-25). The cancellation fixture now signals independently of the third request;
headers and isolated-body checks pass. [The evidence](experiments/http-status/README.md)
retains timeout sensitivity under competing local work for serial release validation;
no production scheduler/deadline fix is claimed for that cancellation fixture. Stream-backed HttpContent is an
explicit author direction (2026-09-25), not yet implemented. Plan ownership/close,
cancellation, known/unknown lengths and repeatability with the stream/framing slice;
current byte buffering is a checkpoint rather than the final content contract.
The [private native cancellation hooks](cancellation-design.md#native-operation-acknowledgement--implemented-2026-09-25)
now cover DNS, connect, accept and transfers, preserving completion ownership.
Managed DNS/socket token registration and acknowledgement are now implemented, with
[focused loopback evidence](experiments/network-cancellation/README.md).
[HTTP token forwarding and GetString](experiments/http-cancellation/README.md) now
complete the client slice, including cancellation cleanup, request isolation and
strict UTF-8 conversion through the same handler pipeline. An [HTTPS feasibility checkpoint](http-client-design.md#https-feasibility-checkpoint--2026-09-25)
records a possible TLS adapter and its required validation; it is not an implementation
or a release-scope decision. IPv6 transport and scope identifiers remain explicit follow-ups. Investigate HTTPS
feasibility after cancellation foundations and decide its scope before the application slice. Finish validation and documentation
for each slice before advancing; a source change alone is not completion.

The initial GET/200, small-buffer and ServeOne POC proved the path, but it was too
narrow to be the whole release experience. Prefer these connected release gates:

| Slice | Proposed release outcome and evidence |
| --- | --- |
| Client contract | Token-aware Send, Get/GetString and common verb helpers; optional string BaseUri with the recorded relative/absolute rules; fake and forwarding handlers use the same pipeline |
| Address values (author-selected) | System.Networking.IPAddress as a closed class hierarchy with immutable IPv4Address and IPv6Address implementations; validated parsing, formatting and value semantics, followed by DNS/socket integration with explicit transport limits |
| Requests and content | Method, resolved URI, usable case-insensitive headers, byte and UTF-8 text bodies with content type; POST round-trip, empty content and non-ASCII content tested |
| Responses and errors | General status values, headers and content; 201/204/400/404/500 examples; typed HttpError with inspectable causes; distinguish HTTP status, transport failure, cancellation, timeout and decoding failure |
| Cancellation and lifetime | A source/token pair usable beyond HTTP; cancel before dispatch and during pending work; configurable request deadline; deterministic connection/buffer cleanup and explicit handler ownership/reuse rules |
| HTTP interoperability | Bounded Content-Length and chunked-body reception, legal no-body responses, close-delimited response handling, malformed/ambiguous framing rejection; client and server each checked with independent peers |
| Server lifecycle | Foundational HttpContext with request/response lifetime and explicit close/dispose, shared by accept and callback styles; repeated requests, GET and POST bodies, application-selected statuses, cancellation while waiting/serving and bounded shutdown; define in-flight request ownership; demonstrate a small bounded concurrent workload without adding a routing framework |
| Release product | A storage-backed notes app: list/read notes, POST UTF-8 content to save one, report missing/invalid input and stop cleanly; packaged client/server samples, API docs and matching SDK/compiler/runtime artifacts |

Keep bodies buffered initially with documented/configurable bounds. A stream-backed
body API is a follow-up unless the chosen sample demonstrates a concrete need;
do not require full duplex streaming just to release ordinary request/response I/O.
The notes app should reuse storage APIs and the bounded JSON work, extracting only
the small public JSON contract the app needs. A serializer/reflection framework is
not a prerequisite. Check the application API with familiar .NET patterns while
retaining Result/Option and explicit ownership.

**HTTPS scope decision:** investigate a maintained native/platform TLS backend early,
including certificate trust, hostname validation, cancellation and target packaging.
Recommend HTTPS client support if this release is meant to call ordinary external
services. If deferred, describe the release explicitly as experimental plain-HTTP
client/server support for controlled environments. Server TLS may remain separate.
Do not implement cryptography or accept invalid certificates as a shortcut.

**Defer by default:** HTTP/2/3, automatic retries/redirects, cookies, proxies,
compression, WebSockets, connection pooling, a general hosting/routing framework,
TcpClient/UdpClient convenience wrappers and runtime suspension. HostEntry and full
IPv6 transport remain follow-ups; IPAddress itself is now author-selected release
scope. Its two address-family cases do not imply implemented IPv6 transport. The additional .NET client machinery has real lifetime
and performance benefits, but also broadens this release's contracts and tests.
Document the costs of connection-per-request behavior and retained IPv4-only limits.

**Author validation direction, 2026-09-25:** for each slice, run only the most relevant
checks and skip website builds. Keep documentation/reference snapshots current;
reserve broad integration/site validation for release stabilization or a concrete
regression need. This overrides the default per-API-change website build requirement
for the current slice sequence.

**Efficient validation:** run common parser, URI, handler and error behavior once per
relevant change. On each supported target, exercise the actual socket/DNS/TLS and
cancellation/cleanup boundary plus one packaged smoke exchange. Perform focused
GC/race/fault checks and independent-peer interoperability before release. Use a
fresh matching Raven release/verified main baseline when porting .NET Raven examples
to distinguish shared compiler defects from neoCLR-specific problems. Refresh API
reference and on-site limitations with each integrated slice. No version is selected.

Comparison sources reviewed 2026-09-25:
[.NET SendAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpclient.sendasync?view=net-10.0)
for the request/token boundary,
[.NET HttpClient lifetime guidance](https://learn.microsoft.com/en-us/dotnet/fundamentals/networking/http/httpclient-guidelines)
for handler ownership and connection reuse costs, and
[RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html)
for framing and completion. These are reference points, not a claim that the bounded
neoCLR stack already implements their full behavior. Reuse the detailed
[client](http-client-design.md), [server](http-server-design.md) and
[socket](socket-api-design.md) research for each implementation slice.

## Scheduling checkpoint before the public socket bridge — 2026-09-24

The author asks that socket work account for runtime-owned async and reevaluate
scheduling beyond TaskQueue. The [runtime scheduling design](runtime-scheduling-design.md)
selects an internal scheduling boundary as the next implementation checkpoint:
operation completion, ready-work arbitration, durable wakeup, continuation destination
and GC ownership must not depend on generated state-machine callbacks. TaskQueue is
transitional compatibility machinery; no public Scheduler class or full suspension
implementation is selected. This refines the socket path rather than replacing it.

The initial private native-host driver now centralizes root collection, rotating
source arbitration and worker wake notifications. Existing TaskQueue behavior is
preserved; cancellation/socket readiness still has a bounded polling fallback.
Next make operation/resumption ownership explicit and settle continuation affinity,
then attach reusable sockets and Task/Result completion for Raven echo. Current TCP/GC/VM probes remain evidence; they do not
establish a portable scheduler or runtime-owned suspension.

**Author clarification, 2026-09-24:** do not implement runtime suspension yet. Keep
application contracts distinct from transitional compiler/TaskQueue protocols and
private scheduler machinery. Build the capabilities needed now with generated state
machines; preserve a replaceable execution mechanism for later suspension. The ready
callback/destination handoff now has explicit traced ownership until active frame
installation. Broader affinity/context machinery must not become a prerequisite for
all useful I/O; any affinity change still needs its own migration checks.

## Authority and use

**This roadmap is authoritative for our work unless the author explicitly directs
work otherwise.** It governs default priorities, milestone sequencing and scope.
An explicit author instruction takes precedence, including a bounded task outside
the current milestone. Such a task does not silently reorder the entire roadmap;
record a lasting change in direction when the author makes one.

**Completed author-directed slice, 2026-09-23:** develop the common Console API and standard
streams. Console stays a class, with static In/Out/Error access. The bounded
[Console sample](experiments/console-streams/README.md) exercises line input, separate
text/byte output channels and reader/writer ownership. This is synchronous I/O;
TaskQueue/suspension exploration remains open and networking stays later.

**Previous author-directed focus, updated 2026-09-24:** consolidate Object/value
semantics and investigate String storage and reference identity. Object is abstract;
class identity and overrides, bounded primitive equality/hash/display, String content
contracts, Path and introspection semantics have checked samples. The
[consistency review](object-model-review.md) records their exact limits. String
ReferenceEquals now follows the shared text owner, separately from content equality.
The [storage investigation](string-storage-design.md) measures current copy/wrapper
costs and a shared-text prototype. Immutable shared String storage now adopts owned
UTF-8 buffers and retains text across value copies. Private ownership checks now
cover Object/interface round-trips, fields,
arrays, erasure, GC pressure, host retention and cyclic/fault teardown. Owner-based
reference comparison and stable identity/base hashes now use that retained owner;
wrapper IDs are not String IDs. The identity slice did not add interning; its follow-up
is recorded below. The internal shared-owner
layout remains provisional. The general Object/value consistency review remains
open; the later socket direction above now selects the next application case.
Do not expand text contracts without a concrete need.

**Socket completion checkpoint, 2026-09-24:** the
[receive ownership probe](experiments/socket-completion/README.md) checks real TCP
with retained destinations, bounded admission, exact range delivery and serialized
completion/cancellation. The six host-side cases now have a
[real-heap TCP follow-up](experiments/socket-completion/GC-OWNERSHIP.md): five added
cases verify collector roots, separate socket/read ownership and bounded ready
callbacks (13 managed-heap cases pass overall). This does not expose a guest Socket API.
The [VM adapter](experiments/socket-completion/VM-INTEGRATION.md) now wires a test-only
TCP source into actual collection safepoints and default TaskQueue dispatch.
Production Socket/Task contracts and the Raven echo application remain pending.

**Author-directed interning exploration, 2026-09-24:** investigate explicit String
interning with a repeated-identifier case before choosing the public contract.
The [owned-pool experiment](experiments/string-interning/README.md) validates exact
content canonicalization, entry/payload quotas, GC retention and pool teardown.
The first development String.Intern now uses an execution-owned pool: repeated host
invocations and isolated workers have separate quotas and pool lifetimes. This fits
existing execution state without introducing a runtime-session abstraction. Explicit
entry/payload limits fault on new insertions at capacity; retained results survive
pool teardown. Automatic literal interning, lookup helpers and shared session pools
remain unselected. This completes the bounded interning follow-up; the active socket direction governs
subsequent API work.

**Author-directed String API slice, 2026-09-24:** support construction from
Sequence<char>, including arrays, and expose String as a read-only Sequence with
explicit Count and public Length/indexer. The [sample](experiments/string-sequence/README.md)
covers immutable copies and grapheme behavior. Compared with .NET's char-array
constructor and UTF-16 indexer, this accepts the platform's sequence abstraction
and grapheme characters, at the cost of traversal/snapshot allocation and scanning
indexes. This bounded addition did not select comparers; the subsequent identity slice is
described above.

**Author-selected end-to-end acceptance case:** Raven record syntax now exercises
class and struct equality/hash/display, assignment, nested components and null default
fields within the documented non-generic component contract. See the
[record sample](experiments/records/README.md) and [HashCode design](hash-code-design.md).
Generic/inherited records, additional component shapes, remaining primitive/calendar
Object contracts and default comparers remain follow-ups. The author also selected
a coherent System.Text API and general/string comparer infrastructure as future work,
using .NET comparison policies as a reference; see the [text design](string-storage-design.md#future-text-and-comparer-direction).
These are not a commitment to culture support or an immediate namespace migration. Generic math is later
exploration. Value removal still requires its own storage migration, not a rename to
Object; String work does not implicitly select that migration.

Planned text work also includes ToUpper/ToLower-style casing and comparison methods.
.NET is a reference, not an API-copy requirement: adapt names and contracts where a
concrete benefit justifies the compatibility cost. Culture selection, Unicode casing
(including length changes), ordering and equality/hash consistency need explicit
choices and tests. Additional casing/comparison APIs remain planned; the existing
bounded ordinal helpers retain their current contract.

When choosing work autonomously, follow the current author-directed focus and the
[immediate next step](#working-rules-and-immediate-next-step). The post-release
concurrency, Storage and file Stream POC has working evidence below; broader
suspension and cancellation remain within M1. The async/Tasks preview has shipped.
The early memory-copy checkpoint has evidence; sockets are now the active API
work and HTTP follows the transport integration.
Use the progression below and its detailed plan for validation.
Later milestone candidates remain provisional: listing them here does not authorize
wholesale implementation of their proposals or freeze their order. Update completion
status with evidence and record significant direction changes in the development timeline.

## How the plans fit together

This is the unified view of platform milestones and their priorities.
The [HTTP POC roadmap](http-poc-roadmap.md) is the detailed M1 work plan, including
urgent experiments, proposal triage and socket/stream acceptance cases. Keep its
implementation details there. [Direction and migration](roadmap.md) preserves dated
directions; the [API plan](runtime-api-plan.md) and [platform backlog](platform-backlog.md)
provide supporting inventories rather than competing execution orders.

Current applications, collections, text, Tasks, file helpers and introspection are
starting assets, not completed milestones below. Their evidence is linked from the
[HTTP baseline](http-poc-roadmap.md#starting-evidence-and-gaps) and feature design
notes. No new sample or runtime capability is implemented by this roadmap.

## Milestones at a glance

| Milestone | Theme | Concrete product / sample | What it should demonstrate | Planning status |
| --- | --- | --- | --- | --- |
| M1 | Communicate | **Hello Service + Hello Client** | Two Raven apps exchange text and JSON on neoCLR using HTTP, sockets, streams and encoding | First major milestone selected by author; detailed scope provisional |
| M2 | Work with data | **File Catalog** | Scan a bounded directory, query entries, write/read a JSON catalog and reuse the same logic with an in-memory source | Candidate after M1 |
| M3 | Handle real waiting and failure | **Download Queue** | Fetch named resources with bounded concurrency, cancellation, deadlines and controlled output to files | Candidate; builds on M1 and the needed part of M2 |
| M4 | Work with human time and presentation | **Activity Report** | Summarize timestamped records with deterministic clocks, explicit time-zone handling and selected cultural formatting | Candidate; can branch from M2 independently of M3 |
| M5 | Explain programs | **Assembly Explorer** | Inspect a sample application's types/members, compare metadata views and report unsupported information clearly | Candidate; a loaded-program version can start independently |
| M6 | Carry the platform across hosts | **Portable Sample Pack** | Run selected earlier products on a second host and report available/missing capabilities consistently | Candidate; target and exact sample set remain open |

M1's memory-copy, encoding, JSON, delayed-read, file-copy and TCP echo programs
are independently useful delivery checkpoints inside M1. Deliver and reassess each
before taking on the next kind of complexity. HTTP is the first major application
destination, not the first API to implement. M2–M6 identify
useful destinations, not a promise to complete all proposals. Progress through them
by the dependency of the next runnable case, not by completing entire API families.

## Release checkpoint — Async and Tasks

Following the author's 2026-09-23 request and subsequent acceptance of the scope,
use [the async preview plan](async-preview-plan.md) as the immediate stabilization
checkpoint. Ship the existing Task/Promise, async/await, composition, producer cancellation, default dispatch and isolated-worker
surface after a fresh evaluator bundle and exact-candidate release gates pass.
HTTP and completion of M1 are not prerequisites. The experimental nonblocking worker
adapter and guest operation cancellation tokens are not part of the supported scope.

This is a release checkpoint within the foundations work, not a replacement for M1
or a declaration that S0 is complete. **Preview 9 was published on 2026-09-23**
at `834028c`; see [release notes](preview-9-release-notes.md) and
[exact-candidate evidence](preview-9-validation.json).
The author also requests a DocFX API reference at `/docs/`. For this release,
provide a useful overview, main async API descriptions and links to tested examples;
complete member coverage is not a release gate. See [documentation maintenance](../api-docs/README.md).
After the release, focus on Streams, Storage and Encoding before networking, as
requested by the author. Reuse memory-backed and file-backed transformation samples;
start Storage with bounded local file operations rather than a full provider model.
Resolve cancellation/lifetime gaps as those cases require. Networking still follows
these foundations, and proposals remain exploratory rather than API specifications.

The [local readiness record](async-preview-readiness.md) now covers a fresh extracted
bundle, full library regeneration, Task editor completion and MSBuild checks.
The final candidate passed editor, migration and six-job source/platform gates.
This checkpoint is complete; resume the foundational Streams/Storage/Encoding cases.
Published release notes stay unchanged; website deployment remains a separate workflow.

## Next-release validation efficiency

The author requests a leaner CI process **for the next release, not during this
release**: isolate platform-specific behavior and stop repeating the full sample
and validation suite on every platform. Follow the [CI efficiency plan](ci-efficiency-plan.md)
before the next release cycle; preserve evidence and targeted platform coverage.
This is a release-engineering follow-up, not a change to the feature sequence.

Before the next release, also perform the author-requested
[Raven example portability pass](ci-efficiency-plan.md#pre-release-raven-example-portability-pass--author-direction-2026-09-23).
First verify shared compiler fixes independently on Raven main, release Raven with
those fixes, and integrate the same fixes into the neoCLR branch. Then compare
original .NET examples and minimal neoCLR ports on pinned builds; investigate crashes,
missing contracts and semantic differences without conflating known shared compiler
issues with neoCLR failures. Record fixes, retests and remaining release dispositions.
This is a later release gate; current Object and record-semantics work continues.

## Post-release concurrency direction — 2026-09-23

The author selects the next implementation checkpoint in this order:

1. Rename `System.Threading` to `System.Concurrency`, keeping `Task` and `Promise`
   in **`System.Tasks`** (explicit author clarification).
2. Add explicit `Thread.Run` and a retained `Thread` with instance `Start()` and an
   awaitable `Task`. Design the general `Task.Run` overload family (completion-only
   and value-producing callbacks, as in .NET) alongside suspension and scheduling;
   do not substitute the existing string-only worker adapter for that contract.
3. Implement the first `System.Storage` File/Directory abstractions from the proposals.
4. Implement the first `System.Streams` capabilities for file access.

**Acceptance product: an application that writes and reads a real disk file.**
The author confirms this is also exploration: investigate the best design consistent
with platform architecture, not mechanical implementation of proposal shapes.
Use the sample to evaluate provider boundaries, stream capabilities, ownership,
typed failures and compatibility with future suspension/scheduling. Record and
revise provisional decisions rather than treating this sequence as an API freeze.
Implement and validate smaller cases along the way; proposals remain design inputs.
The explicit Thread slice is implemented and checked: retained Task before Start,
one-shot Start, construction-queue affinity and native termination before completion.
Evidence: [worker sample](experiments/raven-target/samples/library-workers.rvn),
[eight contract cases](experiments/task-contract/verify_workers.py), default-queue
regressions and Rust worker tests. Task/Promise remain in System.Tasks. The file-resource backend and first directional Raven stream wrappers are now
implemented. [FileInputStream and FileOutputStream](../api-docs/streams.md) provide
blocking, bounded reads/writes, exclusive creation, typed failures and explicit close.
The [provider-bound disk/memory sample](experiments/storage-provider/README.md)
uses both whole-text helpers and the new byte streams. Three-byte caller buffers
and a memory backend restricted to two-byte transfers exercise partial-transfer
loops; the verifier checks actual UTF-8 disk bytes. This supplies the checkpoint's
first disk read/write stream application, not a completed Storage model or async I/O.

**Author clarification, 2026-09-23:** establish working stream reads and writes
before aligning the Storage APIs, including investigating/implementing the Path
value object. That Storage alignment began with an application-owned immutable Path value
object: static Parse returns Result<Path, InvalidPathError>, with a private
constructor and explicit logical grammar. The disk/memory sample now passes Path
values and maps host storage beneath a configured native root. That initial
experiment is now integrated as System.Storage.Path, as described below; its grammar
remains provisional. The author intends future parsing of Unix and Windows formats,
normalized through the Path object; the current logical grammar is not the final
format model. Format selection, drive/UNC roots and normalization/equality rules
remain to be defined. The author defers broader Path operations
and leaves string overloads on Storage APIs open. The author further clarifies
that Path belongs to Storage rather than being imposed system-wide: other APIs may
accept strings, with callers optionally parsing and passing Text. The native metadata
and file-stream APIs retain string parameters; the assistant recommends
convenience overloads that parse and forward, not a separate unchecked path.
The [lookup/identity probes](experiments/storage-provider/README.md#lookup-and-identity-evidence-2026-09-23)
now distinguish metadata observations, provider-bound addresses and open handles,
with a .NET FileInfo comparison. They support typed metadata lookup rather than
opening content to probe existence. The first provider lookup implementation follows below.
The memory provider now shares byte payloads across text helpers and streams, with
cross-API sample checks and bounded multi-file storage. Its replacement policy
retains old payloads for open streams; common identity semantics remain exploratory.
The existing native metadata service now has a typed development wrapper:
System.Storage.Metadata.GetKind(string) returns Result<EntryKind, StorageLookupError>.
The sample providers expose GetFile(Path) and Directory.GetFile(name), preserving
missing versus wrong-kind errors without retaining a stream. These remain blocking
observations, not stable item identity or a finalized provider surface.
Directory now has an application-owned GetFile(Path) overload for nested relative
resolution, rejecting absolute inputs while retaining its provider context. The
string overload still accepts one child name; this exploratory distinction is not
a system-wide restriction on string APIs. Disk/memory contract checks cover both.
**Future direction:** the author expects a similar structured-value versus string
boundary for a Uri class. Evaluate its parsing and relative-reference contracts
with networking. The author subsequently prioritized the [Uri/HTTP addressing slices](uri-design.md), which now supply an initial managed Uri contract.

**Author priority, 2026-09-23:** integrate the working Storage slice as soon as
possible, rather than expanding isolated experiments first. The first integration
moves the validated value into System.Storage.Path while preserving its native
Combine/GetFileName string helpers; the sample now imports the platform type.
The initial integration placed stream capability contracts in System.Streams and
put byte opening on StorageProvider, with optional StorageLookup and concrete item
descriptors. The current model below supersedes that intermediate split: System.IO
owns stream contracts, StorageProvider resolves items, and File opens content.
Directory.FileAt now returns StorageLookupError, replacing the sample's FileReadError.
Native static text behavior is retained, with the development naming migration
described below. All public
members have on-site reference coverage, including a manual WriteAllText entry.
**Author-selected target, 2026-09-23:** align with the
[Storage proposal's object model](proposals/storage-api.md#selected-object-model--2026-09-23).
StorageItem, File and Directory are interfaces. StorageItem has exactly the two
permitted branches File and Directory; providers supply concrete implementations.
StorageProvider resolves paths. GetItem returns one StorageItem; Directory.GetItems
enumerates StorageItem values. The existing descriptor classes and separate
StorageLookup/byte-provider split are intermediate implementation choices, not the goal.

The interface migration is now implemented: StorageItem is closed to File and
Directory in Raven metadata and the strict importer, while provider implementations
of either branch are accepted. The disk/memory sample owns ProviderFile and
ProviderDirectory. Raw neoIL does not enforce that metadata. Native static text
helpers move to FileText in the development Raven API; legacy raw aliases remain.
Provider resolution is now consolidated on StorageProvider: GetItem returns
StorageItem, and GetFile/GetDirectory return their branch interfaces. The temporary
StorageLookup type and platform provider byte methods are removed. Byte routing
belongs to concrete item implementations; the sample keeps that helper private to
its own provider model. FileSystem now integrates the host provider with internal
File/Directory implementations, relative directory traversal and bounded mixed
GetItems snapshots.
**POC scope reaffirmed by the author, 2026-09-23:** deliver a small Storage API that
can demonstrate the abstraction and real file access. The completion evidence is a
runnable, documented app obtaining a directory through the provider contract and
resolving/creating a file, writing bytes and reading them back through streams, with
expected failures visible. Keep the selected StorageItem/File/Directory interface
model; finish the minimal provider integration needed by this product before
expanding metadata, mutation operations or provider breadth. The current disk/memory
sample demonstrates the workflow using the platform FileSystem provider; the memory
provider remains a sample comparison.

The author also suggests StreamReader for this POC and specifies a TextReader
interface. Consumers depend on TextReader; StreamReader implements it as a minimal
provider-independent UTF-8 reader over InputStream to remove byte-buffer/decoding plumbing from the text
consumer; [scope and contract questions](proposals/streams-api.md#minimal-text-reader-for-the-storage-poc--2026-09-23)
cover bounds, partial reads, errors and ownership. The author includes these readers,
a small seekability case and the System.IO grouping in the POC objective, alongside
provider integration and enumeration. Move byte streams and text readers together
to System.IO; keep storage item/provider/path abstractions in System.Storage. These
capabilities are now implemented in the development library; they do not aim at full .NET parity.

**POC evidence, 2026-09-23:** the [standalone Storage app](experiments/storage-poc/README.md)
uses platform FileSystem through StorageProvider, writes UTF-8 bytes to a new File,
reads via TextReader/StreamReader, discovers SeekableStream, rewinds and rereads,
enumerates mixed items and demonstrates AlreadyExists/NotFound. Its verifier checks
exact output and native file bytes. Streams and readers now live in System.IO.
Readers are bounded, strict UTF-8 and explicitly own or leave open their inputs.
FileAt/CreateNew remain the minimal transitional creation model. The
[disk/memory suite](experiments/storage-provider/README.md) supplies complementary
partial-transfer, error, ownership and seek contracts. On-site API documentation
includes a downloadable walkthrough. This meets the requested synchronous Storage
POC product scope; production provider breadth is not implied.

**File consumer follow-up:** the [File Transformer](experiments/file-transformer/README.md)
connects the existing bounded JSON document experiment to Storage and System.IO.
It validates and serializes before exclusive output creation, preserves input and
existing destinations, and documents the possibility of partial output after an I/O
failure. JSON remains application experiment code, not a new platform API.

**Pending-operation evidence:** the [pending-read contract experiment](experiments/pending-read/README.md)
uses real Task/Promise, await and GC with queued guest producer events. Six cases
separate cancellation requests from terminal acknowledgement, permit completion to
win, and reject late writes/releases/notifications. The buffer is private while
pending. Resource release is a counter, not native I/O; no public async signature
or token API is selected.

The [host-backed follow-up](experiments/host-pending-read/README.md) now uses real
isolated workers, notification dispatch, joined producer acknowledgement and Raven
await. Five cases cover deferred cancellation, completion and bounded payload outcomes
with GC and unrelated ready work. Cancellation before delivery discards a completed
result; it does not interrupt the worker. The existing runtime and public API are unchanged.

Experimental raw worker services now give each job an independent cancellation
token and a cancellation-aware terminal join. Registry/direct-IL evidence covers
acknowledgement, sibling isolation and retained notification state; ordinary Raven
Thread/Task/Storage APIs are unchanged. See [worker contracts](isolated-workers.md#per-job-cancellation-experiment-2026-09-23).

The [Raven cancellation adapter](experiments/worker-task-cancellation/README.md)
now maps acknowledged job cancellation to Promise.Cancel. Four real-worker/await
cases preserve cancelled destinations and successful siblings with actual GC; negative
fixtures retain UserFault identity and reject bootstrap service access. Cancellation
submission uses fixture-only wiring, not a new public Thread member.

**Scheduling clarification, 2026-09-23:** TaskQueue is provisional scaffolding.
The author directs adapting the model when requirements arise, including runtime
suspension; neither retaining nor replacing TaskQueue is a commitment. No public
Scheduler API is selected. The [scheduling exploration](task-contracts.md#scheduling-and-suspension-exploration--2026-09-23)
and cross-queue worker fixture document current producer-bound await resumption
versus caller-bound result observation. They characterize a limitation, not a future
affinity guarantee.

**Next direction:** use a concrete pending-operation case to settle continuation
ownership/progress as needed, then address byte payload accounting before introducing
a filesystem producer. Default-only worker notification remains experimental;
explicit-queue worker submission and public cancellation handles remain open.
Keep M1 suspension and cancellation work ahead of networking. Evaluate task-returning provider operations
against that suspension model; do not expand Storage metadata or mutations without
a concrete case. Preserve the working disk/memory consumer.
Task-based interaction is the intended extension direction; its scheduling and
cancellation contract must be established before claiming nonblocking I/O. GetItems'
eager/incremental representation remains open. Rich metadata, topology, Path format
expansion and unneeded mutations remain outside the initial POC.
**Additional information direction, 2026-09-23:** the author expects queryable
attributes/information through the Storage structure while keeping the object model
independent of filesystem or other provider-specific models. Compare System.IO and
WinRT approaches; [metadata query design](proposals/storage-api.md#querying-additional-information)
keeps typed versus extensible representations open. Do not impose a universal native
attribute set. Define availability, failure and freshness when implementing the first
useful query. This does not move metadata ahead of the interface/provider migration.
**API design clarification, 2026-09-23:** learn from WinRT StorageFile/StorageFolder
without importing their breadth. Start with address properties (Path and File.Name),
explicit lookup and byte-stream access. Add metadata properties only for a concrete
consumer need; timestamps, size snapshots, query APIs and enumeration are follow-ups.
[The comparison and next-slice choices](experiments/storage-provider/README.md#minimal-storage-design-winrt-lessons-2026-09-23)
keep temporary whole-text requirements open for simplification. The sample now
derives File.Name from Path and constructs descriptors directly: the redundant
StorageProvider.FileAt factory has been removed. Directory.FileAt remains an address
operation, distinct from GetFile lookup; no metadata properties were added.
The author permits continuing directly into the next slice after validation.

The author additionally directs stable runtime Fault codes, including StackOverflow,
with a distinct UserFault for explicit guest faults and no guest-selectable code.
This bounded diagnostics improvement does not reorder the Storage/Path follow-up.
See [the fault reference](../api-docs/faults.md).
Networking remains later.

Task represents work independently of an OS thread. The current host can submit to
isolated worker threads; a future WebAssembly host might use Web Workers. Explicit
Thread is a target capability and may be unavailable. An optional package named
`System.Concurrency.Threads` remains a packaging possibility, not a namespace choice.
See [concurrency direction](concurrency-direction.md) for .NET comparisons and
current callback/isolation limits. General .NET-style shared-memory callbacks and
arbitrary result types are not implied by the first bounded worker implementation.

## Future networking and web namespaces — consideration, 2026-09-23

The author suggests considering `System.Networking.Sockets` for socket-level APIs
and `System.Web.Http` for HTTP APIs. Record this as a candidate separation for the
later networking slices, not a finalized naming decision or an immediate rename.
Current foundational Object work retains priority.

The proposed boundary separates transport primitives from HTTP requests, responses,
clients and serving contracts. HTTP consumers should not need to manipulate sockets;
a provider could use sockets or a host HTTP facility. This is a design objective to
validate with TCP echo and the HTTP client/server samples, not implemented behavior.
Namespace organization does not itself settle packages, provider availability or
whether client and server APIs share all dependencies. URI placement remains open.

.NET comparison (primary documentation checked 2026-09-23): .NET uses
[System.Net.Sockets](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets?view=net-10.0)
and [System.Net.Http](https://learn.microsoft.com/en-us/dotnet/api/system.net.http?view=net-10.0).
The proposed names make the transport/web distinction explicit, at the cost of
familiar .NET imports. The name System.Web also has an existing
[ASP.NET association in .NET Framework](https://learn.microsoft.com/en-us/dotnet/api/system.web?view=netframework-4.8.1);
using it here would not imply compatibility with that framework. Compare the
proposed split with retaining .NET names or placing HTTP under System.Networking
when the samples establish the API boundaries. Do not infer a web framework scope
from this naming suggestion.

## Minimal HTTP application dependencies — consideration, 2026-09-24

The author expands the future minimal HttpServer application direction with this
candidate namespace map. These are planning boundaries, not implemented APIs or
an instruction to move networking ahead of the current Object/record work.

| Candidate namespace | Role in the future application |
| --- | --- |
| `System.Cryptography` | Cryptographic and certificate support if HTTPS is included; the TLS transport/provider boundary still needs design. |
| `System.Data.Json` | JSON request/response data, independently usable outside HTTP. |
| `System.Networking.Sockets` | Socket transport primitives beneath HTTP. |
| `System.Web.Http` | HTTP serving, client, request and response contracts. |
| `System.Text` | Encoding support; evaluate an Encoding abstraction against the existing bounded Utf8 API. |
| `System.Web` | Possible home for a conceptual WebApplication above the HTTP layer. |

The author also identifies a potentially richer time API, more String methods and
StringBuilder as longer-term needs. Candidate application cases include deadlines,
HTTP date formatting, token parsing and response construction; these examples are
assistant proposals, not selected member contracts. String remains in System;
System.Text is a candidate home for StringBuilder. Reuse existing clocks and text
support first, and select additions through bounded application cases. The initial
cleartext POC still precedes HTTPS, and a WebApplication framework is not required
for its low-level server acceptance case. Shared System.IO streams and System.Tasks
completion remain supporting foundations; the namespace map is not an exhaustive
list of imports every application must use.

**Comparison and tradeoffs.** Primary .NET documentation checked 2026-09-24:
[System.Text.Json](https://learn.microsoft.com/en-us/dotnet/api/system.text.json?view=net-10.0)
places JSON under text, whereas the proposed System.Data.Json emphasizes structured
data. [System.Text](https://learn.microsoft.com/en-us/dotnet/api/system.text?view=net-10.0)
already groups Encoding and StringBuilder in .NET, while
[TimeProvider](https://learn.microsoft.com/en-us/dotnet/api/system.timeprovider?view=net-10.0)
is a clock/time abstraction to compare before inventing new time machinery.
.NET separates TLS streams in
[System.Net.Security.SslStream](https://learn.microsoft.com/en-us/dotnet/api/system.net.security.sslstream?view=net-9.0)
from cryptographic/certificate APIs in System.Security.Cryptography. HTTPS therefore
needs an authenticated transport contract as well as cryptographic capabilities;
the proposed System.Cryptography name alone does not settle that contract.
ASP.NET Core's [WebApplication](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/minimal-apis/webapplication?view=aspnetcore-10.0)
provides an application-level comparison for the conceptual System.Web layer.
These are library/framework boundaries, not evidence that new CLR mechanisms are
needed. The proposed grouping may make application roles clearer, at the cost of
different imports and possible confusion with existing .NET namespace meanings.

Compare retaining .NET names, using this role-based split, and keeping a small
HttpServer API without WebApplication before settling public contracts. Namespace
placement does not decide assemblies or require applications to configure sockets
or cryptography directly. Validate layering with the existing TCP/HTTP slices;
later HTTPS cases must cover authentication failures and ownership, time cases
must separate elapsed time from civil timestamps, and text cases must distinguish
Unicode text from protocol bytes. Broader platform/library comparisons and exact
API design remain open under the design-research process; this records a scenario
and candidate organization, not a completed design review.

## Progressive delivery before networking — revised 2026-09-23

The author asks whether sockets/HTTP should move down in priority so that features
and existing APIs can evolve gradually, while retaining difficult cases when they
teach us something about GC or other runtime needs. The following is the revised
working sequence, not an author endorsement of every API choice. Preserve the HTTP
application destination, but prioritize cheap, informative cases before transport.

| Order | Runnable product / checkpoint | New question to answer | Advance when |
| --- | --- | --- | --- |
| 1 | **Byte Copy** — copy between two bounded memory-backed endpoints | Which buffer ranges, ownership and partial-transfer contracts are actually useful? | Empty/short transfers, bounds, aliasing and error outcomes are explicit; ordinary arrays/interfaces suffice or their concrete gap is recorded |
| 2 | **Text/JSON Transformer** — decode small UTF-8 chunks, inspect/change an explicit JSON field, encode output | Can text and structured data layer on the byte contracts without changing their meaning? | Split UTF-8, malformed input, byte/grapheme distinctions and bounded JSON cases work; no reflection or dynamic machinery required |
| 3 | **Delayed Copy** — the same operation with controlled external completion | What must stay alive while work is pending, and when is it safe to cancel/release it? | Actual guest GC retains the destination and continuation; cancellation and teardown have tested ownership rules; ordinary callbacks progress. Use the existing host probe as partial evidence |
| 4 | **File Transformer** — reuse the pipeline for a bounded file input/output | Does a second backend reveal a weak abstraction or hidden resource assumption? | File opening, stream cleanup and failed output behavior are documented and tested; no provider hierarchy or directory catalog is required |
| 5 | **TCP Echo** — reuse the byte contracts over a connection | Which additional demands come from OS readiness, peer failure and backpressure? | Pending read/accept, short transfers and shutdown work without blocking unrelated progress |
| 6 | **Hello Service + Hello Client** | Do the pieces compose into the intended HTTP application? | M1's independent-peer, protocol, resource and reproduction checks pass |

These are a default learning order, not a demand to finish every earlier API family.
A missing primitive may be explored earlier when a small case demonstrates why it
is needed. The delayed-copy case deliberately introduces suspension/GC before real
networking: complexity is justified by the question it answers, not by proximity to
HTTP. File transformation borrows a narrow slice of M2; its directory/catalog work
remains later. File I/O can begin with an honest synchronous backend; it must not be
labelled nonblocking merely because a Task wrapper is added.

After each checkpoint, review the previous APIs against the new consumer. Change
provisional contracts, update earlier samples and record migration effects together.
Keep both backends in validation so a convenient change for one does not break the
other. Do not freeze an API early just because the first sample passed. In particular,
compare .NET MemoryStream/Stream and existing neoCLR arrays before choosing a new
buffer family; compare Encoding/Decoder and explicit JSON readers before adding
serialization infrastructure. Reuse [stream research](stream-design.md) and
[the M1 comparisons](http-poc-roadmap.md#evidence-comparisons-and-costs).

## M1 — Communicate: Hello Service and Hello Client

**Product:** a small server with a text route and a JSON echo route, plus a client
that calls both and displays the decoded results. Both run on neoCLR. This is the
first proof that the runtime can host an application rather than only isolated API
examples.

**Build up through:** byte copy in memory; UTF-8/JSON transformation; controlled
delayed copy with guest GC; a bounded file transformer; TCP echo; standalone HTTP
server; standalone HTTP client; combined application pair. The [detailed sequence](http-poc-roadmap.md#small-executable-slices)
contains dependencies and negative cases.

**Proposal inputs:** networking, streams, Task, strings/encoding, with only the
necessary collection and runtime-host support. The small JSON API is an explicit
author requirement rather than a supplied standalone proposal.

**Finish when:** the pair works on the bounded documented protocol subset, each
side also interoperates with an independent peer, cancellation and shutdown release
resources, and a matching packaged toolchain reproduces the checked output.

**Compare with .NET:** Socket/TcpListener, Stream, Encoding/Decoder, HttpClient,
HttpListener and System.Text.Json provide the functional baselines. Test directional
streams and Task/Result against those roles; they may improve explicitness but cost
adapters and different error handling. Reuse the [primary-source comparison](http-poc-roadmap.md#evidence-comparisons-and-costs).
Do not require new suspension machinery, intersection types or reflective JSON to
prove the application. Proposed loopback scope and API names are still open to evidence.

## M2 — Work with data: File Catalog

**Product:** a command-line tool accepts a root directory, lists supported file
metadata, filters/orders entries and writes a bounded JSON catalog. A second command
reads the catalog and produces a short report. Begin with names and sizes; add
content inspection only when it gives the sample a useful additional operation.

**Build up through:** file-to-memory copy using M1 streams; shallow directory listing;
filter a fixed set of entries; JSON catalog round trip; inject an in-memory source
into the same catalog logic. Recursive traversal is a later bounded increment with
explicit limits and a deliberate symbolic-link policy.

**Proposal inputs:** [storage](proposals/storage-api.md), [async storage](proposals/storage-api-extensions.md),
[collections](proposals/collections-api.md), [environment](proposals/environment-api.md),
streams and text. This is the second concrete backend for stream contracts and the
first useful test of contextual storage access.

**Explore:** path values versus resolved handles; explicit provider injection versus
contextual convenience; iterator cleanup; missing versus inaccessible entries; save
failure and overwrite policy. A local host source plus a small in-memory implementation
is enough to test substitutability. It does not select a provider hierarchy or require
archive storage, every collection family, or a general ShellEnvironment first.

**Finish when:** fixture catalogs are deterministic, malformed input and access errors
produce documented outcomes, interrupted writes follow the declared save policy,
handles are released and the same catalog operation works with both sources. State
whether saves are atomic; do not imply durability from a successful stream write.

**Compare with .NET:** File/Directory/Path/FileStream and collection/query APIs are
the baseline. Injected capabilities can make tests and host restrictions clearer,
but introduce resolution and ownership contracts absent from simple static calls.
Reuse [filesystem comparisons](filesystem-design.md) and [collection review](collection-contracts.md).
Keep path and provider policy in libraries/host adapters unless runtime enforcement
is actually needed; a provider interface alone is not a sandbox.

## M3 — Handle real waiting and failure: Download Queue

**Product:** a terminal program reads a small manifest, downloads resources to a
chosen directory, reports progress and permits cancellation. Start against a
controlled local server with delayed, failed and truncated responses, then add a
selected HTTPS scenario before describing the sample as an internet downloader.

**Build up through:** one response streamed to a file; two queued downloads with a
concurrency limit; a cancelled pending read; a deadline; deterministic failure/retry
cases; DNS and TLS integration. Reuse the M1 client and M2 file stream rather than
create a separate networking or storage stack.

**Proposal inputs:** networking, Task, streams, storage extensions, date/time and
runtime architecture. This is the motivating product for backpressure, deadlines,
resource budgets and controlled concurrent work.

**Explore:** explicit tokens versus context-based cancellation, group lifetime,
monotonic deadlines, retry policy and connection reuse. Cancellation requested is
not cancellation completed. Keep retries bounded and select eligible operations;
do not turn an arbitrary failure into an automatic repeated side effect. Compare
host event progress with a bounded worker implementation under the same workload.

**Finish when:** slow connections do not halt unrelated work; buffer, task and handle
counts stay within declared limits; cancellation stops pending work and leaves output
in a documented state; certificate/hostname failures are rejected in the HTTPS case;
shutdown is bounded. Record throughput and allocation measurements before claiming
an improvement. HTTP/2/3, an application hosting framework and universal structured
concurrency remain optional follow-ups.

**Compare with .NET:** HttpClient's streaming/lifetime choices, Task cancellation,
Stream and TimeProvider are reference roles. neoCLR's explicit outcomes may clarify
failure flow but require different cancellation/cleanup contracts. Use [M1 evidence](http-poc-roadmap.md#evidence-comparisons-and-costs),
[Task contracts](task-contracts.md) and [time design](date-time-design.md); pin backend
versions and complete TLS/cancellation research when selecting implementation details.

## M4 — Human time and presentation: Activity Report

**Product:** a console report reads a small timestamped activity log, filters a time
window and groups entries for display. Keep the stored JSON invariant; make display
culture and time zone explicit inputs. This can later consume File Catalog or server
activity, but a fixed input file is enough for the first case.

**Build up through:** invariant timestamp round trip; a fixed clock; elapsed-duration
summary; grouping by local calendar date in a selected zone; formatting the same
values for two selected cultures. Add a scheduling example only if date-to-instant
conversion needs a clearer consumer.

**Proposal inputs:** [date/time](proposals/datetime-api.md), [globalization](proposals/globalization-api.md),
strings, collections and contextual environment. These proposals should converge
through the sample rather than impose their entire type catalogs.

**Explore:** instant versus local date/time, elapsed versus calendar arithmetic,
ambiguous/missing local times, explicit versus contextual culture and the source and
version of cultural/time-zone data. Normalization, collation and alternate calendars
need their own use cases; displaying two cultures does not imply complete support.

**Finish when:** fixed inputs, clock and data versions give reproducible results;
daylight-transition cases have explicit outcomes; unknown culture/zone data is
reported; changing display culture does not change stored values or wire parsing.

**Compare with .NET:** DateOnly/TimeOnly, DateTimeOffset, TimeSpan, TimeZoneInfo,
TimeProvider and CultureInfo distinguish the relevant roles. More explicit types
or immutable context could reduce accidental mixing but add conversions, data
shipping and portability costs. Start from [time comparisons](date-time-design.md)
and [globalization design](globalization-design.md); complete the existing broader
[Noda Time research direction](design-research.md#broader-api-review-scope-2026-09-13)
before settling new contracts. No calendar redesign is required merely to format a report.

## M5 — Explain programs: Assembly Explorer

**Product:** a developer tool prints a structured inventory of a sample application's
types, members and signatures and can write it as JSON. Start by inspecting the loaded
program using current introspection. Add an offline input mode as a separate extension,
then compare the two views on a shared supported subset.

**Build up through:** list loaded declarations; filter/query them; stable report;
read a small artifact without executing it; compare supported signatures; diagnose
unknown metadata. This gives introspection and metadata proposals a consumer before
redesigning the complete descriptor model.

**Proposal inputs:** both [introspection](proposals/introspection-and-reflection-api.md)
and [capability-model](proposals/introspection-model.md) texts, [metadata](proposals/metadata-format.md),
collections and runtime architecture.

**Author clarification, 2026-09-24:** ensure the six TypeInfo classification flags
(IsAbstract, IsOpen, IsClosedHierarchy, nominal IsUnion, IsEnum, IsValueType) are
available now; evolve toward the proposal's richer closed interface hierarchy
incrementally. See the [classification contract](introspection-design.md#type-classification-flags--development-2026-09-24).
This bounded API work does not reprioritize the active product milestones.

**Explore:** descriptor identity and lifetime, nominal versus compound type views,
missing metadata versus unsupported operations, and compatibility of the container
with conventional readers. Keep description separate from invocation. Dynamic loading,
reflection execution and Emit are separate experiments, not implicit explorer features.

**Finish when:** loaded and offline reports agree for the selected subset, offline
inspection does not run the inspected program, malformed/unsupported metadata is
handled explicitly and golden artifacts document compatibility limits. The initial
loaded-only product may ship as a checkpoint without claiming offline capability.

**Compare with .NET/CLR:** Type/TypeInfo and System.Reflection.Metadata/ECMA-335 are
the reference layers. A shared descriptive model could reduce duplicated tooling,
but context identity and unavailable capabilities become explicit obligations.
Reuse [introspection evidence](introspection-design.md) and the [model review](reflection-model-review.md).
A new metadata representation must earn its cost against the existing CLI boundary.

## M6 — Across hosts: Portable Sample Pack

**Product:** a reproducible bundle of selected earlier samples, plus a runner that
records outputs, declared capabilities and unsupported cases on two selected hosts.
Begin with another supported desktop environment; selecting a constrained target is
a separate decision, not a requirement to build a microcontroller port.

**Build up through:** inventory sample dependencies; package a matching toolchain;
run memory/text/JSON cases; run file and HTTP cases on a second host; exercise an
explicitly unavailable host service. Earlier milestones still need ordinary packaging
and honest platform limits; this milestone deepens portability rather than postponing it.

**Proposal inputs:** [runtime architecture](proposals/runtime-architecture.md),
environment, storage, networking and metadata. Reuse the same public sample code
where its capabilities exist; expose host-specific limitations rather than silently
substituting a different contract.

**Finish when:** the supported cases produce equivalent documented outcomes, missing
capabilities have deliberate diagnostics, resource limits are recorded and a clean
checkout or package can reproduce both runs. OS-dependent path, clock and socket
behavior must be accounted for. This is not a claim of universal binary portability.

**Compare with .NET/CLR:** common API contracts, platform-specific services and
runtime deployment are separate concerns. Smaller explicit capability sets may aid
embedding but create a conformance matrix and packaging burden. Use [capability planning](runtime-api-plan.md#common-platform-contract-and-target-capabilities-2026-09-14),
[target profiles](raven-target-profiles.md) and [execution architecture](execution-architecture.md).
JIT, AOT and compiler self-hosting are not prerequisites; evaluate them only against
a recorded deployment or performance problem in this sample pack.

## Research products alongside the milestones

These are bounded experiments, not additional mandatory milestones. Their deliverable
is a runnable comparison and a decision record; “the existing mechanism is adequate”
is a successful result. Run them when a product exposes the question, keeping them
off M1's critical path unless it cannot proceed safely without the result.

| Research product | Proposal inputs and motivating case | Comparison and decision evidence |
| --- | --- | --- |
| **Capability Composition Lab** | Runtime unions/intersections and generic relationships; pass a duplex connection to code that requires read plus write | Compare CLI nominal interfaces/constraints and existing Result/Option with proposed type expressions. Show a concrete ergonomic or enforcement gain, account for signatures, dispatch, GC and tooling, or retain nominal composition |
| **Dynamic Record Adapter** | Dynamic dispatch; access fields from a small external record in a typed consumer | Compare explicit JSON lookup, typed adapters and .NET-style dynamic binding. Test missing members, type mismatch, access control and cache invalidation; prefer a library solution if runtime hooks add no needed guarantee |
| **Metadata Compatibility Probe** | Metadata format and introspection; read ordinary and proposed extended signatures | Compare existing CLI artifacts with a reduced extension. Record what conventional readers can enumerate versus understand; do not generalize a successful container read to semantic compatibility |
| **Execution Cost Probe** | Runtime architecture and Task; rerun an earlier bounded workload | Compare current state machines/interpreter with one reduced suspension or execution alternative only when justified. Measure correctness, GC/lifetimes, diagnostics, allocation and execution cost on recorded versions; no new backend is selected in advance |

These experiments cover the wider proposals without making type-system redesign,
dynamic dispatch or new execution backends a price of admission for useful apps.
Nullability and other existing backlog contracts should likewise be promoted when
a concrete sample reveals a problem, with direct-IL validation where runtime guarantees
are proposed. A language-only check is not evidence of runtime enforcement.

## Active progress — 2026-09-23

M1 remains active. Its first [host-side S0 experiment](experiments/external-io-progress/README.md)
passes seven ownership/progress/cancellation checks. This is a reduced Rust model,
not a guest networking API or completed S0. A [real-heap ownership follow-up](experiments/external-io-progress/GC-OWNERSHIP.md)
now verifies pending and ready roots, receiver graphs, reclamation and terminal
cleanup in eight test-only cases. A [VM Delayed Copy checkpoint](experiments/delayed-copy/README.md)
now retains real invocation roots, posts ready worker notifications to TaskQueue and
resumes a Raven consumer through GC pressure. Its worker-library adapter is isolated;
normal worker APIs still use queued joins. A self-reposting callback sample now
verifies cooperative completion progress without queue quiescence. S0 remains partial:
queue affinity, operation cancellation races and bounded native I/O ownership remain open.
S1 now has a [runnable Byte Copy checkpoint](experiments/byte-copy/README.md):
checked ranges, overlapping copies and short reads/writes over ordinary managed
arrays, with synchronous GC-retention checks. It is an application-local experiment,
not a completed general stream API. S2 now has a [UTF-8 chunk consumer](experiments/utf8-chunks/README.md)
that reuses those primitives, preserves incomplete scalars across reads and compares
strict decoding with .NET. S3 now has a [JSON document consumer](experiments/json-document/README.md)
that reads a sensor report and constructs an acknowledgement. It covers objects,
arrays, strings, booleans, null and preserved number text with checked conversions,
plus explicit duplicate-key and resource limits. The public JSON API remains open.
These experiments do not select a new storage representation;
the host probe does not make networking urgent.
No later major milestone has started.

## Working rules and immediate next step

The [pending transfer checkpoint](socket-api-design.md#pending-transfer-deadline--2026-09-24)
now returns TimedOut after five seconds per nonempty Send/Receive, releasing buffers
without closing the caller's socket. HTTP closes its owned connection on that error.
The [HTTP socket-handler budget](http-client-design.md#socket-backed-http-exchange-budget--2026-09-24)
now carries a 15-second absolute deadline across lookup, connect and repeated transfers.
Public Socket operations keep their phase bounds. Request construction, custom pipeline
work outside transport, server accept and server application handlers remain outside
that budget. A configurable general timeout/token API is not yet selected.

**Active next step, 2026-09-24 (author-directed):** complete union migrations in a
batch before resuming HttpError and HttpClient.BaseUri. Uri is implemented.
SocketError is now the first public error migrated to normal Raven union syntax:
consumer/bootstrap references compile embedded source, native callers use generated
case matching, and nested HttpError probes pass copying/boxing/GC checks. Real TCP,
managed listener/client and selected HTTP cases pass. Rebuild matching SDK/library/apps;
SocketError's Is*/Get* helpers are removed and its default is inactive. The next
empty-case group migrates DnsError and UriError through the same source projection,
with generated case patterns and inactive defaults. All 23 case names, nested
managed payloads, copies/boxing/defaults, URI grammar/resolution and DNS/TCP pass; payload-bearing explicit
layout is separately tested as an unsupported boundary.

Per-case Is* properties are not a required convention, including for Option/Result.
The remaining empty-case errors are migrated as one group, with EntryKind changed
to an enum following author direction (File = 1, Directory = 2). Their integrated
case/default/boxing checks pass. Generic Option/Result/TaskOutcome still require
companion/generic reference projection and payload-body import, plus propagation
and task-contract validation; their projection is explicitly rejected today.

**Enum helper slice:** System.Enum now has names/values helpers retaining TypeInfo
arguments and adding constrained enum type-parameter overloads. Shared metadata
orders names/values and formats boxed enums. The [SDK probe](experiments/enum-helpers/README.md)
checks typed/discovered results, non-enum rejection and retained snapshots under GC.
The historical Neo bootstrap retains explicit legacy carrier/enum snapshots; Raven
uses migrated unions without mandatory per-case predicates.

**Author direction, 2026-09-25:** after finishing this slice, put other additions on
hold, close out applicable union migrations, and resume HTTP. MinValue/MaxValue,
Boolean.FalseString/TrueString and further Enum flags helpers are recorded follow-ups,
not prerequisites for HTTP. Grapheme Char should not receive UTF-16 bounds. The future Error interface
for diagnostics, composition and decoration remains a proposal, without current
stack capture or wrapper contracts. Raven case/companion metadata remains a provisional bridge concern,
not a runtime dependency or a standardized platform case map. The [nongeneric payload-library probe](experiments/http-error-unions/README.md#nongeneric-payload-bearing-library-projection)
now projects a sequential HttpError prototype, compiles a separate consumer and
executes imported payload bodies under GC pressure. Generic companion projection
and nonempty overlapping layouts remain rejected. The migration batch is closed
for applicable nongeneric families. Typed HTTP errors are now integrated into the
client/handler/server contracts, with public consumer, independent-peer,
framing/timeout, payload-GC and signature rejection checks passing.
The [IPAddress slice](ip-address-design.md) now follows this with a closed class
hierarchy, typed DNS results and Socket overloads. Cancellation foundations and
string/Uri overloads with BaseUri are implemented; HTTP token wiring remains next.
Do not extend the manual carrier catalog.
The author's 2026-09-25 HTTP contract makes
`Send(HttpRequest, CancellationToken) -> Task<Result<HttpResponse, HttpError>>` the
client primitive; Get, later verb helpers and GetString layer on it through the
same handler pipeline. Token propagation and operation cleanup are part of this
integration target; public CancellationToken foundations now exist, but HTTP/socket
token wiring is not implemented yet. See the
[contract and validation gates](http-client-design.md#core-request-contract--author-direction-2026-09-25).
Server cancellation ownership and public JSON contracts remain later work. The [JSON report](experiments/http-json/README.md) now passes between two
neoCLR applications and independent peers; it does not promote a public JSON API. The
[HTTP sample](experiments/http-client/README.md) imports development System.Web.Http
client, request/response/content and handler APIs; the bounded GET/200 client is the
client checkpoint. The [server POC](experiments/http-server/README.md) adds bounded
request parsing, received headers and an owned response exchange. These are not yet
the completed web application milestone.
[Address fallback](socket-api-design.md#address-sequence-connection-poc--2026-09-24)
now accepts a Sequence of IPv4 addresses with a shared five-second connection budget,
per-address progress limits and one owned native connection. DNS still has a separate
five-second deadline. The HTTP case must establish header/body bounds and phase
limits; the socket handler now carries a shared 15-second exchange budget while preserving shorter phase limits. The [two-process echo](experiments/socket-echo/README.md)
now uses Listen/Accept on the server and the hostname client on the other side. The [hostname client](experiments/socket-client/README.md) now uses public
Dns.GetHostAddresses with Task/Result and a read-only IPv4 address sequence over the
[bounded resolver](socket-api-design.md#public-hostname-lookup-and-networking-poc--2026-09-24).
Immediate author direction (2026-09-24): complete Uri, HttpError and
HttpClient.BaseUri, retaining string and Uri request overloads. The 2026-09-25 Send direction adds client token propagation to this scope, without
reprioritizing broader scheduler or runtime-suspension exploration. The [managed Uri slice](uri-design.md) adds
bounded ASCII parsing, RFC reference resolution and lexical value semantics; typed
HTTP errors and BaseUri request integration follow. The author clarifies on
2026-09-25 that BaseUri is an optional string (Option<string>): when configured,
verb methods expect relative URLs and resolve them against that base; otherwise,
they take absolute URIs. Keep Uri parsing internal to validation/resolution and
retain string/Uri address overloads. Parameterless HttpClient already
selects the socket handler. The 2026-09-25 release direction now includes an
IPAddress closed class hierarchy with IPv4Address/IPv6Address implementations
(superseding the initial union selection); HostEntry remains a later
networking evaluation. Author-directed
later work also includes URI/URL encoding utilities, with component-specific
contracts evaluated separately from the current parser.
Bounded address fallback and its shared connection deadline now have an executable
echo case; preserve these bounds when building the first HTTP client. Networking has its own feature page and homepage box;
the integrated client now also has its own Web page and homepage box.
The [public TCP client](experiments/socket-client/README.md) now has
Connect/Send/Receive/Close and a Task/Result bridge over nonblocking transfers.
Its compiled sample sends bytes and receives a host echo with collection while pending. Keep exact addressing/lifecycle choices provisional and maintain the API
reference with each addition. Later evaluate TcpClient (or TcpConnection) with stream
adaptation and UdpClient when a datagram case needs it; these convenience layers are
not prerequisites for the first Socket case. Preserve current affinity. Generated
state machines remain the execution mechanism; runtime suspension and general
context customization are not prerequisites for TCP echo. The author clarifies that
the product is a web-app demo receiving and sending HTTP messages: follow echo with
the minimal provisional stream/text/HTTP chain, not completion of every layer.
The [HTTP prototype essentials](socket-api-design.md#http-prototype-essentials-and-dns--2026-09-24)
now include near-term host-backed DNS resolution after send, independently of listener
completion. Build HTTP directly over Socket behind a private byte-I/O boundary;
TcpClient is not required. Concrete products are socket echo, hostname byte exchange,
a bounded HTTP GET/POST client, and the web-app request/response demo. Include request,
response and header models, correct framing, bounds and deadline/error behavior.
Start with a declared local HTTP subset; HTTPS requires a later TLS slice.
The author also calls out character/string encoding: keep Socket byte-oriented,
then exercise explicit text-to-byte and byte-to-text conversion in the HTTP path,
including multibyte characters split across transfers. Reuse existing UTF-8 work;
select any broader encoding API from that concrete case rather than adding it now.
Earlier foundation checkpoints below are dated evidence, not competing priorities.

Each selected slice should leave a checked sample, expected output, a matching build/run
path, failure cases and a short decision record. Record the .NET baseline, alternatives,
selected layer (language, library, metadata, runtime or host), benefits, costs and open
questions. Existing research can support planning; deepen primary-source and independent
comparisons before adopting a substantive new contract. Proposals are not proof that
a feature compiles, runs or improves on .NET.

Update this roadmap's status only with linked evidence. Keep detailed API progress in
its feature plan, and update changelog, relevant feature pages and integration docs.
Samples begin as small programs, not miniature frameworks. Existing release/debugging
requirements and Raven branch/integration rules continue to apply.

**Current checkpoint, 2026-09-23:** Preview 9 is released. The synchronous disk/memory
Storage POC and Console stream slice have executable evidence. The author requests a
consistency review before expanding the platform: keep, adapt, redesign or remove
provisional choices based on concrete use and comparison with .NET.

The library composition repair is complete before the author-selected Object/Value
review. StorageProvider belongs to the composed Raven library alongside StorageItem,
File and Directory; the legacy bootstrap library keeps its static File helpers.
The repair restores the existing Console regression suite without changing public APIs.
The [Object/Value review](object-model-review.md) now records those dependencies and
missing members. The bounded class display/override case now preserves
.NET-style class reference sharing and value copying. Class equality/hash and the first integer record-class gate now have checked evidence.
The record sample now also covers non-null strings and nested same-compilation record
classes, including nullable record references and deconstruction. Broader component
support and boxed-value behavior remain bounded; nullable value types are deferred.
Boxed Int32 equality/hash now has a bounded intrinsic and sample. Named value-type Object overrides, exact copied unboxing and the first Raven record-struct
slice now have [checked evidence](object-model-review.md#struct-object-slots-and-record-structs--2026-09-24).
The author-directed struct/record-struct gate is complete for non-generic types with
the documented component contract. Nested same-compilation record structs now compose in both record classes and structs,
with a checked Point/Rectangle sample and value-output deconstruction. Boxed Boolean
now joins Int32 for exact-type Object equality and hashing, with a checked flag sample.
Boxed Int64 now also has full-width exact-type equality and a .NET-compatible hash;
the [Int64 sample](experiments/int64-object/README.md) checks colliding Object map keys
and collection. Boxed Int32/Int64/Boolean now also provide bounded Object display:
culture-independent decimal integers and True/False. Boxed Single/Double now have
exact-type Object equality and hashes consistent for NaNs and signed zero, with a
[floating map sample](experiments/floating-object/README.md). Floating display remains open. Boxed Char now aligns exact grapheme equality/hash
and display with typed Char; the [sample](experiments/char-object/README.md) checks
combining text, emoji and Object map keys through GC. String Object content equality,
hash and display now work through existing wrappers, with a [map sample](experiments/string-object/README.md);
String reference identity remains deferred.
Other primitive Object implementations and generic struct components
remain follow-ups.
Nullable value types and their boxing behavior are not part of the current scope. Default reference fields now have a checked record case:
null is preserved even for non-nullable declarations, and generated equality/hash/display
handle it. Object.Equals and ReferenceEquals now annotate nullable comparison
arguments for Raven compatibility. The author keeps reference annotations for now
while leaving the future nullability model and metadata format open. Explicit
nullable-string components remain separate work. Generated Object.Equals preserves
the inherited nullable parameter contract, and typed record-class Equals now accepts
a nullable record reference. Typed record-struct parameters remain values; the checked
sample covers absent/present Key? locals and literal null. Generated class `==`/`!=`
also accept nullable references, with symmetric null handling and value equality.
Internal null guards use reference identity, independent of overloaded operators.
Value retirement remains a separate storage migration.

**Library consistency direction, 2026-09-24:** the author requests applying Object
contracts throughout the runtime class library. The [library audit](object-model-review.md#library-wide-consistency-checkpoint--2026-09-24)
identified Path typed/Object equality, hash and display alignment as the next library
consumer case. That slice now passes through existing HashMap callbacks. The
[wider audit](object-model-review.md#path-integration-and-wider-library-audit--2026-09-24)
confirmed the RuntimeTypeInfo mismatch. The author-directed introspection slice
now aligns represented-type equality/hash/display, with GC-tested map reuse.
AssemblyInfo/ModuleInfo now use scoped catalog equality/hash/display. Field, method
and property equality includes descriptor kind, closed declaring owner and definition
index. Parameter snapshots now retain that owner key plus position, with equality/hash
and Name display. Public owner resolution remains future work. The Object-keyed
HashMap importer gap is now closed, with mixed keys, Object payloads and GC-tested
map growth using explicit callbacks. Keep
Equatable<T>.Equals(T) explicit;
nullable value operands are not introduced by equality contracts. Reflected context,
inherited queries and a public parameter Member property remain design work. Broader primitive,
formatting and default-comparer coverage remains incremental; do not imply universal
Object support from the completed record gate. The author also requires correct GC
behavior for boxing: verify copied payloads, aliases, reference-field tracing, roots
and collection pressure alongside observable equality/hash/display behavior.

**Absence-model direction, 2026-09-24:** prefer Option<T> for intentional absence in
API/domain models, for both value types and reference types. Retain nullable reference
annotations for Raven compatibility and APIs whose contract already involves null,
such as Object equality. Do not implement Nullable<T>/nullable structs or nullable-value
boxing in the current work; reconsider only when a concrete future need warrants it.
This does not promise Option components in generated records before they are supported,
remove runtime reference nulls, or settle the future metadata representation.
The author additionally requests an opt-in Raven diagnostic for nullable value
declarations. The development target now selects RavenAllowNullableValueTypes=false:
RAV0407 rejects known nullable value declarations while allowing reference annotations.
Raven's ordinary .NET default remains unchanged. See the
[record sample and rejection checks](experiments/records/README.md).

**Earlier author-directed investigation, 2026-09-24:** use the now-supported value
semantics to establish efficient async state-machine ownership. Raven already emits
struct states by default; neoCLR still explicitly selects classes. Prioritize the
[value-state-machine gates](async-state-machine-assessment.md#value-state-machine-priority--2026-09-24)
ahead of Path: by-reference startup, one retained state across suspension, GC/root
checks, then measured Release comparison. The initial probe exposed by-value boxing and an import rejection. The development
ref protocol now starts value states in place and retains one state across pending
awaits. Ready/pending/cancelled, unit and Result cases run under GC pressure; managed
object counts show one saved state for ready completion and pending allocation parity.
Keep the working heap default while broader shapes and byte/copy costs remain unmeasured.
Generated-code builder APIs are explicitly transitional and may be removed when
runtime-owned suspension replaces them; this work supports building the platform now.

This was a bounded foundation review; the later socket direction above now governs
next work. Console ownership,
cleanup on propagated errors and buffering remain follow-up questions, not selected
redesigns. The scheduling/operation-cancellation work below remains open.

[Host invocation cancellation orderings](cancellation.md#worker-completion-boundaries--development)
now cover ready results before/after notification dispatch, interrupted output delivery
and producer acknowledgement during teardown. This does not implement guest operation
cancellation. Broader host-memory accounting remains open. A [worker payload quota](isolated-workers.md#completion-payload-quota)
now bounds successful retained text/output (1 MiB per worker by default), excluding
temporaries, diagnostics and allocation overhead. Cooperative progress under
self-reposting ready work is now checked; broader scheduling policies remain open.
Keep the experimental worker adapter provisional until the operation cancellation
contract is checked, before reusing the pipeline for files and then sockets. Keep the
application-enum and protected-constructor importer limitations exposed by the JSON experiment as explicit follow-up probes; they do
not require a metadata redesign or block this next lifetime checkpoint. Reassess
priorities at each checkpoint and M2 onward after the first major HTTP application
milestone.


### Later exploration: generic math interfaces — 2026-09-24

The author suggests possible generic math support later; no implementation or release
commitment is made. Compare .NET's [generic math interfaces](https://learn.microsoft.com/en-us/dotnet/standard/generics/math),
including narrow operator/identity contracts and broader numeric hierarchies. A future
slice should establish Raven static interface-member and runtime dispatch support,
conversion/overflow policy, and floating special-value behavior before choosing a
surface. Start with a comprehensible generic sum sample across integer and floating
types; compare interface-based dispatch with today's concrete overloads. Interfaces
could reduce overload duplication but add compiler/runtime and contract complexity.
This remains after current primitive Object consistency work and does not reprioritize
the active milestone.

### HTTP status names and pattern contracts — discussion, 2026-09-25

The author asks about an HttpStatusCode enum; the proposed next bounded follow-up
is named statuses with unknown numeric values preserved. The author also suggests
Deconstruct on HttpResponse and HttpRequest for Raven pattern-based inspection.
Evaluate those shapes against the request/content sample in slice 5; no positional
contract is committed yet. See [the status design](http-client-design.md#follow-up-httpstatuscode-enum).
