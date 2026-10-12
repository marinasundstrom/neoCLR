# Shared TLS development checkpoints

The [provider design](../../shared-security-provider.md) describes ownership, status
codes, platform scope and remaining managed API work. The chronological checkpoints below distinguish private foundations from the
current public development APIs. None is a published release claim.

Run the provider and native adapter checks with:

```sh
python3 scripts/test-native-security.py
```

Run identical ownership service cases in interpreter, JIT and AOT with:

```sh
python3 docs/experiments/shared-tls/validate-bindings.py \
  --runtime target/debug/neoclr \
  --compiler tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/tls-binding-validation
```

The output directory must be new. A local OpenSSL executable generates disposable
PEM fixtures; keep the output private because it contains the test private key.
Build the runtime and native compiler first. macOS ARM64 requires clang and an SDK;
Windows x64 requires an MSVC developer shell and uses the same Rustls provider.
The focused host is compiled by the harness and disposes TLS resources before
destroying its arena, including Fault exits. Default CLI host packaging is separate.

The native binding requires `--bind-tls --compile-system --reference-arena --native-gc`.
The harness checks seven ownership cases, exact admission and missing prerequisites;
encrypted I/O remains covered by the provider tests. This is not yet a managed
SslStream or HTTPS client qualification. Windows CI execution remains pending.

## Default JIT and source AOT hosts

New development bundles prepared by `scripts/prepare-jit-bundle.py` include the
TLS adapter/provider and advertise `jit.tlsOwnership`. Rebuild the runtime,
native compiler and host together. This remains private ownership support, without
public Raven certificate or secure-stream APIs.

```sh
python3 docs/experiments/shared-tls/validate-bundle.py \
  --bundle path/to/bundle --output target/tls-installed-validation
python3 docs/experiments/shared-tls/validate-project.py \
  --bundle path/to/bundle --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/tls-project-validation
```

The bundle harness generates disposable credentials (or accepts `--fixtures`) and
checks nine default-JIT cases, including terminal Fault teardown with owned sessions.
The source project uses test-owned InternalCalls to check status/lifetime admission
in interpreter, default JIT and standalone AOT. A second unused-declaration build
must omit the TLS provider. The AOT builder reports `managedTls` and links the
provider only when selected; released native build kits remain unsupported.

HTTP hosts also dispose TLS after stopping async work and socket/task scopes.
A local async-Main HTTP-profile probe passes. A synchronous HTTP-profile probe with
no queue bindings still fails to link its generated drain export; this existing
host/compiler gap remains open and is not a TLS API restriction.

## Borrowed-buffer transport

The same `scripts/test-native-security.py` harness now exercises the private Rust
and C ciphertext pump with disposable credentials. It covers fragmented handshake,
bidirectional backpressure, partial output, half-close, explicit EOF/truncation,
terminal certificate rejection and invalid buffer ranges. The native managed
adapter test also calls the exported C pump. See
[local evidence](buffered-transport-validation.json) and the
[pump contract](../../shared-security-provider.md#borrowed-buffer-transport-pump-development-2026-10-12).
Managed pumping service bindings and public certificate/SslStream APIs remain open;
Windows execution is pending the existing shared-provider CI gate.

## Managed SSL and HTTP composition

`System.Networking.Security.SslStream` now implements async InputStream/OutputStream
with explicit PEM client trust and server certificate/private-key options. HttpClient
selects it for HTTPS using `HttpSocketHandler(options)`; HttpServer selects it through
`HttpServerOptions.SslOptions`. Both retain cleartext HTTP support. See the
[API and limitations](../../../api-docs/ssl-streams.md).

```sh
python3 docs/experiments/shared-tls/validate-streams.py \
  --bundle path/to/bundle --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/tls-streams-validation
```

[Local evidence](streams-validation.json) records interpreter, default JIT and AOT
nested JSON exchange with GC and direct stream ownership checks, plus a separate
cleartext HTTP interpreter regression. Use an optimized interpreter; its explicit
100-million-instruction, 1,048,576-element and 64-MiB array-storage budgets are test
configuration, not widened runtime defaults. Timings are not performance evidence.
The fixture has a common 60-second server deadline to accommodate interpreted setup.

The tested Raven compiler is `6fe7d630a25d0e27f3e9e89b1403987caf30d9d6` on its
neoCLR branch. Rebuild compiler, libraries and host together: bounded
[function admission](native-selection-budget.md), [layout admission](native-layout-budget.md)
and matching root hooks are required. Public signatures have a refreshed API snapshot.
Windows managed execution remains pending; the earlier native HTTPS host result does
not qualify this new managed composition. HTTP handshake rejection, timeout,
cancellation and recovery checks are covered below. Full-duplex simultaneous stream operations, OS trust
stores, general X509 inspection, mTLS and release packaging remain outside this slice.

### HTTP handshake failure checks

Use `--http-failures` with `validate-streams.py` to build the focused HTTP consumer.
It checks wrong-hostname rejection followed by success on the same client/listener,
a stalled handshake under HttpClient.Timeout, caller cancellation after ClientHello,
and the server read deadline while its TLS peer sends nothing. The ClientHello
observation makes the two client cases handshake checks rather than DNS/connect checks.
The runner observes cancellation through an explicit outcome adapter so cancelled
Tasks cannot accidentally skip the remaining assertions.

[Local qualification](http-failures-validation.json) passes all four scenarios in
interpreter, default JIT and standalone AOT on macOS. Windows uses the same consumer
in CI; execution is pending. These checks required no additional runtime change.

### Independent HTTPS peers

`validate-interop.py` runs the managed HTTP client against a Python/OpenSSL server,
then Python's HTTPS client against the managed HTTP server. Each pair exchanges
nested JSON containing multi-byte UTF-8 in TLS 1.2 and TLS 1.3. The external server
returns chunked HTTP, splitting chunk framing and UTF-8 across TLS records. Server
handlers collect garbage before producing their response. This is interoperability
qualification, not a performance comparison or a production hosting claim.

```sh
python3 docs/experiments/shared-tls/validate-interop.py \
  --bundle path/to/bundle --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/tls-http-interop
```

Python must link an SSL library with TLS 1.3 support; the harness rejects older
system Python builds before compiling. The local macOS setup uses Homebrew Python
with OpenSSL. Credentials are disposable test fixtures with explicit localhost
trust; no certificate verification is disabled. Keep the generated output private
because it contains the private key. The Windows workflow invokes the same harness.

The first external-peer attempt exposed a managed pump deadlock: a short output
record could leave another handshake record queued, but SslStream waited for input.
The adapter now drains any output progress before reading again. The initial local
interpreter failure surfaced as authentication failure after waiting for the peer. The provider's pending status did not imply read readiness.
`--reuse-build` can repeat peer checks after validating recorded build inputs and
consumer source; use a new output directory when either changes.

[Local interoperability evidence](interop-validation.json) passes all twelve
peer/protocol/execution-mode combinations on macOS ARM64. The four existing HTTP
failure scenarios also pass again in the interpreter with the corrected library.
Windows execution remains pending.
