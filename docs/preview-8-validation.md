# Preview 8 validation

Candidate version: 0.1.0-preview.8, with experimental Raven 0.1.12-neoclr.15.
Published 2026-09-19 after exact-candidate validation. Results for other commits do not certify this one.

Required evidence (completed):

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

## Extracted package results — 2026-09-19

Candidate `07ecdbee0a5ad4eceaee7df30824c4dfa3719a45`, Raven
`ddf10eca80d599ede28f9da59f145497a21e854e`; runtime .8 and Raven .15 assets.
The six-job source matrix passed: each stable/minimum-Rust Linux, macOS and Windows
job passed 1,280 tests, 25 source/artifact smoke programs and native interop.
All six validated source archives match the released archive’s 1,705 files.
See [exact-candidate CI](https://github.com/marinasundstrom/neoCLR/actions/runs/35452668923).

The extracted macOS arm64 runtime/SDK passed all 19 verifier suites, including
83 saved-project outcomes, 22 MSBuild checks, 137 signatures, 30 query checks,
13 library-import checks, 5 compiler-target checks, 15 application checks,
4 array samples and 17 match cases. The primary and application/library projects
built and ran through the packaged MSBuild assets. The grapheme sample built from
a saved project and matched its expected output exactly.

The editor matrix passed 86 primary-project sections, 87 project-reference sections
before the first library build, and 86 sections against the installed VSIX server.
An isolated VS Code profile opened the grapheme project and started its matching
language client. This does not claim Raven source debugging on neoCLR.

All 846 runtime manifest file hashes matched. The native runtime links only to
macOS system libraries (libiconv, CoreFoundation and libSystem), not .NET or a
developer-local library. Tool notice audits covered 26 NuGet
dependencies, 34 package notice sets and 8 production npm dependencies; the rebuilt
production extension JavaScript matched the installed VSIX. The SDK archive excludes
AppleDouble sidecars without changing the staged compiler payload. The deferred
upstream packaging candidate is recorded in [integration findings](raven-target-evaluation.md).

Local host: macOS 27.0 (26A428), arm64; .NET SDK 11.0.100-rc.1.26425.128.
The release manifest and validation archive are assembled from these final assets
and exact-candidate CI reports; earlier candidate failures are not passing evidence.

## Published evidence

[Preview 8](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.8)
was published with eight assets. GitHub's SHA-256 digests matched every local upload,
including the manifest, validation archive and checksum file. See the attached
[manifest](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.8/release-manifest.json)
and [checksums](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.8/SHA256SUMS).
The runtime also produced the expected primary-program output with dotnet absent
from PATH; that check does not claim the host's .NET installation was removed.
