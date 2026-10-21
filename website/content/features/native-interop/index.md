# Native libraries and foreign functions

**Development:** Raven `extern` functions and static methods can declare native
imports through `System.Runtime.InteropServices` attributes. The interpreter executes
these imports; JIT/AOT lowering and installed compiler bundles remain unqualified.
The interface describes native libraries, ABI and value representation on any host.

## Declare the native contract

The compiled foreign-function tests exercise this shape:

```raven
import System.Runtime.InteropServices.*

[NativeImport("sample", EntryPoint: "neoclr_bool32")]
[return: MarshalAs(NativeType.Bool32)]
extern func Test([MarshalAs(NativeType.Bool32)] value: bool) -> bool;
```

`extern` supplies no managed body. NativeImport records the library and exact export;
EntryPoint can be omitted when the names match. CallingConvention selects Cdecl or
System. System follows the platform ABI, using stdcall only on 32-bit Windows.
That 32-bit distinction is not yet qualified by local execution evidence.

## Describe each value explicitly

| Setting | Contract |
| --- | --- |
| NativeType.Bool8 / Bool32 | One-byte or four-byte integer booleans. Inputs use 0/1; nonzero results are true. |
| NativeType.Utf8 / Utf16 | Terminated input String buffers, owned for the duration of the call. |
| NativeImport.StringMarshalling | Optional Utf8/Utf16 default for unannotated String parameters; None otherwise. |
| Parameter MarshalAs | Overrides the function default for that parameter. |

No platform Boolean width or string encoding is guessed. Incompatible descriptors
are rejected during compilation and runtime metadata validation. Native code must
not modify, free, retain or return addresses into temporary string buffers. Embedded
NULs fault instead of silently truncating text. UTF-16 is 16-bit code units on every
host, not C's host-dependent wchar_t.

## Managed and native buffers

Native libraries can use explicit pointer-and-length parameters. A foreign-owned
buffer must be released through its matching native API; retaining its pointer does
not transfer ownership to neoCLR. Raw pointer imports currently execute in the
interpreter.

Slice and Memory remain managed views. A future scoped native borrow must guarantee
both address stability and owner lifetime, while persistent native-backed Memory
needs a disposal/lease contract. Neither bridge is implemented yet. Imported
functions do not need to accept Slice or Memory themselves: a library wrapper can
provide the managed API around a conventional C ABI.

Arbitrary native imports under JIT/AOT remain an evaluation gap, including when
using the development editor's default JIT task.

## Loading and limits

Libraries load on first invocation; successful symbol resolution is cached and the
library remains alive for the execution. Metadata validation does not load native
code. Missing libraries/symbols produce a terminal Fault. Incorrect actual native
signatures or pointer lifetimes can corrupt memory or crash, so native execution
requires trust.

The low-level neoIL spelling is `.foreign`; `.pinvoke` remains a compatibility alias.
Explicit scalar and pointer ABI imports remain supported. Returned strings, bounded
input/output buffers, automatic record layout, error-state capture and retained callbacks need
further contracts. The current evidence covers macOS ARM64 interpreter execution,
including mixed encodings, boolean results and static class imports.

See the [attribute API reference](/docs/native-interop.html) for constructors,
properties, enum values, errors and ownership rules.

Metadata tools can inspect the immutable ABI descriptor without loading native code.
Raven preserves `extern` when reading a native import from another assembly.

Runtime-owned internal calls remain separate from foreign imports. Both use `extern`,
while `MethodImpl(InternalCall)` identifies a runtime service and `NativeImport`
identifies a library/export. Metadata inspection preserves both distinctions.

The runtime supports binding a native
address to a structural Function signature in the interpreter. It also accepts
managed Functions as call-scoped native callbacks and wraps returned native callable
pointers as Functions. Each callable has its own convention and conversion metadata;
captured managed state stays alive through the call. The initial callback scope is
same-thread scalar signatures. Native APIs that store callbacks need a future
registration/unregistration lifetime contract. Raven mapping syntax and JIT/AOT support
remain next. The longer-term goal is an AOT library plus versioned public-interface
metadata for runtime and native consumers. Export stubs and that sidecar format are
not implemented yet.


## Typed pointers in development

[`System.Runtime.InteropServices.Ptr<T>`](/docs/typed-pointers.html) wraps a borrowed native `*T`, with
`T: struct`. It supports typed address equality through `==`, `!=` and `Equals`,
with consistent value hashing. Copying it copies only the pointer; callers retain
responsibility for allocation lifetime and the matching release operation.
`Pointer` exposes the raw address for native calls. The built-in native-pointer
suite tests aliases, null defaults and equality/hashing across boxing and collection.

A struct constraint does not establish an unmanaged layout. Native layout checks
still apply. Pointer arithmetic and safe native Slice borrowing remain future work;
this wrapper does not change the interpreter-only native import limitation.
