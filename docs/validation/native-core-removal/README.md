# Native bootstrap removal — 2026-10-17

The reduced native core baseline compiles/emits without CLI references. Raven's
native project provider now preserves explicit RavenTypeOfAssemblyName,
RavenTypeOfInfoType and RavenTypeOfContextType settings. Missing mappings reject
instead of silently falling back; explicitly empty settings disable the contract.

Validation: NeoClrMetadataProbe --native-core-project using NativeCore.dll/Input.dll
from docs/experiments/native-core-bootstrap/Probe.csproj passes native references,
compilation/emission, namespace completion, contract retention/clearing/rejection
and nine malformed project configurations. The matching native-only LSP
[completion result](completion.json) contains Int32 and no Action/Func transport
types. Run check-completion.py SERVER_DLL PROJECT_DIR OUTPUT_JSON to repeat.
The server used for completion is the installed 20261017 SDK; the new contract
regression runs against the rebuilt compiler adapter.

## Remaining production work

A project selecting target/local-libraries-20261017/System.Runtime.dll with
RavenNeoClrNativeCoreReference, matching System.Runtime metadata/target core names
and `func Main() -> int => 42` fails with RAVT003:
`missing or mismatched native dependency: NeoCLR.CoreProbe`.

The production build still uses CLI primitive declarations, an ownership manifest
and retained runtime services. Replace these with native core production and
explicit runtime contracts, rebuild all dependent libraries, then validate the
reflection/async consumers and install a fresh SDK. Do not remove dependency
validation, hide completions, or use fixture Object implementations to claim
production readiness. Structural feature expansion is parked.

## Artifact-derived core identity

The next native project slice derives metadata/emission core identity directly
from RavenNeoClrNativeCoreReference when RavenMetadataCoreAssemblyName is omitted.
The project probe compiles/emits both explicit and inferred configurations, keeps
explicit mismatches rejected and leaves the legacy CLI configuration checks intact.
The website now explicitly distinguishes the production CLI semantic dependency
from native metadata output and the reduced native-only qualification.

## Native parameter-array migration

SDK `99eabe63f`/`609fff182` and Raven `a6006e458` close the local/imported and
symbol-authored parameter-marker gap. `System.Runtime` now declares ParamArrayAttribute;
bootstrap preparation removes its duplicate CLI and seed declarations. This is a
staged migration, not a bridge-free bundle. [Fresh artifact evidence](native-parameter-production.json)
shows no bootstrap TypeRefs in Runtime/Data/Networking/Web/PostgreSql; retained service
AssemblyRefs still remain. A fresh production consumer compiles expanded arguments and
returns 42 in the interpreter. The parameter-array physical calling convention is unchanged.

[Language-server completion](native-parameter-completion.json) against native-only core
and marker references includes ParamArrayAttribute and Int32 without CLI delegates.
This reduced editor fixture establishes native metadata loading, not complete production
editor migration. The production reference snapshot is refreshed from the matching native
bundle and documents the marker under System.Runtime. The installed SDK is unchanged.

## Native queue-service migration

Raven `ff4cff61b` and SDK `bdbe3298d` preserve generic native InternalCall identity.
System.Runtime now declares all three generic queue services in native Raven source,
and staged bootstrap preparation removes the old facade methods and seed functions.
The four-library `target/native-queue-libraries-v3` bundle compiles against that staged
input. The [consumer](QueueConsumer.rvn) exercises queue registration/default identity
and async completion. [Execution evidence](queue-results.json) records 42 in the
interpreter, managed JIT and sanitized AOT, with fresh runtime/backend binaries.
Focused signature tests preserve rejection of malformed getters/registrations.

Raven emits no-result registration, unlike the old seed's inhabited-unit result.
Both registration forms are supported; getters still return their exact generic T,
and the runtime retains nominal TaskQueue identity/layout validation. This does not
relax arbitrary runtime-service declarations.

[Production editor completion](queue-completion.json) resolves TaskQueue.Default
against the same bundle. This production project still uses the CLI core; the check
is member-import validation, not a claim of bridge-free System completion. The
public API surface is unchanged and source fingerprints pass the API snapshot check.
Remaining facade services, intrinsic owners and native source-core construction are
still migration blockers; the installed SDK is unchanged.

