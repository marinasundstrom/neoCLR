# Next preview — draft release scope

Unpublished draft. Release version and date are not selected. Intended binary
targets are macOS ARM64 and Windows x64; both require candidate qualification.
See the [release readiness work order](release-readiness.md) for remaining gates.

## Native compiler and library contracts

System.Runtime and dependent libraries are built from Raven source using native
metadata and explicit runtime-service contracts. Production construction no longer
requires the NeoCLR.CoreProbe/CLI semantic bootstrap. The compiler and editor import
native library types; the Raven tooling itself still runs on .NET.

## Language and runtime support

The candidate includes checked operators and contexts, consistent exclusive/inclusive
range endpoints, overflow-safe signed integer iteration, and reference-only nominal
interface variance. Static readonly fields and type initializers retain their native
metadata and execution contracts. Introspection describes fields, initializers and
generic variance; reflection performs supported operations under runtime guards.
Native Function types support typed patterns and provisional dynamic invocation.

These are bounded capabilities, not full .NET API parity. Further structural type
work remains parked; published documentation must retain the exact supported forms
and backend limits. The release will use matching compiler, libraries and runtime
artifacts; applications need to be rebuilt with that cohort.

## Tools and documentation

Native editor completion and source-local declarations share the compiler's native
metadata references. Hover improvements display union alternatives without expanding
unrelated nominal cases. Website guides describe current contracts and the native API
snapshot includes the source-owned core types.

## Performance and qualification

Runtime benchmarks must be rerun on the optimized macOS and Windows packages after
correctness acceptance. Compilation is outside execution measurements; startup and
steady-state results are reported separately. No performance improvement is claimed
until those measurements and regression investigations are recorded.

Final release notes must link exact package hashes, platform qualification and
benchmark results, document remaining API-reference gaps and state the actual
optional-provider contents. Prior release results do not certify this candidate.
