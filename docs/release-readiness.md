# Upcoming release readiness

Audit: 2026-10-17. Required release targets: **macOS ARM64 and Windows x64**.
No new release number, tag, publication date or published artifact is selected.
This is the active release work order, supporting the platform roadmap and
[toolchain tracker](tracking/toolchain-release.md). Published release records remain frozen.

## Current assessment

Native compiler bootstrap construction is complete for the qualified repository
build. That is not yet a qualified release package. At audit time neoCLR is
`677279b35` and Raven `codex/neoclr` is `57dbabb175f8d934745c22f990657b2b8d1bd5c6`.
There are 279 neoCLR commits after Preview 15, so the old release's delta-only
qualification cannot certify this candidate.

Evidence already available:

- Five native libraries build without a CLI semantic core. Native parsing/async
  consumers pass interpreter/JIT/sanitized AOT and editor completion; see
  [native-core qualification](validation/native-core-removal/README.md).
- The isolated macOS SDK/editor sample compiles, prints 42 and exits zero. Its
  runtime binaries are debug builds, not optimized release artifacts; see
  [installation evidence](validation/native-core-removal/installed-preview.json).
- Website HTML and local links pass for 4,441 pages, alongside 22 website and four
  native snapshot tests. Published download links still identify Preview 15.

## Candidate validation checkpoint (2026-10-17)

An optimized macOS runtime/backend and extracted native-core bundle were built
locally. Archive acceptance and untimed CPU/JSON preparation exposed two native
codegen gaps before measurements began:

- Native UTF-8 byte arrays and grapheme arrays needed exact InternalCall result
  admission. UTF-8 results also needed an ordinary managed-array tag and inclusion
  in nominal interface dispatch. The focused fix and executable evidence are in
  [text-vector validation](validation/release-candidate-20261017/README.md).
- The packaged JSON consumer still fails on a scalar-union instance test in an
  async function. This is a blocker in an existing packaged workload, not an
  expansion of the parked structural-type scope.

The original archive retains the pre-fix backend and is not qualified. Rebuild it
from a validated revision after the remaining correctness failures are closed.
Windows execution, editor package qualification and runtime measurements remain
pending. The local Raven revision is ahead of the remote branch; candidate pinning
must establish a remotely available, matching compiler/server/extension cohort.

## Required before release

| Gate | Observed gap | Required outcome |
| --- | --- | --- |
| Candidate identity | The native-toolchain workflow pins Raven `51dd22f5a`, before the `57dbabb17` execution-core deduplication and native Console fixes. Its package version still identifies the old release. | Select exact available Raven/runtime revisions and a distinct candidate package version. Record hashes for compiler, server, extension, libraries, runtime, codegen and native hosts. |
| Source qualification | No fresh complete source gate is recorded for this candidate. The native source validator still checks the legacy seed fixture. Its pinned hash was reproduced successfully, but that is not production-core acceptance. | Run canonical extracted-source full tests once, formatting/Clippy and minimum-Rust checks; retain focused macOS/Windows host/ABI checks. Keep legacy fixture evidence distinct from the native library/package gate. |
| Optimized packages | Local installed preview is debug; no fresh Windows native-core package acceptance is recorded. | Build optimized macOS ARM64 and Windows x64 runtime/backend/host packages from the same selected revisions. Extract and validate each actual archive, not a working checkout. |
| Runtime/library correctness | Narrow bootstrap consumers do not qualify all production wrappers or all newly integrated type-system behavior. | On both targets exercise packaged collections, parsing/text, checked arithmetic, readonly/type initialization, variance/imported generics, Function invocation, reflection/JSON, async, HTTP/TLS and GC. Use interpreter/JIT/AOT for supported workloads. Reuse unchanged focused evidence only with explicit dependency justification. |
| Editor | The new preview has focused completion evidence, not the full packaged editor gate on both hosts. | Validate native-only completion, hover, navigation, inlays, configuration recovery and build/run tasks with the exact shipped SDK/VSIX on both platforms. Test stale-output rejection and HTTP task inputs. |
| API coverage | The website reports 123 unmatched legacy routes. Inspection shows 104 flat redirect routes and 19 hierarchical pages, not 123 missing APIs. | Verify redirect destinations; classify the 19 pages against native metadata. Preserve documented current APIs, remove obsolete scaffolds from current navigation, and retain honest notices/manual references for real coverage gaps. |
| Documentation and packaging | The previous next-preview draft described the already released unified runtime. Native-core migration changes the compiler/library cohort. | Finalize current release notes, supported-feature limits, notices, package prerequisites and checksums. Change published download URLs only when actual new assets exist. |
| Runtime performance | Existing published CPU/JSON/HTTP measurements use Preview 14; the installed debug preview is not a release benchmark baseline. Windows workload preparation is incomplete. | Complete the benchmark preparation below, then retain and review runtime measurements on both release targets before publication. |

