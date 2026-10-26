# Next-preview preparation — 2026-10-25

Status: Preview 18 candidate preparation commenced by author direction, 2026-10-25.
Latest published baseline: [Preview 17](../release-20261023/README.md).
Runtime version: `0.1.0-preview.18`; SDK/editor: `0.1.18-neoclr.20261025.1`.
Raven `codex/neoclr`: `ed1d08defe2778f965951747d5f48311ce3daf83` (clean,
already on the remote branch). Exact neoCLR qualification revisions and workflow
runs will be recorded as they start. Publication date and artifact hashes remain
pending. This is not package acceptance or publication.

## Scope freeze

The author asked to wrap up optimization and commence macOS/Windows release
procedure. Stop adding optimization experiments during qualification. Keep sync GC
and production JIT memory placement as defaults. Grouped stack bounds, split-frame
validation and contiguous/fallback placement remain opt-in; the last composition
study is retained as development evidence rather than newly enabled release behavior.
Concurrent native GC remains private qualification tooling. Package current public
API/value/HTTP changes and integrated runtime fixes with a matching compiler.
Published Preview 17 downloads stay current until new artifacts pass their gates.

The author asked to integrate the improvements and prepare a release while
questioning whether concurrent GC should become the default. The assistant
recommends retaining synchronous GC as default: repeated native HTTP still favors
sync, and broad concurrent pause/scaling/default acceptance is incomplete. The
release narrative covers platform APIs, HTTP/value efficiency, general runtime/GC
improvements and clearly labeled concurrent experiments, rather than a GC diary.

## Integration inventory

| Area | Current evidence/state | Before candidate freeze |
| --- | --- | --- |
| Optional / Result.Failure / Error | Source/API migrations and focused validation recorded in the archived development history; Optional compiler evidence pins Raven main `694bbeae8` and neoCLR `5b9298b62` | Resolve current matching Raven/neoCLR revisions; preserve separate Raven branches and rerun affected package consumers |
| Environment / MediaType / Uri | Implemented source contracts and focused three-engine/macOS checks; public API documentation was refreshed | Rebuild matching reference/SDK snapshots and validate packaged consumers on claimed hosts |
| Ordinary runtime / synchronous GC | Root-publication, metadata/heap lookup, initialization and async-context improvements are in source with focused evidence | Audit active defaults versus opt-in flags; qualify the exact source and packaged native hosts |
| Concurrent scalar stores | Integrated within the private candidate, with explicit generic-store control; [validation](../../../benchmarks/gc-suite/matrix/scalar-store-integration.md) retains matching compiler/host evidence. Earlier standalone gains and small/mixed HTTP effects remain scoped | Qualify exact candidate/package inputs; this does not expose a public concurrent selector |
| Concurrent stack/byref policy | [Integrated complete-caller proofs and immutable byrefs](../concurrent-stack-policy-20261025/README.md), with matching standalone objects and native root/async qualification | Retain nonmoving collector and complete-closure boundaries; qualify exact package inputs |
| Concurrent native implementation | [Canonical runtime sources and selected owned policy](../concurrent-native-integration-20261025/README.md); runtime-only generation/compile and native safety checks pass | Keep private admission boundaries; select native-kit/package exposure explicitly and qualify final host artifacts |
| Other concurrent/worker policies | Multiple qualified and rejected experiments, different ownership/topology limits | Select only coherent supported combinations; do not enable every experimental flag or treat restricted shared access as arbitrary mutators |
| Concurrent execution exposure | `runtime/codegen/src/lib.rs::run` passes `live_stores=false`; private qualification entry enables it | Decide package exposure explicitly; do not describe a public selector that does not exist. Sync stays default |
| Persistent HTTP | Private close/keep-alive/pipeline matrix qualifies the internal driver | Resolve public timeout/resource/ownership policy separately before claiming public persistent-server support |

The pre-existing stack-transfer JIT failure was traced to missing macOS ARM64
instruction-cache publication and corrected at the shared JIT memory boundary.
[Investigation and validation](../jit-code-publication-20261025/README.md) include
reproducing negative controls; final packages still need their own qualification.

## Release gates

1. Complete the selected integration inventory and freeze exact clean neoCLR/Raven
   inputs. Keep Raven main and its neoCLR line separate. The source package version is reconciled to the explicitly selected Preview 18
   candidate; this version change does not certify or publish its packages.
2. Review [draft notes](../../next-preview-release-notes.md), migration guidance and
   API coverage against those exact inputs. Published notes/sections stay frozen.
3. Run canonical source/archive/notice validation under the
   [native distribution process](../../source-release.md), plus applicable stable
   host and minimum-Rust CI checks. Earlier release passes do not certify new bytes.
4. Build matching macOS ARM64 and Windows x64 SDK/editor packages and native kits;
   run extracted-source/project, interpreter/JIT/AOT, HTTP/HTTPS, API and editor
   gates applicable to each artifact. Retain unsupported additional hosts as gaps.
