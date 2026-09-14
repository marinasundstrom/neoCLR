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
