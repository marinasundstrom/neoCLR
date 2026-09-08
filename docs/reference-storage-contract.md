# Stored and returned managed-reference contracts

Status: design recommendation, 2026-09-08, following the
[runtime groundwork review](runtime-groundwork-review.md). The runtime implementation
remains at the readonly input/receiver milestone. This document settles a proposed
semantic direction and bounds the next implementation; it does not add syntax or
claim that the verifier already enforces these rules.

## Observed gap

The [return probe](experiments/reference-contracts/return-gap.neoil) forwards a readonly
input through an ordinary Int32& return and local, then attempts a write. With the
current implementation:

```sh
cargo run --locked -- verify docs/experiments/reference-contracts/return-gap.neoil
cargo run --locked -- run docs/experiments/reference-contracts/return-gap.neoil
```

Verification succeeds. Execution intentionally faults at Main instruction 6 with
"readonly managed reference cannot be used for writable access". The attempted write
does not succeed. This demonstrates missing static contract information, not an
upgrade of the runtime capability. The probe was run against milestone 0e1b2e1.

[Function metadata](../src/metadata.rs) has readonly parameter indices and a receiver
flag. [Value::for_storage](../src/value.rs) compares ordinary storage types, while
[verifier local loads and returns](../src/verifier.rs) do not preserve the full access
contract. Adding another return Boolean would fix one position but leave nested
generic arguments and reference-valued array elements unresolved.

## Recommended semantic model

Keep three independent facts:

| Fact | Meaning | Where enforced |
| --- | --- | --- |
| Stored value signature | What value the slot contains, including permissions on any managed-reference signature nodes | Loader, storage conversion, calls, returns, verifier |
| Slot replacement policy | Whether the stored value may be replaced after initialization | Runtime storage, independent of the alias used |
| Live reference permission and provenance | Which operations this access path permits, and which frame or heap owner it addresses | Runtime reference operations and lifetime checks |

A writable managed reference and a readonly view can designate the same location.
Narrowing must preserve reference identity, referent type, allocation and lifetime.
Readonly remains shallow and can observe writes through another writable alias.

Use the following notation only for discussing contracts: W(T) is a writable managed
reference to T; R(T) is a readonly managed reference to T. Neither notation proposes
a Ref<T> wrapper or a second object hierarchy.

| Incoming value | Destination contract | Proposed result |
| --- | --- | --- |
| W(T) | W(T) | Accept, retaining write permission |
| W(T) | R(T) | Accept a narrowed copy of the reference |
| R(T) | R(T) | Accept, retaining restriction |
| R(T) | W(T) | Reject at the storage/call/return boundary |
| Reference to T | Owned T | Only an explicit value-read/copy operation; no alias conversion |

The caller's W(T) alias remains writable when another copy narrows to R(T). The rule
must apply equally to ordinary stores, returns, calls, interface projections, erasure
and eventual reflection/delegate invocation. Check before committing a store so a
failed assignment leaves the destination unchanged.

A declared W(T) return must never successfully return a restricted reference.
A declared R(T) return accepts either input permission and exposes R(T) to its caller.
Both still reject a reference into the returning frame. Readonly grants no escape,
heap promotion or ownership extension.

For erased references, preserve the live permission as today. Extraction to W(T)
must check it and fault if restricted; extraction to R(T) may narrow. Opaque storage
is not permission erasure.

## Representation and identity

Recommend a recursive signature representation with access recorded on each managed
reference node, used consistently by parameters, returns, locals, fields, array elements
and generic arguments. Use structured parameter/receiver contracts for direction
(input/out/conditional output), rather than encoding initialization as reference access.
Keep slot protection in storage declarations.

This is a logical schema recommendation, not a mandated Rust enum or JSON spelling.
Evaluate whether to extend the current Type tree or separate nominal identities from
use-site signatures during implementation. Do not maintain two independently mutable
sources of truth for the same permission. Normalize today's readonly flags/indices at
preparation, or migrate them explicitly; conflicting encodings must fail loading.

Nominal type definition and addressed-object identity remain unchanged. Qualified
constructed signatures must preserve their arguments: a container of W(T) and a
container of R(T) must not collapse into the same closed storage signature. Reflection
and generic substitution must retain these distinctions even though the referenced T
objects have the same runtime type.

Retain the current rule that methods cannot overload solely by readonly access.
Definition identity identifies the callable; binding must additionally validate its
full access contract. Initially keep exact interface implementation matching, including
returns, rather than introducing permission variance and inheritance variance together.
A future relaxation requires a separate substitutability proof.

### Containers do not acquire implicit variance

Narrowing a single W(T) to R(T) does not authorize aliasing Array<W(T)> as
Array<R(T)> or ArrayList<W(T)> as ArrayList<R(T)>. If both views could replace elements,
the latter could store a restricted reference that the former would read as writable.
Keep mutable containers invariant in their complete element signatures. An explicit
element-by-element copy into a new destination can narrow each reference; it is a
separate operation with allocation and copy behavior.

Also distinguish R(Array<W(T)>) from an array containing R(T). The first restricts
writes to the owned array's elements through that view, while loading an existing W(T)
element preserves its own permission. A readonly ArrayList receiver restricts the
descriptor; its separately referenced backing array is another shallow boundary.
Neither contract proves collection purity or freezes the contained objects.

