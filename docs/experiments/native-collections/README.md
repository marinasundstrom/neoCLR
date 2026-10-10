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
remain separate programs. Windows qualification for these overloads is pending.