## Direct math and environment services

Production callers now select native namespace functions directly, removing 18
forwarding methods and their bootstrap RuntimeServices extension dependency.
No public API or runtime implementation changed. Raven `290de084b` builds the fresh
four-library `target/native-direct-services-v2` bundle. The
[consumer](DirectServicesConsumer.rvn) checks math, command-line arguments, current
directory and environment-query error behavior; [execution](direct-services-results.json)
returns 42 in the interpreter. [LSP completion](direct-services-completion.json)
resolves Math functions against that bundle. API source fingerprints are refreshed.
This remains a production project with CLI core input, not native-only completion.
Existing JIT/AOT implementation evidence is unaffected by these forwarding removals.

An attempted native-integer consumer could not bind Int32-to-IntPtr/UIntPtr casts
or imported CompareTo through the current project configuration. Those two adapters
remain unchanged and the gap must be isolated before migrating them. Other wrappers,
intrinsic ownership and production native-core construction remain open.

## Native integer ownership

The native-integer member gap was caused by missing IntPtr/UIntPtr entries in both
`libraries[].types` and `nativePrimitives` in the production ownership manifest.
Both now select System.Runtime, preserving native-width primitive storage while
exposing the source nominal contracts. Their two forwarding adapters are removed;
source methods and Console call the existing native services directly.

Raven `04632d5e2` builds `target/native-integer-library` with all five libraries.
The [consumer](NativeIntegerOwnerConsumer.rvn) invokes both imported comparisons;
[interpreter execution](native-integer-owner-results.json) returns 42, and
[LSP completion](native-integer-owner-completion.json) exposes CompareTo on nint.
The matching API audit renders IntPtr/UIntPtr under System.Runtime and the native
snapshot/source fingerprints are refreshed. Local file browser preview was blocked
by URL policy; the renderer checks and generated HTML ownership/member links were
inspected instead. No website publication or SDK installation occurred.

This resolves member ownership, not the separate general Raven integral conversion
rules, which still exclude native integers at this compiler revision. It adds no
unchecked allocation or synthetic runtime implementations. Other bootstrap services
and native production core construction remain open.

## Native integral conversions

Raven main `babd5fad0` implements built-in integral conversions involving nint/nuint;
its focused conversion suite passes 104 tests. The neoCLR carry emits native-width
conversion instructions, preserves high UInt32 bits and signed widening, and selects
checked conversion signedness from the source. No new Runtime Contract switch is
needed. IntPtr/UIntPtr instance methods use the same verified borrowed-receiver
projection as the previously supported primitive owners in JIT/AOT.

The [consumer](NativeIntegerConsumer.rvn) compiles against the five-library
`target/native-integer-library` bundle from the ownership slice. It returns 42 in
interpreter, managed JIT and sanitized AOT. [Editor completion](native-integer-cast-completion.json)
resolves CompareTo on `((nint)42).`. This is production native library import with
CLI core input, not proof of complete bootstrap removal. Execution is qualified on
macOS ARM64; 32-bit runtime execution remains unverified.

Target-typed native integer literal assignment and full floating-point/decimal
conversion support remain open; samples use explicit integral casts. Other service
wrappers and production native core construction remain open. Structural work is
parked and the installed SDK is unchanged.

Raven neoCLR `d83ea51e7` is the validated compiler carry. The
[overflow consumer](NativeIntegerOverflow.rvn) reports ArithmeticOverflow in interpreter,
managed JIT and sanitized AOT. [Execution evidence](native-integer-conversion-results.json)
records commands, result codes and binary hashes for both consumers.

## Direct time-zone and heap services

The five time-zone and eight GC forwarding methods are removed from System.Runtime's
source list. Public TimeZone and GC members now call the existing native declarations
directly; their signatures, result contracts, service ownership and runtime bindings
are unchanged. This is the same semantic intent as the old forwards and introduces
no .NET API divergence or Runtime Contract setting.

