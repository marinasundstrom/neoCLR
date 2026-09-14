# Preview 7 validation requirements

Candidate: `v0.1.0-preview.7`. Publication requires the committed candidate to pass all
six Linux/macOS/Windows stable/minimum-Rust CI jobs. Source archives, notice inventories
and runtime tests are checked by those jobs.

The extracted macOS arm64 runtime bundle and matching Raven .14 SDK/VSIX must pass:

- MSBuild compilation, project references, changed-library rebuild, failures and
  stale-output invalidation; separately execute both primary demos.
- Completion against the standalone project and the referenced-library project before
  building the library; also exercise the installed VSIX server.
- Direct neoIL, saved-project, file/process/clock, union/match, application/order,
  collection/query, native-buffer, separate-library and normal compiler/import suites.
- Generic array API and importer signature probes, artifact hashes and dependency notices.

Use the [release procedure](experiments/raven-target/RELEASING.md). The attached
`release-manifest.json` and validation archive are the evidence; this checklist alone
is not a claim that the candidate passed. Never overwrite published Preview 6 assets.


## Published outcome — 2026-09-14

Preview 7 was published from `5da27a7f5fc7ee6db3c7cad9da336ddc3a93619d` after
[all six CI jobs passed](https://github.com/marinasundstrom/neoCLR/actions/runs/34877491381).
Each source report identifies that commit, reports full tests and 26 smoke programs,
and audits 47 Rust dependencies with 94 notice files. The distributed source archive's
file contents match all six independently validated source archives.

All 18 packaged script suites passed, including 63 saved-project checks, 22 MSBuild
scenarios, 30 query checks, 13 separate-library checks and 121 signature-probe checks.
The primary editor suite passed 68 checks; the project-reference suite passed 69,
also with generated outputs removed before checking source completion. The installed
VSIX server and both build/run demos passed. All 765 runtime manifest entries and
all eight GitHub asset digests were verified. The tool notice audit covered 26 NuGet
dependencies and all 34 package notice sets.

The [release assets](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.7)
include the six CI reports, packaged logs, provenance, a release manifest and SHA256SUMS.
Published artifacts and release notes are unchanged by this outcome record.
