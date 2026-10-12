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

## Cryptographic primitive foundation

The same private archive now exposes the C declarations in
`tools/native-tls/include/neoclr_crypto.h`: secure random fill, one-shot SHA-256,
HMAC-SHA-256 and HMAC verification. All delegate to pinned ring 0.17.14, already
used by Rustls. Its safe Rust implementation now lives in the separate
`tools/security-provider` crate, without TLS dependencies; the TLS archive retains
the existing C ABI wrappers. There are no public managed methods yet. Private interpreter services now project
these operations through verified byte-array contracts; JIT/AOT lowering, the public
managed facade and API reference generation remain open.

The intended baseline is .NET's
[RandomNumberGenerator.Fill](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.randomnumbergenerator.fill?view=net-10.0),
[SHA256.HashData](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.sha256.hashdata?view=net-10.0)
and [HMACSHA256.HashData](https://learn.microsoft.com/en-us/dotnet/api/system.security.cryptography.hmacsha256.hashdata?view=net-10.0).
These are synchronous CPU/provider operations; no extra task or scheduler is needed
for each hash. Async stream hashing can compose the stream contract later.

Private ABI input buffers are borrowed. Hash/MAC write exactly 32 bytes into a
caller-owned destination of at least that capacity, allowing overlap with inputs.
Empty inputs and keys are valid; null nonempty inputs are rejected. The `u32`
lengths are a private ABI bound, not a promised managed payload limit. Hash/MAC
success is 0 and invalid arguments return -1 without altering output. Random fill
returns -2 on provider failure and clears any partial output. These return codes
still need projection into the platform's managed Result/error conventions.

[ring SystemRandom](https://docs.rs/ring/0.17.14/ring/rand/index.html) supplies OS
randomness. [ring HMAC verification](https://docs.rs/ring/0.17.14/ring/hmac/index.html)
avoids a handwritten early-exit comparison; verification returns 1 for mismatches,
including truncated tags. No key-length policy, password hashing, encryption,
exportable key handle, secret-zeroization guarantee or FIPS certification is implied.

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

The cryptography slice adds five passing local tests: SHA-256 known answers with
empty/multiblock/overlapping inputs; [RFC 4231](https://www.rfc-editor.org/rfc/rfc4231.html)
HMAC cases 1, 2 and 6 plus altered/truncated tags; invalid arguments preserving output;
random fill bounds; and injected provider failure clearing partial output. All ten
provider tests pass together on macOS. The same CI command covers these on Windows;
Windows execution remains pending. Random tests check behavior, not statistical quality.

The extraction checkpoint reruns all ten tests (nine C-ABI/TLS tests and one shared
provider failure test). Both CI workflows watch the shared crate, and HTTPS project
build manifests include its source inputs.

## Typed interpreter boundary

`runtime/neoCLR/Runtime/Cryptography.neoil` declares the private SHA-256, HMAC,
verification and random-fill operations. Their `arrayref<Byte>` arguments and exact
result signatures are checked before dispatch. The interpreter uses the same safe
provider crate as the native wrappers, copies initialized inputs and writes into
caller-owned destinations. It requires the Cryptography and ManagedArrays runtime
capabilities; it does not dynamically load foreign symbols or pass guest pointers.
See [runtime service semantics](runtime-services.md#cryptographic-primitives-development-2026-10-12).

Focused integration tests exercise known answers, aliasing, preserved destination
tails, short buffers, MAC tampering, empty/random buffers, reserved output,
uninitialized input rejection, exact signatures and capability reporting. These
are interpreter tests; they do not establish Raven API, JIT or AOT support. The
macOS/Windows provider workflow includes the same focused interpreter tests.

Local qualification: all seven cryptography integration tests and all twelve
runtime-service regression tests pass on macOS. Windows is wired into the same
CI job but has not been executed locally.
