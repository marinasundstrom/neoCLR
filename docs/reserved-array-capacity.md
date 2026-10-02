# Managed capacity without a default element

The Raven library profile uses `array.reserve T` for ArrayList backing capacity.
It consumes a length and returns an ordinary managed `arrayref<T>`. Every slot starts
as a checked unreadable reservation; stores publish typed values, and a read through
an index or element address faults until that slot has been written. The allocation
obeys normal heap, array-budget and GC rules. This extends the existing `array.alloc`
reservation behavior to the target profile's ordinary array-reference category.

This is needed for lists of Result/Option carriers and other values without a valid
default. Capacity is not Count: creating an empty list must not fabricate an Ok,
Error or None case. The existing Add/growth/Copy algorithms write the populated
prefix and expose only Count elements. Source programs still use the same ArrayList
API. Ordinary `newarr` retains default initialization and does not acquire unreadable
slots; allocating an ordinary array whose elements have no default remains rejected.

.NET's [GC.AllocateUninitializedArray](https://learn.microsoft.com/en-us/dotnet/api/system.gc.allocateuninitializedarray?view=net-10.0)
may omit zeroing. Its [CoreCLR implementation](https://github.com/dotnet/runtime/blob/main/src/coreclr/System.Private.CoreLib/src/System/GC.CoreCLR.cs)
retains reference initialization (consulted 2026-09-13). neoCLR's reservation is a
different contract: slots contain interpreter markers and cannot reveal arbitrary
memory. It supports non-defaultable union values, at the cost of a tracked state and
a read check. No performance improvement is claimed.

`array.reserve` is an internal-library operation, not an ordinary CLI opcode or a
consumer allocation API. Binary CLI imports continue using newarr. The metadata feature
branch now preserves reservation in native PE/#Neo as described below. The original Neo library keeps its existing array.alloc
profile. Regression tests cover publication, direct/indirect unreadable-slot faults,
union capacity and unchanged ordinary array defaults.


## Raven library authoring — 2026-09-15

The bootstrap reference assembly has a provisional
`System.Runtime.CompilerServices.CheckedStorage.Reserve<T>(length: int) -> T[]`
contract. Only `--reference-library-core` includes it; the normal consumer reference
assembly does not. Only matched library implementation imports recognize the call.
Raven emits an ordinary generic CLI call and the library importer lowers its validated
signature to `array.reserve T`. Compiler configuration and ordinary `newarr` behavior
are unchanged. The reserved array uses normal indexing and length access; each element
must be stored before reading. This exposes existing runtime behavior to library
source rather than adding a public allocation API.

The intrinsic is bootstrap machinery and its placement is provisional. The .NET
comparison and read-check tradeoff above still apply. It is not an unsafe request to
expose uninitialized bytes. The importer retains generic element identity across
fields, locals and array instructions, including Void as an actual generic element.


## Native metadata producer — development, 2026-10-02

The metadata builder exposes ReserveArray and its raw typed opcode. Native JSON and
PE/#Neo preserve existing array.reserve semantics. Executable CLI writing explicitly
rejects the extension; declaration-only CLI projection remains available. Raven's native
adapter maps the exact CheckedStorage.Reserve contract only when NeoClrEmitOptions binds
an explicit registered BootstrapReference. The default compiler and consumer core do not
gain reservation semantics. This is authoring bridge machinery, with native declarations
and target intrinsic contracts as the eventual replacement for the CLI seed.

C# metadata and Raven producer tests verify generic element substitution, a written slot
returning 42 and an unread slot faulting after binary loading. The established CLR
comparison and tracked-state tradeoff above apply unchanged; no speedup is claimed.
