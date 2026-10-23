# Tuple

Development neoCLR tuples are structural value types with ordered elements, native
names and `TupleTypeInfo` introspection. The neoCLR target selects this representation
without feature switches. No nominal `System.Tuple<T...>` declaration or constructor
is required. The old runtime carriers have been removed; rebuild consumers with the
matching compiler and library bundle. Ordinary .NET tuple representation is unchanged.

Names are preserved in metadata and reflection but do not participate in positional
type compatibility. Use tuple syntax such as `(offset: int, length: int)` and values
such as `(2, 42)`. Access names or positional `Item1` / `Item2` aliases. Assignment
copies the tuple; reference elements preserve the referenced object identity.

## Range results

`System.Range.GetOffsetAndLength(length: int) -> (Offset: int, Length: int)` returns
the validated start offset and count. Negative lengths, reversed boundaries and
out-of-bounds ranges fault. `TryGetOffsetAndLength(length: int)` returns
`Result<(Offset: int, Length: int), InvalidRangeError>` with the same validation.
Both methods now use structural tuple signatures, including across library imports.
This changes metadata compatibility: rebuild callers of the old nominal result.

`()` remains the inhabited unit contract. Tuple-specific equality, formatting and
general construction through introspection are not implied by structural identity.
Tuple alternatives inside unions remain rejected by the current metadata authoring
profile; that combination is an integration gap, not a nominal-inheritance rule.

[All structural families](structural-types.md)

## Resolved host tuple descriptors (development)

In the standard runtime, `LoadedProgram::resolve_type_identity` returns
`TypeIdentity::Tuple(Vec<TypeIdentity>)` for admitted closed tuple signatures.
Element keys retain resolved nominal identities where applicable. Order, repeated
elements and nesting contribute to identity; a one-element tuple differs from its
element. The tuple has no TypeDef or nominal metadata token.

`LoadedProgram::describe_type` returns a `TypeDescriptor` with ordered
`tuple_elements: Vec<TypeDescriptor>` and a recursive `tuple<...>` name. Each element
has its own descriptor. `generic_arguments` and `constituents` remain empty for a
tuple; `tuple_elements` is empty for other shapes. These APIs use the same feature,
closedness, arity and element-admission checks as executable tuples, returning
`Fault` for unsupported signatures. Descriptor access does not grant layout or
construction rights. This host inspection underlies the guest `TupleTypeInfo` provider.
Unlike .NET ValueTuple reflection, the tuple itself has no nominal generic definition.


## TupleTypeInfo (development)

[`System.Introspection.TupleTypeInfo`](xref:System.Introspection.TupleTypeInfo)
extends `TypeInfo`. Its read-only `ElementTypes: Sequence<TypeInfo>` property
returns a snapshot in positional order, including repeated types and recursive
`TupleTypeInfo` descriptors for nested tuples. It is not a nominal generic type;
elements are separate from `GetGenericArguments`. `IsValueType` is true and
`IsUnion` is false. Element labels are not part of runtime identity.

`ElementNames: Sequence<string>` returns a fresh snapshot of effective names in
matching positional order. Declared native names are preserved; an unnamed position
uses `ItemN` (one-based). Equal tuple types can expose different names because each
signature retains its descriptive metadata. This query does not distinguish an
explicit `ItemN` label from the same synthesized default; host signature APIs retain
that distinction. Identity-only handles cannot recover declared names.

Requires the standard runtime and the matching native source library.
There is no public constructor. The runtime selects this provider when a type handle
contains an admitted structural tuple signature. The same arity, closedness and
element-admission limits apply as for tuple execution. Unsupported signatures fault.
Nominal `System.Tuple<T...>` descriptors do not implement this interface. Synthesized
field/constructor introspection, nominal module ownership, native layout and unrestricted
Raven tuple emission remain incomplete. Do not infer those APIs from ElementTypes.

