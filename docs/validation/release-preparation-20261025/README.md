# Next-preview preparation — 2026-10-25

Status: author-directed preparation, not a frozen or qualified release candidate.
Latest published baseline: [Preview 17](../release-20261023/README.md). Version,
publication date, source/compiler pins and final artifact hashes remain unset.

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
   inputs. Keep Raven main and its neoCLR line separate. Cargo's source package
   currently still says preview.13 while the published product is Preview 17;
   use the release tooling's actual version inputs and reconcile them deliberately,
   rather than inferring a new release from Cargo.toml alone.
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
