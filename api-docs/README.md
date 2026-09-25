# API reference maintenance

RavenDoc publishes this reference inside the single neoCLR website at `/docs/`.
It reads a checked-in compiler reference assembly and the authored XML sidecar,
then renders Raven signatures with the same layout, navigation and development
notice as the Markdown guides. No DocFX build, metadata YAML or second site exists.

The site documents development after Preview 9 and may be published before a
runtime release. Generated reference pages are development documentation; per-page
release labels and published sample/toolchain versions remain authoritative.

## Build and refresh

```sh
python3 scripts/build-website.py
python3 -m unittest discover -s scripts -p 'test_build_website.py'
```

Normal CI requires only Python and .NET 10, using the pinned portable
[RavenDoc build](../tools/ravendoc/README.md). To refresh after a public API change,
first build the matching Raven-target bridge following its README, then:

```sh
mkdir -p target/api-docs/input
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-core target/api-docs/input/NeoCLR.CoreProbe.dll
python3 scripts/build-api-docs.py \
  --refresh target/api-docs/input/NeoCLR.CoreProbe.dll --check
python3 scripts/build-website.py
```

Commit `reference/NeoCLR.CoreProbe.dll`, `snapshot.json`, `types.json` and relevant
XML/Markdown changes together. The assembly is a small documentation/compiler
reference, not executable runtime code. The build copies `NeoCLR.CoreProbe.xml`
beside it locally; that generated copy is ignored. Source fingerprints and the
assembly SHA-256 reject stale/corrupt snapshots. A fingerprint is not proof that an
arbitrary input DLL was produced by those sources: always regenerate from the
freshly built matching bridge. CI never requires that compiler checkout.

## Public API coverage

RavenDoc generates every public type, even without XML documentation. `types.json`
is an automatically refreshed inventory of all externally visible types in the
reference assembly, not a curated allowlist. `tools/api-inventory` uses .NET metadata
reading without loading or executing the reference. Snapshot checks reject an
inventory that omits a public type or includes an internal type.

Provide useful type/member summaries, parameters, results, ownership and limitations
as APIs develop. Missing prose is reported but does not block page generation.
Missing public type pages still fail validation. Authored XML entries must resolve
to real member pages. Documentation is maintained in the XML sidecar, not claimed
to be extracted from Raven source comments.

`manual-types.json` records the exact pinned-renderer limitations and manual routes:
the two namespace-function containers and IsReadOnlyAttribute. These remain part of
the public inventory, with linked manual entries and checked routes. No public type
is excluded from the inventory. `exclusions.json` records intentional page exclusions
with reasons: non-generic Array, Option, Result and TaskOutcome are importer/exporter
scaffolds, not additional application types. Their generic APIs remain documented;
CLR case-carrier types inside the non-generic containers are also explicitly
excluded. Coverage checks honor these exact type exclusions.
Keep [reference support](reference-support.md) aligned
when those renderer limitations change.

