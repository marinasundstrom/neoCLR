# Generic bounds and consistent Raven ranges

Status: proposed design, 2026-10-15. No compiler or runtime behavior changes in this
slice. The author requested generic bounds and consistent endpoint semantics,
followed by integer iteration, and explicitly parked structural-type work. The
author then clarified that all known Raven consumers are owned by the project and
compatibility is not a concern. A coordinated source migration is appropriate;
no legacy mode or deprecation period is proposed.

## Recommendation and current evidence

Use one Raven endpoint meaning across .NET and neoCLR, loops, collection expansion,
patterns and slicing: `a..b` excludes b; `a..=b` includes b. Remove `..<` during the
coordinated migration. Keep System.Index and System.Range as the familiar .NET-style
collection-position API. Add generic bounds independently of those types. Neither
this design nor its implementation requires further structural-type development.

At Raven main 41abd72ea, the control-flow specification makes numeric `..` inclusive
and `..<` exclusive. BlockBinder.BindRangeExpression instead binds standalone
endpoints through System.Index; ExpressionGenerator.EmitRangeExpression constructs
System.Range, whose end is exclusive. Range-loop and collection-expansion emitters
have separate comparison/increment paths. This is source inspection, not a fresh
execution qualification. neoCLR main 71a845fce supplies half-open System.Range and
from-end System.Index in ordinary Raven library structs.

The problem is observable: moving bounds between a loop, a stored value and a slice
must not silently change whether the endpoint belongs to the selection. The language
owns endpoint inclusion; a backend must not reinterpret it based on its available API.

## External comparison

Primary sources retrieved 2026-10-15:

- [.NET 10 Range implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Range.cs):
  shipped library value storing Index boundaries and resolving offset/count. Preserve
  that public contract. C# syntax and compiler slicing patterns are distinct from
  the library and CLI metadata; no new runtime opcode is needed for this proposal.
