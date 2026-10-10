# Basic collection library qualification — 2026-10-10

Development library slice: Queue<T>/ArrayQueue<T>, Stack<T>/ArrayStack<T> and
Set<T>/MutableSet<T>/HashSet<T>. These use ordinary generic library code, existing
Option values and existing comparer/array contracts. No new runtime intrinsic,
thread-safety guarantee or scheduler integration is introduced.

The [consumer](Main.rvn) passes native macOS ARM64 and matching interpreter execution
with the compiler `71cafd353` development bundle. It covers FIFO/LIFO ordering,
wraparound, growth, clear/reuse, reference identity, interface dispatch, shallow
snapshot iteration, hash collisions, chain head/middle/tail removal, freed-slot
reuse, growth after removal and ordinal Unicode membership. [Evidence](validation.json).
Windows uses the dedicated `windows-native-collections.yml` action; qualification
passed for Queue/Stack/Set at a5378b44; [verified Windows evidence](windows-validation.json) records verification of 1,481 downloaded artifact hashes and seven source-input checks. The [interpreter GC probe](../native-collections-gc/)
checks that draining a still-live queue releases its element objects.

Run `scripts/validate-native-collections.py --bundle <bundle> --output <fresh-dir>`
on macOS ARM64. Windows rebuilds the pinned bundle if --bundle is omitted. The
validator builds the matching interpreter and AOT tool, records hashes and runs an
isolated native executable plus the same assembly in the interpreter. No timing
comparison is claimed. The earlier experiment remains evidence for the design,
not the production implementation: HashSet now unlinks chains and reuses freed
optional slots instead of rebuilding the map on removal.

## Iterable Map pairs

The consumer now also checks Map/MutableMap/Iterable dispatch, imported generic
KeyValuePair construction/copying/getters/deconstruction, pair snapshots after value
replacement/insertion, and foreach iteration. It requires compiler
`d5fadf968feaea2c1073255ef5cb04f997fc5604` from Raven’s existing native integration
line `codex/source-object-metadata-resolution`, not Raven main. The native record
contract is constructor/getter/deconstruction storage only; full record helpers
remain unsupported. Four rejection fixtures cover equality helpers, mutable
components, record classes and imported read-only assignment. The validator requires
compiler diagnostics for each, rather than a successful or crashed compilation.

The pinned bundle helper can build that exact compiler before rebuilding the source
libraries. Windows HTTP/JSON/collection development qualification shares this pin
because all three rebuild System.Collections. The original Queue/Stack/Set report
is preserved; map-specific results are retained separately in map-validation.json.

The [focused JSON regression](json-map-regression.json) also passes native macOS
and interpreted execution with this bundle, preserving nested object/list/map JSON
behavior after Map gains Iterable. [Map validation](map-validation.json) records
the positive consumer and four explicit rejection cases.

