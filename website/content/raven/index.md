# Raven language and neoCLR integration

Raven is a typed, general-purpose language for .NET. It brings functions, pattern matching, classes and interfaces into one language, and is the first language being adapted to neoCLR.

Raven targets both .NET and neoCLR. Each target has its own libraries and runtime capabilities.

[Raven website ↗](https://marinasundstrom.github.io/raven/) · [Try the language playground ↗](https://marinasundstrom.github.io/raven/playground/)

The playground introduces Raven syntax; use the [neoCLR bundle](../try/) to run
programs against neoCLR’s libraries.

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

Raven normally targets .NET and can use its libraries. The Raven compiler and Raven Language Server run on .NET; the VS Code extension uses that server for editor features. neoCLR itself and programs running on it do not depend on .NET. The native target emits PE assemblies directly and imports native library metadata into Raven symbols. The matched bundle supplies its primitive bootstrap and runtime seed.

This gives us a source language for applications and for the Raven-authored System.Runtime library. It also makes language and runtime gaps visible through real programs. It is not a promise that arbitrary .NET libraries or every Raven feature already run on neoCLR.

<a id="try"></a>

## Try Raven in a neoCLR project
Start with a `.rvnproj` project in VS Code or the terminal. Our setup guide covers downloads, prerequisites, the project file, build/run tasks and expected output.

[Try neoCLR with Raven →](../try/)

For the language’s ordinary .NET target, visit [Raven’s language website](https://marinasundstrom.github.io/raven/). The target and available libraries differ from this experiment.

<a id="direction"></a>

## Interfaces

Interfaces can supply public default implementations and static helpers. A class's
own implementation takes precedence. Private helpers remain private to the interface.
Private instance helpers and static virtual defaults are not supported.

## Entry points

Use a static, nongeneric `Main`, either a file-scope function or a class method.
Native programs support no-result and integer entry points, plus `Task<unit>` and
`Task<int>` for async entry points. An integer result becomes the process exit status;
a no-result entry exits with status zero on success.

An async Main can await directly. The runtime drives its default task queue and
registered host operations before obtaining the result. Cancellation and unresolved
tasks fail explicitly. Generic async methods and native `Task<Result<…>>` entry
points are not supported.

[Tasks and await →](../features/tasks/#await)

<a id="native-target"></a>

## Native libraries and tools

Raven compiles application and library projects for neoCLR. Consumers import compiled
library metadata without including library sources. The class library covers collections,
Unicode text, JSON, networking and HTTP. The bundled samples demonstrate callbacks,
shared object identity, Tasks and client/server requests.

In VS Code, native metadata supplies completion, hover documentation and read-only
declaration navigation. The project file selects the same references for the editor
and compiler. Library XML and Markdown sidecars supply API descriptions where available.

`neoclr disassemble` inspects metadata and instructions without executing the assembly
or loading its dependencies. Its output is a diagnostic listing, not reassemblable neoIL.

The native target does not support every Raven or .NET feature. Local nongeneric class
virtual dispatch is supported; external class overrides and generic virtual classes
remain limited. Runtime suspension and green threads are not implemented.

[Install and run →](../try/) · [HTTP application case →](../cases/http-server/) ·
[API reference →](../docs/)
