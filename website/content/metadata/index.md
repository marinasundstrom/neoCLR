# Metadata and compiled libraries

Metadata describes the contents of a compiled neoCLR assembly: its modules, types,
functions, constants and references to other assemblies. It lets the compiler and
editor understand a library without its source files, and lets the runtime resolve
and check the code it executes.

## One library, several uses

Suppose an application imports a separately compiled library. The compiler reads its
public declarations and checks calls against their signatures. The editor uses those
same declarations for completion and navigation. At execution time, the runtime binds
the application's references to the supplied libraries and verifies instruction bodies.
Reading a library's metadata does not run its application code.

[Modules](../features/modules/) form namespaces for declarations. Assemblies package
those modules and supply dependency identity. Two assemblies can declare modules with
the same name without becoming the same owner.

<a id="what-an-artifact-contains"></a>
<a id="neox-framing-and-payload"></a>

## The artifact layers

![PE transport, NEOX framing, execution encoding and semantic module](../metadata-format.svg)

Native metadata and instructions are carried in a **NEOX** container. It can be stored
as a standalone file or inside a PE file's **#Neo** stream. The layers separate file
framing, payload encoding and the meaning of the declarations and instructions.

| Layer | Purpose |
| --- | --- |
| PE wrapper, when present | Carries metadata streams in a familiar executable-file container |
| NEOX envelope | Identifies versioned sections and their boundaries |
| Native payload | Encodes declarations, references and instruction bodies |
| Resolved program | Binds those references to the actual supplied dependencies |

The current binary payload uses a restricted CBOR object encoding. JSON remains a
legacy/intermediate representation. Neither representation contains live objects,
open files or other execution state.

<a id="pe-is-a-container-not-a-compatibility-promise"></a>
<a id="validation-and-evolution"></a>

## Compatibility and validation

A PE file is not automatically a .NET assembly that the CLR can execute. neoCLR runs
the native model carried by `#Neo`; ordinary CLI tools may recognize the outer file
without understanding that model. The [platform comparison](../comparison/) explains
the broader relationship to .NET.

The runtime checks container structure and payload encoding, then resolves dependencies
and verifies executable code. A file that parses successfully has only passed the first
part of that process. Versioned contracts allow readers to reject features they cannot
preserve rather than silently misinterpret them.

For application development, keep compiler, library and runtime artifacts together as
shown in [setup](../try/). [Introspection](../features/introspection/) exposes supported
metadata views to programs; [reflection](../features/reflection/) adds controlled
construction and invocation.

## Implementing a reader or writer

The [repository format specification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/metadata-format.md)
covers byte layout, schema versions, size budgets, ownership validation and test evidence.
The [architecture guide](../architecture/) explains how metadata fits into the platform.
