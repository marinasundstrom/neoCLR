# Architecture of neoCLR

neoCLR brings together a managed runtime, a Raven class library, compiler integration
and development tools. The layers are familiar from .NET, but the runtime owns its
metadata, execution and service contracts. It is an independent experimental platform,
not a runtime for arbitrary .NET applications.

## Modules organize the library

The class library and its APIs are organized into **modules that form namespaces**.
Modules are the primary means of organizing neoCLR code. A module owns types,
functions and constants; its name supplies their namespace for qualified lookup and
imports. This makes the organization explicit and easy to follow.

For example, `System.Runtime` is an assembly containing modules such as `System`,
`System.Math` and `System.Time`. Assemblies provide packaging and dependency identity,
while modules organize declarations. The [Modules feature page](../features/modules/)
explains declarations, imports, ownership and the current limits.

## Runtime layers

![Applications, library, managed execution, host services and operating system](../runtime-layers.svg)

The boxes group responsibilities, not separate processes or an exhaustive API inventory.
Applications call Raven library code for collections, JSON, Tasks, Storage and HTTP.
That code runs through the interpreter or supported native backend. Runtime primitives
provide calls, managed storage, garbage collection, text, numeric operations and queues.
Host adapters connect those operations to external resources such as files and sockets.
Not every service has identical interpreter and AOT coverage yet.

For example, an HTTP handler uses the Web library to parse and route a request. Library
code uses networking services to read and write bytes; runtime support maintains managed
objects and roots while the OS supplies the socket. See the working
[HTTP case](../cases/http-server/) and [native compilation evidence](../features/native-compilation/).

## From source to execution

![Source, metadata, resolution, verification and execution pipeline](../architecture.svg)

[Raven](../raven/) binds source and emits metadata and instruction bodies through its
neoCLR target. The neoIL assembler supplies another input path for low-level programs
and runtime tests. Native library metadata also supplies symbols to the compiler,
language server and documentation tools.

The loader checks the artifact's structure, resolves its declared dependencies and binds
references into a prepared program. Typed verification then checks instruction and call
contracts. Decoding a file alone does not establish that its code is valid. Execution
creates frames and managed state from this prepared model; metadata contains no live
heap objects or OS handles. Read the [metadata format](../metadata/) for the artifact layers.

## Runtime and library responsibilities

| Layer | Responsibility |
| --- | --- |
| Compiler and project tools | Source semantics, native symbol import, target emission and editor integration |
| Metadata, loader and verifier | Definition identity, dependencies, signatures and executable-code checks |
| Execution backend | Calls, control flow, storage operations and the selected execution mode |
| Runtime primitives and services | GC, text/numbers, initialization, scheduling, Faults and host resource access |
| Raven library | Modules organizing application APIs, built from library code and explicit service contracts |

The current Raven target has copied values and reference-semantic classes and arrays.
Managed byrefs address slots; unmanaged pointers have a separate interop contract.
Strings use UTF-8, with distinct byte, scalar and grapheme operations.
[Option and Result](../features/outcomes/) express absence and expected errors;
terminal Faults are a separate runtime mechanism. These choices affect libraries,
verification and backend code, not just Raven syntax.

## Interpreted and native execution

The Rust interpreter runs the native instruction model without requiring .NET to be
installed. Raven's compiler and editor tooling have their own .NET dependency.
The [ARM64 AOT proof of concept](../features/native-compilation/) lowers supported
metadata/IL and links runtime support into a standalone executable. Checked cases
include routing and bounded HTTP serving; general interpreter/native parity is open.
A native executable still needs implementations of GC, text, queues, I/O and Faults.

JIT, general native AOT, native hot reload and trimming remain future work. The native
bootstrap also retains explicit primitive-core/runtime inputs; source-built library
coverage does not establish a fully independent production core. Use the matching
artifacts described in [setup](../try/).

## Relationship to .NET

The assembly, type, generic and managed-execution concepts aim to preserve familiar
.NET ergonomics. UTF-8 text and explicit service boundaries make different platform
choices possible, but require target-specific libraries and tooling. Familiar names or
a PE container do not establish CLR binary compatibility or a performance advantage.
The [platform comparison](../comparison/) explains the practical differences.

For implementation detail, see the [architecture document](https://github.com/marinasundstrom/neoCLR/blob/main/docs/architecture.md).