Raven `d83ea51e7` builds the fresh five-library `target/native-time-heap-library`
bundle. [TimeZone completion](time-heap-completion.json) exposes Find/DatabaseVersion;
[GC completion](heap-completion.json) exposes Collect/KeepAlive. API source fingerprints
are refreshed; public signatures and rendered API content are unchanged.

This still uses the staged CLI core input. Console/storage/string/object/introspection
forwards and retained payload/intrinsic services need further migration; production
native-core construction remains open. Structural work stays parked. No installed SDK
or website deployment is changed.

The [consumer](TimeHeapConsumer.rvn) returns 42 in the interpreter: it checks
unknown-zone rejection, explicit UTC offset/mapping, system-zone query, all heap
counters, collection progress and KeepAlive for a reference and null.
[Execution evidence](time-heap-results.json) includes compiler/runtime commands and
bundle hashes. JIT/AOT service implementations are unchanged; this forwarding-only
slice does not repeat their platform matrix or claim new backend coverage.

## Direct console and storage services

Twelve console, file-text and storage forwarding methods are removed. Their public
callers select the existing native namespace functions directly. Payload interpretation
still uses IsValue/UnpackValue; this does not establish native-only production core
construction or change public API/error contracts. No new Runtime Contract setting
or runtime binding is introduced.

Raven `d83ea51e7` builds the five-library `target/native-console-storage-library`
bundle. [Console completion](console-completion.json) resolves ReadByte and
OpenStandardOutput, and [FileText completion](storage-completion.json) resolves
ReadAllText/WriteAllText against that bundle. Public signatures and reference content
are unchanged; matching API source fingerprints pass. The installed SDK is unchanged.

The [consumer](ConsoleStorageConsumer.rvn) uses piped input and a dedicated `files`
directory beneath its working directory, never application files. It exercises
console byte input/EOF, unsigned formatting, awaited output/flush, UTF-8 file round
trip, size-limit errors, path helpers, kind lookup and directory enumeration.
JIT/AOT implementations are unchanged; this forwarding-only slice does not extend
backend coverage or repeat their matrix. Broader structural work remains parked.

[Execution evidence](console-storage-results.json) records compilation and interpreter
exit 42, exact console output, commands, working directory and artifact hashes.
Create an empty `files` directory beneath the recorded working directory before
reproduction; supply the single input byte `A` followed by EOF. The harness also
verifies `files/roundtrip.txt` remains `Hello, värld!` after the rejected oversized
write.

## Direct object and string-join services

Four forwarding adapters are removed; Object identity/equality/hash and String/
StringBuilder join callers bind existing native namespace declarations directly.
Raven `d83ea51e7` builds all five libraries in `target/native-object-join-library`.
The [consumer](ObjectJoinConsumer.rvn) returns 42 for reference identity, distinct
objects, null equality, stable identity hashing, UTF-8 join and empty join.
[Execution](object-join-results.json) records commands and hashes;
[completion](object-join-completion.json) exposes String.Join. API fingerprints pass.
Public signatures, Runtime Contract settings and runtime implementations are
unchanged. This forwarding-only slice does not repeat JIT/AOT implementation checks.
Production CLI core input and payload dependencies remain; structural work stays
parked and the installed SDK is unchanged.

## Native payload helper qualification

The `--intrinsics` native-core probe now authors IsValue<T> and UnpackValue<T>
with SDK ValueIs/ValueUnpack bodies, plus an exact native ConsoleReadByte declaration.
System.Value has an explicitly owned native primitive declaration/signature; the
SDK deliberately disallows bare PrimitiveType.Value signatures. No compiler
bootstrap interception or new Runtime Contract setting is used.

The [probe](../../experiments/native-core-bootstrap/PayloadConsumer.cs) compiles
against native-only references. With input `*` then EOF it checks byte membership,
rejects Int32 membership, extracts byte 42 and recognizes the unit end-of-input
payload. Wrong-type extraction faults inside UnpackValue. [Execution evidence](payload-results.json)
records build/run commands and hashes. [Native-only LSP completion](payload-completion.json)
exposes both generic helpers. Prepare the empty System seed as for the existing
intrinsic probe; the interpreter is the qualified execution mode for this slice.

