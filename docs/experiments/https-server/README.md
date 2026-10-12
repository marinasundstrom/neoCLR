# HTTPS server hosting proof

Author-selected next slice, 2026-10-12. This is an experimental native host capability,
not a new managed TLS API or installed HTTPS support. Raven's existing HttpServer runs
over authenticated TLS records without changing its HTTP parser/handler pipeline.
The server presents a configured certificate; the independent client validates its
trust and hostname. No client certificate authentication is provided.

## Scope and layering

`tools/native-tls` pins Rustls 0.23.45 with ring and TLS 1.2/1.3 support behind a
private C ABI. The runtime owns socket descriptors, nonblocking polling, deadlines,
cancellation and managed roots. Rustls owns cryptographic state and record buffers;
it receives borrowed I/O callbacks and never closes sockets, schedules work, holds
managed pointers or invokes Raven. This boundary can support future client and SQL
adapters, but neither is implemented here.

Build every native host source with `NEOCLR_HTTPS_SERVER` and link the matching Rust
archive. The flag changes the private socket-scope layout; do not mix objects/headers
from different configurations. In this **HTTPS-only host**, every accepted connection
uses TLS. Outgoing connections remain ordinary TCP. Without this flag all Socket and
HttpServer behavior stays cleartext. This is a temporary explicit host capability,
not a silent change to the managed Socket contract or a mixed-listener API.

Set `NEOCLR_HTTPS_CERTIFICATE` to a PEM certificate chain and
`NEOCLR_HTTPS_PRIVATE_KEY` to its unencrypted PEM private key before launching the
host. Configuration is loaded once per socket scope, before binding; invalid/missing
credentials fail host admission and never enable cleartext fallback. Each file is
limited to 64 KiB. Keys are not logged, copied into guest memory or embedded in code.
Certificate rotation, encrypted-key stores and OS certificate selection remain open.

The first receive drives authentication before releasing plaintext to HTTP. Its
existing nonrenewing request-read deadline includes the handshake. Send acknowledges
plaintext only after its encrypted output is flushed; partial/would-block polls do
not replay input. Cancellation or expiry poisons the TLS session, preventing reuse
of a partially completed operation. The HTTP layer then closes it and can accept a
new connection. Failures currently map to transport errors; a richer TLS error
contract is deferred.

ALPN advertises only `http/1.1`; an incompatible offer fails. Clients omitting ALPN
may use HTTP/1.1. No TLS 1.0/1.1, early data, client authentication, session resumption
or ticket issuance is enabled. Each poll performs bounded record I/O without a
blocking handshake loop. Rustls queue limits are 64 KiB and individual callback I/O
is at most 16 KiB; these are not a total TLS memory cap. Crypto/configuration storage
is outside managed GC accounting, subject to the existing 64-socket host limit.

Normal close attempts a nonblocking close_notify. Cancellation, failure and host
teardown release TLS state and descriptors without waiting for peers. Guaranteed
asynchronous graceful TLS shutdown under backpressure remains a gap; a synchronous
Close cannot promise it. Abrupt TLS EOF is not converted into a clean stream EOF.

## Comparison and provisional choices

Primary sources reviewed 2026-10-12:

