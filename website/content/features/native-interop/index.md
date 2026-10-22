# Native libraries and foreign functions

**Development:** Raven `extern` functions and static methods can declare native
imports through `System.Runtime.InteropServices` attributes. The interpreter executes
these imports. A bounded development AOT profile additionally supports free Int32/pointer
C imports on macOS ARM64; general AOT/JIT and installed-bundle support remain open.
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

## Pass a Raven function to C

A development interpreter import can take a structural function type directly:

```raven
{{NATIVE_CALLBACK_IMPORT}}
```

The native fixture accepts an `int (*)(int)` and invokes it synchronously. The
built-in Raven test passes an ordinary module function:

```raven
{{NATIVE_CALLBACK_FUNCTION}}
```

The compiler records an explicit callback ABI using the import's calling convention.
The runtime keeps captured state and bound receivers alive for the call, including
across collection. Native code must call back on the same thread and must not retain
the callback after the imported function returns. Managed faults are reported after
control returns from native code; they do not unwind through C.

Numeric scalar parameters/results and no-result callbacks are supported. Raven keeps
`unit` as its keyword, mapped to neoCLR `System.Void`. A `(int) -> unit` parameter
can receive a Raven callback for C `void (*)(int)`; the compiler adapts its logical
unit result at the native boundary. This currently allocates a managed adapter per
call. Strings, pointers, nested callbacks and implicit Boolean widths inside callback
signatures are rejected. Retained callbacks and JIT/AOT callback execution remain open.

A native import can also return a function directly:

```raven
{{NATIVE_RETURN_IMPORT}}
```

The returned function remains callable after GC and can be passed back to C as a
callback. The runtime retains its loaded code library; foreign state still follows
the native library's lifetime contract. A returned `(int) -> unit` function maps C's
no-result call to a logical System.Void value, with a compiler adapter allocated once
per returned function. This source path is currently qualified in the interpreter.

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

Development interpreter `NativeMemory.Copy` overloads connect byte slices, Memory
windows and String.Utf8Bytes to live tracked native allocations without an
intermediate managed transfer array. They copy bytes explicitly and do not pin
managed storage. Native output must be initialized before the call under the current
tracking model. [Exact contract and limits](/docs/native-interop.html#native-byte-copies).

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
registration/unregistration lifetime contract. Raven scalar and unit callback parameters use
the source syntax above, including returned functions; JIT/AOT support remains open. The longer-term goal is an AOT library plus versioned public-interface
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
the bounded AOT profile transports raw pointer metadata but does not yet project
this public wrapper automatically.


## Development AOT import foundation

The low-level macOS ARM64 scalar backend accepts `--native-imports` for free C
functions taking up to sixteen Int32, native-sized integer or pointer arguments and
returning those types or no value (C void). Pointer metadata targets are Void, Byte or Int32; address transport through
locals/helpers is supported, with ownership remaining in the native contract. Generated code
resolves symbols lazily and retains successfully loaded libraries until process exit.
Lookup failures return a runtime fault without writing the result. The default
continues to reject imports; this opt-in requires explicitly linking the native
resolver. It is not yet available through the ordinary SDK application pipeline.

A tested Raven consumer now goes directly from NativeImport declarations to native
metadata and AOT machine code. It allocates, reads and releases C-owned memory through
raw pointers and a `unit`-returning helper, including typed/void-pointer casts and
null/default pointers and `nint`/`nuint` transport for C offsets and `size_t` values. The backend accepts an explicit seed/core
load context for verification and uses `@entry` for the Raven entry point; dependency
libraries are not compiled by this scalar profile.

Managed buffers, automatic Ptr<T> wrapper mapping, callbacks, other numeric types, full
class-library composition and managed JIT imports remain future work for this path.
[Execution contract](/docs/native-interop.html#aot-scalar-imports) ·
[Tests and reproduction](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-native-imports/README.md)

### Development IL spelling

No-result imports and helpers use `-> void`; no-result function signatures use
`fn<void>`. The obsolete `noresult` spelling is rejected. This syntax migration
preserves the existing native call ABI. `System.Void` now has the same no-result return semantics as `void`, including
function shapes. Removing the legacy bare `Void` spelling remains open; see the
[Void contract and migration status](https://github.com/marinasundstrom/neoCLR/blob/main/docs/void-semantics.md).

### Development AOT pointer checks

The opt-in macOS ARM64 scalar NativeImport profile accepts pointer equality and
null checks, and equality/zero tests for native-sized integers. Comparisons preserve
all address bits and do not dereference memory. Raven can check a native allocation
against `default(*int)` before use. A non-null pointer does not prove ownership,
allocation lifetime or accessible bounds; ordered pointer comparisons remain outside
this profile. Managed memory pinning remains unimplemented.

### Development AOT native memory access

The same trusted scalar profile supports direct Byte/Int32 pointer reads and writes.
Raven can dereference an imported allocation, update it and release it through the
native owner. Null and misaligned Int32 access returns a runtime fault. Bounds,
initialization and lifetime are the caller's responsibility; other invalid addresses
can crash the process. This does not expose managed storage or provide pinning,
native-backed Slice borrowing or managed-library AOT integration.

Future work is ordered around qualifying NativeImport across interpreter, JIT and
AOT first, followed by stable exported names and a callable ABI for AOT libraries.
Those exports should be usable by native consumers and importable back into neoCLR.
Metadata sidecars mapped to the native binaries are a later step; current internal
AOT symbols are not a published export contract.

The development `--jit-scalar` probe now accepts `--native-imports` on macOS ARM64,
sharing the AOT import thunks, native resolver and process-lifetime library cache.
Tests cover repeated allocation/read/write/release, native-sized results and loader
faults. A tested Raven consumer now executes through both scalar JIT and AOT using
explicit seed/core verification inputs and `@entry`. Dependencies remain verification-only;
this does not enable imports in the ordinary managed JIT pipeline. Default execution continues
to reject imports unless explicitly enabled.

A shared Raven transport sample is now tested across interpreter, scalar JIT and
AOT: C allocation/read/release, pointer casts/nulls, no-result calls and native-sized
integer values. C performs the reads in this shared sample. Direct access to
untracked foreign storage remains an interpreter gap; the separate direct pointer
read/write sample is qualified in scalar JIT/AOT only. This baseline does not mean
that full NativeImport support is complete across engines.


Fixed-width 64-bit transport (development, 2026-10-22): the opt-in macOS ARM64
scalar JIT/AOT import profile now admits Int64/UInt64 parameters, results and
locals, including private helper calls, defaults and equality/zero checks. The
shared Raven transport consumer exercises negative signed and high-bit unsigned
values against C int64_t/uint64_t functions across interpreter, JIT and AOT.
This follows ordinary .NET fixed-width integer ABI semantics; it adds no wrapper,
marshalling allocation or new metadata representation. Arithmetic expansion,
64-bit pointer dereference and broader managed import composition remain open.


Floating-point transport (development, 2026-10-22): the same opt-in scalar ABI
now admits Single/Double parameters, returns, locals, constants and defaults.
The shared Raven consumer checks negative zero, infinity, quiet-NaN payloads and
finite values through helper calls and mixed C arguments, including floating-point
arguments spilled to the stack. It preserves their declared F32/F64 representation,
following the platform C ABI as .NET interop does; no marshalling allocations are
introduced. Arithmetic, floating-point comparisons/conversions and pointer loads
remain outside this bounded profile.
