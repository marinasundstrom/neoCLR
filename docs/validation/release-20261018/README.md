# Preview 16 qualification — in progress

Package version `0.1.16-neoclr.20261017.1`, Raven `c0854cdda`. Publication remains
pending Windows package/editor acceptance and final-package measurements.

The macOS archive passes [extracted sample acceptance](macos-acceptance.json) and
[HTTP client/server acceptance](macos-acceptance-http.json). The installed extension
passes the [main suite](macos-editor-main.json), [first](macos-configuration-1.json)
and [second](macos-configuration-2.json) ten-cycle diagnostic-recovery runs and
[exact inlays](macos-inlays.json). [Range checks](macos-ranges.json) pass across
interpreter, JIT and AOT. [Identity](macos-identity.json) records archive/runtime/
backend hashes, matching SDK/extension server files and four installed-server
selections without overrides.

The [runtime/library comparison](macos-runtime-library-comparison.json) confirms
unchanged runtime and backend binaries but changed library hashes. Earlier candidate
measurements therefore remain separate; the final package will be measured again.
Full runtime-await validation is also in progress.

[Canonical source CI](../release-candidate-20261017/canonical-ci-passed.json)
passes all source, host and minimum-Rust jobs. The
[source report](../release-candidate-20261017/canonical-source-passed.json)
records full tests and 25 smoke programs at `25828f59e`.

Windows qualification: https://github.com/marinasundstrom/neoCLR/actions/runs/39073227105.
Its benchmark job consumes that exact archive only after successful qualification:
https://github.com/marinasundstrom/neoCLR/actions/runs/39073249377.
The earlier Windows run failed explicit helper clearing; the
[isolated repair](../release-candidate-20261017/async-helper-clearing.json) passes
all nine empty/whitespace/generator interpreter/JIT/AOT cases. No earlier failed
archive is relabeled as a passing package.