5. Run the [release benchmark procedure](../../../benchmarks/release/README.md)
   against final artifacts. Keep developmental GC experiments and package results
   labeled separately, with throughput, CPU and memory scopes explicit.
6. Audit dependencies/notices, archive membership, compiler/runtime manifests and
   checksums. Select version/date and promote the curated Unreleased entries only
   when preparing the actual candidate. Publication/tagging and website deployment
   remain separate actions; none has occurred here.

## Preserved history and measurement evidence

- [Full former Unreleased history](../../development-history/unreleased-through-2026-10-25.md)
  is preserved verbatim. `changelog-history.json` records its source commit/hash and
  verifies that published changelog sections were not changed by consolidation.
- [Standard GC suite](../../../benchmarks/gc-suite/README.md) and
  [scalar HTTP evidence](../../../benchmarks/gc-suite/matrix/scalar-store-http.md)
  retain qualification and repeated comparisons. These are development artifacts,
  not qualification of the forthcoming release packages.

## Candidate preflight

- `cargo metadata --locked --no-deps` accepts the Preview 18 manifest/lock pair.
- Initial formatting check found accumulated differences in runtime/codegen and
  two root files; `cargo fmt` applied them and the final check passes. This is
  release-gate normalization, without intended behavior changes.
- Initial API snapshot check found stale internal HTTP source fingerprints.
  Rebuilt the matching bridge/reference with Raven `ed1d08def`; public reference
  bytes/type inventory are unchanged and the refreshed fingerprint check passes.
- macOS SDK packaging starts from an isolated Git archive of that Raven revision,
  with the explicit neoCLR metadata project and candidate version. Packaging and
  qualification outcomes remain pending until recorded below.

## Started qualification and first repairs

