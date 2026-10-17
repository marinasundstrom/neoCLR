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
