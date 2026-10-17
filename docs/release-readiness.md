# Preview 16 release readiness

Release targets: **macOS ARM64 and Windows x64**. SDK/extension version
`0.1.16-neoclr.20261017.1`, Raven `c0854cdda`. The
[qualification record](validation/release-20261018/README.md) owns exact revisions,
archive hashes, raw reports and the remaining publication status.

| Gate | Evidence and status |
| --- | --- |
| Source | Canonical extracted-source full suite and 25 smoke programs pass at `efa1687c8`, with strict Clippy/formatting, release-host checks and minimum-Rust checks on Windows/macOS/Linux. The qualified-source archive is that exact snapshot. |
| Optimized packages | Both extracted platform packages pass. README-only publication repacking preserves every executable/library byte and verifies every manifest hash. |
| Native metadata | Production System.Runtime and dependent libraries are built from native metadata and explicit runtime-service contracts, without CoreProbe semantic references. The retained seed supplies execution support. |
| Runtime/library | Packaged collections, JSON/reflection, HTTP, GC and 24 await cases pass their supported interpreter/JIT/AOT matrices. Unchanged earlier focused type, checked-arithmetic and field/initializer evidence remains linked from the candidate record. |
| Editor | Both platforms pass installed-server completion/hover/navigation checks, exact Task<int> inlays and two ten-cycle two-project diagnostic-recovery runs. SDK/VSIX server files match. |
| HTTPS | Hosted security-provider checks pass on both platforms. Matching-package macOS HTTPS passes 15 AOT security/recovery/cleartext cases. Matching-package Windows JIT/AOT project qualification also passes. |
| Runtime performance | Final macOS CPU/JSON and plain/typed-JSON HTTP measurements pass correctness checks; final Windows measurements also pass against the exact qualified archive. Compilation is excluded from steady-state measurements. |
| API coverage | Native snapshot and website contract tests verify the reference and navigation. The route audit classifies 19 legacy-only hierarchical pages and validates 104 compatibility redirects; documented coverage limitations remain explicit. |
| Publication | Final release notes, asset hashes and website links are being prepared. No published Preview 15 artifact or notes are modified. |

The [earlier candidate record](validation/release-candidate-20261017/README.md)
retains the six repaired source-test failures, UTF-8/array and JSON codegen fixes,
callback inference and repeated stale-project-diagnostic repairs. Failed candidate
runs are not relabeled as release qualification.

## Scope and measurement limits

The release keeps broader structural work parked. Supported structural types,
nominal interface variance, Function reflection, ranges, checked operations and
field/type initialization retain their documented bounded contracts. Raven/.NET
and Raven/neoCLR remain separate compiler lines.

CPU/JSON and HTTP measurements use fresh processes, five rounds and explicit
warmups. Native CPU batches contain 10,000 operations; smaller pilot batches are
not used for native CPU conclusions. Startup is separate from throughput. Preserve
raw data, correctness checks, host/toolchain details and artifact/input hashes.
Interactive macOS and hosted Windows measurements are separate series, not a
cross-platform ranking or a production capacity guarantee. The syntax-adapted
Preview 15 comparison documents the historical endpoint spelling; it does not
justify precise speedup claims under different host load. PostgreSQL, allocation
rates, sustained concurrency and HTTPS throughput are not newly measured.

All execution gates pass. Publication still requires verified uploaded asset
checksums and the separate manual website deployment.
