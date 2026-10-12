# Shared security provider

Development checkpoint, 2026-10-12. The private Rust provider in
`tools/native-tls` now serves both TLS client and server sessions. Both host
platforms use the same Rustls implementation and nonblocking transfer contract;
sockets, cancellation, deadlines and scheduling remain runtime responsibilities.
This is groundwork for `System.Networking.Security.SslStream`. The bounded managed
`System.Security.Cryptography` facade is available as described below; managed SSL
streams and certificate objects remain under development.

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
that clients trust them. Configuration creation now has typed private status codes (below). Authentication
failures still collapse into a private terminal status; a managed error model and
more precise handshake diagnostics remain needed.

`neoclr_tls_handshake` returns success (0), pending (-1) or terminal failure (-2),
without consuming application plaintext. A host serializes access to each session
and polls after readiness; it must not busy-spin. Existing transfer semantics remain:
a pending write retains its operation until ciphertext flush completes; a retry
must not replace its payload. An authenticated close delivers buffered plaintext
before EOF; a truncated transport is a terminal failure. Destruction sends
close-notify best effort and never waits. Explicit pollable local shutdown is now
available in the private ABI (below); its managed awaited projection remains open.
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

## Certificate configuration foundation (development, 2026-10-12)

The private `neoclr_tls_server_config_create_pem` and
`neoclr_tls_client_config_create_pem` ABI functions return a status plus an owned
configuration handle. The output slot is cleared before validation and remains
null on failure. It must not alias input buffers. Existing pointer-returning
functions delegate to these operations, preserving existing HTTPS hosts.

| Status | Meaning |
| --- | --- |
| 0 | Configuration created; release with the matching client/server free function |
| -10 | Invalid borrowed-buffer/output argument or input exceeds 64 KiB |
| -11 | Missing/malformed server certificate or invalid leaf DER |
| -12 | Missing, malformed or unsupported private key |
| -13 | Leaf public key and private key do not match |
| -14 | Missing/malformed/unusable explicit trust anchor |
| -15 | ALPN identifier exceeds 255 bytes |
| -16 | Provider protocol configuration failed |

These are private boundary codes, not public managed enum values. They do not
include provider strings or key material. Null output pointers are rejected;
other pointer validity and correct handle kinds remain trusted-host requirements.
Configuration parsing/key matching does not establish remote trust, certificate
validity at connection time or successful authentication. Chain/identity validation
remains part of each client handshake. No expiry override or custom clock is exposed
through the ABI; tests inject time directly into Rustls configurations.

