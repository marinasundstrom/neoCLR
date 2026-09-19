# Preview 8 validation

Candidate version: 0.1.0-preview.8, with experimental Raven 0.1.12-neoclr.15.
Publication is pending validation; no results for other commits certify this one.

Required evidence:

- Six exact-commit Linux/macOS/Windows jobs using stable and Rust 1.85.0, with
  source archive membership, dependency notices, full runtime tests and smoke runs.
- Extracted macOS arm64 runtime, SDK and VSIX: primary MSBuild project and library
  reference, saved-project suite, full target editor checks and stale-output failures.
- Text, Introspection and existing application/library demonstrations, signature
  admission, native-buffer, file/process/clock, collection/query and direct IL checks.
- Source and binary revisions, platform/toolchain versions, artifact SHA-256 hashes,
  bundled tool notices and production extension dependency notices.

The implementation baseline passed 1,280 runtime tests after two service-count
assertion corrections, 94 focused compiler tests, 77-slice reproducible regeneration,
137 signature checks and local saved-project/editor checks. Release packaging must
repeat relevant checks against the actual extracted assets.

Follow [the release procedure](experiments/raven-target/RELEASING.md).
