# Release candidate validation

The [toolchain/release tracker](tracking/toolchain-release.md) owns candidate status.
Preview 11 extends the completed HTTP POC. Its [qualification record](preview-11-validation.json)
records full-suite, host and package evidence, including the validator-only correction.
For later candidates, start with the [pre-release assessment](tracking/toolchain-release.md#pre-release-assessment--2026-09-27). Follow the [CI design](ci-efficiency-plan.md)
for validation placement and the [package procedure](experiments/raven-target/RELEASING.md)
for the matching runtime, Raven SDK and editor assets.

## Source archive

With Python 3.9+, Git, Rust and the host's native build prerequisites:

```sh
python3 scripts/validate-release.py --revision COMMIT --toolchain stable --release --output /fresh/output
```

The validator archives committed source, rejects unexpected paths/member kinds,
compares archive membership with Git, audits dependency notices, runs the full test
suite, builds the runtime and checks source/artifact sample outcomes. Native interop
and embedding checks are included. Its report records revision, archive hash,
platform/toolchain, profile, timings and results. Uncommitted changes are excluded.

`--smoke-only` skips full tests and records `full_tests: false`; use it for a local
packaging check alongside the canonical full-suite result. Omit `--release` for an
explicit debug-profile run. Never present a smoke-only report as full-suite evidence.

## Hosted checks

Default CI runs the full archive/test gate once on Linux stable. macOS and Windows
run the focused host/ABI inventory in `scripts/validate-host.py`. All three hosts
compile all targets with Rust 1.85.0. A manual `full_matrix` dispatch runs the full
stable suite on all three hosts when warranted. Format and strict Clippy remain
canonical gates. Reports are retained for successful and failed jobs.

Require passing checks on the selected code revision. A later documentation-only
tag may identify that validated code revision explicitly and record its exact delta;
a green run does not certify untested code. Review actual release notes, changelog,
notices, package contents and checksums before publication.

## Binary and editor checks

Source CI does not certify packaged binaries. For every shipped target, archive and
extract the matching runtime/SDK, compare payload hashes, run the bundled samples,
check failed-build/stale-output rejection, and verify editor completion/build/run
with the matching extension. Preview 11 ships Raven tools for macOS arm64 and
a Windows x64 native-runtime ZIP: CI builds it, compares extracted payload
hashes and executes four direct-runtime samples using `scripts/package-native.py`.
This package carries the library and dependency notices, but no Raven SDK/bridge.
Do not claim Windows Raven SDK/editor qualification from native host checks.
Record compiler/runtime revisions, prerequisites and asset hashes in the release
validation manifest. Preserve known limitations and distinguish observed failures
from fixed defects.

The validator does not publish or deploy. Runtime release and manual website
publication are separate operations. Earlier release notes and validation records
remain frozen evidence for their own revisions.
