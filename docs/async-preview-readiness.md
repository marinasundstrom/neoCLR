# Async preview readiness — 2026-09-23

This is local preparation for the [author-selected async/Tasks release](async-preview-plan.md),
not a release certification. The release version/date and final candidate remain unset.
The first evaluator build uses clean isolated neoCLR and Raven worktrees; Raven stays
on `codex/async-preview-readiness`, based on the existing neoCLR feature revision.
No developer SDK selection or VS Code installation is changed.

## Gate inventory

| Gate | Concrete check / deliverable | Evidence boundary |
| --- | --- | --- |
| Library sources and generated artifacts | `build_runtime_library.py --check-snapshot`; full `--check --compiler … --bridge …` regeneration | All slices regenerate identically on the local recorded compiler/bridge; final-candidate repetition remains required |
| Fresh compiler/editor distribution | Raven `scripts/codex-build.sh`, `package-sdk.sh`, `package-vscode.sh` | Local macOS arm64 build; VSIX packaging is separate from an installed editor test |
| Runtime bundle | `package_bundle.py` with clean recorded repositories, freshly built runtime and SDK | Manifest records revisions, tools and file hashes; archive/extract before testing |
| Saved-source Async Workbench | Packaged `tools/verify_async_workbench.py --bundle … --sdk … --report …` | Eight exact-output cases through standalone MSBuild, in fresh temporary projects |
| Task/Promise contracts | Packaged `tools/task-contract/verify_tasks.py` and `verify_composition.py` | Completion, cancellation, capability boundaries and composition |
| Async lowering and default dispatch | Packaged `verify_async.py`, `verify_default_queue.py`, `verify_map_result.py` | Immediate/resumed paths, Result, cancelled await, GC and rejected constructs |
| Worker boundary | Packaged `verify_workers.py`; Rust `workers` and `runtime_services` tests | Dedicated/pool workers, default queue, invalid captures, quotas and host cancellation |
| Editor and project workflow | `configure.py --sdk …`; packaged `tools/task-contract/verify_editor.py msbuild-demo` and `tools/verify_msbuild.py` | Protocol probes do not replace interactive VSIX installation/build-run evidence |
| Source release matrix | `scripts/validate-release.py` at the selected commit; six platform/toolchain CI jobs | Still required for the final candidate; local macOS is not Linux/Windows evidence |
| Distribution review | Complete diff since Preview 8, migration notes, notices, checksums and website | Focus is async; every included compatibility change still needs review |

The six deeper packaged Task probes take `demo/Demo.rvnproj`, `--bridge
 tools/bridge/Probe.dll`, `--system lib/System.neoil` and `--runtime bin/neoclr`.
Run from the extracted bundle; the editor probe takes the configured project folder.

## Findings and fixes

- The bundle previously omitted the current Task probes. They are now packaged under
  `tools/task-contract`; source and extracted-package helper paths are both supported.
- The worker probe still expected failure without an explicit queue. It now checks
  the supported default-queue behavior. The ordinary worker library remains unchanged.
- Async Workbench now builds eight saved samples and records exact outputs, using only
  the supplied bundle and SDK. Existing samples remain the source of the demonstration.
- The release formatting check exposed three existing whitespace discrepancies;
  those were corrected without behavior changes.
- A fresh native build initially failed because the installed linker could not parse
  `arm64e.x1` in the macOS 27 SDK's libSystem stub. Selecting the installed
  `/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk` through `SDKROOT` allowed
  the clean release build to pass. This is a local toolchain prerequisite, not a
  claim that the default SDK build passed or a change to system settings.

## Local results

- SDK and VSIX packaged successfully from Raven `e56fc1ddf25d570d6aaa2bddfdf34138198f9941`,
  version `0.1.12-neoclr.async.20260923` (local experiment label).
- Runtime bundle built from neoCLR `20a964e0a9a4522e7bc0dd549042b943fa5690dd`,
  archived and extracted into new directories. No package is published.
- All six packaged contract probes passed: Tasks, composition, async, default queue,
  workers and MapResult. They used the extracted bridge, library and runtime.
- The new Workbench verifier initially failed: MSBuild command-line roots did not
  survive Raven's separate reload of the saved project. The verifier now persists
  both roots in the temporary `.rvnproj`. All eight samples pass using this corrected
  source verifier and the extracted packages. The subsequent clean package at
  `97b8d91` also passes all eight cases using its own packaged verifier after extraction.
- Strict Clippy passes; Task (7), property (6), worker (14) and runtime-service (7)
  regression tests pass locally. Formatting passes.

