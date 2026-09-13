# Preview 4 publication evidence

Published 2026-09-13 as the [Preview 4 GitHub prerelease](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.4).

- neoCLR release commit/tag target: `c135659b591536d88296a3253143cb06ca1abe36`.
- Raven source: `9b269f9d0d71c4302c9008ff6bf6d0c6b1d21c9c`, published on
  [codex/neoclr-target-resolution](https://github.com/marinasundstrom/raven/tree/9b269f9d0d71c4302c9008ff6bf6d0c6b1d21c9c).
- Raven SDK/VSIX: `0.1.12-neoclr.6`; tested .NET SDK `11.0.100-rc.1.26425.128`.
- Prebuilt runtime/Raven platform: macOS arm64. Python 3.9+ supports the project tasks.

All six jobs in [the exact-source CI run](https://github.com/marinasundstrom/neoCLR/actions/runs/34746830953)
passed: Linux, macOS and Windows on stable Rust and Rust 1.85. Each archived-source
report records `full_tests: true`, successful source/artifact and native smoke checks,
and the same source archive SHA-256 as the uploaded source payload. The final fix
updated smoke validation for guest-only stdout and explicit result diagnostics on stderr.

Ten package suites passed outside both development checkouts: saved Raven projects,
files, process input/environment, clock, error values, unions, delegates, native
buffers, editor completion/hover and direct neoIL samples. The runtime archive's
payload hashes were verified after compression. A configured local release demo
also produced the expected seven-line propagation output.

The release includes eight assets: runtime archive, source archive, matching SDK,
VSIX, companion toolchain notices, validation evidence, manifest and SHA256SUMS.
All eight GitHub asset digests were compared to their local files before publication.
The [release manifest](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.4/release-manifest.json)
contains the reports, versions, commands, hashes and scope; the evidence archive retains
CI reports and package logs. Source checks do not claim prebuilt Raven support on
Linux or Windows, nor a full Raven product-release gate.

Published Preview 1–3 notes remain unchanged. Preview 4's notes/changelog section are
now frozen; later corrections belong under Unreleased with a reference to this release.
