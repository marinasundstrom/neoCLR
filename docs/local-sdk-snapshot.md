# Local SDK snapshot — 27 September 2026

Installed local development tools, not a published release:

- Raven SDK and VS Code extension: `0.1.12-neoclr.20260927.cpu1`.
- SDK: `~/.raven/sdk/0.1.12-neoclr.20260927.cpu1`.
- Runtime, bridge, library and editable samples: `~/.neoclr/experiments/development-20260927-cpu`.
- Runtime sources: neoCLR `46b40f80`; SDK Raven neoclr `2a3fbb346`.
- Bridge/VSIX packaging revision: Raven neoclr `66c33f7a0`, which adds only a
  separately integrated editor completion-order change. Compiler/metadata sources
  are unchanged; the workspace explicitly uses the SDK language server.
- Extension installed in default VS Code and the isolated neoCLR profile. The
  workspace pins the matching SDK/server; other projects keep their SDK selection.

## Open and run the HTTP client/server

```sh
~/.neoclr/experiments/development-20260927-cpu/open-vscode.sh http-json
open ~/.neoclr/experiments/development-20260927-cpu/launch-http-server.command
open ~/.neoclr/experiments/development-20260927-cpu/launch-http-client.command
```

Start the server first and wait for its printed loopback port. The client launcher
reads that port from `http-server.log`. The launchers run the already built apps;
use **neoCLR: Build** after source edits. Stop the server with Ctrl+C in its terminal.
The server handles up to 100 connection attempts, including rejected requests. The
local launch verified on this date used port 63796; a new launch selects a new port.
Malformed/oversized request headers close that connection without terminating the
server; no HTTP error response is produced on that decoding path.

The workspace contains Server and Client projects using Raven async/await, the
HTTP APIs and generic JSON helpers. The client GETs `/report` into a ReportPayload,
POSTs it to `/reports`, and deserializes the acknowledgement. Expected output:
`{"accepted":true}`. Each project has a copy of Application.rvn; update both when
changing the wire contract.

Alternatively, run **neoCLR: Run** for Server, then Client. Supply the printed URL
at the client's prompt. These tasks build saved sources before running them. Raven's
ordinary .NET toolbar Run/Debug commands are not the neoCLR target pipeline.

GET `/report` returns `{"station":"Café"}`. POST `/reports` accepts that shape and
returns 201. Invalid JSON receives 400; unknown routes receive 404. Run the workspace
**neoCLR: Verify HTTP JSON flow** task for independent Python requests, or include
the Raven client with:

```sh
python3 ~/.neoclr/experiments/development-20260927-cpu/tools/verify-local-http-json.py \
  --bundle ~/.neoclr/experiments/development-20260927-cpu --managed-pair
```

The HTTP runner uses 1,024 heap objects and 100,000,000 instructions, preserving
existing transport deadlines. It prints timing/GC counters on stderr. This is an
experimental loopback sample, not a production host.

## Evidence and boundaries

[Exact revisions, hashes and results](experiments/http-json/local-cpu-toolchain-validation.json)
record all 440 installed SDK files matching their build, both extension installations,
installed language-server completion, independent HTTP requests and the Raven pair.
[Runtime validation and performance](experiments/http-json/lookup-comparison-20260927.md)
include 45 focused native checks, thirteen Task.Run consumers and about 24% lower
median GET/POST CPU in local measurements. The sample remains a bounded interpreter
POC. Quota aggregation remains a performance follow-up. SDK/extension builds also
compile Raven.Core and Raven.Macros; API snapshot checks pass. No website build or
full suite was run. The [earlier async installation evidence](experiments/task-run/local-async-toolchain-validation.json)
remains available; its installed bundle is preserved.

Generic methods on nongeneric owners and ordinary instance async methods on bounded
generic classes are supported. Generic methods on generic owners remain an importer
limit. Silent target interpolation output loss and canonical format/Clippy/signature
probe gates remain open in [release tracking](tracking/toolchain-release.md).
This local installation does not close those release gates.

Other prepared samples are `library-calendar` and `library-async`, selected as an
argument to open-vscode.sh. Sources and existing editable installations are preserved.
See [packaging](experiments/raven-target/RELEASING.md) and
[sample configuration](../scripts/configure-local-samples.py) to reproduce a fresh
installation. On this host the release runtime/runner used the Xcode MacOSX26.2 SDK.
