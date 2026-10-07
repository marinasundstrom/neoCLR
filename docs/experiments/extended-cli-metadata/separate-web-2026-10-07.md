# Separate Web execution — 2026-10-07

**Original slice: execution passes with the recorded artifacts; clean rebuild reliability
was open.** Follow-up: [canonical bootstrap Void binding](unit-bootstrap-2026-10-07.md)
fixes the reproduced lookup failure, with six clean builds and five passing consumers.
The original evidence and limitations below are retained as the investigation record.
System.Runtime (175 sources), Data (6), Networking (9 including native service adapters)
and Web (9) are independently compiled native assemblies. Consumers import their emitted
artifacts without library sources. Primitive Core and the retained runtime seed are still
explicit bootstrap dependencies. This is not a completed bootstrap or release gate.

Networking now owns NetworkDeadline and five public typed Until overloads. Default is
expired; copies share the same process-local monotonic deadline. Pre-cancelled tokens
retain cancellation precedence; otherwise expired deadlines return TimedOut before
admission. Raw stamps remain internal. HTTP client/server retain one 15000ms exchange
budget. See [API limits](../../../api-docs/sockets.md#shared-deadlines-development-native-assemblies)
and [design comparison](web-boundary-2026-10-07.md#the-shared-deadline-boundary).

## Evidence

[Commands, stdout, diagnostics and hashes](separate-web-2026-10-07.json) record Raven
65f554a49, runtime/dependency artifacts and source hashes. The new driver compiles,
verifies and runs deadline, headers, base-address, JSON-client and routing consumers.
All five match exact stdout, exit 0 and empty stderr. The deadline consumer awaits a
real localhost DNS lookup and checks shared-copy/default expiry and cancellation.
The four existing HTTP consumers are unchanged.

Both loopback cancellation cases (before response headers and during the response body)
observe closure of the cancelled sockets and successful independent UTF-8 text work.
Managed live objects finish at zero in both runs. The existing .NET 10 comparison passes.
C# bootstrap/application visibility checks and the Rust bounded-stamp test pass.
API reference fingerprints and type inventory pass. No full website build or publication
was performed; relevant development content and the generated reference were updated.

## Reproduce

Use the explicit Runtime inputs from [separate Data](separate-data-2026-10-07.md).
The recorded paths below refer to the local qualification artifacts; use fresh output
paths and the matching compiler/metadata/runtime revisions when reproducing.

```sh
python3 scripts/audit-optional-libraries.py --compiler /tmp/array-final-compiler1007/rvnc.dll --compiler-revision 65f554a49 --core /tmp/failure1006b/Core.dll --runtime-library-directory /tmp/array-runtime1007/runtime-owned --output /tmp/web-libraries-new --include-web
python3 scripts/verify-separate-web.py --compiler /tmp/array-final-compiler1007/rvnc.dll --compiler-revision 65f554a49 --core /tmp/failure1006b/Core.dll --runtime-library-directory /tmp/array-runtime1007/runtime-owned --data /tmp/web-libraries-new/System.Data.dll --networking /tmp/web-libraries-new/System.Networking.dll --web /tmp/web-libraries-new/System.Web.dll --runtime target/debug/neoclr --output /tmp/web-consumers-new
python3 docs/experiments/http-cancellation/verify.py --compiler /tmp/array-final-compiler1007/rvnc.dll --core /tmp/failure1006b/Core.dll --seed /tmp/array-runtime1007/runtime-owned/System.runtime.neox --ownership /tmp/array-runtime1007/runtime-owned/ownership.json --native-library /tmp/array-runtime1007/runtime-owned/System.Runtime.dll --native-library /tmp/web-libraries-new/System.Data.dll --native-library /tmp/web-libraries-new/System.Networking.dll --native-library /tmp/web-libraries-new/System.Web.dll --object-library System.Runtime --object-root /tmp/array-runtime1007/runtime-owned/System.Runtime.dll --runner target/debug/neoclr
```

## Remaining blockers and limits

A clean audit built Data/Networking but Web failed with NEOMETA003:
`native type missing or ambiguous: System.Void`. No Web output was published. Re-running
the identical recorded Web command succeeded. This is intermittent compiler/metadata
behavior, not solved by the deadline API and not grounds to accept a release build.
Preserve the failing invocation; investigate unit-owner selection/import ordering next.
Do not add retries to disguise it.

The new async fixture initially rejected Task<()> with `Cannot convert from '()' to
'Task<()>'`; it now uses explicit Task<int> status. This exposes a separate unresolved
compiler case, not a claim that all async forms pass. The driver must explicitly select
`--async-library System.Runtime` to bind native Task/builders. Existing consumer sources
were not rewritten to bypass either issue.

After reliable rebuilds, implement real native project dependencies and Platform service
ownership, propagate catalogs to the LSP and qualify collected shipping artifacts.
Current Raven native workspace tests explicitly reject ProjectReference; the legacy
bridge project targets are not evidence of native project-graph support. API bundle and
precise source-link requirements remain in the release backlog. Per author direction,
merge codex/native-system-bootstrap into neoCLR main only when bootstrap is ready, then
continue development there.
