# Environment discovery API prototype

2026-10-23. Author-requested, executable **Raven on .NET 10** design experiment.
This does not change the shipped `System.Environment` module or add native neoCLR
host bindings. Inspect [Model.rvn](Model.rvn) first, then the
[consumer and checks](Main.rvn) and isolated [host adapter](DotNetHost.rvn).
The prototype namespace avoids colliding with the host's System types; intended
library placement is System, subject to API review and native validation.

## Implemented prototype

- A static Environment class with GetOperatingSystem(), NewLine,
  ProcessArchitecture and OSArchitecture. Discovery occurs at execution time.
- Closed OS record cases: Windows, Linux, MacOS, Android, IOS and OtherOS.
- WindowsVersion has four numeric components; ProductVersion has three.
  KernelRelease preserves raw text; DistributionRelease keeps optional identity
  and release strings, including nonnumeric releases. AndroidApiLevel is separate.
  There is no common Version property or comparison across version domains.
- Architecture distinguishes x86, x64, ARM, ARM64 and a preserved other name.
  Process and OS architecture are separate facts; neither is a CPU-feature probe.
- Explicit ToString overrides on the OS base, every case, all version types and
  architecture types. These are readable diagnostic/display representations, not
  serialization formats or inputs to platform detection. Equality remains typed.

Examples verified by the consumer:

```text
Windows 10.0.26100.0
Linux 6.12.0-custom (debian testing)
Linux 6.12.0-custom
macOS 15.2.0
Android API 35
iOS 18.0.0
ARM64
```

The author's broader direction is to give meaningful domain types useful string
representations where appropriate; this prototype applies it to the types above,
without implying that every existing platform type has been audited.

## Run and inspect

From the repository root, supply a matching Raven compiler and Raven.Core:

```sh
python3 docs/experiments/environment-discovery/run.py \
  --compiler /path/to/rvnc.dll \
  --raven-core /path/to/Raven.Core.dll
```

The runner emits into `target/environment-discovery-prototype`, fails on a compiler
or consumer failure, and records exact compiler/core/source/assembly hashes and
output in `validation.json`. [Recorded validation](validation.json) passed on the
local macOS ARM64 .NET host. Its reported product version is preserved as observed.
Other OS examples are constructed fixtures, not cross-platform host qualification.
Checks include distinct Windows builds, kernel suffix preservation, nonnumeric
release identity, unknown architecture retention, newline delegation, every OS
case's string output through OperatingSystem and Object references, missing
distribution fields, and architecture formatting/dispatch.

## .NET comparison and choices

.NET uses a [static Environment class](https://learn.microsoft.com/en-us/dotnet/api/system.environment?view=net-10.0)
and [Environment.OSVersion](https://learn.microsoft.com/en-us/dotnet/api/system.environment.osversion?view=net-10.0).
The proposed method and case-specific payloads are deliberate deviations: they
make different version meanings explicit and support pattern matching, at the cost
of more public types, migration work and case-set evolution. Retaining a closed
set requires an explicit unknown/other case and compatibility review when adding
new named cases. Final method failure semantics remain open.

[Environment.NewLine](https://learn.microsoft.com/en-us/dotnet/api/system.environment.newline?view=net-10.0)
supplies platform-default text delimiters. Existing explicit protocol/reader/writer
delimiters are not changed. Architecture follows .NET's separation of process and
[OS architecture](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.runtimeinformation.osarchitecture?view=net-10.0);
OS detection remains best effort under emulation, not guaranteed physical hardware
identity. Placement on Environment rather than RuntimeInformation is provisional.

The [Unix .NET version parser](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Environment.OSVersion.Unix.cs)
extracts numeric components from a release string. Keeping KernelRelease text avoids
losing suffixes; it does not supply ordering. Distribution VERSION_ID from
[os-release](https://www.freedesktop.org/software/systemd/man/latest/os-release.html)
is separate from the kernel. Android requires
[SDK_INT](https://developer.android.com/reference/android/os/Build.VERSION#SDK_INT),
not a kernel-version component. These sources describe host contracts, not proof
that the prototype runs on those systems.

## Boundaries and next implementation work

- The .NET adapter detects the local OS and architecture. Linux reads the full
  `/proc/sys/kernel/osrelease` text, with I/O errors propagated; distribution discovery
  is not bound and returns null. Android detection explicitly throws until an API
  level binding exists; constructed Android values work. Windows/Linux/iOS paths
  compile but were not executed on those hosts. No mobile platform support is claimed.
- Version records currently accept their components without validation, parsing or
  ordering. ProductVersion is shared by macOS/iOS for this experiment only. Decide
  whether platform-specific version types, absent components, Result errors and
  Option payloads improve the final contract. Numeric formatting is provisional.
- Raven uses `sealed record ... permits ...` for this closed hierarchy; it is not
  a new `closed` keyword implementation. Records remain ordinary reference types.
- Standard union architecture was attempted first. The local CLI build produced
  an inactive-default diagnostic when forwarding a union-valued property; changing
  to method forwarding compiled but an execution printed an uninitialized value
  and crashed. Closed records are a prototype workaround, not a permanent departure
  from the repository's union convention. Root cause and a minimal compiler
  regression remain open; this experiment makes no compiler changes.
- An abstract ToString override in the base compiled but failed Object dispatch in
  this local compiler. Concrete base overrides and derived overrides pass the
  retained dispatch checks. Architecture interpolation also produced empty text;
  explicit ToString calls pass. These observations need isolated compiler follow-up.
- Native integration must supply host-owned OS/version/architecture services, check
  the source module-to-class migration and imported call sites, preserve runtime
  capability contracts, and validate interpreter/JIT/AOT consumers. Update native
  reference selection/snapshots and compiler/integration docs when APIs are promoted.
  Raven main and the neoCLR compiler branch remain separate; neither was edited.
- This experimental API is not added to the published reference or website as a
  shipped capability. Native implementation and release acceptance remain open.

## Native integration follow-up (2026-10-23)

The prototype files and validation.json above preserve the initial .NET experiment.
The integrated API now lives in [Platform.rvn](../../../runtime/raven/src/System/Environment/Platform.rvn)
and [Functions.rvn](../../../runtime/raven/src/System/Environment/Functions.rvn).
Cases are nested (`OperatingSystem.Linux`, `Architecture.Arm64`); GetOperatingSystem
returns Result. Native Android API-level binding exists but has not been qualified
on Android. Earlier statements about pending integration/compiler changes describe
the prototype stage, not the current development implementation.

The authoritative [contract](../../environment.md) and built-in framework suite
in `runtime/raven/tests/environment-discovery` supersede the prototype for native
behavior. Raven main a271da192 and codex/neoclr bf2acea78 fix the isolated abstract
override/empty record issues; native nested-case support is a subsequent branch
extension. Explicit permits naming nested cases still diagnoses before their base
relationships are bound; use the standard same-file inferred sealed hierarchy.
The native metadata abstract-override limitation and uninvestigated union forwarding
observation remain separate gaps, not permanent platform semantics.

[Native integration evidence](integration.json) and [built-in suite results](native-tests.json):
eight environment plus fifteen existing record-formatting tests pass in all three
engines (69 executions). The latter also verifies the dispatch projection does not
regress inherited formatting or value records.