- Runtime/code source candidate: `179a2d8763a0541aea6594faa233012d95a68e55`, pushed
  to main. [Windows SDK/editor qualification](https://github.com/marinasundstrom/neoCLR/actions/runs/40213940464)
  uses the exact Raven pin and `0.1.18-neoclr.20261025.1`; no prior bundle is reused.
- [First source run](https://github.com/marinasundstrom/neoCLR/actions/runs/40213931017)
  exposes a nonexistent `CoreOptional.neoil` include. Canonical/minimum-Rust jobs
  fail on it; this run is not acceptance. The prior source run also exposed legacy
  examples/test strings still using Error instead of Failure.
- Repair scope: use the retained `CoreOption.neoil` filename (artifact identity,
  not a public type), port three retained IL fixtures' Result case references to
  Failure, and update active legacy source examples/test strings to Optional and
  Failure. Their source maps remain historical emission provenance; these fixture
  spellings are adapted to the current runtime, not newly emitted compiler output.
- Focused run passes 125 tests and initially fails the imported corpus; after its
  IL spelling repair the remaining test passes. This gives 126 passing affected
  tests, with the failed attempt retained separately. Broader source CI must rerun.
- Website validation passes all 6,133 pages, including local links and shared API
  navigation. This is a local build, not website deployment. Preview 17 downloads
  remain unchanged.
- Fresh local runtime/backend release binaries build successfully. macOS SDK
  packaging's first bootstrap attempt exits 1 without a useful error diagnostic;
  the same bootstrap command with normal logging then succeeds. Packaging resumes
  against the same pinned sources; no completed SDK acceptance is claimed yet.

`migration-first.txt.gz`, `import-retest.txt.gz` and `website.txt.gz` retain the local
results. Source-only fixture/docs repairs do not change the runtime/compiler bytes
being built by the original package job; final artifact lineage must list the exact
source/test revisions rather than relabeling old artifacts with a newer commit.

## Qualification follow-up — 2026-10-26

The macOS SDK, matching VSIX, libraries, runtime and JIT-host packaging completed.
The archived/extracted bundle passes default-JIT and interpreter sample/HTTP
verification. Exact archive and manifest hashes and compressed reports are linked
from [candidate.json](candidate.json). The initial temporary extraction no longer
existed on resumption; interpreter validation uses a fresh extraction of the same
archive under `target/preview18-release/extracted-20261026`. Local strict Clippy
passed. This is partial package acceptance, not publication.

[Repaired source CI](https://github.com/marinasundstrom/neoCLR/actions/runs/40214540493)
passes minimum Rust on all three hosts and both macOS/Windows host checks, but
canonical tests expose five task-fixture failures: references to removed
`Option.methods.neoil`/`Option.helpers.neoil`. Update the fixture to `Optional`.
[Windows packaging](https://github.com/marinasundstrom/neoCLR/actions/runs/40213940464)
passes SDK/editor construction and extracted JIT samples, then stops at MSVC C4996
in the root-validation fixture. Replace its fixed-literal `strcpy` with the same
NUL-inclusive bounded `memcpy` used by adjacent cases; keep warnings-as-errors.
These are validation-fixture repairs; production package inputs remain unchanged.
Rerun both gates before claiming acceptance. Windows editor execution and the
remaining package/API, HTTPS, native-kit and benchmark gates remain pending.

The repaired macOS [native GC gate](native-gc-retest.json.gz) passes all 188
commands, including intentional rejection cases. Retained [source failure log](source-repaired-failure.log.gz)
and [Windows compiler diagnostic](windows-root-build.log.gz) establish the original
failures; macOS success does not certify MSVC.

All five previously failing task tests pass locally after the filename repair
([focused retest](task-fixtures-retest.log.gz)). Production runtime source is unchanged.

The macOS packaged async gate passes 24 executions and JSON passes all three
engines. Header acceptance initially failed to compile a retained `Error` pattern;
the header fixture and both release HTTP benchmark fixtures now use `Failure`.
The repaired header gate passes 132 cases. Both HTTP benchmark preparations compile
successfully; preparation is not a measurement result. Source requalification run
[40295956669](https://github.com/marinasundstrom/neoCLR/actions/runs/40295956669)
and Windows reused-package run
[40295956855](https://github.com/marinasundstrom/neoCLR/actions/runs/40295956855)
were started at `802cae647`; the latter predates the header-fixture repair and must
not be treated as validating it.

Native-kit console/HTTP and C/C++ interop acceptance pass. HTTPS passes 15
security, recovery and cleartext-isolation cases. All 320 packaged notice/source
inputs match the checkout. The packaged editor passes inlays; its full suite first
stopped after 18 checks because the harness still expected `Option<Order>`.
The harness now expects `Optional<Order>`. A retry in the previously mutated
workspace timed out at its initial `VersionOne` probe (the prior run had changed
it to `VersionTwo`); preserve that harness-state failure and use a freshly prepared
workspace for acceptance. The 87 SDK/VSIX server files include 81 byte-identical
files and six differences: SDK publishing uses the osx-arm64 RID, while VSIX
publishing is RID-independent. Both builds use the same archived Raven revision
and explicit metadata project; do not claim byte identity between separate builds.
Installed editor execution remains the acceptance gate.

### Compiler pin reaffirmed — 2026-10-26

The author clarified that this release remains pinned to the previous Raven
compiler. Backtick-delimited identifiers landed afterward in Raven `036111ec0`
and are deferred from this release; retain `ed1d08defe2778f965951747d5f48311ce3daf83`
for every candidate artifact and do not migrate release tests to that new syntax.
Source run [40296297625](https://github.com/marinasundstrom/neoCLR/actions/runs/40296297625)
passes canonical archive/full tests, strict Clippy/formatting, minimum Rust on
three hosts and both macOS/Windows host boundaries. Windows run `40296298072`
passes runtime/SDK contracts but fails editor acceptance; inspect its retained
report before treating the corrected local harness as Windows evidence.

The fresh macOS editor workspace passes all 26 checks with the Optional expectation
repair ([report](editor-final.json.gz)). Windows
[failed at the identical stale expectation](windows-editor-first.json.gz), after
18 successful checks; rerun against the same archive with this harness repair.

### Final measurements and remaining gates — 2026-10-26

The [macOS candidate measurements](../../../benchmarks/release-20261026/macos-preview18/README.md)
pass all five stages: 55 CPU/JSON processes and 1,152 HTTP responses, with clean
Git status before and after. Startup, ranges and limitations are retained on the
website. Both ten-cycle macOS editor recovery runs pass. All 469 standalone SDK
files match the bundle, 27 NuGet dependencies have reviewed attribution, and the
production extension JavaScript is reproduced with five bundled npm packages
covered by the existing eight-package inventory.

Windows HTTPS run [40298850434](https://github.com/marinasundstrom/neoCLR/actions/runs/40298850434)
passes against the original package archive. Windows package requalification
[40298792405](https://github.com/marinasundstrom/neoCLR/actions/runs/40298792405)
uses the repaired editor harness; benchmark run
[40298846341](https://github.com/marinasundstrom/neoCLR/actions/runs/40298846341)
waits for its success. Windows final acceptance/measurements and publication remain
pending; macOS success is not substituted for them.

The refreshed benchmark website builds and checks all 6,133 pages. This is local
validation; no website deployment or Preview 18 publication has occurred.

### Windows qualification complete — 2026-10-26

Run [40298792405](https://github.com/marinasundstrom/neoCLR/actions/runs/40298792405)
passes against the unchanged archive from `40213940464`. The retained
[Windows reports](windows-final/) cover extracted JIT/HTTP, GC, 24 async executions,
three JSON modes, 132 headers, all 26 editor checks, inlays and two ten-cycle
configuration recovery runs. The earlier editor failure is superseded by the
corrected harness, not erased. Windows HTTPS separately passes 27 cases.
The exact canonical source archive also matches its hosted SHA-256. Final Windows
measurements and publication remain open. Lazy JIT compilation with trampolines
is an author-requested post-release investigation, with no candidate-code change.
