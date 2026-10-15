# Intersection

An intersection `A & B` describes a value satisfying both types. It composes
contracts without declaring another nominal type. Interface conformance remains
explicit: an object must implement the named interfaces; matching method names
alone is insufficient.

## Native contract

The integrated development native target supports intersections in Raven/native signatures,
including parameters, results and supported generic constraint positions. Inputs
and constraints are common uses, but a result may also describe a combined view.
An intersection does not combine two separate objects or copy their fields.

Bounds must be interfaces or reference classes that permit subclassing. A class
closed to extension in CLR terms is rejected. Raven's sealed hierarchy concept is
different: a sealed hierarchy can participate when its bound remains extensible
under its hierarchy rules. Value types are not intersection operands in this
profile. This is a conservative platform restriction, not a general theorem about
intersection types.

Constituent order and duplicates do not change identity. Guaranteed nominal
contracts can be projected for interface dispatch. Supported scoped nominal
constituents retain generic parameters until substitution. Import and constraint
support remain bounded; arbitrary symbolic structural constraints are not qualified.

## Introspection

`IntersectionTypeInfo.Types: Sequence<TypeInfo>` returns a snapshot of the
constituent descriptors. The intersection is structural, with no TypeDef or nominal
declaration name. Its runtime representation is currently a checked reference view.
Public construction of arbitrary guest intersection descriptors is not available.

<a id="branch-only-reference-experiment"></a>

## Availability

Native source, metadata and interpreter support live on
`codex/structural-union-intersection` and Raven's paired
`codex/neoclr-union-intersection` branch. Matching compiler/runtime libraries are
required. The native backend now supports bounded reference views, with macOS ARM64
AOT checks for interface conjunctions, extendable class/interface bounds, identity,
nulls, managed output slots and explicit interface dispatch. Missing conformance
produces the same InvalidCast diagnostic as interpretation. Views use one reference
slot and allocate no carrier. Final-class bounds remain rejected.

Native descriptor reflection, managed collector stress, dedicated intersection JIT
execution and compiler-produced application qualification remain integration work.
No published release is claimed.

[All structural families](structural-types.md) · [Union](unions.md)