This establishes native metadata ownership and interpreter behavior, not general
payload/JIT/AOT coverage or production core replacement. The fixture's Object slots
remain test-only. Production System.Value/RuntimeServices ownership must move as
part of coherent native-core construction; existing payload callers remain unchanged.
The .NET analogue is runtime-owned representation/intrinsic helpers behind CoreLib,
not a new public .NET API. Structural work remains parked.

## Payload native execution and production ownership boundary

The native-owned byte/Int32/unit payload path now passes managed JIT and sanitized
AOT in addition to the interpreter. Successful byte extraction returns 42; wrong-type
extraction reports RuntimeError in all modes. The host also checks root cleanup.
[Native evidence](payload-native-results.json) records compile/link/run commands and
artifact hashes; [the runner](validate-payload-native.py) reuses the existing macOS
ARM64 host recipe and real console input adapter. Supply `*` followed by EOF. This
qualifies the tested scalar/unit shapes, not arbitrary reference payloads or all
System.Value storage shapes.

The native-only rvnc project builds, and fresh [LSP completion](payload-native-completion.json)
exposes both helpers. No CLI semantic reference or BootstrapReference is involved
in these checks. The source and SDK helpers are unchanged from the preceding slice.

**Production ownership is not yet moved.** System.Value already belongs to native
System.Runtime in ownership.json. IsValue/UnpackValue still belong to the CLI
RuntimeServices declaration and retained-seed methods. The production build currently
compiles Raven source first, then translates retained implementations against the
result. Raven's source bodyless-method contract permits exact native InternalCalls;
these generic helpers are managed opcode bodies, not InternalCalls. Removing their
CLI/seed declarations alone therefore does not provide production definitions.
The next construction slice must supply native method definitions/bodies coherently
with their owner and update reference selection before removing the old declarations.
The SDK already authors those bodies and Raven already imports/calls them; the
remaining work is production construction/ownership, not an interpreter or scalar
JIT/AOT payload implementation. Do not use the reduced fixture Object as the
production core. No production bundle or installed SDK was replaced here.

## Production source-owned payload helpers

Raven `eef6c6e3c` closes the preceding managed-body construction gap. The existing
explicit native System.Value ownership contract authorizes only internal extern
`neoCLR.Runtime.IsValue<T>(System.Value) -> bool` and
`neoCLR.Runtime.UnpackValue<T>(System.Value) -> T` declarations, with one
unconstrained invariant type parameter, one by-value argument and no attributes or
source body. The backend supplies managed ValueIs/ValueUnpack bodies. Other externs
remain subject to import/InternalCall rules; .NET behavior is unchanged. Missing
ownership, wrong result signatures and constrained declarations are rejected.

System.Runtime owns both definitions and System.Value. Production callers, including
Networking, now use native namespace functions. Staged preparation removes both
CLI helper declarations and retained-seed methods. This requires the matching Raven
compiler: an older compiler rejects the new declarations rather than silently
falling back. The [ownership evidence](payload-production-ownership.json) records
internal visibility, zero InternalCall flags, native bodies and admission checks.

A narrowly scoped InternalsVisibleTo grant to System.Networking is necessary for
its DNS/socket implementations to decode private service payloads. Public APIs expose
typed Results and offer no replacement for decoding this internal ABI. The helpers
remain internal; no friend grant is added for applications or unrelated libraries.
Revisit the grant if Networking can use a suitable typed runtime service contract.

The fresh five-library `target/native-payload-production-v2` bundle builds with
the newly prepared `target/native-payload-bootstrap` inputs. The
[public consumer](PayloadProductionConsumer.rvn) checks byte and unit console payloads,
valid/invalid integer parsing and string slicing, returning 42 in interpreter,
managed JIT and sanitized AOT. [Commands and hashes](payload-production-results.json)
record the tested compiler/runtime/bundle; input is `*` followed by EOF.
[Production completion](payload-production-completion.json) resolves Console.ReadByte.
API source fingerprints pass; public signatures are unchanged. Wrong-type fault
coverage from the native-only helper matrix remains applicable to the same opcode
bodies. No general payload-cast language syntax is added.

