# Process environment

Implemented 2026-09-08. This bounded, read-only System.Environment subset supports
small command-line programs without a configuration framework.

| API | Result |
| --- | --- |
| GetCommandLineArgs() | String[]; a fresh owned copy of the arguments supplied by the host |
| GetCurrentDirectory() | Result<String,EnvironmentError>; the host process's current directory |
| GetEnvironmentVariable(String name) | Result<Option<String>,EnvironmentError>; process variable lookup |

A missing variable is Ok(None); an existing empty variable is Ok(Some("")). Invalid
names (empty, containing NUL or '=') and values that cannot be represented as UTF-8
return Error. Directory lookup or representation failures also return Error. The
initial concrete EnvironmentError has no detail fields; ToString returns
EnvironmentUnavailable. Detailed OS error classification remains future work.
Variable-name casing follows the host OS. These are live process reads, not an
immutable environment snapshot. There are no setters, enumeration, user/machine
registry scopes, exit/termination APIs or implicit .env-file loading.

## Host and CLI contract

Rust hosts supply `ExecutionOptions.arguments: Vec<String>` per execution. Its
default is empty; the runtime never substitutes the embedding process's own command
line. Repeated reads produce independent array values subject to the ordinary array
budget. Adding the field is a preview Rust API change: exhaustive struct literals
must initialize it or use `..Default::default()`.

The CLI includes the selected guest input path at index zero, followed by arguments
after `--`. Runtime options before `--` are excluded. Quoting is handled by the shell;
the runtime preserves the resulting strings, including empty strings. Run and debug
use the same contract. This does not change Neo's parameterless Main entry point or
the existing CLI convention of printing its returned value.

```sh
NEO_DEMO=hello cargo run --locked -- run examples/source/environment.neo -- "first argument"
cargo run --locked -- debug examples/source/environment.neo -- "first argument"
cargo test --locked --test environment
```

The example prints guest arguments, the current directory, and a single explicitly
named demonstration variable. The CLI currently requires Unicode command-line
arguments; arbitrary native argument bytes are not part of this API.

## .NET comparison and responsibility

