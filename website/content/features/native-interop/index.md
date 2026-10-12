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

## Loading and limits

Libraries load on first invocation; successful symbol resolution is cached and the
library remains alive for the execution. Metadata validation does not load native
code. Missing libraries/symbols produce a terminal Fault. Incorrect actual native
signatures or pointer lifetimes can corrupt memory or crash, so native execution
requires trust.

The low-level neoIL spelling is `.foreign`; `.pinvoke` remains a compatibility alias.
Explicit scalar and pointer ABI imports remain supported. Returned strings, bounded
input/output buffers, automatic record layout, error-state capture and callbacks need
further contracts. The current evidence covers macOS ARM64 interpreter execution,
including mixed encodings, boolean results and static class imports.

See the [attribute API reference](/docs/native-interop.html) for constructors,
properties, enum values, errors and ownership rules.

Metadata tools can inspect the immutable ABI descriptor without loading native code.
Raven preserves `extern` when reading a native import from another assembly.
