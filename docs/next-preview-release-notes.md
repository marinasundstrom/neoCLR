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

The refreshed macOS package passes installed-editor acceptance, including exact
`Task<int>` inlays and repeated two-project diagnostic recovery. The six extracted
source-test failures are closed by focused array-interface and reflection checks;
the repaired source smoke gate also passes. Rust 1.85 all-target checks pass on
macOS, Windows and Linux. These checks do not replace Windows package acceptance.

Fresh macOS [CPU/JSON](../benchmarks/execution/candidate-refreshed-20261017/README.md)
and [plain/typed-JSON HTTP](../benchmarks/http-server/candidate-refreshed-20261017/README.md)
measurements pass correctness checks across interpreter, JIT and AOT. Compilation
is outside execution measurements; startup and steady-state results are separate.
Active host indexing limits performance conclusions. Windows measurements and
matched HTTP baseline comparisons remain open; no cross-platform speed claim is
made.

Final release notes must link exact package hashes, platform qualification and
benchmark results, document remaining API-reference gaps and state the actual
optional-provider contents. Prior release results do not certify this candidate.
