# Option and Result operators

Development API after Preview 8, implemented in Raven on 2026-09-19. Rebuild the
reference assembly, callers and System library together. Published Preview 8
packages do not include these operators. This is a bounded port of Raven.Core's
operators, not complete Raven.Core compatibility.

## Contract

Patterns remain the direct way to inspect outcomes. Operators compose callbacks;
none of them turn runtime faults into Result errors. Callbacks execute immediately
and only on the selected branch. Tap methods return the original outcome.
UnwrapOr takes an already evaluated default; UnwrapOrElse calls its factory only
for None or Error. Result.UnwrapOrElse follows Raven.Core's parameterless factory;
use OrElse or Match when the error is needed.

| Option operation | Behavior |
| --- | --- |
| Map | Transform Some; preserve None. |
| Then | Bind Some to another Option. |
| Filter | Retain Some if its predicate passes; otherwise None. |
| OrElse | Produce a fallback Option only for None. |
| UnwrapOr, UnwrapOrElse | Obtain a value or an eager/lazy fallback. |
| Match | Choose the Some or None callback and return its result. |
| Tap, TapNone | Perform an action on the selected branch. |
| ToIterable | Materialize an independent zero/one-element iterable. |
| ThenResult | Bind Some to Result; obtain an error from a factory for None. |
| MapResult | Map Some to Ok; obtain an error from a factory for None. |
| OkOr(error), OkOr(errorFactory) | Convert to Result with eager/lazy absence error. |
| Flatten | Unwrap one level of Option<Option<T>>. |

| Result operation | Behavior |
| --- | --- |
| Map | Transform Ok; preserve Error. |
| Then | Bind Ok to Result; preserve Error. |
| MapError | Transform Error; preserve Ok. |
| Match | Choose the Ok or Error callback and return its result. |
| Tap, TapError | Perform an action on the selected branch. |
| OrElse | Recover Error with another Result of the same type. |
| UnwrapOr, UnwrapOrElse | Obtain a value or an eager/lazy fallback. |
| ToIterable | Materialize Ok as one element, Error as no elements. |

There are 15 Option overloads and 10 Result overloads. ToIterable allocates an
ArrayList behind Iterable<T>; it is an explicit conversion, not an implicit
iteration contract on the carrier. Errors are discarded by this conversion, so
use Match or patterns when an error needs handling. Generic payloads retain their
normal value/reference semantics; the adapter does not deep-copy objects.

See the [executable example](experiments/raven-target/samples/library-outcome-operators.rvn)
and [expected output](experiments/raven-target/samples/library-outcome-operators.expected.txt).
Use imported case patterns and Raven extension declarations as described in the
[conventions](raven-conventions.md).

## Raven.Core and .NET comparison

The source is Raven.Core at revision `ddf10eca80d599ede28f9da59f145497a21e854e`:
[Option](https://github.com/marinasundstrom/raven/blob/ddf10eca80d599ede28f9da59f145497a21e854e/src/Raven.Core/Option.rvn)
and [Result](https://github.com/marinasundstrom/raven/blob/ddf10eca80d599ede28f9da59f145497a21e854e/src/Raven.Core/Result.rvn).
Map, Then and the outcome-specific names are retained. Option.Where becomes Filter
to match neoCLR's collection vocabulary; ToEnumerable becomes ToIterable to name
neoCLR's actual interface. This is a deliberate source compatibility difference.
The collection equivalents have a separate [.NET mapping table](raven-query-api.md).

Unlike nullable values or exceptions commonly used with .NET APIs, these carriers
make absence and recoverable failure explicit in a return type. This adds types
and handling to callers but supports exhaustive patterns and typed errors.
[F# Option](https://fsharp.github.io/fsharp-core-docs/reference/fsharp-core-optionmodule.html)
is a shipped .NET alternative with map, bind, filter and flatten. Raven uses member
syntax and Then for binding rather than copying every F# name.
[Rust Option](https://doc.rust-lang.org/std/option/enum.Option.html) and
[Result](https://doc.rust-lang.org/std/result/enum.Result.html) offer comparable
branch-selective transformations and eager/lazy fallbacks. neoCLR does not adopt
Rust's ownership semantics. These comparisons concern library contracts, not
claims that the runtimes implement allocation or error propagation identically.

UnwrapOrDefault and throwing unwrap helpers are deferred: generic default and
exception contracts require separate decisions. Nullable adapters, JSON support,
Result.WithContext (which depends on Raven.Core's context-error model), and .NET
GetEnumerator integration are not part of this port. Existing union ABI accessors
and propagation adapters remain; ordinary samples use patterns instead.

## Implementation and validation

The runtime sources use public Raven extension containers with self receivers.
Raven emits ordinary CLI static methods with ExtensionAttribute and flattened
generic method parameters. The bootstrap declarations retain matching C# this
receivers; the importer checks both sides carry the core-scoped extension marker.
This prevents a reference-only extension declaration from silently standing in for
an unmarked static runtime implementation. No Raven compiler or Runtime Contract
configuration change was required. Generic application extension bodies remain a
separate bridge limitation; the library uses an explicit closed-signature catalog.

Run verify_outcome_operators.py against a collection-profile saved project and its
matching library. It checks the website example, all operator branches and callback
selection, reference and completion payloads, callback faults and invalid calls.
The signature probe additionally verifies all 25 overloads and rejects wrong
receivers and virtual calls. Editor checks require the operators on both carriers.

Validated on 2026-09-19 using Raven SDK 0.1.12-neoclr.15 and the Preview 8 runtime
with rebuilt development references/library: 9 outcome integration scenarios,
251 signature checks, 86 editor sections, and clean reproduction of all 80 Raven
library slices. The MSBuild .rvnproj example prints the checked expected output.
No Rust runtime or Raven compiler modification was needed for this port.

The iterable regression suite also passes all 58 checks after the extension-source
conversion. Callback types are inferred where supported; the sample's Then
signature remains explicit because of a [recorded inference limitation](raven-target-evaluation.md#callback-inference-candidates-exposed-by-outcome-operators-2026-09-19).
