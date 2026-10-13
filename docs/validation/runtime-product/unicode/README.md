# Rooted Unicode qualification

Generated GC calls protect source owners during casing, grapheme access/slicing and
character assembly. Grapheme/scalar vector construction retains its source and
partial array through every allocation, with frame cleanup on all exits. Existing
native/non-GC helpers remain noncollecting. This relies on nonmoving storage during
Rust Unicode visitation, not a future pinning contract.

All 27 local sanitized GC/root executions and package checks pass. The fixture uses
the real pinned Rust Unicode library, combining marks, emoji and embedded NUL;
checks missing/foreign contexts, live exhaustion, borrowed-input omission, partial
vector failure, retained elements and complete cleanup. Six compiled character tests
pass, including GC/non-GC interpreter comparisons and fault retention/reclamation.
The initial test setup omitted a link input, then left an unused source unrooted in
its full-heap control; both fixture issues were corrected before retained passing runs.

[Hosted run 38448803103](https://github.com/marinasundstrom/neoCLR/actions/runs/38448803103) passes at `d87af0477b74f6e2fab036697a5ef2652b1f1e84`: 27 GC/root executions (including the real pinned Unicode library), sockets and relocated-package checks on macOS ARM64 and Windows x64, plus macOS profiler attachment. Reports retain source hashes and command provenance; this is not a new managed Windows/JIT/HTTP or Windows ARM64 matrix. No new throughput or pacing claim.
