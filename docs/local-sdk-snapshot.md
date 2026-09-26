# Local SDK snapshot — 26 September 2026

The installed SDK is `~/.raven/sdk/0.1.12-neoclr.20260926`.
The matching runtime, bridge, library and samples are in
`~/.neoclr/experiments/development-20260926`. The Raven VS Code extension
`0.1.12-neoclr.20260926` is installed in both the default VS Code installation and
our isolated neoCLR profile. This is a local development build, not a release.

## Open the HTTP and JSON web application

```sh
~/.neoclr/experiments/development-20260926/open-vscode.sh http-json
```

Omitting `http-json` opens the same workspace. The launcher uses the isolated
profile in `~/.neoclr/vscode/raven-port-20260919`, including its extension directory.
The workspace pins the installed SDK and language server. Reload the VS Code window
if an older extension session is still active.

The workspace contains **Server** and **Client** projects. Both use the public
`JsonSerializer` Object/TypeInfo overloads, HTTP client/server APIs and Raven
async/await. Edit `Application.rvn` for payload models and serialization,
`Server.rvn` for routes, and `Client.rvn` for the request flow. Each project has its
own copy of `Application.rvn`; update both when changing the wire contract.

1. Run **Tasks: Run Task → neoCLR: Run** and select **Server**. It builds and
   verifies the saved sources, starts the server on loopback, and prints its port.
2. Run **neoCLR: Run** for **Client** in a second terminal. At the URL prompt, enter
   `http://127.0.0.1:PORT/`, using the server's printed port.
3. The client serializes a report, posts it to `/reports`, and deserializes the
   response. Expected output: `{"accepted":true}`.
4. To exercise both programs automatically, run the workspace task
   **neoCLR: Verify HTTP JSON flow**. It builds both projects, starts its own
   server, checks GET, a valid JSON POST, invalid JSON, and unknown routes,
   then stops its server. This default check uses an independent Python HTTP client.

You can also open `http://127.0.0.1:PORT/report` in a browser. It returns
`{"station":"Café"}`. A manual request from a terminal is:

```sh
curl -i http://127.0.0.1:PORT/reports \
  -H 'Content-Type: application/json' -d '{"station":"Café"}'
```

Expect HTTP 201 and `{"accepted":true}`. Invalid JSON returns 400; unknown routes
return 404. The interactive server handles up to 100 requests; stop it with Ctrl+C
or restart its task. It is an experimental loopback sample, not a production host.
Use these explicit tasks, not Raven's ordinary .NET toolbar Run/Debug commands.

The HTTP tasks use the bundled `tools/http-runner` (built from `measure_async`),
with 1,024 heap objects and a 100,000,000-instruction budget. The standard CLI's
100,000-instruction default is not the HTTP test configuration. The runner uses the
same runtime implementation, verifies the program, and reports timing/GC statistics
on stderr. It does not change the existing HTTP transport deadlines.

## Select another sample

```sh
~/.neoclr/experiments/development-20260926/open-vscode.sh library-calendar
~/.neoclr/experiments/development-20260926/open-vscode.sh library-async
```

Each opens its own editable project with **neoCLR: Build** and **neoCLR: Run** tasks.
The launcher never overwrites sample edits. These are the three prepared choices;
other source samples are retained in the bundle's `tools/samples` directory.

## Build and validate without VS Code

```sh
python3 ~/.neoclr/experiments/development-20260926/tools/verify-local-http-json.py \
  --bundle ~/.neoclr/experiments/development-20260926
```

To prepare the same editable projects in a fresh matching bundle, from this repo:

```sh
cargo build --locked --release --example measure_async
python3 scripts/configure-local-samples.py \
  --bundle /absolute/path/to/bundle \
  --sdk /absolute/path/to/raven-sdk \
  --http-runner target/release/examples/measure_async \
  --source-ref cbf7bb70
```

The configuration script reads committed sources and refuses to overwrite an
existing HTTP sample. It does not install the extension: build/install the matching
VSIX first using [the packaging procedure](experiments/raven-target/RELEASING.md).
On this host the runtime build used `SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk`
to avoid a linker/SDK incompatibility with the selected macOS 27 SDK.

## Provenance and validation limits

This bundle uses neoCLR `cbf7bb70` and Raven `54ec1c718` on its `neoclr` branch.
The runtime implementation is the committed 25 September code; the website repair
is included in the recorded neoCLR revision. JSON generic overload edits that were
staged when this build began are excluded, even if subsequently committed. Use the
Object/TypeInfo overloads shown in these samples. The source record and artifact
hashes live in the installed bundle's `manifest.json` and `local-validation.json`.

For the optional Raven-to-Raven check, add `--managed-pair` to the verifier command.
It is currently unreliable on this host: later isolated retries also timed out,
although the initial reference-verifier run passed. The default check exercises
the server and public JSON mapping without depending on that unresolved path.

The eight async examples and collection/union/string editor protocol checks pass.
The initial managed HTTP pair passed; later isolated repeats failed. A combined run after three preceding requests
encountered the existing transport timeout; use the isolated verifier and avoid
heavy concurrent builds when testing this experimental flow. No predictable
latency or load-handling claim is made. Full MSBuild acceptance is **not** claimed: its
negative test `func Main() { missing() }` unexpectedly compiled into an empty body
with this compiler. This unresolved compiler diagnostic defect is separate from the
validated HTTP flow; successful compilation alone is not proof of correct source.
The original failed-test logs are retained under `~/.neoclr/builds/20260926`.

---

# Local SDK snapshot — 24 September 2026

The local development workspace is
`~/.neoclr/experiments/nullable-records-20260924/Platform.code-workspace`.
It pins its own Raven compiler and language server and includes a matching neoCLR
runtime, library, bridge and reference assembly. It is not a published release.
The bundle's `snapshot.json` records source revisions and artifact hashes.

Run `~/.neoclr/experiments/nullable-records-20260924/open-vscode.sh` to open it
with the existing isolated experimental Raven extension profile. Select a project
and run its **neoCLR: Run** task (the default build task). Console accepts input in
the task terminal. Each Storage run creates a separate retained `storage/runs/`
folder, so its exclusive-create example can be repeated without deleting files.

Projects: integer/string/nested record equality, hashing and deconstruction, Storage file read/write
and enumeration, and Console input/output/error streams. Records are currently
limited to non-generic classes with integer, non-null string or same-compilation
record-class components and a direct Object base. Record references may be nullable;
nullable string/value components and record structs remain unsupported.
The Object equality project also checks boxed Int32 value equality and hashes while
separate boxes retain distinct identities. Other boxed values remain unsupported.
HashCode supports Add(int), Add(string), ToHashCode and Combine(int, int).

Validation: all four samples build and run; language-server protocol checks
return HashCode.Combine, System.Concurrency and nested-record member completions,
and hover retains Person? for nullable properties. The compiler record suite
passes 46 tests; boxed/class Object equality passes 12 runtime cases and the
.NET comparison passes 22 assertions; the runtime reference-slot suite passes 32. The prior hash suite
passed three tests; the HashCode implementation is unchanged in this slice. This does not claim a manual
VS Code UI test or broad record parity with .NET. The existing SDK base version is
retained; snapshot provenance distinguishes these rebuilt local tools.
