# neoCLR Preview 14 — JIT, HTTP and secure streams

Preview 14 provides matched **macOS ARM64** and **Windows x64** bundles with the
Raven compiler, language server, VS Code extension, runtime, libraries and samples.
**JIT is the default execution mode** in both bundles. The interpreter remains
available explicitly; AOT is a separate experimental publishing workflow.

This is an experimental preview. Rebuild applications with the matching tools and
libraries; APIs and native metadata can change between previews.

## Install and run

Install .NET 11 SDK **11.0.100-rc.1.26425.128** (RC1), a .NET 10 SDK, Python 3 and
VS Code. The Raven compiler and language server require .NET; guest execution on
neoCLR does not.

- [macOS ARM64 bundle](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.14/neoclr-preview14-osx-arm64.tar.gz)
- [Windows x64 bundle](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.14/neoclr-preview14-win-x64.tar.gz)

Extract the archive and keep its folders together. In VS Code, select
**Extensions → ⋯ → Install from VSIX…** and install `editor/raven-vscode.vsix`.
Open `samples/collections` and choose **Terminal → Run Task → neoCLR: Run**.
The `tasks` and `json` folders contain other standalone examples.

From the extracted `neoclr-native-poc` directory, verify the complete installation:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

On Windows use `python`. The verifier checks package hashes, compiles the samples
and runs collections, Tasks, JSON and paired HTTP consumers using default JIT.
Keep the compiler, extension, runtime, native JIT helper and libraries together.
Standalone SDK/VSIX assets are also attached; their companion tool-notice archives
supply attribution. The complete bundles already contain these notices.

## Try the HTTP showcase

Open `samples/http-json-server`, run **neoCLR: Run** with request count `2`, then
open `samples/http-json-client` and supply the server's printed URL. The client
retrieves a typed report, posts it back and prints `{"accepted":true}`.

[HTTP walkthrough](https://marinasundstrom.github.io/neoCLR/cases/http-server/) ·
[API reference](https://marinasundstrom.github.io/neoCLR/docs/)

## Changes worth trying

- **JIT and native execution:** native type/object admission and callback dispatch
  now handle the HTTP async consumers. Windows AOT heap admission follows the
  configured HTTP build budget, with guarded-allocation lifecycle checks.
- **HTTP, TLS and JSON:** async streams compose through managed `SslStream` and
  certificate/cryptography foundations. Shared HTTPS/JSON and failure cases pass
  on macOS ARM64 and Windows x64 in interpreter, JIT and AOT modes. Tests include
  certificate rejection/recovery, deadlines and cancellation; Windows also passes
  twelve independent TLS 1.2/1.3 interoperability cases.
- **SQL mapping:** an optional managed PostgreSQL provider supports plain positional
  values and named parameter objects. The mapper accepts a connection and SQL
  statement and maps buffered rows to objects. The provider is included in the
  macOS bundle; the Windows bundle contains the four base libraries without it.
  See the [PostgreSQL sample](https://marinasundstrom.github.io/neoCLR/cases/postgresql/).
- **Library and tooling:** collections and Index/Range access, JSON nodes and object
  mapping, encoding/async streams, reflection-driven tests and benchmark cases
  have expanded. Packaged HTTP tasks accept arguments and prepare reflection roots.
  The website includes tested examples and the matching native API snapshot.

## Migration and limits

Task APIs live in `System.Concurrency.Tasks`. Stream operations return tasks;
use `await`, including in async `Main`. The bundled run task now uses JIT;
select interpreter execution explicitly when needed.

- Windows JIT uses compiler-generated async state machines. Runtime-owned
  suspension is a separate target-specific development path, not enabled by this
  Windows package.
- AOT supports bounded applications, not arbitrary .NET code or unrestricted
  reflection. Build instructions and host tools remain necessary; the bundles
  are primarily qualified for development JIT execution.
- HTTP/1.1 remains bounded, with explicit TLS credentials/trust. Connection pooling,
  redirects and HTTP/2/3 are not implemented. Source debugging is not available.
- PostgreSQL currently supports local IPv4 loopback and trust authentication only.
  Password authentication, SQL TLS, remote servers, pooling and streaming are not
  implemented. Windows SQL and standalone SQL deployment remain unqualified.
- Windows ARM64 and Linux JIT/AOT are outside this preview's qualification. The
  primitive core/runtime seed remains an explicit source-bootstrap dependency.
- Benchmark results describe named development configurations. This release makes
  no new cross-runtime performance claim, stable ABI or production-support promise.

## Qualification and provenance

Both extracted bundles pass their default-JIT samples. The matching macOS and
Windows editor extensions each pass 26 installed VS Code checks. Canonical
extracted-source tests, smoke programs, notices, format/Clippy, focused host checks
and minimum-Rust compilation pass. Focused backend fixes are qualified separately.

The full source gate ran on `a9adc97cb`; the callback-fixed Windows package ran on
`566fb34dc`. Windows AOT heap repair `a77e20dba` passes the complete 27-stage Windows
HTTP/HTTPS/framework gate. The macOS bundle retains its recorded component inputs
and archive hash. These are exact per-revision results, not a claim that all gates
ran on the release tag. The attached qualified-source archive is the canonical
source-gate input; the tag also includes later qualified fixes and documentation.

[Validation record](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.14/preview-14-validation.json) ·
[SHA-256 checksums](https://github.com/marinasundstrom/neoCLR/releases/download/v0.1.0-preview.14/SHA256SUMS) ·
[Detailed qualification](https://github.com/marinasundstrom/neoCLR/blob/v0.1.0-preview.14/docs/experiments/release-20261013/README.md)
