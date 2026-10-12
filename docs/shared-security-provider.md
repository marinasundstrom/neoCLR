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

The standalone `tools/native-crypto` archive exposes the C declarations in
`tools/native-crypto/include/neoclr_crypto.h`: secure random fill, one-shot SHA-256,
HMAC-SHA-256 and HMAC verification. All delegate to pinned ring 0.17.14, already
used by Rustls. Its safe Rust implementation now lives in the separate
`tools/security-provider` crate, without TLS dependencies. The C ABI wrappers
are packaged separately from TLS; native cryptography consumers no longer link
Rustls or certificate parsing. A bounded public managed facade now exposes these operations (see below). Private interpreter services now project
these operations through verified byte-array contracts; explicit JIT/AOT lowering is now qualified on macOS through a source host. Packaged build kits and broader cryptography APIs remain open.
Source AOT projects now select and link the standalone provider on demand.

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

The standalone archive checkpoint reruns all ten tests (four C-ABI tests, five TLS
tests and one shared provider failure test). The security CI workflow watches both
cryptography crates. HTTPS build manifests now include only their TLS dependencies.

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

## Native execution checkpoint

The [explicit cryptography bindings](experiments/native-cryptography/README.md) now
pass twelve identical cases across interpreter, JIT and AOT on macOS. The same
Windows CI gate is configured but not yet executed. The qualification host links
the standalone cryptography archive, with dependency checks excluding TLS. Packaged native build kits and broader cryptography APIs remain open. Source AOT projects
link the provider on demand, and prepared JIT hosts include these services for
default CLI execution. The bounded public cryptography facade is described below;
SslStream remains open.

## Managed cryptography facade (development, 2026-10-12)

`System.Runtime` now owns public `System.Security.Cryptography.SHA256`,
`HMACSHA256`, `RandomNumberGenerator` and the standard Raven `CryptographyError`
union. HashData writes to a caller-provided byte array and returns `Result<int,
CryptographyError>`; Fill returns `Result<unit, CryptographyError>`. HMAC VerifyData
returns bool and delegates tag verification to the provider. The private native
services remain internal library declarations.

This first slice follows .NET's static algorithm names and destination-buffer
HashData semantics. Deliberate differences are byte arrays instead of spans,
Result instead of throwing for a short destination or RNG failure, and a direct
provider-backed HMAC VerifyData helper. These choices preserve neoCLR's existing
recoverable-error model and avoid asking callers to implement tag comparison.
Costs are different signatures from .NET and a bounded surface; allocating
HashData overloads, stream hashing and stateful algorithms remain support gaps.
The existing .NET/ring sources above are the design baseline. No new Runtime
Contract setting or Raven compiler behavior is introduced. The reference bridge
contains declaration-only stubs and projects the error union from Raven source;
all executable bodies come from the native managed library.

[API contracts](../api-docs/cryptography.md) and the
[managed test suite](../runtime/raven/tests/cryptography/Tests.rvn) document borrowed
buffers, overlapping inputs, preserved tails, short destinations, exact tags,
empty inputs and provider failure. Lower-level fault/RNG-injection evidence remains
in the provider and runtime tests; the managed API adds no recoverable Fault path.

Local managed qualification passes three discovered tests in interpreter, JIT and
AOT (nine executions). Native project validation also fixes acceptance of the
existing optional PostgreSQL bundle layout without requiring that provider at
execution. Both aggregate and native API snapshots include these public types.
Windows runs the same managed suite in its JIT workflow; execution remains pending.
