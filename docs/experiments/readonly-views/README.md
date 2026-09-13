# Read-only view contract experiment

2026-09-13. This is a closed, application-local prototype, not a new collection
hierarchy, standard-library API or implementation of generic covariance. It uses the
existing runtime and installed Raven `.10` tools without compiler changes.

## Question and outcome

Can code read Dog elements as Animal objects, keep familiar indexer/member syntax,
and retain the original mutable array without copying its elements?

Yes, through an explicit adapter with a private array reference and a getter-only
interface. `Main.rvn` demonstrates replacement through the original array, mutation
of a returned object, and a view returned from a function. Output is:

```text
1
7
1
42
99
1
11
```

This proves an API shape and existing runtime behavior. It does not prove that
`Sequence<Dog>` can implicitly convert to `Sequence<Animal>`. The adapter is one extra
heap object and has a different identity from the array. No second array or copied
element objects are created in the direct IL probe. No throughput claim is made.
The closed AnimalView name avoids committing to a generic collection vocabulary.

## Run

From the neoCLR repository, provide an extracted/configured bundle that includes the
array-invariance compiler option (the installed `.10` build was used):

```sh
python3 docs/experiments/readonly-views/verify.py \
  --bundle /Users/robert/.neoclr/experiments/array-diagnostics-20260913
cargo test --test readonly_view_prototype
```

The Python runner uses temporary projects and preserves the installed demo. It checks
the working program, an empty view, forbidden setter/backing-field access, and retained
mutable-array invariance. Copy Main.rvn into a separate Raven target project to edit it.

`view.neoil` is the equivalent low-level fixture, with a Boolean result for the alias
check. The Rust tests exercise it directly, including forced GC, out-of-bounds access,
null element preservation, inaccessible backing storage, nonexistent setter calls,
forged array casts and element-address attempts. Some malformed calls fail resolution;
other operations fail verification and/or runtime checks. Unsafe native access and
privileged host mutation are outside this ordinary managed-access claim.

The .NET comparison is:

```sh
dotnet run --project docs/experiments/readonly-views/dotnet/Comparison.csproj
```

It uses IReadOnlyList covariance over Array.AsReadOnly, shows the same mutable-alias
and element-object behavior, and contrasts an adapter with an interface directly over
an array. The latter can be cast back to its exact mutable array type. This is why a
read-only interface alone is not a security boundary or a promise of immutable storage.

## Current contract audit

| Contract | Relevant element use | Consequence |
| --- | --- | --- |
| Iterator<T> | Current returns T; MoveNext and Dispose mutate cursor/lifetime state but do not consume T | Candidate producer variance for ordinary reference T; cursor mutation alone does not prevent covariance |
| Iterable<T> | Current legacy library signature returns Iterator<T>& | Do not widen this byref as though it were an ordinary object reference. The adapted Raven surface returns an ordinary Iterator<T>, so reconcile the runtime/imported contracts first |
| List<T> | Getter returns T; setter and Add consume T | Mutable contract stays invariant |
| ArrayList<T> and arrays | Store T and expose mutable operations | Preserve exact element type; derive future read interfaces separately |
| Application-local AnimalView | Returns Animal values/references, count; no setter or element address | Sound explicit Dog-array adapter in this experiment, with no generic conversion |

Runtime TypeDef currently stores generic names and constraints but no variance flags.
Reference assignability/dispatch use exact constructed interfaces. The Raven bridge's
collection metadata validator requires NonVariant; generic application definitions are
also outside its admitted subset. Marking declarations `out` without updating these
layers would not implement covariance. A consumer-facing readonly method returning a
writable byref would not satisfy the proposed element-read restriction.

General variance needs a separate slice: standard CLI variance flag mapping, declaration
polarity validation (including nested types and byrefs), reference-type-only conversion,
constructed-interface dispatch and consistent return adaptation. Include malformed
metadata, conflicting interface instantiations, base/interface conversions, value types,
Void, arrays and ref/out signatures. A setter must not become callable by manufacturing
a wider interface. Keep mutable arrays invariant throughout.

## Collection taxonomy remains open

The later [collection-contract review](../../collection-contracts.md) preserves both
supplied hierarchies and concrete-type lists, including the Set/Map variance questions.


The author proposed a possible separation into Iterable<T>, Collection<T> (count),
Sequence<T> (ordered/indexed), Map<K,V>, Set<T>, with separate MutableSequence,
MutableMap and MutableSet. The author explicitly presented this as an illustration of
the problem, not a chosen hierarchy. Review the whole vocabulary later rather than
accumulate readonly, immutable and frozen names now.

Iteration, count, indexing, key lookup and mutation are distinct capabilities. A Set
membership test consumes T; a Map can both accept keys and expose them. Those contracts
cannot be declared covariant just because no setter appears. Ordering and random access
also need not be synonymous. Immutable snapshots and frozen lookup structures answer
different needs from live read-only views; no extra interfaces are selected here.

## Comparison and evidence

Primary sources consulted 2026-09-13:

- [.NET generic variance](https://learn.microsoft.com/en-us/dotnet/standard/generics/covariance-and-contravariance): reference-type argument conversions for variant interfaces/delegates; class and mutable-list invariance. Keep this baseline instead of inventing read-only array covariance.
- [.NET collection-interface proposal #31001](https://github.com/dotnet/runtime/issues/31001): the author reports confusion around mutable/read-only inheritance. At review it is open, labelled api-approved and blocked, with a Future milestone. This is design feedback, not shipped behavior or proof that all compatibility objections are resolved.
- [Frozen-array discussion #111661](https://github.com/dotnet/runtime/discussions/111661): participants distinguish read-only interfaces from guarantees about repeatable contents and discuss wrappers. It is an open design discussion, not an adopted frozen-array contract.
- [Java Collections.unmodifiableList](https://docs.oracle.com/en/java/javase/25/docs/api/java.base/java/util/Collections.html#unmodifiableList(java.util.List)): a backed unmodifiable view illustrates alias visibility. Java's use-site generic model and mutation methods that reject writes should not be copied accidentally into a capability-only interface.
- [LanguageExt collections](https://github.com/louthy/language-ext): an independent .NET project's immutable Arr/Seq collections illustrate a different goal from a live view. Its documentation is a comparison lead, not measured evidence that those structures are faster or appropriate for this slice.
- [.NET ImmutableArray](https://learn.microsoft.com/en-us/dotnet/api/system.collections.immutable.immutablearray-1?view=net-10.0): an immutable-collection alternative to compare if a future consumer actually requires snapshot stability.

Validated on the existing runtime source at `2348ad9` and the `.10` bundle (neoCLR
`6e4051e`, Raven `f2a4af608`): six direct runtime tests and five Raven checks pass.
The baseline array/interface suites passed 27 tests. The C# comparison targets .NET
10.0, built with SDK 11.0.100-rc.1.26425.128; it is executable comparison evidence,
not a survey of every .NET collection behavior. No runtime or Raven implementation
changes are required by this prototype.
