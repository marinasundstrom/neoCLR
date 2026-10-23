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

The runtime separates three central responsibilities:

- **Execution engines:** JIT and interpreter execute managed programs.
- **Memory management:** allocation, roots, GC and object lifetimes.
- **Scheduling and suspension:** ready work, I/O waits, suspension and resumption.

Managed Tasks and Promises describe operations above these services. A suspended
operation keeps its live state rooted until it resumes or is abandoned. Metadata,
types, faults, diagnostics and platform adapters support the components.

**AOT is a publishing target.** Its output uses the runtime services it needs without
requiring a JIT or interpreter. Integration keeps these components part of one product;
it does not require one implementation or identical coverage on every target.

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

The runtime resolves the application's library dependencies and checks that its
instructions and calls are valid before execution. Metadata supplies declarations and
code; execution creates the live objects and other program state. Read
[metadata and compiled libraries](../metadata/) for how those artifacts connect the tools.

## Runtime and library responsibilities

| Layer | Responsibility |
| --- | --- |
| Compiler and project tools | Source semantics, native symbol import, target emission and editor integration |
| Metadata, loader and verifier | Definition identity, dependencies, signatures and executable-code checks |
| Execution engines | JIT and interpreter; calls, control flow and execution contracts |
| Memory management | Allocation, roots, object lifetimes and collection |
| Scheduling and suspension | Ready work, I/O waits and suspended activation lifetimes |
| AOT publishing | Native compilation and linking with required runtime support |
| Supporting runtime services | Type services, text/numbers, initialization, faults, diagnostics and host resource access |
| Raven library | Modules organizing application APIs, built from library code and explicit service contracts |

The current Raven target has copied values and reference-semantic classes and arrays.
Managed byrefs address slots; unmanaged pointers have a separate
[native interop contract](../features/native-interop/).
Strings use UTF-8, with distinct byte, scalar and grapheme operations.
[Optional and Result](../features/outcomes/) express absence and expected errors;
terminal Faults are a separate runtime mechanism. These choices affect libraries,
verification and backend code, not just Raven syntax.

## Interpreted and native execution

The Rust interpreter runs the native instruction model without requiring .NET to be
installed. Raven's compiler and editor tooling have their own .NET dependency.
The [native AOT target](../features/native-compilation/) lowers supported
metadata/IL and links runtime support into a standalone executable. Checked cases
include routing and bounded HTTP serving; general interpreter/native parity is open.
A native executable still needs implementations of memory management, scheduling, text, I/O and faults.

JIT bundles default to native JIT execution on macOS ARM64 and
Windows x64. Interpreter execution remains available explicitly and is the default
on other hosts. JIT currently compiles eagerly through a separate backend process.
Broader native parity, native hot reload and trimming remain future work.
[Setup](../try/) describes the available toolchain, and
[native compilation](../features/native-compilation/) lists its execution limits.

Runtime-await retains locals and cleanup across suspension on macOS ARM64 and
Windows x64. Compiler state machines remain an explicit fallback. Matching compiler,
library and runtime contracts are required; see [Tasks and async](../features/tasks/).

Memory-management boundaries also leave room for future specialized distributions;
swappable collectors or general non-GC managed execution are not current features.

## Relationship to .NET

The assembly, type, generic and managed-execution concepts aim to preserve familiar
.NET ergonomics. UTF-8 text and explicit service boundaries make different platform
choices possible, but require target-specific libraries and tooling. Familiar names or
a PE container do not establish CLR binary compatibility or a performance advantage.
The [platform comparison](../comparison/) explains the practical differences.

For implementation detail, see the [architecture document](https://github.com/marinasundstrom/neoCLR/blob/main/docs/architecture.md).
