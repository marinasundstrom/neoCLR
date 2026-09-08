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