The existing prohibition on nested managed references is unchanged. Designing a
reference to a reference slot or a ref-rebinding parameter is outside this slice.

## Verifier and runtime agreement

Declared contracts must determine local loads and call results even without analyzing
a callee's body. Stores and returns validate compatibility before a value is accepted.
The runtime repeats the necessary permission checks when verification is skipped.

For compatible evaluation-stack reference shapes, propose the conservative join
W(T) + R(T) = R(T). Both readonly inputs remain R(T); both writable inputs remain W(T).
Do not weaken unrelated type checks or definite initialization. At joins, every local
must still be initialized on every incoming path. A local declared W(T) rejects an
R(T) store even if a later branch might replace it with W(T).

Track access, initialization and known lifetime provenance as independent facts.
Current StackType::Readonly(Type) must not cause known frame-origin information to
disappear. The first implementation may retain conservative rejections for complex
aliases; it must neither manufacture write permission nor claim complete escape proof.
Raw IL receives runtime checks regardless of verifier precision.

## Immutable storage is the following slice

The four combinations remain distinct:

| Slot | Stored reference | Replace reference value | Mutate target through it |
| --- | --- | --- | --- |
| Mutable | W(T) | Allowed by storage contract | Allowed |
| Immutable | W(T) | Forbidden after initialization | Allowed |
| Mutable | R(T) | Allowed by storage contract | Forbidden |
| Immutable | R(T) | Forbidden after initialization | Forbidden |

This table describes runtime operations. Neo assignment to a managed reference already
accesses its target automatically; the table does not redefine assignment as rebinding.
Any future rebinding syntax needs a deliberate language rule.

Protecting an owned value must cover its owned subfields/elements, including writes
through previously formed aliases. Loading a separately stored reference still follows
that reference's contract. For the first protected-slot slice, prefer complete-value
initialization followed by sealing. Constructor field-by-field initialization and
output-based initialization need explicit authorization rules before exposing them.

local.reset cannot simply reopen a sealed lifetime. Define declaration re-entry as
fresh storage lifetime under a checked scope/region rule, preserving the prohibition
on invalidating live references. The precise region encoding and constructor/output
initialization protocol are still open; they are not prerequisites for readonly
local/return signatures and should not be smuggled into that implementation.

For now, out T& means a writable initialization/output contract for the addressed T,
not replacement of the reference value held by the caller. An outer readonly reference
cannot satisfy it. This preserves the existing output model.

## .NET comparison and alternatives

Sources checked 2026-09-08:

- [C# readonly reference documentation](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/readonly)
  describes ref-readonly returns and distinguishes reference-target access from readonly
  field replacement. It provides familiar behavior to compare; it does not specify
  neoCLR's runtime permission representation.
- [C# 12 ref-readonly parameter design](https://github.com/dotnet/csharplang/blob/main/proposals/csharp-12.0/ref-readonly-parameters.md)
  documents attribute/modifier encodings and compatibility choices. It is design
  evidence, not a general runtime capability specification.
- [C# array specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/arrays)
  describes array covariance and its runtime element-assignment checks. The proposed
  permission invariance avoids adding a comparable alias conversion obligation;
  it is not a proposal to change all inheritance conversion rules.

Compiler-only annotations would leave raw IL and other languages without the same
enforced boundary. More parallel flags would be a smaller first patch but do not
compose through containers. Recursive signature permissions cost metadata migration,
substitution/identity work and runtime validation, but cover all signature positions
with one contract. This is the recommended tradeoff, not a measured JIT speedup.

Preview compatibility must be explicit: a restricted reference returned through today's
unqualified T& would fail at the return boundary under the proposed rules. Producers
must declare a readonly result and consumers use compatible destinations. New metadata
must be rejected by unsupported runtimes; do not silently drop qualifiers or invent
a published format/version number here.

## Bounded next implementation and acceptance

Implement readonly local/return declarations, a shared signature access model and
boundary checks; migrate current input/receiver contracts to that model. Project a
minimal Neo spelling, tentatively readonly T& in type positions, clearly distinguished
from readonly func. Do not update the implemented grammar until that parser exists.

Cover these acceptance cases before declaring the slice complete:

1. Writable input returned as readonly stays the same location and leaves the original
   writable alias usable.
2. A readonly return stored locally can be read and passed to another readonly API.
3. A readonly value cannot enter a writable local, return or call; unchecked IL faults
   at the offending boundary, before a destination is changed.
4. Permission joins are conservative without weakening initialization or frame checks.
5. Field/array-derived references, interface views and erased references preserve access.
6. Generic substitution and artifact round trips preserve nested access signatures;
   mutable container conversions cannot manufacture writable elements.
7. Reflection and debugger output expose declared versus live access coherently.
8. Returning a current-frame reference still faults, regardless of access permission.

If the recursive metadata migration exceeds a bounded slice, separate representation
and existing-contract normalization from local/return syntax. Do not compensate with
another permanent side table. Protected slots, initialization regions, nested references,
general variance and new lifetime policies remain subsequent work.
