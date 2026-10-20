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

`var` marks a mutable binding. These callbacks capture the same variable: writing 42 changes what the reader observes. Native function signatures describe both value-producing and no-result callbacks; see [Function types](../features/functions/) for their runtime identity and invocation contracts.

For properties, `val` expresses read-only access. Explicit mutability helps readers distinguish a stable local binding from state that may change.

## Operators and checked arithmetic

Raven supports arithmetic, bitwise, shift, comparison, unary and conversion
operators, including in-place compound operators such as `func +=(amount: int)`.
In-place operators preserve reference identity and mutate value storage. Nominal
struct arrays remain interpreter-only. Parameterless instance `++`/`--` support
prefix results and discarded postfix mutation; consumed postfix uses a static overload.
Paired `true`/`false` operators support conditions and short-circuit operations
through `&`/`|`; direct `&&`/`||` declarations are not supported.

Use `checked(...)` or `checked { ... }` for overflow checking, and `unchecked` for
wrapping integer operations. The nearest context wins. Checked overloads require
matching ordinary declarations and follow .NET selection conventions. Native
arithmetic overflow causes a terminal Fault; .NET uses OverflowException.
Compile-time fixed-width arithmetic and casts diagnose overflow by default;
explicit unchecked integer constants wrap. Constant zero divisors are diagnosed.

Native emission supports Int32 class constants without static storage. Broader
constant-expression coverage, optional parameter declarations, lifted arithmetic,
nullable/pointer conversions and extension/interface mutation operators remain gaps.
[Operator contracts and supported forms](https://github.com/marinasundstrom/neoclr/tree/main/docs/validation/compound-operators#checked-blocks-and-constant-evaluation)
provide implementation details.

## Record values in development

Native positional records now support generated equality and matching hashes for
qualified components, including canonical String/Char text, integers, Boolean,
enums, nested source records and closed tagged-union payloads such as `Option<int>`.
This includes ConsoleKeyInfo's optional scalar. Nongeneric imported components
with an explicit `EquatableTo<Self>` contract and public typed equality/hash methods,
such as `Index` and `Range`, are also supported without boxing their components.
[Imported component validation](https://github.com/marinasundstrom/neoclr/tree/main/docs/experiments/record-components)
records the limits and allocation measurements. Typed and boxed record comparisons
are tested across interpreter, JIT and AOT. Generic record owners, unsupported union
payloads, floating-point components, imported value-record discovery and broader generated formatting remain gaps;
this is development support, not full record parity in a published release.
Supported records can explicitly implement `EquatableTo<Self>` through their
generated equality method and ordinary interface dispatch. Automatic native record
interface synthesis remains a gap; unsupported promised contracts are rejected.

Development native record formatting now covers scalar and nested positional record
structs, with named members, quoted text and boxed `ToString()` dispatch. It uses
one builder for nested values and typed scalar formatting; result and builder
storage still allocate, and the current 65,536 UTF-8-byte builder limit applies.
Closed declared union components now display their case and payloads through the same
builder; an uninitialized carrier remains distinct from an empty case. Enum,
imported-value and generic record formatting remain gaps. This is
pre-release compiler work, not a claim of full record support.

[Record validation and limits](https://github.com/marinasundstrom/neoclr/tree/main/docs/experiments/record-values)

<a id="targets"></a>

## The .NET and neoCLR targets

Raven normally targets .NET and can use its libraries. The compiler and Language
Server run on .NET; the VS Code extension uses that server for editor features.
neoCLR and its applications do not require .NET at runtime.

The neoCLR target emits native PE metadata and instructions directly. Both compiler
and editor import native library metadata, so completion reflects neoCLR's types
and members. System.Runtime is built from Raven source using explicit runtime
contracts; production library construction requires no CLI semantic core or
CoreProbe bridge. Compiler tooling running on .NET does not make host .NET types
available to a neoCLR program.

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

## Generics and numeric capabilities

Local and imported constructed generic classes support inherited storage, virtual
slots, abstract bases and concrete overrides. Generic virtual methods remain
unsupported. [Collection interfaces](../features/collections/#interface-variance)
provide reference-only variance through explicit nominal contracts.

System.Numerics provides AdditionOperators, SubtractionOperators, MultiplyOperators
and DivisionOperators using native Self. Number inherits these same-type
capabilities; generic algorithms can request only the operation they need.
Constrained calls work across library boundaries. A Complex demonstration exercises
arithmetic without imposing ordering; mixed-type contracts and production Complex
edge semantics remain future work.

Checked expressions fault on native overflow. Recoverable numeric APIs are separate
contracts: Int32.Divide returns an expected division error, while the source-local
Result-based addition example remains an API experiment.

## Fields and initialization

Native static readonly initialization is supported. Raven emits
field initializers and explicit static constructors, preserving strict or relaxed timing.
Source and imported consumers run in interpreter, JIT and AOT, including closed generic
owners. The [reflection APIs](/features/reflection/) describe readonly/literal fields and
type initializers, and execute public static field access through the same initialization
guards. Ordinary Raven/.NET behavior remains unchanged.
