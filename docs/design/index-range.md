# Index, Range and slicing — development

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
String slicing must preserve that unit and avoid repeated scans from the beginning
for each character. A bounded scan to locate the two byte boundaries followed by a
single copy is the intended next layer; exposing raw storage bytes through string
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

String/sequence support and performance measurements remain open. Array copying is
one allocation plus a linear element loop; no timing claim is made by these checks.