The initial evaluator archive SHA-256 is `ae0b86efcba52412f7137e7ddb3e574df4ac6bd39601a2dda8dfe84b2e07d0b2`.
Local raw reports and logs are under `/tmp/neoclr-async-readiness.0ceuuk/`:
`packaged-probes.json`, `workbench-fixed.json`, `clippy-final.log`,
`worker-regressions.log` and `lint-regressions.log`. These temporary machine-local
files are supporting evidence, not published or durable release artifacts.

## Follow-up package validation

The [durable local result record](async-preview-local-validation.json) identifies
neoCLR `97b8d91`, the matching Raven revision, archive/tool hashes and case results.
The new archive was extracted to a fresh directory and configured with the extracted
SDK. Its packaged verifier passes all eight Workbench samples. All 22 standalone
MSBuild checks pass, including failed-build cleanup, paths with spaces, changed
library rebuilding, project-reference ordering and invalid dependency graphs.

The packaged language server passes Task/Promise/TaskQueue completion checks;
`Default` is a property and internal `Current` is hidden. This probe does not test
hover descriptions or interactive VS Code installation/build/run behavior.

Full Raven library regeneration matches every checked-in slice. It used the fresh
SDK and earlier clean bridge; the record includes their hashes, and their relevant
sources are unchanged between these bundles. Dependency notice audits cover all
23 bundle and 26 SDK NuGet dependencies, verifying all 34 preserved notice sets.
This is local evidence; it does not select a final release candidate.

## Source-archive follow-up

The first minimum-Rust archive check at `556fd7a` failed at compile time: worker
notification dispatch used two let-chain conditions unavailable in Rust 1.85.
The implementation now uses equivalent nested conditions, preserving the advertised
minimum. A fresh `cargo +1.85.0 check --locked --all-targets` passes locally.
Formatting, strict stable Clippy on all targets and all 14 worker regression tests
also pass with the fix, including self-reposting default-queue progress and explicit
queue isolation. Logs are `msrv-fix-check.log`, `msrv-fix-clippy.log` and
`msrv-fix-workers.log` in the local validation directory.
The original stable full-suite run was stopped after this finding because its
pre-fix archive is superseded; it is not a passed source-archive gate. Both
`source-minimum/report.json` and `source-stable/report.json` remain in the local
validation directory with their unsuccessful status. The fixed commit requires
fresh archive/platform evidence and a rebuilt binary package before publication.

The [migration draft](async-preview-migration.md) covers published Preview 8 breaks
as well as intermediate development Task renames, separating those two audiences.
The VSIX installed successfully into an isolated profile under the validation
directory. Interactive build/run inspection remains unverified: the available UI
binding selected the existing user editor instance, which was not reconfigured.

## Corrected runtime and sample package follow-up

The [follow-up package record](async-preview-package-followup.json) preserves the
new evidence separately from the earlier `97b8d91` results. The runtime was rebuilt
at `a9c1131` with the Rust compatibility fix. All eight Workbench samples pass using
freshly extracted runtime and SDK archives. Four packaged direct neoIL samples pass.
A fresh source archive also builds all 13 authored website pages and the eight-page
DocFX output with no warnings or errors.

The broad saved-project verifier exposed two stale fixtures still using the removed
Error wrapper and former IO namespace. Commit `6512c31` corrects them. All 84
saved-project outcomes pass with the corrected source fixtures and extracted tools;
a fresh package containing those fixes also passes its focused CasePayloads and
ErrorValues checks plus the verifier's mandatory edit/rejection cases. The runtime
binary is identical between these two packages; only samples and release instructions
changed after the runtime build.

System tar produced macOS AppleDouble sidecars in the first archives. Python tarfile
archives omit those entries; every regular payload file was hash-compared with staging
before fresh extraction. The runtime bundle has 935 regular files and the SDK 439.
The record contains archive checksums and outcomes. This is a packaging workaround,
not a claim that Raven's upstream packaging script was fixed.

Fresh stable and Rust 1.85 source validation runs at `a9c1131` are still in progress
when this package evidence is recorded. Neither is claimed as a passed full source
gate. Their eventual reports and exact revision must be recorded separately; these
local macOS runs cannot satisfy the Linux/Windows matrix or certify later revisions.

## Remaining release work

Finish exact-candidate source/archive validation, interactive editor checks
and the complete migration/distribution review. Rerun affected evidence after candidate
changes. Keep publication, website deployment and version/date selection separate.
