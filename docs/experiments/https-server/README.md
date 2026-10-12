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
