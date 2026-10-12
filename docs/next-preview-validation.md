# Release candidate validation

The [toolchain/release tracker](tracking/toolchain-release.md) owns candidate status.
The [installed macOS default-JIT preview](local-preview-20261012.md) records the
current local SDK/editor and HTTP acceptance. It does not replace source validation
or qualification of the Windows package. Follow the [CI design](ci-efficiency-plan.md)
for validation placement and the [package procedure](experiments/raven-target/RELEASING.md)
for the matching runtime, Raven SDK and editor assets.

## Source archive

With Python 3.9+, Git, Rust and the host's native build prerequisites:

```sh
python3 scripts/validate-release.py --revision COMMIT --distribution native --toolchain stable --release --output /fresh/output
```

The validator archives committed source, rejects unexpected paths/member kinds,
compares archive membership with Git, audits dependency notices, runs the full test
suite, checks the native bootstrap seed, builds the runtime and checks source/artifact
sample outcomes. The legacy profile audits the older CLI bridge snapshot and is not
the current native distribution gate. Native interop
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
with the matching extension. Qualify default JIT explicitly on the extracted bundle;
source host checks alone do not certify its backend, host library or compiler. Run
the HTTP client/server twice through the installed helper to catch stale generated
output, and exercise the client URL/server request-count task inputs.
Record compiler/runtime revisions, prerequisites and asset hashes in the release
validation manifest. Preserve known limitations and distinguish observed failures
from fixed defects.

The validator does not publish or deploy. Runtime release and manual website
publication are separate operations. Earlier release notes and validation records
remain frozen evidence for their own revisions.
