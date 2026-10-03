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

Raven normally targets .NET and can use its libraries. The Raven compiler and Raven Language Server run on .NET; the VS Code extension uses that server for editor features. neoCLR itself and programs running on it do not depend on .NET. In the neoCLR integration, Raven emits CLI metadata and IL; a bounded importer translates supported programs for execution on neoCLR’s independent runtime.

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

Development native emission now includes bounded top-level source structs with generic
inline payloads, constructors, accessors and value-copy behavior checked against .NET.
Source-union emission and separate native value-library consumption remain in progress.

The development native emitter also handles bounded nested source class/value declarations
under nongeneric owners. Paired .NET/native execution checks constructor and copy behavior;
Byte discriminator signatures, storage and conversions now execute on both targets;
generated union declarations remain work in progress.

The development metadata API supports value-type interface relationships and bounded
constrained calls through its IL generator. Mutation/copy checks execute on CLR and
neoCLR. Raven admits the declarations and concrete calls; broader constrained lowering,
boxing and complete source-union emission remain in progress.
