# Preview 5 publication evidence

Published 2026-09-13 as the [Preview 5 GitHub prerelease](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.5).

- neoCLR release commit/tag target: `c76ee5777424d281054463c2647683eb4765cfc0`.
- Raven experimental branch: `codex/neoclr-target-resolution`, source
  `000ed511e4b1e124dce34ba4208ac0cda9ce1245`.
- Raven SDK/VSIX: `0.1.12-neoclr.8`, reused byte-for-byte from the validated local build.
- Prebuilt platform: macOS arm64. Raven prerequisite: .NET SDK
  `11.0.100-rc.1.26425.128`; project tasks require Python 3.9+.

All six jobs in [exact-source CI](https://github.com/marinasundstrom/neoCLR/actions/runs/34759299418)
passed: Linux, macOS and Windows on stable Rust and Rust 1.85. Every source report
records full tests, successful archive validation and the same archive SHA-256 as
the published source payload. The release source was not changed after validation.

Thirteen suites passed against the freshly extracted macOS arm64 bundle outside
both repositories: queries, application types, order workflow, saved projects,
direct neoIL, editor completion/hover, files, process, clock, error values, unions,
delegates and native buffers. These include 14 query cases, 12 application cases,
6 order-workflow checks and 50 saved-project checks. The configured default demo task
produced the documented seven-line propagation output. The packaged bridge/runtime
also passed the 17-fixture match matrix using the source verification driver.
The separately extracted SDK reported its expected version. The retained Raven
regression log records 29 passing focused compiler tests.

All 687 runtime-bundle payload hashes matched after compression. The actual bundle
and SDK dependency inventories were covered by the preserved notices; Raven's npm
dependency manifests were unchanged from the reviewed build. SDK and VSIX hashes
matched the prior local build record. All eight uploaded GitHub asset digests were
compared with local artifacts before publication.

The release includes the runtime bundle, source, matching SDK, VSIX, companion
notices, validation evidence, manifest and SHA256SUMS. The
[release manifest](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.5/release-manifest.json)
records hashes, source IDs and test results. The bundle's internal manifest preserves
its prepublication provenance; the external release manifest supplies final status.
Source CI does not claim prebuilt Raven support on Linux or Windows or a full Raven
product-release gate. See [release notes](preview-5-release-notes.md) for limitations.

Published Preview 5 notes and changelog content are frozen. Later corrections belong
under Unreleased and must identify the affected release.