Windows pair run [38002073187](https://github.com/marinasundstrom/neoCLR/actions/runs/38002073187)
failed during compiler preparation: the nested evidence path made a generated
BoundNodeGenerator executable path 262 characters long. Compiler staging now uses
a short repository-level temporary checkout; evidence remains at the requested
output path. A focused regression covers path length, failure diagnostics and
checkout cleanup. Pair execution was not reached in that failed run.

The short-checkout rerun [38032202226](https://github.com/marinasundstrom/neoCLR/actions/runs/38032202226)
passes Windows x64 native/interpreter collection and pair checks plus all four
rejection fixtures. [Verified evidence](windows-map-validation.json) records 1,560
downloaded file hashes, 11 source-input hashes, matching isolated executable/build
hashes, and a standalone dependency list containing only KERNEL32.dll. The sibling
[reflection/JSON run](windows-json-map-validation.json) also passes; 1,571 downloaded
file hashes and 11 source-input hashes were checked. Interpreter/AOT executables
outside the uploaded artifacts were stability-checked in CI, not rehashed locally.

## Iterable construction and ToMap (development, 2026-10-10)

The consumer additionally checks HashMap copying from a Sequence of pairs and an
existing map, callback comparer construction, pair queries, key/value selectors,
empty input, independent storage, shared reference values and case-insensitive
lookup. Three duplicate consumers exercise constructor, pair ToMap and selector
ToMap rejection. Their iterator prints `disposed` exactly once; reading beyond the
duplicate faults differently, detecting accidental continued enumeration.

Run the same collections validator for native/interpreter checks on macOS and the
existing Windows collections action. The source now uses the `module` convention.
[.NET comparison source](dotnet-baseline/Program.cs) runs with
`dotnet run --project docs/experiments/native-collections/dotnet-baseline/Baseline.csproj`;
.NET 10.0.0 rejects duplicates in all three entry points and confirms shallow
independent storage. The neoCLR duplicate failure is terminal, unlike ArgumentException.

A pair-array literal passed the interpreter but exposed the separate native
`NewArray(record)` allocation rejection. The native consumer uses ArrayList/Sequence
pairs; record-array literals remain an explicit backend follow-up.

[Materialization validation](materialization-validation.json) records the macOS checks.
The root sample project explicitly includes Main.rvn so nested rejection fixtures
remain separate programs. [Windows map qualification](windows-materialization-validation.json)
passes; 1,686 downloaded files and 24 source inputs were hash-verified.

## Basic collection constructor follow-up

The consumer checks ArrayList/ArrayQueue/ArrayStack/HashSet construction from an
observed iterable: four iterator requests, twenty MoveNext calls, sixteen Current
reads and four disposals. It covers list/queue order, stack reversal (including
stack-to-stack construction), set deduplication, empty arrays, lazy queries,
independent copies and shared class values. The .NET baseline checks the same
ordering/deduplication policy. Run the existing Windows action for that platform;
local focused validation builds this consumer with the matching native bundle.

[Scoped constructor evidence](copy-validation.json) records successful native macOS
ARM64 and matching interpreter execution. Windows coverage is in the existing action.


## Native iterator scope exits (2026-10-10)

Compiler 494dede841bc09c9e4876ab51f8ba91c74d64ab7 repairs native for cleanup
and preserves unrelated return expressions when a method owns an iterator. The
consumer checks exhaustion (including empty input), break, continue, early return
and labeled outward continue with exactly one disposal per acquired iterator.
Compiler tests additionally cover nested use disposal order, outward goto and match
returns. The integration compiler remains on codex/source-object-metadata-resolution.
Windows qualification uses the shared compiler pin; prior Windows evidence above
does not qualify this new compiler.

The same consumer checks query acquisition/disposal counts and exact MoveNext/Current
counts before library simplification: advance-only Any and Count do not read Current;
predicate queries stop at the required element; Single probes a second element
without reading it. These are behavioral checks, not a throughput benchmark.

[Scope-exit evidence](for-cleanup-validation.json) records passing macOS ARM64
native and matching interpreter execution against the exact compiler revision.

## Selective runtime loop simplification (2026-10-10)

The follow-up converts collection-copy loops and element-consuming queries to for;
advance-only/stateful query operations use scoped explicit iterators. Count and map
duplicate rejection retain explicit disposal before terminal Faults. The same
consumer checks exact MoveNext/Current/disposal counts, now also asserting Single
results, predicate multiple-match short-circuiting and no-match First completion.
[Audit decisions](../../raven-conventions.md#runtime-source-audit) explain retained
manual ownership and the deferred character-snapshot candidate.

The [JSON regression](json-for-cleanup-validation.json) passes native and interpreted
execution with the corrected compiler before these library-body simplifications.

[Refactored-library evidence](scoped-library-validation.json) records macOS ARM64
native/interpreter success against the final source hashes. Reference fingerprints
and the native API snapshot are refreshed; public signatures remain unchanged.

[Verified Windows compiler-cleanup evidence](windows-for-cleanup-validation.json)
passes all five native/interpreter cases at 4d34f35b, with 1,694 artifact files and
24 source inputs checked. This qualifies the compiler fix and iterable constructors;
it predates the final runtime source simplification commit.


## Migration to the Raven test framework — 2026-10-10

The Map iteration/materialization checks in `CheckMaps` and `CheckMapConstruction`
now also live as five focused `[Test]` module functions in
[MapMaterialization.rvn](../../../runtime/raven/tests/collections/MapMaterialization.rvn).
They report individual assertion failures through the Raven runner, and can be
selected with `--filter Map`. The first port preserves snapshot timing, comparer
behavior, independent storage and shared reference values; it also uses `use` for
captured iterator lifetime and `for` for ordinary traversal.

Keep this larger executable as an integration smoke test until the migrated cases
have Windows equivalence evidence. New behavior tests should be added to the Raven
suite rather than duplicated here. Terminal duplicate-key fault/disposal programs
and compiler rejection cases still require their specialized harnesses.

The queue/stack entry checks and `CheckSnapshots` are also ported into six
[QueueStack.rvn tests](../../../runtime/raven/tests/collections/QueueStack.rvn).
They retain FIFO/LIFO growth, wrapped contents, shared reference identity, clear/reuse
and mutation-after-snapshot cases. Explicit small capacities guarantee that growth
is exercised; the tests do not pin an internal growth factor.