The 19 hierarchical API pages comprise two Enum helper pages, two namespace
containers and 15 exception/compiler-scaffold pages. This is a route inventory,
not a conclusion that any particular API can be deleted. Inspect native definitions
and supported contracts before removing or redirecting their pages.

## Runtime benchmark work order

Compiler performance is outside this gate. Build source programs and AOT outputs
before timed execution. Report process startup separately: it can include loading,
verification and JIT preparation, but must not be mixed into steady-state throughput.

1. Host selection is implemented for CPU/JSON and plain/JSON HTTP preparation:
   macOS ARM64 uses `console`/`http` and `app`; Windows x64 uses
   `windows-console`/`windows-http` and `app.exe`. Thirteen framework tests pass.
   Actual Windows execution remains unqualified. Windows database preparation
   explicitly remains unsupported.
2. Run untimed correctness/preparation checks against each fresh extracted release
   bundle. Do not time builds concurrently with a benchmark or publish failed runs.
3. Rerun CPU summation, JSON DOM serialization/deserialization, and HTTP plain/typed
   JSON in interpreter, JIT and AOT on macOS ARM64 and Windows x64. Preserve each
   result checksum and validate every HTTP response. PostgreSQL remains a separately
   identified optional workload with a pinned disposable database; its absence must
   not be reported as a successful measurement.
4. Use at least five fresh-process rounds with rotating mode order and recorded
   warmups. Preserve the existing larger native CPU batches (10,000 operations,
   20 warmups) rather than reverting to sub-millisecond samples. Keep interpreter
   and JSON batch settings explicit. Retain raw samples, median, range, errors,
   hardware/OS/toolchain, power/load conditions, exact artifact and harness hashes.
5. Record allocation/GC diagnostics separately where supported by existing profiling
   harnesses. Do not describe RSS as managed allocation rate or infer allocation
   savings from timing alone. Profile suspicious regressions before changing code.
6. Compare the new candidate with Preview 15 on the same host using the same workload
   and harness where compatible. Keep Preview 14 published numbers as historical
   evidence, not a controlled before/after comparison. If an old bundle cannot run
   the workload unchanged, document the incompatibility rather than invent a ratio.
7. Review regressions and repeat material or noisy differences under matched
   conditions. These fixed-count exploratory harnesses have no established universal
   regression threshold; do not manufacture one. Record unresolved performance
   risks explicitly before declaring release readiness.
8. Update benchmark pages from the accepted raw evidence. Keep macOS and Windows
   results separate; different hardware does not establish a platform speed ranking.

Existing procedures: [execution](../benchmarks/execution/README.md),
[HTTP](../benchmarks/http-server/README.md),
[framework](../benchmarks/framework/README.md),
[source gates](next-preview-validation.md).

## Order and completion rule

First reconcile candidate pins and API inventory, then prepare and qualify both
optimized packages. Port/check benchmark preparation while those gates are being
established. Measure the frozen packages after correctness passes, investigate
regressions, and complete release metadata and artifact review. A later code fix
invalidates affected measurements and checks; a prose-only correction does not
require rerunning runtime workloads.

Both target packages and their benchmark records are required by the author.
No fresh runtime measurements have been taken by this audit. It does not claim
remote CI status, Windows execution or publication. Structural expansion remains
parked; known bounded limitations do not imply a requirement to implement every
unsupported feature before this release.
