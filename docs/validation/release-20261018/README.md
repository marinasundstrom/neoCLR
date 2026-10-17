# Preview 16 qualification — in progress

Package version `0.1.16-neoclr.20261017.1`, Raven `c0854cdda`. Publication remains
pending final Windows measurements and matching-package HTTPS qualification.

The macOS archive passes [extracted sample acceptance](macos-acceptance.json) and
[HTTP client/server acceptance](macos-acceptance-http.json). The installed extension
passes the [main suite](macos-editor-main.json), [first](macos-configuration-1.json)
and [second](macos-configuration-2.json) ten-cycle diagnostic-recovery runs and
[exact inlays](macos-inlays.json). [Range checks](macos-ranges.json) pass across
interpreter, JIT and AOT. [Identity](macos-identity.json) records archive/runtime/
backend hashes, matching SDK/extension server files and four installed-server
selections without overrides.

The [runtime/library comparison](macos-runtime-library-comparison.json) confirms
unchanged runtime and backend binaries but changed library hashes. Earlier candidate measurements remain separate. The
[final macOS measurements](../../../benchmarks/release-20261018/macos/README.md)
pass 55 CPU/JSON processes and 1,152 HTTP responses. The JSON harness resolves
reflection roots from the final assembly; its earlier stale-root failure is retained.
The [full runtime-await matrix](macos-runtime-await.json) passes all 24 cases:
language, cleanup, fault, HTTP server/client, empty/whitespace state-machine
configuration and generator, each in interpreter/JIT/AOT.

[Canonical source CI](../release-candidate-20261017/canonical-ci-passed.json)
passes all source, host and minimum-Rust jobs. The
[source report](../release-candidate-20261017/canonical-source-passed.json)
records full tests and 25 smoke programs at `25828f59e`.

Windows qualification: https://github.com/marinasundstrom/neoCLR/actions/runs/39073227105.
Its benchmark job consumes that exact archive only after successful qualification:
https://github.com/marinasundstrom/neoCLR/actions/runs/39075237776.
The earlier Windows run failed explicit helper clearing; the
[isolated repair](../release-candidate-20261017/async-helper-clearing.json) passes
all nine empty/whitespace/generator interpreter/JIT/AOT cases. No earlier failed
archive is relabeled as a passing package.

## Final Windows package and publication archives

[Windows identity](windows-identity.json) verifies 36 SDK/VSIX server files and
four installed-extension server selections. [Sample acceptance](windows-acceptance.json),
[HTTP acceptance](windows-acceptance-http.json), [editor](windows-editor-main.json),
[two](windows-configuration-1.json) [ten-cycle recovery runs](windows-configuration-2.json)
and [inlays](windows-inlays.json) pass. The [24-case await matrix](windows-runtime-await.json),
[JSON](windows-json.json), [GC](windows-gc.json) and [HTTP headers](windows-headers.json)
pass the supported interpreter/JIT/AOT checks against this package.

The publication archives correct only their README's obsolete CLI-bootstrap notice.
[macOS](macos-documentation-repack.json) and [Windows](windows-documentation-repack.json)
record both archive hashes and confirm every executable/library payload is unchanged.
All manifest files verify after independent extraction (1,602 macOS, 1,597 Windows).
[macOS publication acceptance](macos-publication-acceptance.json) also passes on the
repacked archive. Windows execution evidence is reused because its executable bytes
are identical; this is not a claim of another Windows execution after repacking.

Windows JSON copies normalize CRLF to LF; their data and embedded input hashes are unchanged.
