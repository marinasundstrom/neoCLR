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
length/ownership, record layout, platform-error capture and callbacks remain open.
The generated type/member reference includes all five types; no API is excluded.
