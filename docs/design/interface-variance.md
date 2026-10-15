# Interface variance and result-value conversion

Status: author-directed investigation, 2026-10-15. No variance implementation or
new source syntax is claimed. Structural-type work stays parked. This follows the
[implemented slicing contract](slicing-capabilities.md).

## Concrete requirement

The author proposed using a `Sliceable<int[]>` as a
`Sliceable<Iterable<int>>`, then directed exploring beyond .NET's restrictions,
while noting that some limits remain useful and supplying Microsoft's variance
guide. Treat this example as a design acceptance case, not a currently supported
assignment. Keep nominal interface conformance explicit.

The array-to-Iterable conversion preserves the same array. The fact that its
elements are integers is irrelevant: the type argument being widened is the
array itself. This does not request `int[]` to `Object[]`, or element covariance.

The method returns `Result<TSlice, InvalidRangeError>`. Therefore a complete
solution must preserve both cases while converting the success payload:

- `Ok(array)` becomes `Ok(iterable view of that same array)`.
- `Error(error)` preserves the error and does not access a success payload.

Merely annotating Sliceable's parameter is insufficient while Result is invariant.
The slice allocation already required by the receiver is distinct from any cost
introduced by widening its returned result.

## Baseline and current gaps

Sources inspected 2026-10-15:

- [Author-supplied C# variance guide](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/concepts/covariance-contravariance/): direction of producer/consumer conversions and the unsafe-write problem with covariant arrays.
- [C# specification §19.2.3](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/interfaces#1923-variant-type-parameter-lists): declaration-site annotations, variance safety and conversion rules.
- [.NET generic variance](https://learn.microsoft.com/en-us/dotnet/standard/generics/covariance-and-contravariance): variant interfaces/delegates and reference-type argument restriction.
- [Scala variance](https://docs.scala-lang.org/tour/variances.html): parameter variance on generic classes as well as producer/consumer roles; invariant mutable containers. This establishes an alternative language model, not a ready-made native value ABI.

C# exposes declared variance on interfaces and delegates. Its conversions use
identity/reference conversions, not arbitrary numeric widening, boxing or
user-defined conversions. A nominal Result class or value type cannot simply be
declared variant in C#. Separate CLI metadata/runtime guarantees from C# syntax
when implementing neoCLR; this investigation does not claim a CLR implementation
experiment has been run.

neoCLR's current metadata builder explicitly exposes invariant generic interfaces
(`tools/metadata/NeoCLR.Metadata.Experimental/Model/InterfaceBuilder.cs`).
`Iterable<T>` produces `Iterator<T>`, which produces T; those contracts are also
currently invariant. `Result<T,E>` is a nominal union implementing
`Propagatable<Result<T,E>,T,E>`, with output extraction through `out` parameters.
Its full inherited and generated member surface needs auditing before variance can
be advertised. Read-only case data alone does not prove a declaration is safe.

## Preserve the safety restrictions

Provisional constraints for the next implementation design:

1. Validate explicit declaration-site intent across all instance members, base
   interfaces and substituted generic signatures. Compose nested input/output
   polarity. Reject contradictory uses; do not trust annotations from a frontend.
2. Writable fields, setters, replaceable array elements and ordinary ref aliases
   remain invariant. MutableSequence/List must not inherit a producer conversion
   that permits writes of a wider type into narrower storage.
3. Keep interface conformance nominal. A variant view derives from an explicitly
   implemented interface and a validated conversion; matching member names is
   insufficient.
4. Initially permit only identity and admitted nominal reference upcasts at the
   leaves. Do not fold numeric conversions, downcasts, user code or automatic
   boxing into variance. Value-type arguments remain exact unless a separately
   validated value conversion applies.
5. Preserve exact generic type identity and reflection. A conversion must not
   declare two constructed Result types identical or reinterpret different layouts.
6. Out/ref need separate treatment. Do not reinterpret a pointer to a wider slot as
   a narrower slot. If output conversion is supported, an adapter uses a temporary
   exact output, converts it and writes back under a specified success/failure rule.
   Aliasing and failure semantics must be tested before admitting this feature.

Contravariance reverses the admitted input conversion, for example an explicitly
implemented consumer of a base reference used where a consumer of a derived
reference is expected. It does not authorize writing arbitrary bases into derived
containers. Nested combinations need polarity checking, not a simple rule based
only on whether a parameter appears in the outermost return type.

## Alternatives and proposed staging

| Approach | Benefit | Cost or limitation |
| --- | --- | --- |
| .NET-style interface/reference variance only | Familiar safety model and narrower runtime work | Does not satisfy Sliceable while Result remains invariant |
| Explicit result mapping plus an explicit Sliceable adapter | Expressible with existing nominal interfaces; visible conversion cost | Boilerplate and potentially an adapter allocation; does not make the original assignment implicit |
| Validated result-value widening plus interface call adapters | Supports the requested case without changing Result into an interface or erasing cases | Metadata, conversion planning, dispatch, GC and backend work; no blanket zero-cost claim |
| Treat generic values as layout-compatible aliases | Superficially cheap | Reject: different payload layouts, output slots and mutable state can make this unsound |

Proposed direction: establish ordinary nominal interface variance first, then add
explicitly validated conversion of immutable nominal result values. An interface
view whose return requires value conversion needs a conversion-aware call adapter;
its metadata must describe the actual implementation signature and required
conversion. Reuse an existing reference where possible, but measure and disclose
adapter allocation, payload-copy and dispatch costs. Decide source syntax only
after the metadata and safety model are precise.

A general mechanism is preferable to privileged treatment of the names Result and
Sliceable, but Result is the first bounded acceptance case. Constructor/static
factory parameters are not writes into an already widened instance; distinguish
these from mutable instance members during the audit. No blanket covariance for
all nominal unions is authorized by this proposal.

Ordinary Raven/.NET must remain valid on its backend. General diagnostics and
polarity checking can be developed on Raven main; native conversion behavior goes
to codex/neoclr deliberately under explicit Runtime Contracts. If the .NET backend
requires explicit mapping/adapters, document that difference rather than accepting
source it cannot safely emit. This is a proposal for a target capability, not a
request to restart structural-type work.

## Acceptance and next bounded slices

1. Inventory native generic metadata, importer/emitter flags and runtime type
   relationships; specify variance metadata and malformed-image rejection.
2. Validate declaration polarity and reference-only interface conversions, including
   inherited/nested producer and consumer contracts and negative writable cases.
3. Specify Result's generated/inherited member safety and value conversion plan;
   test both cases, default behavior, differing layouts, exact generic identity and
   output-slot rules. Do not simply set a variance flag on the current declaration.
4. Execute the author's Sliceable conversion against a separately compiled library
   in interpreter, AOT and JIT. Exercise empty/success/error results, exact receiver
   identity, reference retention, faults and collector cleanup.
5. Verify reverse conversions and writable views are rejected, overload selection
   prefers exact matches and reports incomparable candidates, and unsupported .NET
   cases get a clear diagnostic or an explicitly selected safe adapter.
6. Compare direct versus widened calls and explicit mapping with allocation counts
   and throughput before making efficiency claims. Keep each validated slice and
   its changelog/API documentation separate.

The immediate next task is the metadata/conversion inventory and a small
reference-only interface proof, not changing the slicing API or adding implicit
value conversions without backend support.