Other RuntimeServices members, native core construction and broader payload storage
shapes remain open. This removes production payload-helper bootstrap ownership,
not every CLI dependency. The installed SDK is unchanged and structural work is parked.

## Direct native networking calls — 2026-10-17

System.Networking now calls the 21 existing DNS/socket declarations in
`RuntimeNetworkCalls.rvn` directly. The RuntimeServices forwarding extension is
removed from production projects and source-list tooling. Native InternalCall
ownership remains System.Networking; callbacks, buffers, cancellation, deadlines
and payload decoding retain their existing contracts. This changes neither the
.NET-style public asynchronous API nor the native provider implementation.
No Runtime Contract setting or Raven/.NET compiler behavior changes.

Raven `eeb6ef0d1` builds the fresh five-library bundle at
`target/native-network-direct-library`, using `target/native-payload-bootstrap`.
The loopback/deadline consumer and the existing callback cancellation regression
validate the interpreter path; Socket.Listen/Connect editor completion and API
source fingerprints are checked against the fresh sources. The callback regression
intentionally retains OnCompleted to observe cancellation/completion ordering.
Backend bindings are unchanged; the earlier JIT/AOT payload evidence remains
applicable, and no new network JIT/AOT qualification is claimed.

See [deadline/listener consumer](NetworkDirectConsumer.rvn),
[execution evidence](network-direct-results.json),
[cancellation evidence](network-cancellation-results.json) and
[editor evidence](network-direct-completion.json). Full native core construction,
remaining introspection adapters and compiler bootstrap dependencies remain open.
The installed SDK is unchanged; broader structural work remains parked.

## Direct native introspection calls — 2026-10-17

Remove the 49-method RuntimeServices introspection extension. Its callers now use
48 existing native InternalCall declarations and the source-owned
RuntimeTypeInfo.FromHandle factory directly. Production and POC ownership source
lists and the JSON mapping harness no longer include the removed file.
System.Runtime continues to own these declarations. Public introspection and
checked reflection APIs, native providers, callback signatures and error mapping
are unchanged; no Runtime Contract setting or ordinary Raven/.NET behavior changes.
This preserves the existing .NET-style metadata inspection/reflective invocation
model and neoCLR's Result-based preflight errors; it adds no new divergence.
Existing structural descriptor calls are only retargeted; structural work is parked.

Raven `589c353e7` builds all five libraries in
`target/native-introspection-direct-library` using `target/native-payload-bootstrap`.
The existing field-reflection, constructor/type-initializer and dynamic-function
invocation consumers are compiled against this fresh bundle and return 42 in the
interpreter. They cover readonly/literal rejection, invalid receiver/value errors,
initialization timing, function argument/receiver rejection and successful calls.
Editor completion resolves GetFields and TypeInitializer on typeof(int).
API source fingerprints are refreshed without changing public signatures.

Evidence: [field consumer](introspection-fields-results.json),
[construction consumer](introspection-construction-results.json),
[function consumer](introspection-function-results.json) and
[completion](introspection-direct-completion.json). Native backend implementations
are unchanged; their earlier JIT/AOT qualification is reused, not rerun here.
Type-handle primitives, strings/parsing, scheduling and other retained core services
still require native ownership and compiler/bootstrap migration. This does not
replace the installed SDK or establish native-only production core construction.

## Native type-handle ownership — 2026-10-17

System.Runtime now owns seven internal native declarations: TypeName,
TypeDisplayName, TypeShape, TypeMetadataToken, TypeEquals, TypeArgumentCount and
TypeArgument. The existing TypeArgumentCount declaration is reused and the other
six are added to RuntimeHandleServices.rvn. Descriptor callers select these native
functions directly. The runtime implementations and .NET-style type identity,
generic argument retention and introspection semantics are unchanged.

Bootstrap preparation removes the seven CLI facade methods and seed facade bodies.
The seed's native service declarations remain only during assembly of its temporary
Object body; preparation removes them from the retained model together with that
Object. Exact-count checks reject missing/duplicate seed declarations. No native
handle implementation is retained as a fallback. Original legacy seed sources stay
available for older probes; production preparation is the migration boundary.