- [C# 8 range specification](https://github.com/dotnet/csharplang/blob/main/proposals/csharp-8.0/ranges.md):
  language lowering uses well-known types and indexable/slice patterns. It does not
  supply a general ordered interval or integer progression abstraction.
- [Rust range expressions](https://doc.rust-lang.org/reference/expressions/range-expr.html):
  `..` is half-open and `..=` is inclusive. Borrow this endpoint consistency, not
  Rust's ownership model or the requirement to expose a separate type per syntax form.
- [Rust Bound](https://doc.rust-lang.org/std/ops/enum.Bound.html) and
  [RangeBounds](https://doc.rust-lang.org/std/ops/trait.RangeBounds.html):
  included/excluded/unbounded endpoints support generic containment and ordered-key
  queries. RangeBounds.is_empty is marked nightly-only in the retrieved documentation;
  do not describe it as a shipped stable precedent for our emptiness contract.
- [Rust std::range](https://doc.rust-lang.org/std/range/index.html): the documentation
  describes replacement range types for a future edition. This is distinct from the
  current syntax mapping to std::ops types; do not conflate the two generations.
- [C# enumeration discussion #3004](https://github.com/dotnet/csharplang/discussions/3004):
  participants discuss inclusive-loop expectations versus exclusive slicing and
  extension enumerators. This is design discussion, not an adopted specification.
  It illustrates why contextual endpoint reinterpretation needs an explicit decision.
- [Noda Time 3.2 Interval](https://nodatime.org/3.2.x/api/NodaTime.Interval.html):
  an independent .NET library represents time intervals with inclusive starts,
  exclusive ends and optional unbounded endpoints. This supports keeping interval
  logic in a library; it does not establish a generic integer iteration contract.

Alternatives: keeping Raven's current contextual behavior avoids work but preserves
surprising results. Making `..` inclusive everywhere preserves current loops but
requires adapting every ordinary .NET Range boundary. The recommended exclusive
`..` aligns existing slicing, .NET and Rust; explicit `..=` retains inclusive use
without endpoint arithmetic in the abstract value. Given the author's clarification,
source compatibility no longer favors either alternative.

## Proposed library model

Names and assembly placement below are provisional API design, not shipped APIs.

- Bound<T>: a nominal union with Included(T), Excluded(T), Unbounded. Store endpoints
  directly; do not encode inclusion by adding/subtracting one or use sentinels.
- Interval<T>: an immutable pair of lower/upper Bound<T> values. This describes an
  ordered set, not a traversal direction or mutable iterator. It supports Contains
  with an explicit ordering policy. Prefer the existing comparer abstraction; do not
  require arithmetic or successor operators merely to describe an interval.
- Int32 progression: immutable start, end, end-inclusion and signed nonzero step;
  each enumeration creates separate state. Descending traversal is a progression,
  not a reversed mathematical interval. A common range-bounds interface can be added
  when more than one API actually needs it; it is not required for the first slice.
- System.Index / System.Range retain collection-relative meanings and existing ABI.
  Index cannot be globally ordered: resolving ^n requires a collection length.

Generic ordering must be a consistent total order supplied by the caller or by a
supported default comparer. Storage itself need not constrain T. Null and floating
NaN behavior follows the selected comparer, rather than mixing operator comparisons
with comparer comparisons. Partial-order support is deferred explicitly.

Contains checks each bound using that order; Unbounded contributes no restriction.
Reversed endpoints contain nothing; equal endpoints contain their value only if both
are included. Do not promise a general IsEmpty solely from endpoint ordering:
(1,2) contains no integers but can contain real-valued points. Domain-aware emptiness,
cardinality, intersection and overlap APIs require separate qualification. No generic
Count, implicit enumeration or automatic zero lower bound is proposed.

Bounds are inspectable ordinary nominal library values on each target. Generic
metadata describes their type parameters and cases, with no new intrinsic signature
kind. .NET uses Raven's ordinary nominal union representation; neoCLR uses its nominal
library representation. Interface conversion/boxing and union-carrier allocation must
be measured, not assumed absent. The Int32 iteration fast path need not allocate a
Bound union or box an enumerator.

## Explicit capabilities (author clarification, 2026-10-15)

The author wants types to distinguish their supported qualities through explicit
interfaces and generic contracts, without adding every capability to every type.
Use nominal conformance, not incidental member-name matching. Bound<T> storage
alone requires no ordering or stepping capability.

neoCLR already has ComparableTo<T>.CompareTo and Collections.Comparer<T>.Compare.
The .NET adapters use IComparable<T>/IComparer<T> where appropriate. A default
Contains operation should require supported ordering conformance; an overload taking
an explicit comparer can work with externally owned types without changing them.
These are two explicit capability sources, not a runtime guess that all types compare.

Iteration needs an additional stepping contract, provisionally a generic stepper
strategy supplied explicitly or by a type's declared supported capability. It must
report whether a successor exists, obey the requested direction and avoid wrapping.
An addition operator alone proves none of these properties. Keep step magnitude/type
separate from element type so future date/duration operations need not invent date
addition to another date. Do not publish final interface names before compiler
support for the required generic/static members is checked.

Marker interfaces are suitable for an assertion of a property; operations that need
behavior must also have a usable member/provider contract. Do not mark all comparable
types as iterable, give all types Count, or infer a stepping capability through duck
typing. The initial integer fast path implements the same contract without per-element
interface dispatch. No performance equivalence is claimed before measurement.

## Syntax, inference and slicing

| Form | Proposed meaning |
| --- | --- |
| a..b | Include a, exclude b |
| a..=b | Include a and b |
| a.. | Inclusive lower bound, unbounded upper bound |
| ..b | Unbounded lower bound, exclusive upper bound |
| ..=b | Unbounded lower bound, inclusive upper bound |
| .. | Both bounds unbounded |

The upper endpoint is mandatory after `..=`. Excluded lower bounds use the Bound
API initially; no additional punctuation is needed. Range patterns use the same
inclusion rules but need not construct an interval object.

Standalone numeric ranges should have a useful numeric interval type, rather than
being forced through nonnegative System.Index. An expected System.Range or a slicing
context can select collection-relative binding without changing endpoint inclusion.
Fully unbounded values need an expected type or an explicit generic factory. A stored
interval can be iterated or passed to a supported slice adapter without changing its
bounds. Negative numeric endpoints are valid interval values but invalid element
positions; conversion must validate them. Final overload-ranking and natural-type
rules need binder tests before implementation is considered complete.

For half-open slicing, map to System.Range normally. For inclusive slicing, use a
length-aware adapter: evaluate receiver and endpoints once, left-to-right; resolve
from-end positions; validate the included last element; then form an exclusive limit.
An inclusive ^0 is invalid because it denotes the boundary after the final element;
inclusive ^1 includes the last element. Empty receivers cannot accept an included
end element. Never implement inclusive conversion as unchecked `end + 1`, especially
for ^n or integer maximum. Plain System.Range cannot retain an inclusive endpoint;
conversion without sufficient context must be rejected or remain an explicit bounds
value, never silently lose the inclusion flag. Custom indexers require a supported
bounds contract or an explicit adapter; they cannot be assumed to have a length.

Syntax semantics agree across targets. Runtime failure transport and text coordinates
remain explicit platform differences: .NET APIs may throw and use UTF-16 positions;
neoCLR consuming APIs have Fault/Option/Result contracts and grapheme string indexing.
This work does not make identical string offsets denote identical text on both targets.
Slice copy/view policy also remains a property of the consuming API.

## Integer iteration: next implementation slice

Start with Int32 on both targets, then qualify other widths separately. Default step
is +1; a negative explicit step descends. A direction mismatch yields no items; zero
step is rejected (a diagnostic for constants, a checked failure for dynamic values).
Do not auto-reverse bounds. Omitted loop start may continue to mean zero as a documented
iteration convenience; it does not turn an abstract unbounded interval into a bounded
one. Missing loop end and from-end loop bounds remain diagnostics in the first slice.

Evaluate start, end and step exactly once before iteration, in source order. Keep
progression state independent of the body binding. Continue performs exactly one
advance; break performs none. Enumeration is repeatable and independent.

The reference Int32 advancement algorithm calculates next in Int64, checks whether it
still satisfies the directional end condition and the Int32 domain, then narrows.
An out-of-domain next value terminates: it is not a yielded value and must not wrap.
This handles inclusive int.MaxValue/int.MinValue, overshooting steps and int.MinValue
as the step without negating it. Count is not an Int32 prerequisite: a full-domain
progression has more than int.MaxValue elements. For Int64/UInt64, use explicit
pre-addition checks or another proven algorithm, not an unavailable wider native type.

For loops, collection expansion and stored-range enumeration must share the same
semantic progression rules. Lower direct loops to scalar locals/branches where possible;
provide ordinary enumeration for stored values. Do not silently materialize arrays.
Floating-point/decimal/char loops already exist in Raven: migrate their endpoint syntax
with the language change, but retain their own stepping contracts pending separate
review of rounding, non-progress, overflow and Unicode domains. Do not claim the
integer termination proof covers them.

## Author-directed sequencing clarification

The first implementation task is Raven syntax/semantics on main, including owned
consumers and range syntax slot names; integrate that before neoCLR-specific semantics.
Integer advancement and generic capability APIs follow this foundation. A preliminary
integer-overflow patch was set aside rather than promoted ahead of this direction.

## Delivery and validation

1. Design slice (this document): record endpoint recommendation, source audit and
   author direction. No semantic changes or target configuration changes yet.
2. General Raven main slice: parser/bound inclusion representation, syntax migration
   across all owned consumers, spec/grammar/editor updates and focused tests. Carry
   deliberately to the neoCLR compiler line; do not merge whole branches.
3. Generic bounds library/binding slice: select assembly and comparer contracts,
   natural types, conversion/overload behavior and nominal metadata round trips.
   No new structural signatures, runtime opcode or target-dependent endpoint switch.
4. Integer iteration slice: shared checked progression lowering and stored enumeration,
   with equivalent tests on .NET and neoCLR interpreter/JIT/AOT. Implement the slice
   without waiting for generalized intervals if explicit integer factories suffice.
5. Measure allocation and throughput against an equivalent handwritten loop before
   claiming efficiency; inspect generated artifacts for accidental boxing. Keep
   slower fallback behavior explicit if a target lacks a required representation.

Required test cases: empty and singleton intervals, open bounds, included/excluded
equal endpoints, negative numeric bounds, type inference and competing overloads;
slices at start/end, ^0/^1, empty collections, inclusive maximum rejection and receiver
side effects; loops around both integer limits, minimum step, overshoot, wrong direction,
zero step, break/continue and two independent enumerations. Use bounded edge cases
rather than billions of iterations to test the full integer domain. Compare constant,
dynamic and imported values. Update pattern reachability/exhaustiveness tests for the
new endpoint meaning. Parser tests must distinguish `..=` from assignment, spread and
other dot tokens, including malformed/incremental edits.

Documentation-only validation for this slice: inspect referenced source and check
links/patch whitespace. No executable examples are presented as newly supported,
no runtime tests are claimed, and no website publication is performed.
