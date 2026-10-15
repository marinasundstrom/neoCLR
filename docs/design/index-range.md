# Index, Range and slicing — development

The [generic bounds review](generic-bounds.md) proposes a coordinated Raven endpoint
change across targets (2026-10-15). It does not change the implemented contracts below.

Author direction (2026-10-10): supply runtime types for Raven's existing indexing
and slicing syntax across arrays, strings and suitable sequences. On 2026-10-11
the author also requested attention to performance, with span-like facilities or
other designs evaluated later rather than copying .NET's constraints automatically.

## Contract and implementation layers

The .NET 10 baseline is [Index](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Index.cs)
and [Range](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Range.cs)
(retrieved 2026-10-11). Index stores a nonnegative distance and a direction. From-end
zero denotes the end boundary, not the last element. GetOffset resolves relative to
a supplied length; the consuming indexer checks element bounds. Range has an inclusive
start and exclusive end; resolving a range checks ordering and bounds. Empty ranges
at either boundary are valid. Equal values describe the same boundaries, independently
of a particular collection length.

neoCLR uses ordinary library structs, typed equality and explicit formatting/hashing.
Index and Range describe positions and boundaries. The author's clarified policy
(2026-10-11) applies faulting behavior to APIs **using** those values: indexers and
slicing syntax are convenience access whose caller must establish valid bounds.
They remain memory-safe and bounds-checked. Array.Get(int/Index) returns Option<T>;
Array.Slice(Range) and Range.TryGetOffsetAndLength return Result with InvalidRangeError.
GetOffsetAndLength and compiler-generated GetSubArray fault for invalid bounds.
A negative Index distance is separately an invalid value construction, as in .NET.
Faulting APIs use the platform's terminal Fault rather than .NET exceptions. Negative lengths are rejected explicitly. Object boxing/equality
parity is a separate value-type gap; typed Equals is the supported comparison.
The returned offset/count tuple uses neoCLR's value Tuple representation, not a CLR
ValueTuple ABI. These are library contracts, not special runtime opcodes.

Raven binding already recognizes the well-known types. Native lowering needs to
construct the same values, preserve evaluation order and call the library helper
for array copies. Normal CLI emission must retain its existing behavior. Numeric
range enumeration and slicing are distinct: a System.Range value describes half-open
boundaries even where numeric `for` ranges use inclusive `..` notation.

## Copies, views and performance