Raven `3d233b07c` builds all five libraries in `target/native-handle-direct-library`
against freshly prepared `target/native-handle-bootstrap`. No Runtime Contract
switch or compiler/.NET change is required. The [consumer](TypeHandleConsumer.rvn)
checks generic int/string/nested arguments, identity, nominal/value/array flags,
names and metadata tokens. [Interpreter evidence](type-handle-results.json),
[native ownership evidence](type-handle-ownership.json) and
[editor completion](type-handle-completion.json) qualify this slice. The API source
fingerprint check passes; public signatures are unchanged. Existing JIT/AOT provider
implementations are reused without a new backend matrix in this slice.

Strings/parsing, scheduling and other retained services plus compiler bootstrap
contracts still require migration before production native-only core construction.
The installed SDK is unchanged. Existing structural descriptor behavior is preserved;
broader structural work remains parked.

## Native direct text/parsing ownership — 2026-10-17

33 direct string/character/formatting/parsing/UTF-8 decode services now belong to
native System.Runtime declarations. All source callers select them directly.
Preparation removes their CLI facade methods and retained seed definitions. It
also removes the obsolete seed Console type/methods: its WriteLine(int) called
Int32ToString across the new internal ownership boundary and caused access denial.
Source Console already supplies the production API; no helper is made public.

The strict invariant parsing grammar, Result error cases, grapheme/UTF-8 semantics
and native providers are unchanged. Existing documented differences from .NET
text representation/parsing are retained, not redesigned. Raven 70bf9be9c builds
five libraries in target/native-text-direct-library-v2 against freshly prepared
native-text-bootstrap-v2. The [consumer](TextConsumer.rvn) checks all eleven
parsers, format/overflow rejection, string operations, graphemes, UTF-8 slicing
and valid/invalid decoding. [Execution](text-results.json),
[ownership](text-ownership.json) and [completion](text-completion.json) record the
checks. API source fingerprints are refreshed; no public signatures change.
Provider implementations are unchanged; no new JIT/AOT qualification is claimed.

StringEquals still has a managed opcode body. StringGraphemes, StringScalars and
Utf8Encode still materialize managed arrays through retained helper bodies. These
are the next text migration boundary, followed by remaining services and compiler
bootstrap contracts. The installed SDK is unchanged and structural work is parked.

## Native text arrays and equality — 2026-10-17

StringGraphemes, StringScalars and Utf8Encode now use existing native managed-array
result support directly. StringEquals is source-owned code: explicit null checks
followed by ordinal comparison, avoiding recursion through String's public operators.
Preparation removes the four CLI facade methods, the three native seed definitions
and all four managed seed bodies. No public signature, runtime provider or compiler
contract changes. The fresh five-library native-text-array-library bundle uses
native-text-array-bootstrap and Raven 304e8675a.

[Consumer](TextArrayConsumer.rvn), [interpreter evidence](text-array-results.json)
and [completion](text-array-completion.json) cover null/value equality, grapheme
iteration, Unicode scalar snapshots, UTF-8 encode/decode and empty arrays. API
fingerprints pass. Existing JIT/AOT provider support is reused without rerunning a
backend matrix. No text helper remains on RuntimeServices; file/time/scheduling
and compiler bootstrap contracts remain open. Structural work is parked.

## Native file/time/worker/task services — 2026-10-17

14 direct services now have internal System.Runtime declarations: file streaming,
host culture/current time, worker start/queue/join and task scheduling. Preparation
removes their CLI facade and retained native definitions. ScheduleTask additionally
admits Raven's no-result signature: interpreter dispatch does not push a unit value,
and native binding generation preserves the no-result return body. Legacy unit
returns and exact callback restrictions remain supported. Focused admission,
interpreter callback completion and native binding tests pass.

Raven 66c31932a builds five libraries in native-core-services-library-v2 using
native-core-services-bootstrap. The [consumer](CoreServicesConsumer.rvn) exercises
Task.Run, thread/pool completion, isolated file write/flush/read/seek, clock and
culture; [execution/test evidence](core-services-results.json) and
[completion](core-services-completion.json) pass. API fingerprints are current.
Awaited results are bound to locals: the attempted nested await/conditional form
hit an existing native lowerer value-block exit restriction, outside this slice.
Native binding validation is not a new full JIT/AOT application qualification.

