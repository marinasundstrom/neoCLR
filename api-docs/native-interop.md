# Foreign-function attributes

**Development, interpreter execution:** `System.Runtime.InteropServices` contains
NativeImportAttribute, MarshalAsAttribute, NativeCallingConvention,
NativeStringMarshalling and NativeType. Matching source compiler/runtime changes are
required; installed bundles and JIT/AOT imports are not yet qualified.

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
experimental Rust API. The independent C# metadata API does not author these fields.

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
