# Raven language and neoCLR integration

Raven is a typed, general-purpose language for .NET. It brings functions, pattern matching, classes and interfaces into one language, and is the first language being adapted to neoCLR.

**In active development.** Raven has its own language and toolchain. The neoCLR target is an experimental integration with a bounded runtime API; support on .NET does not automatically mean support on neoCLR.

[Meet the language ↓](#examples) · [Raven’s language website ↗](https://marinasundstrom.github.io/raven/)

<a id="examples"></a>

## Functions and typed results

```raven
{{RAVEN_SAMPLE}}
```

`func` introduces a function and `let` an immutable local binding. Types can be inferred where the expression makes them clear. Here `?` continues with a successful value or returns the error to the caller; the Result return type makes that outcome visible.

This example uses neoCLR’s Math API, where an overflow is a Result. The same language can consume different library contracts on another target.

[Explore Option and Result →](../features/outcomes/)

<a id="patterns"></a>

## Pattern matching

```raven
{{UNION_SAMPLE}}
```

Patterns expose a case and bind its payload. There is no need for a special method to extract Some’s value. Raven also supports classes, interfaces and object-oriented code when identity, lifetime or polymorphism is part of the problem.

[See patterns with the sealed MemberInfo hierarchy →](../features/introspection/)

<a id="callbacks"></a>

## Bindings, mutation and callbacks

```raven
{{FUNC_SAMPLE}}
```

`var` marks a mutable binding. These callbacks capture the same variable: writing 42 changes what the reader observes. On neoCLR, System.Void can be a generic result type, so one Func family also describes callbacks with no payload. The Raven spelling `unit` and its value `()` map to System.Void in this target.

For properties, `val` expresses read-only access. Explicit mutability helps readers distinguish a stable local binding from state that may change.

<a id="targets"></a>

## The .NET and neoCLR targets

Raven normally targets .NET and can use its libraries. The Raven compiler and Raven Language Server run on .NET; the VS Code extension uses that server for editor features. neoCLR itself and programs running on it do not depend on .NET. The published integration uses a bounded CLI importer. The development native target emits NeoCLR PE assemblies directly and imports native library metadata into Raven symbols; it still needs an explicit primitive bootstrap and compatible runtime seed.

This gives us a source language for applications and for the Raven-authored System.Runtime library. It also makes language and runtime gaps visible through real programs. It is not a promise that arbitrary .NET libraries or every Raven feature already run on neoCLR.

<a id="try"></a>

## Try Raven in a neoCLR project
Start with a `.rvnproj` project in VS Code or the terminal. Our setup guide covers downloads, prerequisites, the project file, build/run tasks and expected output.

[Try neoCLR with Raven →](../try/)

For the language’s ordinary .NET target, visit [Raven’s language website](https://marinasundstrom.github.io/raven/). The target and available libraries differ from this experiment.

<a id="direction"></a>

## Compiler integration and planned work

The neoCLR work exercises runtime contracts, metadata and library ergonomics. General compiler improvements belong in Raven’s ordinary development; neoCLR policies remain isolated until they are ready. Both projects are open to feedback, and their APIs can change.

[Explore neoCLR’s proposals →](../proposals/)

## Interface bodies in development

Application interfaces can supply public default implementations and public or private
static helper methods. Defaults execute on the original object, and a class's own
implementation takes precedence. Private helpers stay private to the interface.
These capabilities are included in Preview 11. Private instance helpers,
static virtual defaults and broader interface accessibility remain future work.

Explicit interface methods are also supported for non-generic
application classes and interfaces. Two interfaces can select different private
implementations of the same-named method while the class keeps a separate public
method. Explicit accessors, generic/value-type import and implementations of external
core-library contracts remain outside this bounded slice.


<a id="entry-points"></a>

## Supported entry points

**Preview 11.** Use the matching compiler, bridge
and runtime. A program has one static, nongeneric `Main`, either a file-scope
function or a static class method. Each return type below accepts no arguments
or one `string[]` argument containing the application arguments, without the
executable name.

| Main return type | Successful process exit status |
| --- | --- |
| `()` (or omitted) | `0` |
| `int` | Returned integer |
| `Result<int, E>` | Integer in `Ok` |
| `Result<(), E>` | `0` for `Ok` |
| `Task<()>` | `0` after completion |
| `Task<int>` | Integer after completion |
| `Task<Result<int, E>>` | Integer in `Ok` after completion |
| `Task<Result<(), E>>` | `0` for `Ok` after completion |

A Result `Error` prints its payload to standard error and returns status `1`.
Cancelled tasks and tasks still pending after available work drains produce a
runtime failure. `unit` is another spelling of `()`; neoCLR uses `Task<unit>`
instead of a separate nongeneric .NET `Task` type.

An async Main can await directly. The runtime drives the default task queue and
registered host operations before obtaining its result. Explicit private task
queues remain caller-driven. Ordinary `Task.GetResult()` stays nonblocking.

[See a tested async Main awaiting a worker →](../features/tasks/#await)

`neoclr run` also uses integer entry values as process status for Neo and neoIL.
Existing scripts that previously ignored a nonzero integer result should account
for that status.

<a id="native-target"></a>

## Native target: Preview 12 POC

**Preview 12, macOS arm64.** Raven's `rvnc neoclr` command
emits assemblies that neoCLR loads and executes directly. Native library references
supply compiler symbols without translating those libraries back to .NET metadata.
The explicit primitive bootstrap, retained runtime seed and matching dependency
artifacts remain required. The ordinary .NET target keeps its existing backend.

All ten selected POC samples now have compilation and checked execution evidence.
They cover collections and queries, interface and virtual calls, class identity and struct copies,
JSON object mapping, Tasks/await, cancellation, and an HTTP client/server pair tested
over localhost. This is a bounded sample gate, not a claim that every Raven program
or the entire System class library compiles natively.

Selected source-library assemblies are also compiled and consumed separately, including
Unicode String/Char, encoding, text streams and JSON work. The text model distinguishes
grapheme clusters, Unicode scalars and bytes; native UTF-8 storage does not change the
ordinary .NET target's Char semantics. Support and limitations remain target-specific.

**Development beyond Preview 12:** the full 197-input class-library audit now emits a
source-owned native assembly. The unchanged orders application imports that assembly
with library sources absent, then compiles and runs with its expected collection,
callback, query and shared-identity behavior. Explicit Object-root ownership, primitive
bootstrap and a retained runtime seed are still required. This is a development acceptance
gate; production library packaging and editor configuration for this layout remain work
in progress, and not every class-library API has execution coverage.

The development class-library split now has native projects for System.Runtime,
System.Data, System.Networking and System.Web. A staged build produces matching
assemblies and a retained seed with artifact hashes; five HTTP/networking consumers
and a separate project consumer execute against that bundle. Platform service extraction,
API-documentation bundling and editor qualification for this split remain open. This
class-library artifact bundle is not a complete SDK release. It now includes a relocatable
project configuration; headless workspace symbol loading and an unchanged HTTP project
pass after relocation. Installed-editor acceptance of this split remains separate.

Native nongeneric async functions and class methods use the existing heap state-machine
lowering. `Task<unit>` and `Task<int>` entries drain registered work before obtaining their
result; cancellation and unresolved tasks fail explicitly. Generic async methods and
native `Task<Result<…>>` entry points remain unsupported. The Preview 11 entry-point table
above describes the published bridge path, not blanket native-target support. Runtime
suspension and green threads are future work.

The unchanged `application-inheritance` sample now executes abstract base, interface,
virtual/override and direct base-call behavior with the same output as the .NET target.
Support is bounded to local nongeneric class slots; external class overrides, generic
virtual classes and new-slot hiding remain outside this native slice.
A local matched compiler/server/extension/runtime bundle now passes extracted
compilation and execution, plus 19 installed VS Code checks on macOS arm64, including
API help and native reference refresh. Primitive bootstrap and retained runtime seed
remain explicit dependencies. The matched download includes bootstrap dependencies with recorded source provenance.

The development `neoclr disassemble` command inspects native metadata and instructions
without loading dependencies or executing the assembly. Its output is a diagnostic
listing, not reassemblable neoIL.

[Tasks and their runtime limits →](../features/tasks/) ·
[HTTP application case →](../cases/http-server/) ·
[Published toolchain setup →](../try/)
