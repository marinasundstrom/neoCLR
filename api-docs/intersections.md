# Intersection

Proposed structural family. There is no implemented neoCLR structural Intersection
member contract to enumerate yet. This reference entry makes the intended family
visible without presenting a proposed API as shipped.

The open design must specify component identity, compatible and conflicting member
signatures, conversions, extension lookup and introspection construction. Absence
of a name would not itself prevent members. It also would not automatically permit
inheritance or imply Function's Object contract.

[All structural families](structural-types.md) · [Open proposals](/proposals/)


## Branch-only reference experiment

The unpublished `codex/structural-union-intersection` branch admits neoIL
`union<A,B>` and `intersection<A,B>` behind the `experimental-compositions` build
feature. It supports reference locals, free-function parameters/results, checked
casts and guaranteed interface dispatch in the interpreter. This is not Raven
source support or a replacement for published union carriers. There are no public
compound reflection constructors or members to enumerate yet.

Fields, generics, byrefs, arrays, boxed/intrinsic reference values and JIT/AOT remain
unsupported. Typed nulls follow ordinary reference behavior; non-null membership
preserves the same class allocation. These are experiment limits, not permanent
platform contracts.
