# Shared security provider

Development checkpoint, 2026-10-12. The private Rust provider in
`tools/native-tls` now serves both TLS client and server sessions. Both host
platforms use the same Rustls implementation and nonblocking transfer contract;
sockets, cancellation, deadlines and scheduling remain runtime responsibilities.
This is groundwork for `System.Networking.Security.SslStream` and
`System.Security.Cryptography`, not those managed APIs being available yet.

## TLS contract

The existing HTTPS file-based configuration still selects HTTP/1.1. New private
byte-oriented entry points accept PEM server credentials or explicit client trust
anchors, and an optional single ALPN identifier. Empty ALPN is protocol-neutral.
Inputs are borrowed during configuration; sessions retain their configuration after
the original buffers and configuration handles are released. Each input is bounded
at 64 KiB, and an ALPN identifier at 255 bytes. Server and client configuration
handles have separate destruction functions and must not be interchanged.

Client sessions require a UTF-8 DNS name or IP address. Rustls verifies the chain,
validity and peer identity against the supplied anchors. There is no insecure
verification switch, automatic OS-store loading, client identity, resumption or
revocation configuration in this slice. Loading server credentials does not assert
that clients trust them. Failures currently collapse into a private terminal status;
a useful managed error model is still needed.

`neoclr_tls_handshake` returns success (0), pending (-1) or terminal failure (-2),
without consuming application plaintext. A host serializes access to each session
and polls after readiness; it must not busy-spin. Existing transfer semantics remain:
a pending write retains its operation until ciphertext flush completes; a retry
must not replace its payload. An authenticated close delivers buffered plaintext
before EOF; a truncated transport is a terminal failure. Destruction sends
close-notify best effort and never waits. An awaited graceful shutdown is still open.
All opaque handles and callbacks are trusted internal ABI, not guest pointers.

## Comparison and boundaries

[.NET SslStream](https://learn.microsoft.com/en-us/dotnet/api/system.net.security.sslstream?view=net-10.0)
provides explicit authentication and encrypted I/O above another stream. We retain
that layering intent. The private provider instead uses runtime-owned nonblocking
callbacks, avoiding a second scheduler or socket owner. This permits the same engine
on Windows and macOS, but explicit roots do not reproduce .NET's default platform
trust-store behavior. That limitation is deliberate for this bounded foundation,
not a substitute for a future certificate/trust API.

[Rustls 0.23.45](https://docs.rs/rustls/0.23.45/rustls/)
provides the actual protocol and verification, using its ring provider. There is no
custom TLS or cryptographic algorithm implementation. The eventual managed facade
will use canonical async InputStream/OutputStream and System.Concurrency.Tasks.

## Validation

Run `python3 scripts/test-native-security.py` with Rust and OpenSSL installed.
It generates disposable credentials and runs all provider tests. The same command
is wired into the macOS/Windows `Native security provider` workflow. The existing
Windows HTTPS workflow also exercises the extended ignored transport tests.

Local macOS validation passes five tests: invalid configuration, wrong peer name,
client/server authentication and bidirectional fragmented/backpressured transport
(with and without ALPN), and the two existing HTTPS transport/EOF regressions.
No Windows execution of the new client path is claimed by this local checkpoint.
Before exposing managed TLS, add negative trust/expiry/key-mismatch qualification,
stream ownership/concurrency tests and application-level JSON round trips.
