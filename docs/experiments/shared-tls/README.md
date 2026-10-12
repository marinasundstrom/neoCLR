# Shared TLS development checkpoints

The [provider design](../../shared-security-provider.md) describes ownership, status
codes, platform scope and remaining managed API work. These are private development
contracts, not published Raven certificate/SslStream APIs.

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
