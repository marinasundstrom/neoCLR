# Preview 6 validation record

Candidate: `v0.1.0-preview.6`. The exact source revision, package hashes and completed
checks are recorded in the release's `release-manifest.json` and validation archive.
This document describes the required evidence; it is not a substitute for those results.

- Exact-source CI must pass all Linux/macOS/Windows stable/minimum-Rust jobs.
- Extracted macOS arm64 packages must pass the runtime IL, saved-project, application,
  collection/query, library-import, normal compiler/import and editor checks.
- SDK/VSIX and runtime package notices and hashes must match the distributed payloads.
- No changed source tree or stale installed SDK may substitute for the pinned candidate.

The experimental Raven compiler was synchronized with main and passed 42 focused
metadata/attribute/namespace checks before packaging. The main stability audit and
attribute correction are documented in Raven's `docs/compiler/main-stability-audit.md`.
That audit is not the neoCLR release gate. MacCatalyst's Xcode prerequisite remains
an unrelated Raven host-sample limitation; no NanoFramework hardware execution is claimed.

See [release notes](preview-6-release-notes.md) for the distribution and supported scope.
