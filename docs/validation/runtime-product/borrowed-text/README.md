# Borrowed path/replacement qualification

Generated native-GC lexical path and ordinal replacement calls retain every string
read after allocating the output. Copy/publication do not collect. Non-GC and raw
native calls remain noncollecting. Replacement no-op identity and error precedence
are preserved. Private generated symbols require matching native hosts.

All 26 local sanitized GC/root checks and relocated-package checks pass. The new
pressure fixture exercises ordinary combination, empty-side and rooted-right paths,
file-name extraction, expanding/shrinking replacement, live exhaustion, unchanged
failure output, missing/foreign contexts and final reclamation, deliberately omitting
inputs from caller roots during successful retry.

Two compiled path tests and two replacement tests pass. Paths compare UTF-8/NUL,
separator and invalid-input behavior with the interpreter; replacement runs twelve
cases in both GC and non-GC modes, including no-op identity paths, Unicode/NUL,
expansion/shrinking and invalid empty search. GC hosts collect before rendering
faults and check complete reclamation after clearing the fault.

[Hosted run 38447179411](https://github.com/marinasundstrom/neoCLR/actions/runs/38447179411) passes at `c2ca5e9165f7d967b9cf49f1c240b02f10da78f9`: all 26 GC/root executions, socket and relocated-package checks on macOS ARM64 and Windows x64, plus macOS profiler attachment. Reports retain source hashes and commands. This is not a fresh managed Windows/JIT/HTTP or Windows ARM64 matrix. Unix lexical semantics are unchanged on
Windows; these are not host-filesystem operations. No pacing or performance claim.