The comparison remains [.NET SslStream authentication options](https://learn.microsoft.com/en-us/dotnet/api/system.net.security.sslclientauthenticationoptions?view=net-10.0)
and [Rustls 0.23.45 configuration](https://docs.rs/rustls/0.23.45/rustls/struct.ConfigBuilder.html)
(reviewed 2026-10-12). We preserve explicit authentication and separate loading
from peer validation. Returning a bounded status lets the managed adapter project
Result errors consistently; it costs diagnostic detail relative to .NET exceptions.
Keeping null-only failure was simpler but insufficient for that adapter. No new
metadata, compiler bridge or managed surface is introduced by this prerequisite.

Local qualification passes all thirteen security-provider tests: eight TLS tests,
four native cryptography tests and one safe-provider failure test. New cases cover
cleared failure outputs, malformed certificate/key/trust, mismatched keys, invalid
ALPN and untrusted/expired/not-yet-valid peers. Rejected sessions remain terminal,
reject application reads/writes, and do not poison new sessions. Existing tests
exercise config/buffer release before authentication, fragmented bidirectional
traffic, backpressure and authenticated EOF. The same harness already runs on macOS
and Windows CI; new Windows execution remains pending. See
[retained evidence](experiments/shared-tls/certificate-foundations-validation.json).

## Shared concepts, independent backends

**Author clarification, 2026-10-12:** HTTP and the SQL provider may share certificate,
trust and authentication concepts/types without being forced onto one native
implementation. A shared managed SSL stream is one transport implementation, not
a required base for every provider.

The intended layers remain:

- Managed contracts describe credentials, explicit trust, peer identity and ownership.
- Each native backend translates supported options and reports unsupported capabilities
  explicitly; it must not silently weaken validation or ignore requested settings.
- Backends using our Rustls adapter can share its implementation. A SQL provider using
  another native library may adapt the same concepts to that library instead.

This is the direction for future API design, not an implemented shared SQL/TLS API.
OS certificate stores, hardware-backed keys and exportability need explicit capability
and ownership decisions; common concepts must not require extracting private-key bytes.
The next managed slice still needs rooted handle ownership, async stream composition,
cancellation/concurrency rules and the client/server JSON consumer. No claim of those
features follows from provider-level qualification alone.

## Pollable graceful shutdown (development, 2026-10-12)

`neoclr_tls_shutdown` queues close-notify once and flushes its ciphertext through the
same bounded nonblocking callback used for other transport operations. It returns
0 when flushed, -1 while pending, -2 for terminal transport/TLS failure, and -3 if
handshake or a prior write operation still needs completion. An invalid-state result
does not consume the write acknowledgement or change the session. The host must
serialize calls and resume after readiness rather than spin.

Once shutdown begins, new application writes return -3; application reads remain
available. Successful repeated shutdown is idempotent. Completion means the local
alert reached the underlying transport, not that the peer received it or returned
its own alert. This does not close the underlying stream. Cancellation/deadline
handling remains host-owned and may abort the session; abort and transport failure
are terminal. Disposal remains immediate best-effort cleanup and cannot substitute
for awaiting a required graceful shutdown.

The baseline is [.NET SslStream.ShutdownAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.security.sslstream.shutdownasync?view=net-10.0)
and [Rustls close-notify](https://docs.rs/rustls/0.23.45/rustls/struct.CommonState.html#method.send_close_notify),
reviewed 2026-10-12. A private readiness-driven poll avoids making a second scheduler
inside the TLS provider. The cost is explicit serialization and lifecycle work in
the managed adapter. No Task or managed SslStream member is exposed by this slice.

Tests force TLS 1.2 and TLS 1.3, fragmented/backpressured alert writes, peer-observed
EOF, repeated shutdown, continued inbound traffic, rejection during an unfinished
write, abort during shutdown and terminal transport failure. All sixteen provider
tests pass on macOS; the unchanged macOS/Windows CI harness covers the new tests.
[Evidence](experiments/shared-tls/shutdown-validation.json) records the local boundary.

## Checked resource ownership boundary (development, 2026-10-12)

The private `neoclr_tls_context_*` ABI now owns configurations and sessions in a
host-created context. It is intended for the forthcoming managed adapters; existing
HTTPS hosts retain their original pointer-based ABI. The host chooses a live-resource
limit from 1 to 65536. Guest-facing adapters can carry 64-bit resource IDs instead of
native pointers; IDs are never reused within a provider instance, even across
contexts. Lookups check context membership and resource kind. These IDs are not
security capabilities or authorization tokens.

Configuration creation copies/parses borrowed credentials and returns a configuration
ID. Session creation retains its configuration independently, so callers may release
the configuration and original PEM buffers before authentication. `release` removes
one resource; context teardown drops every remaining resource, including configuration
references retained by sessions. Both are abortive and perform no callbacks or I/O.
Explicit graceful shutdown remains separate. Repeated release returns invalid-handle
rather than touching freed storage; a future managed Dispose should clear its own ID.

Additional private statuses are -17 (live-resource/ID limit), -18 (missing, wrong-kind,
stale or foreign-context ID), and -19 (invalid client peer name or nonempty server
name). Creation outputs are zero on failure. Failure does not publish a resource or
consume live capacity. Buffer sizes retain the private 64 KiB bound. The private
transport functions also validate pointer/count combinations and sending direction;
valid native buffer storage and writable, non-aliasing output slots remain trusted
host obligations.

All operations, releases and context teardown must be serialized by the host. A
transport callback must not reenter the context. Callbacks and transient I/O buffers
are never retained. The raw context pointer is host-owned and must not be exposed to
guest code or used after teardown. Integer checks do not make those host rules optional.

The .NET baseline is [SafeHandle lifetime protection](https://learn.microsoft.com/en-us/dotnet/fundamentals/runtime-libraries/system-runtime-interopservices-safehandle)
(reviewed 2026-10-12): resources must not be freed or recycled while in use. Instead
of exposing an OS/provider pointer, this bounded adapter uses typed context lookup,
non-reused IDs and retained configuration ownership. This adds a table lookup and
requires runtime serialization; it is not a replacement for managed rooting or a
performance claim. Per-object GC cleanup, pending-operation roots, service admission,
interpreter/JIT/AOT bindings and managed certificate/SslStream APIs remain required
before exposing the boundary to Raven. Context teardown alone is insufficient for
long-lived runtimes whose managed wrappers become unreachable.

The focused [ownership evidence](experiments/shared-tls/resource-context-validation.json)
passes twenty provider tests locally: fifteen TLS, four native cryptography and one
safe-provider test. New tests cover quotas, failed creation, invalid names, wrong-kind,
stale and foreign handles, configuration lifetime through session/context teardown,
and a complete fragmented transfer plus graceful shutdown after configuration and PEM
release. Windows runs this unchanged harness in the existing provider workflow;
execution of the new cases remains pending. This Rustls-specific adapter does not
require an SQL backend to adopt its resource representation or TLS implementation.

## Interpreter service and heap ownership (development, 2026-10-12)

`runtime/neoCLR/Runtime/Tls.neoil` introduces exact private InternalCall declarations
for TlsCreateClient, TlsCreateServer, TlsCreateSession, TlsRelease and TlsAbort. The
interpreter dispatches only the declared byte-array signatures and Int32 return type;
capability analysis reports Tls, ManagedArrays and ManagedHeap. Native JIT/AOT lowering
is not enabled for these imports yet. No Raven public declaration is added by this slice.

Every resource is associated with a distinct, zero-length managed byte-array owner
token. The token is a temporary private representation, not a proposed public
certificate type. Calls validate the token's provenance against the active heap;
foreign, collected or incorrectly shaped references fault. Credentials/ALPN/peer-name
arrays must contain initialized bytes. Configuration and session creation return the
provider's status; reusing an owner with a live resource faults. A session is created
from another owner's configuration, retaining the provider configuration independently.
There is no native pointer or numeric provider handle in the guest contract.

The heap keeps an untraced owner-ID-to-resource association. After a successful mark,
resources whose owners are unmarked are released before the collection completes.
Failed tracing does not release anything. A managed wrapper or suspended operation
must keep the token reachable; the resource table itself does not keep it alive.
Explicit release is idempotent while the owner is live. Heap teardown also drops the
resource context. Cleanup is abortive, performs no I/O or managed callbacks, and is
not a user finalizer mechanism. Awaited graceful shutdown remains a separate operation.

This is a focused adaptation of the SafeHandle ownership concern described above,
using the existing tracing heap rather than adding guest finalizers. The native
resource quota remains 1024 per heap; collection scans active TLS associations only
when a TLS context exists. Costs include a byte-array token and a resource-table entry
per owner, a bounded scan on collection and the shared Rustls dependency in the
interpreter. No performance improvement is claimed. The native compiler's dependency
lock is updated too; all provider builds retain matching pinned TLS dependencies.

Validation includes executable IL service admission, invalid credentials, explicit
release, foreign/invalid owner rejection, uninitialized input faults, retained sessions
and collection of unrooted owners. The provider CI harness now runs the focused heap
lifetime tests with its disposable certificates on macOS and Windows. Results and
remaining gates are recorded in the [validation checkpoint](experiments/shared-tls/interpreter-lifetime-validation.json). Full managed
stream composition still needs ciphertext pumping, pending-operation ownership and
native JIT/AOT service/collector integration before Raven certificate/SslStream APIs.

## Native collector resource ownership hook (development, 2026-10-12)

The private native collector now supports caller-owned resource registrations with
a managed owner and a trusted native cleanup function. Registration does not retain
the owner. Once tracing succeeds, unreachable owners trigger cleanup before sweep;
explicit release uses the same unlink-and-clear path. Cleanup can destroy its own
registration, runs once, and must not perform I/O, allocate managed objects, call
guest code, reenter runtime services or fail. Failed tracing leaves registrations
untouched. This is resource disposal plumbing, not managed finalization.

The hook addresses the same SafeHandle ownership concern as the interpreter adapter
above. It differs from CLR finalization by allowing only trusted, abortive native
cleanup at collection. This avoids a finalizer thread and guest-code execution during
GC, but requires adapters to provide bounded, nonblocking destruction. Registrations
are thread-affine and limited to 1024 per context. The collector scans registrations
without allocating storage; adapters own that storage. No throughput improvement is
claimed. Before arena teardown, adapters must release their records; entry reset and
session exclusive-ownership checks reject outstanding registrations.

The collector contract covers rooted/unrooted owners, foreign/invalid owners,
duplicate attachment, explicit release, failed tracing, address reuse, per-context
quotas and cleanup that frees its registration. Host-root, collector pass and session
regressions also pass locally under UndefinedBehaviorSanitizer. AddressSanitizer
stalled in its runtime initializer before test code executed on this host and is
not counted as passing. The Windows host workflow runs the collector contract with
safepoint collection enabled; execution of the new cases remains pending.
See [native lifetime evidence](experiments/shared-tls/native-lifetime-validation.json).

This hook is not yet wired into a native TLS adapter. Native TLS service lowering,
adapter teardown, pending-operation roots and shared ciphertext pumping remain
required before implementing and qualifying Raven certificate/SslStream APIs.