RavenDoc now renders the previously excluded TaskQueue Post/Run, Task OnCompleted,
ITaskAwaiter OnCompleted, FileText WriteAllText and OutputStream/FileOutputStream/
TextWriter/StreamWriter Flush signatures, including Func<Void> and Result<Void,E>.
Their Markdown guides remain behavioral reference, not renderer exclusions.
Existing selected types include Tasks, Thread/ThreadPool, Storage, IO, Console,
Object/Value, HashCode, Equatable, compiler async support and Introspection descriptors.
Host Fault/FaultCode and debugger fields remain a complete manual [reference](faults.md),
not synthetic CLI types. The Rust-only StringValue payload and owned-text migration
are documented in the [Object guide](objects.md#rust-host-string-payloads) and source
rustdoc; StringValue is not a guest CLI type selected for RavenDoc.

Sequence, Collection, Iterable and Iterator now have generated reference coverage,
including String construction and its read-only grapheme indexer. String Count is
an explicit Collection implementation, visible through Sequence/Collection only.

The current audit covers every public reference type through generated type pages,
three explicit manual entries and ten explicitly excluded metadata scaffolds. Collections, arrays, delegates, query operators,
Option/Result, TaskOutcome, numeric types, text/encoding, environment, time/calendar,
resource capabilities and interop now have type/member descriptions. Compiler-reference
scaffolds are identified as such; they do not promise executable CLR services.
Future work can deepen examples and parameter/error descriptions without withholding
undocumented public types from generation. The snapshot fingerprints all Raven
library source files, so older API families receive the same freshness checks.

The generated reference starts at `/docs/api/`. `legacy-routes.json` preserves the
former DocFX type/namespace routes and member anchors as redirects into RavenDoc.
Keep those routes stable for external links. New links should use xrefs or the
current generated routes. The build validates all local page, sample and asset links
under both root and project-path hosting. Publication remains manual and separate
from runtime releases.


`exclusions.json` records the one reference-only ThreadPool constructor that is not
an application API. This preserves the former scaffold exclusion; it is not a
renderer workaround. The combined build verifies every public type and authored member has a generated
or explicit manual route, reports missing summaries, and validates links across the
entire result. Documentation-only XML changes can
refresh the manifest against the existing matching reference assembly; source/API
changes still require regeneration from the bridge.

Char now has generated coverage for FromString, ToString, Equals(Char) and CompareTo.
Its grapheme semantics and boxed Object behavior are documented; the other public primitive types are now included too.

String's existing public text methods, properties and operators now have generated
coverage. Its reference-only parameterless scaffold constructor is excluded in
exclusions.json and explained in the [Object guide](objects.md#string-through-object-development);
it is not an executable application API. UTF-8 slice errors are described on the
method; remaining text/encoding type coverage is still incremental.


String.Intern now has generated member documentation. Its execution-owned retention
and host quotas are described there and in the manual Fault reference, including
InternPoolLimitExceeded. This does not add a public interning-pool type or IsInterned.

Socket and SocketError now have generated reference coverage under
System.Networking.Sockets, including every error case. The on-site socket guide
describes the first TCP client slice and pending listener/send/addressing work.
Private completion classes and runtime operation handles are not public APIs.

Socket.Send has generated member coverage. Transfer snapshots, short writes, shared
budgets and simultaneous send/receive behavior are described in the socket guide.
The private transfer-result service replaces the former receive-only result service;
refresh matching library, importer and runtime artifacts together.

Dns, DnsError and every error case now have generated reference coverage. The public
lookup returns Sequence<String>; private operation and completion types stay hidden.
The Rust reachability service `RuntimeService::NameResolution` identifies host lookup
requirements; submission additionally requires TaskDispatch, not SocketIo or
IsolatedWorkers. Result consumption requires NameResolution without TaskDispatch.


Socket listener coverage includes Listen, Accept, GetLocalPort, AddressInUse and
InvalidOperation, with XML descriptions and generated member routes. The guide
links the separate-process echo sample; private handles/completions remain hidden.


Socket.Connect now includes a `Sequence<string>` overload with generated member coverage.
The socket guide documents full preflight validation, snapshot ownership, duplicate
removal and fixed shared/per-address connection deadlines. The native array service
and completion helpers remain internal; matching reference/bridge/library/runtime
artifacts are required. This does not supply a combined DNS/HTTP request deadline.


System.Web.Http has generated coverage for HttpClient, HttpHandler, HttpSocketHandler,
HttpRequest, HttpResponse, HttpContent and HttpHeader, including constructors and all
public members. The bounded response parser and per-request continuation object are
internal and excluded from the public inventory by visibility. The Web guide documents
provisional string errors, body/ownership limits and the fixed socket-handler exchange budget and remaining handler/server lifetime gaps.
Socket Send/Receive and TimedOut documentation cover the new five-second per-transfer
bound, one-shot completion and connection preservation. Signatures are unchanged.

HttpServer and HttpRequest.Headers have generated member coverage. The server guide
covers one-request ownership, malformed-request closure, computed framing and the
unsupported cancellation/deadline cases. Internal request parsing, encoding and
operation adapters remain outside the public inventory.

The HTTP socket handler now carries a private monotonic stamp through DNS, connection
and transfer submissions. Public signatures remain unchanged. Deadline helpers and
Until methods stay out of the normal application/reference surface; bootstrap-only
cross-slice visibility is restored to internal before importer contract validation.
Refresh reference, bridge, library and native runtime together for this private ABI.

System.Uri and UriError now have generated type/member coverage. Parse and both
Resolve overloads document strict ASCII grammar, the 4096-byte bound, unsupported
IP literals, lexical equality and RFC relative resolution. HttpError is integrated.
HttpClient.BaseUri now provides optional-string base configuration and
matching string/Uri Get overloads. URI syntax failures preserve UriError; HTTP policy
remains bounded. Token-aware HTTP Send and GetString remain pending.


SocketError is now projected from its normal Raven union source when the bridge
builds either core reference. Its old Is*/Get* helpers are removed; the generated
HasValue, Value and conditional TryGetValue members are documented. IUnion is a
provisional ordinary compiler-support interface. RavenUnionCaseAttribute is excluded
as compiler-reference metadata, with the exact reason in exclusions.json; it is not
an executable runtime API. Rebuild the bridge after changing the embedded source.

DnsError and UriError also use embedded-source projection. Their generated HasValue,
Value and conditional TryGetValue members replace handwritten per-case Is*/Get*
helpers. Their default values are inactive. The core projection runs sequentially
so every family uses the same supplied IUnion identity.

The same generated union documentation now covers StreamError, TextReadError,
StorageLookupError, FileReadError, FileWriteError, ConsoleReadError, Utf8SliceError,
Int32ParseError, IntegerDivisionError and SingleError. EntryKind is an enum instead:
its named fields replace nested union cases and Is*/Get* accessors. Zero is unnamed.

Development cancellation source/token/registration APIs now have generated type and
member descriptions. They are invocation-local and do not yet wire HTTP/native
operations to tokens. Keep this boundary visible when adding consumer overloads.

Native `SocketCancel`/`DnsCancel` operation hooks are bootstrap-only implementation
services, not new public reference APIs. Normal application reference metadata still
omits RuntimeServices; the importer admits these calls only while building the
runtime library. HTTP and managed DNS/socket token forwarding remain pending.

Development networking token overloads (2026-09-25) are included on the existing
Dns and Socket reference pages, with per-member cancellation/ownership descriptions
and the [socket guide](sockets.md#per-operation-cancellation-development). Private
provider registration and shared-deadline helpers remain excluded from application
reference navigation. The matching reference snapshot includes all eight new overloads.

Development HTTP token/text contracts (2026-09-25) are covered on the existing
HttpClient, HttpHandler and HttpSocketHandler reference pages. HttpHandler's former
tokenless Send entry is removed; implementations must accept and forward or honor
CancellationToken. Both GetString address forms, token variants, strict UTF-8 errors,
success-status policy and ownership behavior are documented with the matching
reference assembly. Website building remains skipped by explicit author direction.

The final-status slice adds HttpResponse.IsSuccessStatusCode and the source-projected
HttpError.UnsuccessfulStatus case. The matching reference documents 200–599 responses,
GetString's 200–299 policy and bodyless status rules. Website sources are updated;
building the website is deferred by the author's focused-validation direction.
