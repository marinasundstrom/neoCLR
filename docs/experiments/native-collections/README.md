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
