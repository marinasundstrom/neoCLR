# Namespace overview

Use this overview to find the part of the neoCLR library that fits your task.
It describes the **current development API following Preview 9**. Development
additions require matching artifacts; the published bundle may expose older names
or fewer APIs. Names and contracts remain experimental.

The list grows as APIs are implemented. It includes areas whose member reference
is still incomplete; links lead to generated reference pages or the available
on-site guides. A namespace here does not imply the full corresponding .NET API
is supported.

## Application namespaces

| Namespace | What it contains | Explore |
| --- | --- | --- |
| `System` | Core types such as Object, Value, HashCode, primitives, String and Array; Func delegates; Option and Result; common capability and error types; Console; dates, times, durations and clocks. Console is a class in this namespace. | [Core reference](xref:System), [Object and Value](objects.md), [Console](console.md), [arrays](/features/arrays/index.html), [outcomes](/features/outcomes/index.html), [dates and clocks](/features/time/index.html) |
| `System.Collections` | Iteration and collection capabilities: Iterable, Iterator, Collection, Sequence, List and map interfaces, with ArrayList and HashMap implementations. | [Collections and queries](/features/collections/index.html) |
| `System.Concurrency` | Explicit Thread lifecycle and ThreadPool execution using isolated workers. This development namespace replaces the Preview 9 System.Threading namespace. | [Reference](xref:System.Concurrency), [thread guide](index.md#explicit-threads-in-development) |
| `System.Environment` | Functions for command-line arguments, the current directory and environment-variable lookup. Lookup distinguishes a missing variable from a host access error through Result and Option. | Member reference coverage pending. |
| `System.Introspection` | Assembly, module, type, member and parameter descriptions, binding flags and metadata tokens for inspecting the loaded program. | [Introspection guide](/features/introspection/index.html) |
| `System.IO` | InputStream, OutputStream and optional SeekableStream capabilities; file byte streams; TextReader, TextWriter, StreamReader and StreamWriter; typed stream and text errors. Current text readers use bounded, strict UTF-8 reads. | [Reference](xref:System.IO), [stream guide](streams.md), [standard streams](console.md) |
| `System.Linq` | Query operators over iterable collections, including filtering, projection and terminal operations. Optional and single-result queries use explicit outcomes, with SingleError for cardinality failures. | [Collections and queries](/features/collections/index.html) |
| `System.Math` | Integer and floating-point functions, including Abs, Min, Max, Clamp, rounding, roots and trigonometry. Integer Abs reports overflow and Clamp reports an invalid range through Result. | [Expected outcomes](/features/outcomes/index.html); member reference coverage pending. |
| `System.Runtime` | RuntimeContext, the application entry point for the executing assembly and type information from runtime type handles. | [Runtime discovery](/features/introspection/index.html) |
| `System.Storage` | Paths, storage items, File and Directory interfaces, StorageProvider lookup, the host FileSystem provider, metadata and FileText helpers with typed errors. Items describe stored resources; their byte and text streams use System.IO. | [Reference](xref:System.Storage), [storage items](storage-items.md), [providers](storage-provider.md), [working POC](storage-poc.md) |
| `System.Tasks` | Task, Promise, TaskQueue, completion state, TaskOutcome and composition/propagation helpers. Tasks describe completion and cancellation; expected errors remain values such as Result. | [Reference](xref:System.Tasks), [Tasks guide](/features/tasks/index.html), [callback methods](callbacks.md) |
| `System.Text` | UnicodeScalar helpers, strict Utf8 encoding/decoding and UTF-8 error types. String itself lives in System. | [Strings and UTF-8](/features/strings/index.html) |

## Runtime and compiler support

These namespaces serve lower-level integration rather than ordinary collection,
text or I/O workflows. Their member reference coverage is still pending.

| Namespace | What it contains |
| --- | --- |
| `System.Runtime.CompilerServices` | Compiler-facing metadata such as UnionAttribute and [IsExternalInit](xref:System.Runtime.CompilerServices.IsExternalInit), plus runtime integration support. Internal bootstrap services are not application APIs. |
| `System.Runtime.InteropServices` | NativeMemory allocation and freeing for the supported unsafe pointer surface. This is separate from managed object and array storage. |

## Finding a type

Expand namespaces in the [generated reference](xref:System) to browse documented
types and members. Some areas currently have only a feature guide; this overview
does not replace their eventual member documentation. Generated declarations use
C# metadata notation; the [API guide](index.md#reading-generated-declarations)
explains how to read them for Raven.

Future namespace ideas belong in [proposals](/proposals/index.html) until they have
implemented APIs. This list will expand alongside the library.

The [transitional async builder guide](async-builders.md) covers generated state ownership and links the compiler-facing System.Runtime.CompilerServices contracts to their temporary role.

[Equatable<T>](xref:System.Equatable`1) defines typed equality as Equals(T), without
imposing nullable value types. Path now supplies that contract and Object overrides.

## System.Introspection

[TypeInfo](xref:System.Introspection.TypeInfo) and [MemberInfo](xref:System.Introspection.MemberInfo) describe types and metadata. See [type identity](introspection.md) for Object behavior and remaining descriptor coverage.
