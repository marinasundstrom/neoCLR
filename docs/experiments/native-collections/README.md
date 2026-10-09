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
is pending for this commit. The [interpreter GC probe](../native-collections-gc/)
checks that draining a still-live queue releases its element objects.

Run `scripts/validate-native-collections.py --bundle <bundle> --output <fresh-dir>`
on macOS ARM64. Windows rebuilds the pinned bundle if --bundle is omitted. The
validator builds the matching interpreter and AOT tool, records hashes and runs an
isolated native executable plus the same assembly in the interpreter. No timing
comparison is claimed. The earlier experiment remains evidence for the design,
not the production implementation: HashSet now unlinks chains and reuses freed
optional slots instead of rebuilding the map on removal.
