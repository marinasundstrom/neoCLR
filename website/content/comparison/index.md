# neoCLR and .NET

neoCLR explores an independent managed application platform with familiar .NET
concepts and some different contracts. It is experimental: CLI metadata is a
compiler boundary, not a promise that existing .NET applications or NuGet packages
will run unchanged.

**Reviewed 27 September 2026.** This page describes Preview 11. Use matching runtime and compiler
artifacts. See [installation and availability](../try/) before running an example.
The comparison concerns modern .NET; language syntax, library policy and runtime
behavior are distinguished below.

## Similarities

Both platforms have value and reference types, generics, interfaces, managed
memory and metadata. neoCLR uses Raven as its current application language.
Raven also targets .NET, so Raven syntax by itself is not a neoCLR difference.
The runtime and System library determine the target contracts.

| Shared concept | What carries over |
| --- | --- |
| [Values and references](../docs/objects.html) | Struct values are copied; class and array references can share an object. Identity and value equality remain different questions. |
| [Generics](../raven/) | Types and methods can describe operations over type parameters. The familiar idea carries over even though import coverage and constraints must be checked. |
| [Interfaces and dispatch](../docs/objects.html) | Interfaces describe contracts, and implementations supply behavior. neoCLR supports ordinary dispatch and bounded default-interface behavior; this does not imply support for every .NET interface feature. |
| [Managed memory](../features/gc/) | Reachable objects are retained by a collector. Applications still need explicit lifetime handling for resources such as files and connections. |
| [Metadata](../features/introspection/) and [async code](../features/tasks/) | Types and members have discoverable metadata, and async methods can await task completion. Metadata coverage, scheduling and failure contracts differ. |

## Deliberate differences

| Area | .NET baseline | neoCLR contract and tradeoff |
| --- | --- | --- |
| Text | `String` stores UTF-16 code units; `Char` is one code unit. See [Microsoft’s encoding guide](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction). | [String uses UTF-8 and Char represents a grapheme](../features/strings/). This makes character operations closer to displayed text, but grapheme boundaries require segmentation. Byte, scalar and character positions are different; algorithms assuming fixed-width characters need adaptation. |
| Arrays | Reference arrays support covariance, with runtime checks on stores. See [.NET variance](https://learn.microsoft.com/en-us/dotnet/standard/generics/covariance-and-contravariance). | [Mutable arrays are invariant](../features/arrays/). This prevents writes through a covariant array view; code needing a more general element type must use a suitable abstraction or conversion. |
| Absence and expected errors | APIs commonly use null, exceptions or `Try` methods. .NET languages and libraries can also model typed outcomes. | [Option and Result](../features/outcomes/) are the standard library’s explicit outcome model. Callers match or propagate cases; API signatures expose expected failure types. Porting exception-based code requires revisiting those contracts. |
| Completion without a value | [.NET Void](https://learn.microsoft.com/en-us/dotnet/api/system.void?view=net-10.0) represents a method without a return value. .NET uses separate shapes such as `Action` and `Func<T>`, or `Task` and `Task<T>`. | [Void as a generic argument](../raven/#callbacks) is represented as unit in Raven. `Task<unit>` and `Result<unit, E>` reuse generic families. This requires compiler/import support and is not ordinary .NET binary compatibility. |
| Async failures | Tasks can complete successfully, be cancelled or become faulted; [exceptions are retained by faulted tasks](https://learn.microsoft.com/en-us/dotnet/standard/parallel-programming/exception-handling-task-parallel-library). | [Tasks](../features/tasks/) complete with a value or cancellation. Expected errors belong in `Result`; a terminal Fault is separate and can fail the invocation. This makes expected failures explicit, but changes recovery and supervision assumptions. |
| Runtime discovery | .NET reflection provides metadata discovery and execution, subject to deployment constraints such as trimming and AOT. | [Introspection](../features/introspection/) exposes descriptors; **Development:** [Reflection](../features/reflection/) adds bounded constructor, method and field execution with typed failures. The separation makes intent explicit, but coverage is smaller and needs checking per operation. |

These are design choices, not evidence that neoCLR is universally faster, safer or
easier to use. A change can simplify one contract while increasing compiler,
conversion or application work.

## Implementation differences and current gaps

| Area | Current neoCLR position |
| --- | --- |
| [Execution and GC](../features/gc/) | A Rust interpreter with a nonmoving tracing collector. There is no JIT backend. Modern .NET includes JIT and [Native AOT deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/), and a [generational collector](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/fundamentals). These differences do not establish a performance advantage; comparisons need matched workloads. |
| [Toolchain](../try/) | Raven compilation, the import bridge, MSBuild and language tooling use .NET. Guest execution uses neoCLR without a .NET runtime. The importer supports a bounded CLI subset; compile against the matching neoCLR reference library. |
| Libraries | Collections, queries, text, time, streams, storage, tasks, networking and HTTP/JSON support useful small programs. The [guides](../guides/) and [API reference](../docs/) define the actual surface; familiar names do not imply full .NET API parity. |
| Work submission | **Development:** `Task.Run` supports shared captures and returns an awaitable task. Guest instruction intervals share a managed graph gate, so this does not promise parallel guest CPU execution. Earlier isolated-worker APIs have different sharing rules. See [Tasks](../features/tasks/#task-run). |
| Web applications | Preview 11 includes bounded typed JSON client/server exchanges, nested JSON, typed arrays and routing. Cleartext HTTP/1.1 and bounded payloads are not an ASP.NET Core replacement. See the [HTTP guide](../features/web/) and [server case](../cases/http-server/). |
| Readiness | Experimental contracts and bounded validation. Import coverage, async shapes, terminal-fault cleanup and packaging still need work. This is a platform to investigate through working cases, not a production compatibility layer. |

## Coming from .NET

Start with a small [tested case](../guides/#cases-apis-in-context), then check the
actual API signatures. Pay particular attention to text positions, array
conversions, expected errors, cancellation and resource lifetime. Build with the
matching SDK and runtime rather than copying an existing .NET assembly.

Performance, deployment size and reliability need workload-specific evidence.
Future execution backends and broader framework coverage remain [directions and
proposals](../proposals/), not capabilities to depend on today.

The linked feature pages own detailed contracts and limitations. This comparison
is reviewed when those contracts or release availability change; its review date
records the last check, not a release date. Microsoft references above were
consulted on 27 September 2026.

## Function types: delegates evolved (development)

[Function types](../features/functions/) preserve the familiar target and receiver
binding of a single-target delegate while making the callable signature a structural
type. Matching signatures share a Function type, so callbacks can use that contract
directly without sharing a named delegate declaration. .NET delegate types instead
have nominal identity.

Function objects support value equality, typed Invoke and target inspection through
the Function property, which returns MethodInfo. FunctionTypeInfo describes the
signature independently of the bound target.
