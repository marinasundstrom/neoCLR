# Source NativeMemory execution — 2026-10-07

Raven **1c9b1f5e7** maps bounded pointer signatures through its semantic symbols and
explicit portable emission capability. The metadata API additionally authors external
module-function pointer contracts from identity/digest/signature facts. No importer
objects enter emission, no native dependency falls back to a CLI projection, and no
new runtime format or service is introduced.

The unchanged `runtime/raven/src/System/Runtime/InteropServices/NativeMemory/Functions.rvn`
is compiled with two internal adapters into NativeMemory.dll. Separate consumers import
that artifact with library sources absent. They exercise both Alloc overloads, explicit
Void-pointer parameter/result identity, locals and Free. The successful case exits 42;
double-free and checked native-size overflow exit 1 with their expected faults.
Unsupported string-pointer signatures reject with NEOMETA001 before publication.

**Validation:** 162 metadata groups; 36 focused Raven pointer/portable tests (including
existing .NET observable execution). API snapshot check passes. The initial .NET
baseline was 34 tests. [Executable evidence](source-native-memory-2026-10-07.json).

Reproduce the input fixture with:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --native-integer-inputs "$CORE" /tmp/NativeWidthInputs.dll
```

Run `verify_source_native_memory.py` with the explicit `--compiler`, `--compiler-revision`,
`--core`, `--seed`, `--numbers`, `--integers`, `--inputs`, `--ownership`, `--runtime` and
fresh `--output` paths. Inputs are the same documented native-width primitive ownership
profile and independently built Numbers/NativeIntegers artifacts. Exact invocations and
all source, assembly and tool hashes are retained in the evidence file.

NativeWidthInputs supplies actual UIntPtr values 6, 7 and the maximum value through
API-generated native methods; it does not implement or replace NativeMemory. The source
UIntPtr class still has no source numeric-literal/conversion contract. This acceptance
therefore does not claim arbitrary source numeric casts, pointer arithmetic/dereference,
or nominal unmanaged pointer targets. Managed arrays remain a separate representation.

## Next System blocker

The full-owned-handle audit now has **no binding errors across 194 inputs**. It reaches
NEOMETA001 for `Array<T>`: the shared type plan explicitly rejects generic reference
classes with the source-defined Object base, matching the metadata builder's current
local-base restriction. No full System assembly is published.
[Audit evidence](native-bootstrap-pointer-2026-10-07.json).

Next support generic classes inheriting a local nongeneric Object root through definition
and builder validation, native materialization, constructed layout and runtime dispatch.
Start with a small generic class plus source Object and a separate consumer; retain the
existing .NET path. This unlocks broad generic class emission rather than changing
individual Array APIs. Later full-System errors are still unknown.
