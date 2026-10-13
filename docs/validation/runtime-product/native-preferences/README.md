# Native preference output qualification

Generated GC calls select rooted output allocation for system culture and pinned
time-zone version strings. The captured OS preference buffer/snapshot or static
version bytes stay alive through retry. No preference acquisition is replayed by
the retry; raw/non-GC entry points remain noncollecting.

All 30 local sanitized GC/root executions pass. The real-provider pressure fixture
checks output parity, exhaustion, missing/foreign contexts, unchanged failures and
complete reclamation. Its short comparison assumes host preferences are not changed
concurrently. The generated consumer passes GC and non-GC compilation/execution,
verifies imported symbols and checks GC root/result cleanup. Fixture declaration
resolution and a local toolchain-path typo were corrected before the retained run.

[Hosted Windows run 38452171576](https://github.com/marinasundstrom/neoCLR/actions/runs/38452171576) passes at `2f61c085f5ffe960cca34f3480be7f226fe415a5`: all 30 GC/root executions, socket and relocated-package checks on Windows x64. Reports retain source hashes and command provenance. Local macOS ARM64 validation is recorded above. The hosted macOS job remained queued for a runner when this evidence was saved; the overall hosted workflow is not yet qualified. No fresh managed Windows/JIT/HTTP or Windows ARM64 matrix is claimed. Linux culture implementation remains
unsupported and explicitly diagnosed; this does not claim Linux ARM64 support or
change public managed contracts or collection pacing.
