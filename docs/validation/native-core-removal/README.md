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
