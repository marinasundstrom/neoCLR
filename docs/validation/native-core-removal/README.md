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
