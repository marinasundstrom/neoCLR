# Generic managed arrays

Implemented for the Raven runtime profile on 2026-09-13. The profile replaces the
old native System.Array<T> descriptor with a managed array definition. Existing
Raven `T[]` syntax and CLI SZARRAY/newarr/ldlen/ldelem/stelem operations remain the
language/compiler boundary. No Raven array syntax change or wrapper allocation is
required.

The loader canonicalizes System.Array<T> signatures to the existing ArrayRef<T>
identity when the managed System definition is loaded. The generic spelling and
CLI spelling therefore share storage, null state, aliases and bounds checks.
Allocation still uses newarr; newobj cannot create an unrelated record with the
array name. The definition cannot add fields, a base class or element constraints
that the intrinsic storage would silently ignore.

The definition explicitly implements System.Collections.MutableSequence<T>, inheriting
Sequence<T>, Collection<T> and Iterable<T>. Interface
reflection and conformance use that declaration; the runtime supplies the existing
array iterator implementation. Direct loops over a statically known Raven array
continue to use indexed access. Passing that array to an Iterable<T> parameter or
an extension method targeting Iterable<T> uses the interface contract instead.
This enables shared query extensions such as ToList without changing array-loop
lowering or introducing array-only extension overloads. Length, Count and the Item getter/setter are ordinary
library members implemented with existing array instructions. Reflection lists
those properties/methods, substitutes their element signatures, and reports T from
GetGenericArguments. Existing diagnostic array spelling and TypeIdentity::ArrayRef
remain; a generic class is not a second identity for the same object.

Mutable arrays remain invariant, including casts through interface views. The
current List<T> includes Add, so arrays do not implement it. The provisional counted/read/replacement contracts are described in the
[collection review](collection-contracts.md#2026-09-13-capability-prototype).
No general interface variance, read-only array, immutable/frozen collection family,
Span or Memory contract is introduced here.

Compared with .NET, the deliberate differences are the generic runtime array shape
and invariant mutable arrays. Familiar allocation, aliasing, default reference
slots and ordinary array IL remain. The profile keeps nongeneric System.Array for
existing static ForEach compiler declarations. Raven still imports ordinary array
signatures. The reference assembly now declares a generic System.Array<T> interface
shape implementing MutableSequence<T>. `RavenIterationArrayShapeType` selects it; Raven
reads its interface metadata instead of hardcoding Iterable on vector symbols.
The selected generic shape and T[] now resolve to the same array in Raven source
annotations and imported signatures. Assignment in either direction, nested arrays,
indexing and typeof use normal vector semantics. The compiler emits ordinary CLI
array signatures. Array base members and interface members such as GetIterator are
available directly; arbitrary members on the metadata class are not projected. The reference declaration and
runtime implementation must stay aligned; the interface probe checks the shape.

## Try the direct IL examples

From the repository root, choose an unused output path (the generator refuses to
overwrite an existing library):

```sh
python3 docs/experiments/raven-target/collection_library.py /tmp/managed-array-System.neoil
cargo run -- run examples/preview/generic_managed_array.neoil --system /tmp/managed-array-System.neoil
cargo run -- run examples/preview/native_memory.neoil --system /tmp/managed-array-System.neoil
```

Both entry points print 42. The first creates one managed array and accesses it
through the generic API; the second writes/reads native storage and frees it.

The saved Raven sample `library-managed-array-metadata.rvn` demonstrates ordinary
indexing and aliasing, the generic argument, properties and Iterable reflection.
Use the freshly built bridge/core declaration assembly and generated System
profile with the [saved-project runner](experiments/raven-target/README.md).
Installed SDKs and extensions are not refreshed by a source commit.

## Native memory migration

The active profile removes Array<T>.Allocate/View/Free and the public Data/Length
native descriptor fields. There is no compatibility alias or LegacyNativeArray.
Use System.Runtime.InteropServices.NativeMemory.Alloc(nuint) or
Alloc(nuint elementCount, nuint elementSize), and Free(void*) for explicit native
allocation. The multiplication is checked as unsigned native-sized arithmetic;
existing runtime allocation budgets and tracked-pointer checks still apply.

These are a bounded subset of [.NET NativeMemory](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.nativememory?view=net-10.0).
They return raw storage, do not own managed references, and do not create a borrowed
view API. Ownership versus views follows [.NET's memory guidance](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/memory-t-usage-guidelines).
Do not rely on Alloc's initial byte contents. Zero-sized allocations can be freed;
Free(null) is permitted. The preview reports invalid tracked accesses/releases as
faults; this is not a promise that arbitrary external addresses are safe.

Raven's metadata rewriter now preserves pointer signatures when targeting the
profile. Native-sized numeric conversions and explicit pointer casts remain
compiler gaps: the Raven native sample is a zero-sized allocation/release smoke
test, while the direct IL sample demonstrates typed native access. Span-like views,
pinning and a deterministic owner wrapper remain future work. The historical Neo
library/samples are outside the Raven-profile migration, as previously directed;
they are not compatibility aliases in the current profile.

## Validation

`tests/generic_array_shape.rs` covers identity, one-allocation aliasing, member
calls, reflection, explicit interface dispatch, bounds, invariance and rejection
of record storage. `tests/native_memory_api.rs` covers native read/write, release,
zero-sized/null release, overflow, bounds and stale-pointer faults. The saved-project
suite includes managed-array metadata and the updated native smoke test.

Validation for this source slice: 46 focused runtime tests passed across generic
arrays, native memory, array interfaces, reference arrays, reflection, scoped types
and type identities; the earlier broader generic/metadata regression run also
passed. All-target Clippy passed. The Raven saved-project suite passed 58 cases,
the query suite passed 28 checks, and the metadata signature probe passed 77 checks.
Native release/removal checks and both direct IL examples passed. Raven's focused
metadata suite passed six tests; its independent fix is commit `3df1b54b0` on
`codex/neoclr-target-resolution`. This is source validation, not a packaged SDK or
VS Code extension release gate.


The follow-up metadata projection slice passes 25 focused Raven compiler/project
tests, 58 saved-project cases and 28 query checks using the generic shape setting.
The interface probe validates the reference declaration and the runtime API inventory
check passes. No SDK or VS Code extension was installed by this follow-up.


## Unified Raven example

`library-array-unified.rvn` demonstrates `let values: Array<int> = [7, 8]`, passing
that array through `int[]` parameters and back, mutating a shared alias, indexed
iteration, explicit iterator acquisition, query extensions and reflection. Nested
`Array<Array<int>>` is likewise interchangeable with `int[][]`. Array allocation
uses normal Raven array expressions; this does not add an `Array<T>(length)` constructor.
The interface declaration supplies the contract, not a second object or boxing step.

The compiler slice passes 22 focused array/iteration tests, including emitted vector
signatures and opt-in/invariance checks. The saved-project suite includes the new
sample; `verify_editor.py --array-shape` checks member/extension completion and alias
hover using the project configuration. The source integration passes 59 saved-project cases and 52 editor checks with the
new SDK server. Packaged bundle validation is recorded separately.