.NET [GetCommandLineArgs](https://learn.microsoft.com/en-us/dotnet/api/system.environment.getcommandlineargs?view=net-10.0)
includes the executable at index zero. NeoCLR uses the guest input path as the
analogous first item rather than exposing the runtime launcher and its switches.
Embedding hosts control their own list, which permits isolated tests and multiple
guest invocations. There is no implicit ambient argument fallback.

.NET [GetEnvironmentVariable](https://learn.microsoft.com/en-us/dotnet/api/system.environment.getenvironmentvariable?view=net-10.0)
uses null for absence and [CurrentDirectory](https://learn.microsoft.com/en-us/dotnet/api/system.environment.currentdirectory?view=net-10.0)
is a get/set property with exceptional failure. NeoCLR uses Option for absence and
Result for failure; GetCurrentDirectory is deliberately a method to make its fallible
host read explicit. This increases call-site handling but avoids nullable reference
defaults and an exception dependency. UTF-8 representation errors remain explicit;
we do not silently replace native bytes or unpaired UTF-16 units. Invalid-name
rejection is a provisional stricter contract rather than full .NET compatibility.

Three InternalCalls perform host access and declare ProcessEnvironment. The argument
helper also requires ManagedArrays; the two erased-result helpers require
ValueStorage. Ordinary library IL constructs public Option/Result carriers. Neo uses
existing calls, arrays and match syntax; no opcode or metadata extension is needed.
Host access is not a sandbox or a configurable environment provider. A provider can
be explored when embedding requirements justify it. Globalization is unrelated to
this slice and remains deferred.

Tests cover argument isolation and limits, Neo/artifact execution, CLI option
separation, current directory and missing/empty/present/invalid-text values. Unix
non-Unicode tests use isolated child processes, without mutating test-process globals.


## Native source bootstrap (development, 2026-10-06)

The Environment facade now builds directly from its unchanged Raven sources and is
consumed through native metadata. GetCommandLineArgs returns a fresh managed string
array, including independent mutation across repeated calls. Its host buffer is an
implementation detail; inline value arrays are not an exposed platform feature.
A separate consumer exercises all three functions, Unicode and absence/error distinctions.
See the [source gate and limitations](experiments/extended-cli-metadata/source-environment-2026-10-06.md).

## Platform discovery (development, 2026-10-23)

`System.Environment` is now a static class in namespace System. Existing qualified
calls retain their spelling; code importing the former module's functions must
use the class. Rebuild native consumers against the matching library/compiler.
This is next-release development and does not change published Preview 17 packages.

| API | Contract |
| --- | --- |
| GetOperatingSystem() | Result<OperatingSystem, EnvironmentError>; execution host classification |
| NewLine | CRLF on Windows, LF elsewhere; independent of protocol delimiters |
| ProcessArchitecture | Architecture of this process |
| OSArchitecture | Host OS architecture, best effort under emulation |

OperatingSystem is a closed record family with nested Windows, Linux, MacOS,
Android, IOS and OtherOS cases. Architecture likewise nests X86, X64, Arm, Arm64
and OtherArchitecture. Unknown cases retain descriptive text. Match the named
cases for exhaustive classification, or destructure only the characteristics of
interest. Adding a named case is a source compatibility consideration for exhaustive
matches. The base families cannot be instantiated.

The compiled suite uses this nested pattern (`result` has type
`Result<OperatingSystem, EnvironmentError>`):

```raven
if result is Ok(OperatingSystem.Linux(let kernel, let distribution)) {
    _ = Assert.Equal("6.12-custom", kernel.Value)?
    _ = Assert.True(distribution is DistributionRelease, "Distribution payload")?
}
```

A case-only pattern is `OperatingSystem.Linux`; a match over OperatingSystem can
list all six qualified cases without a catch-all. The suite exercises both forms.

Windows carries WindowsVersion(Major, Minor, Build, Revision); macOS/iOS carry
ProductVersion(Major, Minor, Patch); Linux carries KernelRelease(Value) and nullable
DistributionRelease(Id, VersionId); Android carries AndroidApiLevel(Value).
Numeric records currently accept components without validation or parsing/ordering
APIs. Kernel and distribution release text retains vendor suffixes and nonnumeric
values. Every family, case and version type has an explicit readable ToString;
these displays are not stable serialization or sorting formats.

The host adapter uses Windows RtlGetVersion/GetNativeSystemInfo, Unix uname,
Apple kern.osproductversion/hw.optional.arm64 and Android ro.build.version.sdk.
Windows Revision is currently zero (the OS revision is unavailable through this
binding). Linux distribution metadata is currently null. Unsupported or failed OS
version reads return EnvironmentError; unknown architecture becomes
OtherArchitecture("unknown"). Mobile and other-platform bindings require host
qualification; local macOS testing does not establish support on those targets.

A private Int32-to-String InternalCall requires ProcessEnvironment. Interpreter
and compiled hosts share `runtime/native/environment/platform.c`; no external
command is executed. Native/JIT admission requires `--bind-environment-platform`,
`--compile-system`, `--reference-arena` and the matching text-arena host adapter.
The private selector ABI is implementation detail, not a public platform contract.

Compared with .NET Environment.OSVersion/System.OperatingSystem and
RuntimeInformation.ProcessArchitecture/OSArchitecture, this API deliberately uses
closed record cases and OS-specific version domains. That enables typed patterns
and preserves Linux release text, at the cost of more public types and exhaustive
match migration when cases are added. Fallible discovery uses the existing Result
convention. Architecture lives beside Environment for one platform discovery entry
point; no .NET RuntimeInformation compatibility is claimed. The earlier
[.NET prototype and research](experiments/environment-discovery/README.md) retain
historical evidence separately from the native implementation.

The built-in framework suite is `runtime/raven/tests/environment-discovery` and
runs through `scripts/test-runtime-library.py --suite environment-discovery`.
It exercises string dispatch, equality, nested patterns, exhaustive matches, live
host facts and GC lifetime. See the [retained integration evidence](experiments/environment-discovery/integration.json)
for execution results and exact artifacts. All eight environment and fifteen record
formatting tests pass in all three engines (69 executions) on macOS ARM64.
