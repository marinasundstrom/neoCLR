# Async preview readiness — 2026-09-23

This is local preparation for the [author-selected async/Tasks release](async-preview-plan.md),
not a release certification. The release version/date and final candidate remain unset.
The first evaluator build uses clean isolated neoCLR and Raven worktrees; Raven stays
on `codex/async-preview-readiness`, based on the existing neoCLR feature revision.
No developer SDK selection or VS Code installation is changed.

## Gate inventory

| Gate | Concrete check / deliverable | Evidence boundary |
| --- | --- | --- |
| Library sources and generated artifacts | `build_runtime_library.py --check-snapshot`; full `--check --compiler … --bridge …` regeneration | Hash check passes locally; full regeneration must be recorded separately |
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
  source verifier and the extracted packages. A newly packaged verifier still needs
  its final extracted-package check; this distinction is intentional.
- Strict Clippy passes; Task (7), property (6), worker (14) and runtime-service (7)
  regression tests pass locally. Formatting passes.

The initial evaluator archive SHA-256 is `ae0b86efcba52412f7137e7ddb3e574df4ac6bd39601a2dda8dfe84b2e07d0b2`.
Local raw reports and logs are under `/tmp/neoclr-async-readiness.0ceuuk/`:
`packaged-probes.json`, `workbench-fixed.json`, `clippy-final.log`,
`worker-regressions.log` and `lint-regressions.log`. These temporary machine-local
files are supporting evidence, not published or durable release artifacts.

## Remaining release work

Finish full
library regeneration, exact-candidate source/archive validation, package/editor checks
and the complete migration/notice review. Rerun affected evidence after candidate
changes. Keep publication, website deployment and version/date selection separate.