[Microsoft's range guide](https://learn.microsoft.com/en-us/dotnet/csharp/tutorials/ranges-indexes)
distinguishes array/string copies from span/memory views. The first array operation
here makes one destination array and shallow-copies the selected elements. Replacing
an element in the result must not modify the source; referenced objects remain shared.
Index/Range are values, with no per-element wrapper objects. No throughput advantage
is claimed by choosing this implementation.

String indexing already counts grapheme clusters, unlike .NET UTF-16 code units.
String slicing preserves that unit and avoids repeated scans from the beginning
for each character. A bounded scan locates the two byte boundaries followed by a
single payload copy; exposing raw storage bytes through string
syntax is not the contract. Known-count Sequence does not imply contiguous storage
or cheap indexing; arbitrary Iterable must not gain hidden eager enumeration.

Future view choices remain proposals:

- A restricted borrowed view, comparable to [.NET 10 Span](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Span.cs),
  avoids copying but requires escape/lifetime checks and rules for suspension.
- An owner-retaining offset/count value is easier to store, but retains backing
  allocations and must define mutation, resize invalidation and GC tracing.
- A specialized immutable text view can retain a source and boundaries, but a small
  view may keep a large source alive. Grapheme boundaries still require locating.

Before selecting one, measure representative JSON/HTTP parsing and repeated slicing:
allocations, copied bytes, boundary-scan work, retained memory, interpreter and AOT
throughput. Include tiny, empty, whole-value and large slices, non-ASCII/combining text,
reference elements and nested slices. Compare copy and view semantics explicitly.
Compiler bounds-check elimination and bulk copying can be optimized underneath the
same observable contract. Avoid promising constant-time text indexing or calling a
new design better until benefits and costs are measured.

## Qualification sequence

1. Value contracts, native construction, boundary resolution and array copies.
2. Grapheme-preserving string slicing and suitable sequence APIs.
3. Focused performance measurements for the chosen text/sequence algorithms.
4. Separately designed borrowing/view support when runtime lifetime rules are ready.

Windows execution and release qualification are separate from local development
checks. Public API snapshots and concrete evidence accompany each completed slice.

## Development implementation limits

Index uses two constructor overloads instead of an optional Boolean parameter because
native optional-parameter export is not yet supported. Range resolution exposes
Tuple<int, int> (Item1 offset, Item2 count); named tuple binding during runtime library
bootstrapping remains a compiler gap. No .NET behavioral divergence is intended by
these temporary declaration limitations. Imported native value fields support reads
from values or initialized managed receivers; external value-field writes/addresses
remain unsupported. Runtime Contract configuration is unchanged.

## Array milestone evidence — 2026-10-11

- Raven `ad3409e06910bdb33bccd0cfdb1431401f248349` on
  `codex/source-object-metadata-resolution`; no structural Function work included.
- Runtime libraries: `target/index-range-libraries-v9`; development bundle
  `target/index-range-bundle`. Built from the preceding compiler working tree;
  exact artifact hashes and dirty source provenance are retained in the manifests.
- `scripts/test-runtime-library.py --suite index-range` report in
  `target/index-range-tests-v2/report.json`: nine tests plus six guard processes,
  each passing in interpreter and macOS ARM64 native AOT. This covers boundary
  semantics, copies/reference identity, evaluation order, readonly receivers and
  safe Option/Result alternatives. Guard faults are expected nonzero exits.
- Metadata `--authored-references` checks authored/imported value fields, generic
  substitution, initialized receivers, invalid receivers and rejected writes.
- Four native libraries build; aggregate and native API snapshots are refreshed.
- Windows collections CI uses the new compiler pin and includes this suite through
  the runtime-library runner. Windows execution has not been performed locally.

String support is qualified below; sequence support and throughput measurements remain open. Array copying is
one allocation plus a linear element loop; no timing claim is made by these checks.


## String milestone — 2026-10-11

String now has Index and Range indexers, Get(int/Index) returning Option<char>, and
Slice(Range) returning Result<string, InvalidRangeError>. Boundaries count extended
grapheme clusters, matching String.Length and the existing integer indexer. This is
a deliberate difference from .NET UTF-16 indexing: complete user-perceived characters
are retained, at the cost of segmentation work and different offsets from .NET APIs.
The shared pinned Unicode segmentation implementation is used in interpreter and AOT.
No normalization, byte indexing or borrowed view is introduced. Empty slices at the
end are valid; ^0 is not an element index. Invalid range indexers fault, while Slice
returns InvalidRangeError. Successful results contain immutable copied text.

Implementation performs one length scan to resolve relative bounds, then a bounded
scan to locate byte boundaries and one payload copy. It does not enumerate/materialize
Char[] or repeatedly rescan for each grapheme. String.Get(Index) currently performs a
second count via Get(int); this is a bounded extra linear pass, not constant-time
access. The private StringSliceGraphemes service independently rejects invalid integer
bounds and publishes no result on failure. Native execution roots allocating calls;
the kernel does not collect or reenter. This is not a public interop ABI.

Evidence: target/string-range-tests/report.json, using target/string-range-bundle
and target/string-range-libraries-v2 with Raven ad3409e06. Twelve framework cases and
eight fault guards run in both interpreter and macOS ARM64 AOT. String cases include
combining accents, ZWJ emoji, regional-indicator flags, empty text and evaluation order.
The standalone string-unicode-test.c passes UBSan and warnings-as-errors, including
one-allocation accounting and unchanged output/allocation state for invalid bounds.
These checks establish allocation shape, not comparative throughput. Windows uses the
same library/kernel through the existing CI runner; execution remains pending.

Suitable sequence APIs and performance comparison remain next. No compiler change
was required beyond the preceding Index/Range lowering commit.


## Sequence and list contracts — 2026-10-11

`SequenceAccess<T>` supplies Index/Range extension indexers, Get(int/Index) returning
Option, and Slice(Range) returning Result<T[], InvalidRangeError>. The default
slice validates against Count, skips to the start with one iterator and copies
exactly the requested elements. It disposes on normal completion and early
exhaustion; invalid and empty default slices acquire no iterator. A provider that
ends before Count produces InvalidRangeError. Provider faults remain terminal.

`List<T>` declares Index get/set and Range get contracts; ArrayList implements them
using its logical Count, never its spare capacity. Range access returns independent
shallow storage using the existing linear copy helper. Sequence.Slice checks for
List<T> after bounds validation and delegates to that interface's Range indexer.
Other sequence implementations retain the iterator fallback. This avoids assuming
constant-time indexed access for an arbitrary sequence. There is no concurrent
snapshot guarantee, and list implementations must honor the copy contract.

Raven prefers applicable instance indexers to extension indexers. A read-only
instance indexer cannot be bypassed by a writable extension. A list-typed expression
therefore selects its declared contract; an expression typed as Sequence uses the
extension fallback, whose Slice method dispatches to a List when available.

Compared with .NET List.GetRange, the result is likewise a shallow independent copy,
but neoCLR returns an array and exposes a Range member in its List interface. The
benefit is a common slice contract and provider-specific copying; the cost is an
additional member required of every List implementer. Unlike IEnumerable, Sequence
requires Count and indexed read capability. Arbitrary Iterable slicing is excluded.
Primary comparisons: [List<T> implementation](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/Collections/Generic/List.cs)
and [C# indices/ranges](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/member-access-operators#indices-and-ranges).

After this slice, investigate view/buffer APIs against .NET Span, ReadOnlySpan,
Memory and ReadOnlyMemory. Candidate designs may change neoCLR allocation and
reference representation: owner-plus-offset views, runtime-managed slice references
and explicit borrowing. Evaluate relocation/GC roots, retention, aliasing, async
lifetimes, native/CLI interop and measured copy/allocation costs before selecting
public types. No representation or zero-copy API is committed by this plan.


Sequence qualification: [retained report](../../runtime/raven/tests/sequence-access-validation.json),
from `target/extension-indexer-final-tests-fixed/report.json`, using
`target/extension-indexer-final-bundle` and `target/extension-indexer-libraries-v4`.
Libraries use Raven 9e542200d; the consumer compiler includes the subsequent
reference-identity candidate-deduplication fix at Raven `163050da2`. Twenty cases and ten
fault guards pass in each of interpreter and macOS ARM64 AOT. The tests prove one
iterator/no indexed reads for the fallback, List dispatch/no iterator on the fast
path, independent copies, generic metadata import, writes and compound assignment.
They establish behavior and algorithm shape, not comparative throughput. Windows
execution remains pending; existing collections CI uses the matching compiler pin.