- [.NET SslStream](https://learn.microsoft.com/en-us/dotnet/api/system.net.security.sslstream?view=net-10.0)
  separates TLS authentication/byte transport from HTTP. This proof follows that
  layering; it does not implement SslStream's managed surface or platform key stores.
- [Rustls ServerConnection 0.23.45](https://docs.rs/rustls/0.23.45/rustls/server/struct.ServerConnection.html)
  and its pinned source define record processing, nonblocking I/O and buffer behavior.
  We use explicit polling instead of a potentially repeated complete_io operation.

An OS TLS backend could better reuse native certificate stores, at the cost of
separate transport/lifetime adapters. Rustls supplies one portable protocol engine
and permits independent testing of the runtime's I/O boundary. Costs include native
package size, separate trust/key configuration, dependency maintenance and memory
outside the current collector profile. This selects Rustls for this proof, not an
irreversible public provider contract. The ordinary managed API, metadata and Raven
Runtime Contract configuration do not change. Future managed secure-transport and
per-listener configuration must replace this HTTPS-only host switch before general
mixed-protocol hosting or PostgreSQL TLS upgrades.

## Validation and reproduction

Use Python linked to OpenSSL with TLS 1.3 (the local system Python/LibreSSL lacks it),
OpenSSL CLI, Rust, and the passing runtime-async HTTP recipe's native compiler:

```sh
python3 docs/experiments/https-server/validate.py \
  --bundle target/generic-cleanup-gc-bundle/bundle \
  --output target/https-server-new
```

The harness compiles `Server.rvn` with runtime async, links matching AOT and JIT
hosts and generates one-use test credentials in a temporary directory. It makes
no system trust-store changes. The default fixture trust is explicit for that peer;
it does not bypass certificate or hostname verification. Generated private keys are
not retained in source or the validation report.

[Recorded validation](validation.json) covers TLS 1.2/1.3, fragmented application
writes, untrusted certificate and wrong-host rejection, incompatible ALPN, plaintext
and truncated handshakes, handshake timeout and cancellation. Every network case
then serves a valid recovery request in the same runtime. Normal success also checks
close_notify and the native host's usual heap/root/async teardown invariants.
Missing/invalid credential files fail before listening. A separately linked ordinary
host serves the same application over cleartext, verifying opt-in isolation.

A Rust transport test uses seven-byte writes, eleven-byte reads and alternating
would-block responses throughout authentication and an 8 KiB response. It verifies
no duplicated plaintext, deferred send acknowledgement, config/session ownership
and terminal cancellation. It runs explicitly with ephemeral fixture credentials;
ordinary `cargo test` marks it ignored because those credentials are required.

Qualification is macOS ARM64 JIT/AOT only. Windows x64/ARM64, interpreter TLS,
expired-certificate fixtures, forced network send-deadline expiry, adversarial load,
SNI certificate selection, client authentication and installed packaging remain open.
No production-readiness or throughput claim is made.

## Source project build

The source builder accepts `--profile http --https-server` on macOS ARM64. This
links the pinned Rustls archive into the ordinary project AOT host. It is an
explicit development capability, rejected for other profiles, retained-session
bootstrap and installed build kits. All accepted connections use TLS; outgoing
connections retain their existing transport behavior. No public managed API is added.

```sh
python3 scripts/build-native-project.py --profile http --https-server \
  --project docs/experiments/https-server/Native.rvnproj \
  --bundle /path/to/qualified/bundle \
  --aot tools/aot-poc/target/release/neoclr-aot-poc \
  --output target/https-project-new
python3 docs/experiments/https-server/validate.py \
  --bundle /path/to/qualified/bundle \
  --project-build target/https-project-new \
  --output target/https-project-validation-new
```

Supply `NEOCLR_HTTPS_CERTIFICATE` and `NEOCLR_HTTPS_PRIVATE_KEY` as PEM file paths
when running the executable directly. Credentials are loaded at runtime, not
embedded in the executable or build report. The validation harness creates and
removes its own ephemeral credentials. Its project mode verifies the build's
input and artifact hashes before using the executable, then runs twelve network
scenarios, two credential rejection cases and two Rust transport tests. Cleartext and
JIT isolation retain the separate runtime-async proof above.

The project path uses compiler-generated state machines. A qualified compiler
must include the async project cleanup configuration fix and unary-operand walker
fix described in [the compiler regression](../async-unary-cleanup/README.md). Existing
published bundles are not silently upgraded by this source-only option.

[Project build evidence](project-build.json) and [host results](project-validation.json)
qualify native compiler `c71baae94` with the unchanged original awaited condition.
The build report replaces its 7 MB native selection stdout with byte count and hash;
`fullReportSha256` identifies the original report checked by the validator. Twelve
host cases and the explicit Rust transport test pass. This locally qualified bundle
retains the existing runtime libraries and replaces only its compiler directory;
the original bundle is preserved. Both compiler and library inputs are hashed.

## Authenticated shutdown qualification — 2026-10-12

The existing TLS EOF policy now has direct regression coverage. For nonempty receive buffers, the private ABI
must deliver all buffered plaintext, report zero bytes only for authenticated
`close_notify`, and report a terminal error after abrupt TCP EOF. A second Rust
transport test exercises both outcomes with fragmented records and checks that a
failed session cannot resume. The runtime implementation required no change.

The independent OpenSSL client also sends an incomplete HTTP request after a valid
handshake, then either sends `close_notify` or disconnects without it. Both requests
must fail before reaching the handler; each is followed by a successful request
on a new connection in the same runtime. A clean TLS shutdown does not make an
incomplete HTTP message valid. Normal complete responses still require close_notify.

[Project AOT results](eof-project-validation.json) pass 14 host cases and both Rust
tests, using [this build](eof-project-build.json). The build evidence retains the
original report hash and replaces large selection output with its hash/byte count.
[Runtime-async AOT/JIT results](eof-runtime-validation.json) pass 27 host cases and
both Rust tests, including the ordinary cleartext-host check. All inputs are hashed;
private credentials remain temporary. Earlier reports above are historical checkpoints.

This closes the bounded authenticated-EOF qualification gap on macOS ARM64. It
adds no managed API or new TLS behavior. Guaranteed asynchronous graceful shutdown
under backpressure, expired-certificate fixtures, Windows/interpreter support and
installed TLS packaging remain open.

## Windows source-host integration — 2026-10-12

The source builder now accepts `--profile windows-http --https-server`, builds the
x86_64 MSVC Rust archive with a static CRT, and links it into the existing shared
socket adapter. OS socket calls remain behind `socket-os.h`; TLS state, deadlines,
cancellation and EOF rules are shared with macOS. Windows project validation uses
the same client cases, plus an optional `--cleartext-build` isolation check.
Windows execution is not yet qualified by this integration change; CI evidence is
required before claiming parity. Windows ARM64 and Windows runtime suspension
remain separate gaps. No packaged build-kit capability is added.

### Reusable secure streams direction

The author requests async stream operations followed by a secure-stream equivalent,
possibly `System.Networking.Security.SslStream`, then clarifies that these concepts
need not copy .NET when other platforms offer a better fit. Current InputStream and
OutputStream interfaces are synchronous. The next managed design should compare
.NET's stream decorator with capability-based async I/O and explicit TLS state
machines, addressing buffer ownership, cancellation, authentication, duplex use and
async shutdown. This is direction, not a shipped API or a settled class hierarchy.
The host-only switch remains provisional while Windows behavior is qualified.