UnixTimeToLocal is the last source RuntimeServices call. A separate native-only
production consumer still reports missing NeoCLR.CoreProbe: compiler source-core
construction and remaining metadata dependencies are not closed by these forwards.
The installed SDK remains unchanged; structural work is parked.

## Native local-time conversion — 2026-10-17

UnixTimeToLocal now uses an internal native declaration and managed Int32 array
results. Preparation removes its CLI/seed facade. Production Raven library source
has no remaining RuntimeServices calls. Exact result-element admission and native
managed-array materialization pass focused tests; legacy array results remain
supported. Raven 85660156b builds five libraries in native-local-time-library with
native-local-time-bootstrap. The [consumer](LocalTimeConsumer.rvn) checks epoch and
negative timestamps under UTC; [execution](local-time-results.json) and
[completion](local-time-completion.json) pass. API fingerprints are refreshed.
This qualifies interpreter execution, not new JIT/AOT local-time support.
Compiler source-core construction and the CoreProbe metadata dependency remain
open; the installed SDK is unchanged and broader structural work stays parked.

## Reference-free native source core — 2026-10-17

Raven 87698cd00 adds explicit `RavenNeoClrSourceCore=true` library construction.
The output owns primitive/root symbols and uses an ownership manifest; it imports
no CLI/native reference core and performs no CLI projection. Native Object,
array views, source attributes and emitted extension ownership have source-aware
binding. Main 2766516e1 fixes exact metadata arity (four regressions fail before
and pass after); codex/neoclr carries it as 0606260b0.

Raven's `tools/NeoClrMetadataProbe --native-source-core <directory>` constructs
a small core, imports it into a consumer, checks completion and rejects conflicting
configuration/native-call flags. The consumer exercises an array interface through
a generic extension and returns 42 against an empty System seed. The existing
native-core project checks also pass. [Evidence](source-core-results.json) records
revisions and hashes. The test Object methods are fixture bodies, never production
implementations. Production core declarations and managed allocation/entry-drain
contracts remain to migrate; installed artifacts and broader structural work are
unchanged.

## Source-core managed array reservation — 2026-10-17

Raven 31da12d29 admits only the exact internal unannotated source-core
`neoCLR.Runtime.ReserveArray<T>(count: int) -> T[]` declaration and emits a
managed body using the existing checked reservation operation. No bootstrap
reference matching or host callback is involved. Non-Int32 sizes and fixed-length
result signatures are rejected before output.

The source-core probe emits three consumers: initialized array/interface/extension
use returns 42; uninitialized reads and negative lengths fault. All pass in the
interpreter, JIT and sanitized AOT using an empty System seed and the source-built
core. [Evidence](source-core-storage-results.json) and the
[runner](validate-source-core-storage.py) preserve the checks. Final compiler
validation produces byte-identical accepted artifacts. This retains checked
uninitialized storage semantics, not .NET zero-filled array allocation. Public
class-library forwarding and the remaining core declarations are still pending.

## No-result entry draining

Native source declarations can expose `neoCLR.Runtime.DrainEntryTasks` with no
result. The interpreter remembers that shape while running queued callbacks and
does not push a synthetic unit on return. The JIT/AOT binding stand-in emits a
bare return for that signature. The existing inhabited-unit contract is retained
for explicit legacy fixtures. Six `entry_results` integration tests and the
task-queue codegen binding test pass; production
backend qualification is recorded separately below. This is startup plumbing,
not a change to task scheduling or broader structural features.

## Native executing-assembly getter

JIT/AOT preparation now validates the source getter's direct reserved InternalCall
as well as the explicit historical two-step facade. Caller-specific assembly
recipes and receiver checks remain unchanged. Four `runtime_context` codegen
tests pass, including native direct calls and malformed/indirect rejection.
This closes a preparation dependency on the removed IntrospectionRuntimeServices
wrapper; it does not relax native service admission.
