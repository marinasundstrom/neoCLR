# Foreign-function attributes

**Development, interpreter execution:** `System.Runtime.InteropServices` contains
NativeImportAttribute, MarshalAsAttribute, NativeCallingConvention,
NativeStringMarshalling and NativeType. Matching source compiler/runtime changes are
required; installed bundles and general JIT/AOT imports are not yet qualified.
A separate [bounded scalar AOT profile](#aot-scalar-imports) supports Int32/pointer C imports.

| API | Contract |
| --- | --- |
| `NativeImportAttribute(libraryName: string)` | Metadata for an extern namespace function or static method. The name/path is required and must contain no NUL when used as an import. Construction does not load native code. |
| `LibraryName: string` (read-only) | Supplied library loader name/path. |
| `EntryPoint: string` (read/write) | Exact export; omission uses the source name. Explicit empty/NUL names are rejected at emission. |
| `CallingConvention: NativeCallingConvention` (read/write) | Cdecl (0, default) or System (1). System uses stdcall on 32-bit Windows and C elsewhere. |
| `StringMarshalling: NativeStringMarshalling` (read/write) | None (0, default), Utf8 (1), Utf16 (2). Applies only to otherwise unannotated String parameters. |
| `MarshalAsAttribute(nativeType: NativeType)` | Overrides one parameter or result. No conversion occurs when constructing the attribute object. |
| `MarshalAsAttribute.Value: NativeType` (read-only) | Utf8 (0), Utf16 (1), Bool8 (2), Bool32 (3). |

Utf8/Utf16 support input String parameters only. Buffers are NUL-terminated and live
through one call. Embedded NULs fault before entry; buffers must not be modified,
retained, freed or returned by native code. UTF-16 means native-endian 16-bit code
units, not platform wchar_t. Empty strings have a non-null terminator-only buffer.

Bool8/Bool32 support Boolean parameters and `[return: MarshalAs(...)]` results.
Input is 0/1; any nonzero result is true. Without a descriptor Boolean is rejected.
Parameter overrides win over the function's string default. Scalar/pointer arguments
use their declared ABI directly; incompatible descriptors are rejected.

Diagnostics reject unsupported conventions, signatures, attribute owners, duplicate
or invalid attributes, generics, instance imports, managed bodies and unspecified
string/boolean conversion. Runtime validation also checks descriptors, parameter
indices and body/entry-point restrictions. Missing library/export produces a terminal
Fault on first invocation. Incorrect actual native signatures or lifetimes can corrupt
memory or crash; they are not generally discoverable from export metadata.

See the [tested source shape and full contract](https://github.com/marinasundstrom/neoclr/blob/main/docs/native-interop.md) and
[website guide](/features/native-interop/). Returned strings, buffers with declared
length/ownership, record layout, platform-error capture and retained callbacks remain open.
The generated type/member reference includes all five types; no API is excluded.

## Raven callback parameters (development, 2026-10-22)

A NativeImport parameter may be a structural function, such as `(int) -> int` or
`(int) -> unit`. The callback uses the import's convention. Numeric scalar arguments
and numeric/no-result returns are supported; native code must invoke it synchronously
on the calling thread and must not retain it. Captures and receivers remain rooted
through the call. Callback faults are delivered after native code returns.

Raven retains `unit` syntax, mapping logical values to `System.Void`. For a unit
callback the native emitter generates a managed source-facing wrapper and an internal
C import with a true no-result Function parameter. The adapter invokes the managed
Function and discards its unit value. This also preserves a generic return closed over
unit. The source-facing wrapper is an ordinary managed method in emitted metadata;
NativeImport metadata belongs to the internal thunk. The adapter currently allocates
per call; no allocation-free callback claim is made. There are no new guest attributes.
An import may also return `(int) -> int` or `(int) -> unit`, mapping the native function
pointer to an ordinary Raven Function. The runtime retains loaded libraries through
that binding; it does not own foreign context/state. Null or tracked-data addresses
fault at binding. For unit results, the compiler emits an adapter around the returned
no-result Function, materializing System.Void after each invocation. That adapter and
binding allocate once per returned result; ordinary numeric results need no additional
compiler adapter. The result can itself be passed as a call-scoped callback.
Callback strings, Boolean widths, pointers, nested callbacks, retained registration
lifetimes and JIT/AOT callable execution remain open.

## Structural pointer binding experiment

Main includes development interpreter/neoIL address mapping
through `function.bind.native`. It produces the existing structural Function type,
not a delegate. [Contract and restrictions](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-interop.md#structural-native-function-bindings-development).
No new guest attribute or RavenDoc type is introduced.

The Rust metadata API adds `metadata::NativeAbi` and
`Instruction::BindNativeFunction { function_type: Type, abi: NativeAbi }`. The ABI
record has public fields `calling_convention: CallingConvention`,
`string_marshalling: Option<NativeStringMarshalling>`,
`parameter_marshalling: BTreeMap<String, NativeMarshalling>` and
`return_marshalling: Option<NativeMarshalling>`. `From<&NativeImport>` copies only
ABI/conversion fields, excluding library/export identity. Validation uses the same
rules as named imports; zero-based keys must be canonical decimal strings. The
function_type must be a supported structural Function signature. The instruction
consumes a pointer and produces that signature's Function value. Unsupported shapes,
conversions and output contracts fail validation; null/tracked data addresses fail
at binding. Loading metadata never resolves the pointer.

Trusted execution is required both to bind and invoke. Native wrappers retain the
loaded library owners, but external pointers still need an explicit host lifetime
guarantee. Clones share binding identity; independent mappings are distinct.
Direct scheduler submission requires a managed wrapper. The independent C# metadata
API and Raven emitter do not yet author this instruction; public AOT exports and
sidecar generation remain [design work](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/native-import-export-contract.md).


<a id="callable-abi-metadata-development"></a>

## Callable ABI metadata

Both Rust `metadata::NativeImport` and `metadata::NativeAbi` additionally expose:

| Field | Type and meaning |
| --- | --- |
| `callback_parameters` | `BTreeMap<String, NativeAbi>`: canonical zero-based parameter index to that callable's independent ABI. Empty by default and omitted in serialization. |
| `function_return` | `Option<Box<NativeAbi>>`: ABI of a returned native pointer mapped to the declared structural Function. None by default and omitted in serialization. |

`From<&NativeImport>` copies these fields as well. The new public fields require
updates to Rust struct literals; wire records without them retain their previous
meaning. A matching runtime is required for records containing them. No new guest
attribute or generated RavenDoc type is introduced; this manual entry documents the
experimental Rust API. The independent C# metadata API now authors the bounded
scalar subset through NativeCallbackDefinition, CallbackParameters and FunctionReturn;
see [host API reference](experimental-metadata.md#native-imports-development-2026-10-12).

Callback parameters require an exact non-null Function and a scalar callable shape.
Numeric types, explicit Bool8/Bool32 and void/no-result returns are supported. The
runtime retains the captured receiver, generates a call-duration entry stub and
reenters the same managed invocation on the calling thread. Managed faults are caught
at the stub, return zero/false to native code and propagate when the foreign call
returns. Later callbacks in that call do not execute guest code after a fault.
Native code must not retain the pointer. Retention beyond the call needs a future
explicit ownership/release contract. The author's extern examples illustrate callable
parameters and results; they do not prescribe registration behavior. Suspension and
other-thread entry are unsupported.

A returned pointer becomes a library-retaining Function using function_return.
Null, tracked-data and current temporary callback addresses are rejected. Provider
library retention cannot guarantee arbitrary native state/code lifetime. Callable
ABI nesting is limited to eight; native callback reentry is limited to 32 and the
remaining managed frame budget. See [the tested example and complete boundaries](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-interop.md#callable-parameters-and-results-development).


<a id="native-byte-copies"></a>

## NativeMemory.Copy (development)

The `System.Runtime.InteropServices.NativeMemory` module adds two unsafe overloads,
implemented by `System.Runtime.dll` and currently executable in the interpreter:

```raven
unsafe func Copy(source: ReadOnlySlice<byte>, destination: *System.Void, destinationByteCount: nuint)
unsafe func Copy(source: *System.Void, sourceByteCount: nuint, destination: Slice<byte>)
```

The first copies all source bytes; destinationByteCount is the available capacity
and must be at least source.Length. The second copies exactly sourceByteCount bytes
and leaves the rest of the destination unchanged. Memory windows project through
existing slice conversions; String.Utf8Bytes can be used as read-only input. These
operations create no intermediate managed transfer array and retain neither buffer.
They perform explicit copies; they do not pin managed storage or transfer ownership.

Both overloads require live, tracked NativeMemory.Alloc storage for nonempty native
ranges. Empty copies do not dereference pointers and accept a null address. Invalid
ranges, insufficient capacity, uninitialized source bytes, read-only output, freed
storage and native allocations currently borrowed by a foreign call cause runtime
faults. Validation occurs before writes for supported single-consumer use; concurrent
mutation of the managed buffer is unsupported. These checks do not make arbitrary
foreign addresses safe or admit foreign-owned pointers.

Foreign writes do not currently update the interpreter's initialization tracking.
Initialize the native range before passing it to an imported function that will
mutate it; a C output write alone does not qualify an uninitialized range for Copy.
The built-in native-buffers fixture copies input first, invokes an in-place C byte
increment, and copies back into a Memory subview. JIT/AOT execution, true pinning
and persistent native-backed Memory ownership remain open.


<a id="aot-scalar-imports"></a>

## Bounded AOT import execution (development)

`neoclr-codegen --native-imports` enables the macOS ARM64 scalar AOT backend's
existing NativeImport descriptors for nongeneric free functions with at most 16
parameters, using Int32, IntPtr/UIntPtr or raw pointer metadata to Void, Byte or Int32 for parameters
and results, or no-result Void returns, without marshalling. Pointer locals and helper calls preserve native
address width. Explicit pointer casts, implicit void-pointer erasure and null/default
pointer values are supported within the same pointee set. Casts retain address bits
without reading storage. Raven's local-address/default initialization is lowered only
for exact scalar locals; managed local addresses cannot escape as native pointers.
Unsupported layouts, mismatched types and integer-to-pointer conversion still reject.
IntPtr/UIntPtr transport uses 64-bit registers/storage on this target. Unchecked
integer widening/narrowing is supported for native sizes, including Int64 conversion
temporaries; native-width arithmetic, checked conversions, pointer/integer conversion
and Int64/UInt64 signatures remain rejected in this profile. The public Ptr<T> wrapper
is not automatically projected yet.
Managed/value profiles, class members, other types, callbacks, managed JIT and SDK integration remain
unsupported. The flag is required; the default still rejects imports.

Development source qualification accepts explicit `--system`, `--module` and
`--object-root` inputs to verify Raven's original metadata references. A load context
requires `--system`; ordinary loader/verification errors reject before output. These
libraries are verification inputs, not emitted managed code. Scalar restrictions and
external-managed-call rejection remain in force. Use `@entry` for the Raven executable
entry point. The tested Raven consumer performs C allocation, pointer transport and
no-result release; no neoIL intermediary is involved.

Link `runtime/native/interop/imports.c` into the executable. Its private adapter
`neoclr_native_import_resolve_v1(const char *library, const char *symbol, void **output)`
returns 0 on success and 3 (RuntimeError) on failure, leaving output unchanged on
failure. Inputs are valid NUL-terminated UTF-8 metadata strings; output points to
writable pointer storage. This helper is a provisional runtime implementation ABI,
not the future public native hosting API. Successful lookups retain code ownership
until process exit, with a 256-entry library/symbol cache. Failed lookups can retry.

Generated thunks preserve the existing private managed status/result ABI while
calling the native C ABI. No interpreter fallback, managed object marshalling or
sandbox guarantee is provided. See [reproduction and limitations](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-native-imports/README.md).


Within this profile, imports and scalar helpers with the existing no-result Void
contract produce no operand-stack value. C void calls have no return register;
success leaves the private result slot untouched. Missing symbols still fault before
continuation and preserve the exported caller's result. The distinct Void value
carrier is not supported by this profile, and the host entry remains Int32-returning.

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

### Development scalar JIT imports

`--jit-scalar <input> <root> <Int32 argument> --native-imports` enables the same
bounded import signatures and pointer operations in the macOS ARM64 development
JIT probe. It shares the AOT C resolver and library ownership; the default rejects
imports. This CLI path does not yet accept seed/core verification inputs for Raven
assemblies, and does not enable imports in the ordinary managed JIT/SDK pipeline.
