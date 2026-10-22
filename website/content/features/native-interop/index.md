# Native libraries and foreign functions

**Development:** Raven `extern` functions and static methods can declare native
imports through `System.Runtime.InteropServices` attributes. The interpreter executes
these imports. Development scalar JIT/AOT profiles also support numeric/pointer
C imports on macOS ARM64; ordinary SDK integration remains open.
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


## Development native execution

The macOS ARM64 scalar JIT/AOT backend accepts `--native-imports` for free C
functions with up to sixteen arguments. Supported numeric types are Int32/UInt32,
Int64/UInt64, IntPtr/UIntPtr and Single/Double; pointer targets are Void, Byte and
Int32. Returns use those types or C void. The shared Raven consumer is tested in
all three engines, including unsigned widening/truncation, negative zero, infinity
and quiet-NaN payloads through mixed register/stack arguments.

| Capability | Interpreter | Scalar JIT / AOT |
| --- | --- | --- |
| Tested numeric/pointer transport and no-result calls | Yes | Yes, explicit opt-in |
| C-owned allocation/read/release | Yes | Yes |
| Direct access to untracked foreign Byte/Int32 storage | Not yet | Trusted raw access; null/alignment checks |
| String/Boolean marshalling and Function callbacks | Bounded support above | Not yet |
| Managed-memory pinning and automatic Ptr<T> projection | Not yet | Not yet |
| Import an AOT-built free-function library | Tested | Tested |

Libraries resolve lazily. The scalar resolver retains successful library handles for
process lifetime; invalid addresses and native ABI violations remain the caller's
responsibility. Explicit seed/core contexts verify Raven metadata, while dependency
bodies remain outside this profile. These are development backend capabilities,
not a claim that ordinary SDK/managed JIT imports are complete.

## AOT libraries callable through NativeImport

A development `--native-library` mode emits selected free functions as platform C
exports. Versioned names encode source assembly, namespace, function and scalar
signature; content hashes and definition row numbers do not affect them. A native C
caller and the same Raven NativeImport consumer test the produced library.

Export adapters return the declared native result on success. An escaping guest
fault terminates execution: it does not unwind through C or return a fabricated
value. NativeExportAttribute source integration is the next step; explicit CLI
selection currently exercises the backend. Class/object mapping and metadata
sidecars are future work.

[Execution and attribute reference](/docs/native-interop.html) ·
[Contract, tests and reproduction](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-native-imports/README.md)

### Assembler spelling

No-result imports and exports use `-> void`, an alias for `System.Void`.
The obsolete `noresult` spelling is rejected. Remaining legacy spelling migration
is tracked in the [Void contract](https://github.com/marinasundstrom/neoCLR/blob/main/docs/void-semantics.md).
