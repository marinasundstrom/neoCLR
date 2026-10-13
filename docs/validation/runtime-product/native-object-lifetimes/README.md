# Native interning and object lifetime qualification

The string-intern slice is `f09da4fe6`; the object-construction slice is
`c82b9e3495f6f293931667a54ef0614f027ebb3b`. Local reports were recorded from their
working trees before commit; reports retain source hashes and commands. The final
header-only clarification describes GC versus arena-reset ownership accurately.

All 32 local sanitized GC/root executions pass on macOS ARM64 (31 at the interning
slice). New pressure tests protect an intern source absent from caller roots and
verify pool identity, embedded NUL, quota failure, exhaustion and reset. Object
tests verify zeroed payloads, headers, rooted receivers, missing/foreign contexts,
unchanged failure outputs and complete reclamation. Raw helpers do not collect.

Six compiled rooted-consumer tests pass, including integer boxes and the interning
binding. The constructor fixture was corrected to use `arrayref<Byte>` rather than
value-array spelling before its successful run. Its final extended version forces
at least three collections: before construction, inside construction and during
callback binding. It reads the surviving argument through the bound receiver,
matches interpreter/non-GC output, clears frames and reclaims the heap. The existing
callback-array lifetime/fault test and relocated-package check also pass.

The generated-consumer tests are local macOS tests, not Windows execution evidence.
The focused hosted platform run validates shared C GC/root/socket/package contracts.
No Windows ARM64, full managed HTTP/JIT matrix, general pacing or performance gain
is claimed. No managed API, Raven Runtime Contract or metadata encoding changes.

[Hosted run 38454170119](https://github.com/marinasundstrom/neoCLR/actions/runs/38454170119)
passes all 32 GC/root executions and socket/package contracts on Windows x64 at
the object-construction revision above. The macOS job was still queued when this
report was saved; local macOS evidence is complete, but the hosted workflow as a
whole is not yet qualified. The preceding preference run `38452171576` ended with
its macOS job cancelled without running steps; its Windows qualification remains
valid. No hosted macOS success is inferred from either run.