The executable [Raven consumer](https://github.com/marinasundstrom/neoCLR/blob/codex/structural-union-intersection/docs/experiments/structural-tuples/TypeInfo.rvn)
checks ordered duplicates, nested names, value classification, equal renamed types and
repeated snapshot queries. A documented test fixture substitutes two native tokens;
this validates guest introspection, not compiler tuple lowering.


## Native tuple element names (2026-10-14 development)

Rust `metadata::Type::Tuple` now contains `metadata::TupleSignature`. Its
`new(elements: Vec<Type>, names: Vec<Optional<String>>) -> Result<Self, Fault>`
constructor stores native names directly. `names() -> &[Optional<String>]` returns
the declared names; empty means entirely unnamed. `element_name(index: usize) ->
Optional<String>` returns the declared name or positional `ItemN`, and returns None
for an out-of-range index. `From<Vec<Type>>` creates an unnamed signature; callers
constructing the old variant migrate to `Type::Tuple(elements.into())`. Deref and
borrowed iteration expose the ordered element types.

A nonempty name vector must have one entry per element. Names must be nonempty,
NUL-free, at most 1024 UTF-8 bytes and unique after unnamed positions receive their
`ItemN` fallback. These bounded validation rules are provisional development limits.
Invalid names/counts return Fault; malformed named serialized shapes fail decoding.
The existing one-to-256-element and nesting checks still apply during normalization.

Named signatures encode as `{"Tuple":{"elements":["Int32","Int32"],"names":["x","y"]}}`.
Unnamed signatures retain `{"Tuple":["Int32","Int32"]}`. Matching native readers are
required for the new object payload; the updated experimental C# reader supports
it while older readers reject it. This is
direct signature metadata, without a nominal carrier or tuple-name attribute.

Equality/hashing of TupleSignature and Type ignore names recursively; TypeIdentity
also remains the ordered element identities. Names are preserved through native
serialization, scope binding and type/method/Self substitution.
`TypeDescriptor::tuple_element_names: Vec<Optional<String>>` returns names in the same
order as `tuple_elements`, including None for unnamed elements; non-tuples return
an empty vector. Descriptors obtained from differently named signatures retain their
respective names while their `identity` values compare equal. Identity alone cannot
recover names: consumers must retain the descriptive signature/descriptor rather
than intern it solely by structural identity. Synthesized runtime field layouts use
declared names with the existing `ItemN` fallback.

Unlike the CLR's ValueTuple/name-attribute representation, native names need no
attribute decoding. The cost is an expanded signature payload and explicit separation
of descriptive metadata from equality. Guest ElementNames and Raven tuple import
now preserve these names; named assembler syntax remains a follow-up.
C# authoring/reading/import and Raven native emission now preserve names; see
[the C# contract](experimental-metadata.md#native-tuple-names-in-c-authoringimport-2026-10-14-development). Existing positional guest element
queries and execution remain supported. Focused tests prove exact name roundtrips,
equal identity/hash under renaming, name-preserving substitution, host descriptors,
invalid-name rejection and interpreter parameter compatibility (result 42).

## Raven native binding contract (development)

Raven hosts select `CompilationOptions.NeoCLR`; structural tuples require no additional switch. `ITupleTypeSymbol` exposes
ordered fields and names with `UnderlyingTupleType == null`, no nominal declaration
or constructors, and empty TypeArguments. Positional ItemN aliases are available.
Names do not affect structural equality/hash; substitution preserves names. CLI
emission rejects this profile. Ordinary .NET tuple APIs are unchanged.

Local-rooted element mutation supports nested writeback and preserves copies;
immutable let roots and nonlocal roots are rejected. The current native emitter
still admits only closed scalar/nested elements. The nominal runtime tuple declarations have been removed. Historical CLI bridge
fixtures are compatibility tooling, not executable native API declarations.
These are compiler-host contracts, documented in Raven's runtime-contract and
neoCLR integration guides; they do not add a guest class-library API.

## Native execution development

The structural branch admits ordered tuple storage in the shared native backend.
macOS ARM64 AOT tests cover nested integer/floating-point tuples, copying, calls,
returns, element mutation through managed addresses and default initialization.
Elements use the existing private eight-byte scalar slots, capped at 256 flattened
slots. Tuple construction itself allocates no heap carrier. Element names remain
metadata; positional access and compatibility are unchanged.

Nested tracing recipes retain reference slots and omit numeric slots. This bounded
storage slice does not complete native tuple reflection, boxing, array storage,
Windows qualification or full Raven application integration. Direct JIT tests also execute two- and sixteen-integer tuple copy/call workloads;
managed-host and collector qualification remain open. These are
integration gaps, not permanent restrictions of structural identity.

Initial AOT microbenchmarks compare two- and sixteen-integer tuples with equivalent
nominal records: each pair emits byte-identical native objects with no allocator
imports. This is evidence for those copy/call workloads, not a general performance
guarantee. Reference-bearing values, boxing and union workloads remain unmeasured.

Development native specialization also admits tuples as closed generic arguments
and inline fields of nominal records. Recursive cycles and the flattened-lane budget
remain checked. See the [native structural qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-structural-types/README.md).
