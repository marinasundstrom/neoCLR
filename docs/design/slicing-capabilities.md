# Nominal slicing capabilities and explicit traversal

Status: author-approved separation of responsibilities (2026-10-15); concrete API
spellings below are a proposal, not implemented or published APIs. Structural-type
work remains parked. This refines [generic bounds](generic-bounds.md).

## Direction and existing behavior

The author requested nominal slicing capabilities, approved separating bounds from
traversal, and clarified that a safe slicing method can return Result while an
indexer may fault. Existing neoCLR Index and Range remain collection-relative
position/bounds values. Range stores an exclusive upper Index and resolves bounds
against a receiver length. Its GetOffsetAndLength faults on invalid bounds;
TryGetOffsetAndLength returns Result with InvalidRangeError. The new capabilities
must build on that validation rather than introduce a second interpretation.

## Proposed contract

| Responsibility | Proposed shape | Meaning |
| --- | --- | --- |
| Safe slicing | Sliceable<TSlice>.Slice(bounds: Range) -> Result<TSlice, SliceError> | Explicit nominal conformance; invalid receiver-relative bounds are recoverable. |
| Convenient slicing | receiver[range] -> TSlice | Optional concrete range indexer; invalid bounds may produce a terminal Fault. |
| Ordered bounds | Interval<T> with Bound<T> endpoints | Endpoint inclusion is data; ordering supports membership, not implicit stepping. |
| Traversal | interval.StepBy(step, stepper) -> Progression<T, ...> | Explicit stepping produces an enumerable value with fresh state per enumeration. |

TSlice is the result type, not necessarily the element type or receiver type. An
array-like receiver may return a copy; a span-like receiver may return a view. The
interface does not require Length, element indexing, mutability or enumeration:
those are separate capabilities. Avoid an IEquatable/ordering constraint on element
types merely to slice their container. A read-only receiver can still be sliceable.

Require the safe method only for nominal Sliceable conformance. A type may also
expose the faulting indexer, delegating both paths to one internal bounds validator.
Do not silently synthesize conformance from a conveniently named method or from a
Range indexer. Generic code constrained to Sliceable uses its declared safe method;
indexer sugar through that constraint is a separate compiler decision, not assumed.

The receiver documents copy versus view, aliasing, lifetime, mutation visibility and
coordinate units. String uses neoCLR's grapheme coordinates; array slices count
elements. Success must not silently clamp invalid bounds. Empty ranges at a valid
boundary succeed. Invalid/reversed/out-of-bounds selections return Error through
the safe method and Fault through the indexer. Validation must precede allocation,
mutation or publication of the resulting view. SliceError naming and reuse versus
wrapping of InvalidRangeError remain open pending the first concrete implementations.

Safe slicing guarantees recoverable *slice validation* after its arguments exist.
It cannot catch a terminal fault during argument evaluation: Index(-1) and an
unrepresentable inclusive System.Range conversion already fail before Slice runs.
If callers need recoverable construction too, provide separately specified Result
factories; do not silently change existing constructors. Generic Interval bounds can
retain an inclusive maximum directly and must not require a successor to represent it.

## Enumeration and capabilities

Range will not become IEnumerable merely because its endpoints are integers: ^n
needs a receiver length and coordinates do not determine a step. Interval<T> also
will not imply enumeration from ordering alone. A progression owns a step policy,
direction and termination behavior and implements the normal enumeration contract.
It is repeatable; mutable cursor state belongs to each enumerator, not the interval.

An explicit nominal stepping capability must specify progress, overflow/end-of-domain,
and incompatible direction/zero-step behavior. Start with signed integer progressions;
consider a default step of one only for that domain. Dates require a calendar/duration
policy, decimal and floating domains need their own progress rules, and arbitrary
ordered types need not be discrete. Scalar loop lowering may optimize an admitted
progression without changing its observable sequence or allocating a collection.
The existing range-loop overflow fix is groundwork, not a shipped Interval API.

## Comparison and tradeoffs

.NET System.Range is a bounds value with GetOffsetAndLength, not an enumerable
progression. C# permits explicit Index/Range indexers and pattern-based slicing
support. Keeping bounds independent follows that model; requiring nominal
Sliceable conformance for generic slicing is a deliberate capability contract,
with explicit discoverability and constraints at the cost of implementing an
interface. See [System.Range](https://learn.microsoft.com/en-us/dotnet/api/system.range)
and [C# indices and ranges](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/ranges).

The proposed Result-returning Slice method deliberately differs from throwing
.NET Slice APIs. It makes invalid bounds recoverable in neoCLR's Fault model;
callers must handle a result, and the paired indexer has a distinct failure contract.
Neither method spelling nor an indexer determines copy/view ownership by itself.

Rust's current std::range::Range separates a copyable bounds value from RangeIter
and provides iter/IntoIterator when the element supports Step. This supports the
separation, but does not require neoCLR to adopt Rust traits or Rust's generic
iteration rules. Its documentation marks the new type stable since 1.96 while
range syntax still constructs the older type; do not conflate that migration with
a completed syntax change. Step remains nightly-only in the consulted documentation.
See [Range](https://doc.rust-lang.org/std/range/struct.Range.html) and
[Step](https://doc.rust-lang.org/std/iter/trait.Step.html) (consulted 2026-10-15).

## Next implementation and validation

1. Compile a nominal generic safe-slicing contract and a concrete owned-array
   implementation against the native bundle, including calls through the interface.
2. Prove Result/indexer agreement for empty, ordinary, from-end, reversed and invalid
   bounds, plus documented copy isolation. Prove any view implementation separately.
3. Reuse the shared endpoint validator and document constructor versus receiver
   validation boundaries. Qualify interpreter, JIT and AOT before publishing APIs.
4. Specify Bound/Interval and integer step policy, then implement independent
   repeatable enumeration with the same domain termination as direct loops.
5. Update both compiler integration docs and public API snapshots when APIs ship.
   No interface or progression API is claimed as shipped by this design document.
