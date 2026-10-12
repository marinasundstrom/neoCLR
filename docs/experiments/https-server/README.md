# HTTPS server hosting proof

**Current qualification (2026-10-12):** macOS ARM64 and Windows x64 native AOT/JIT
pass the bounded HTTPS server checks. [Cross-platform evidence](platform-parity.json)
compares the shared AOT cases and Windows JIT outcomes. This remains a private
source-host capability; managed secure-stream APIs and installed TLS are not added.

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

The original qualification was macOS ARM64 JIT/AOT; Windows x64 now passes below.
Windows ARM64, interpreter TLS,
expired-certificate fixtures, forced network send-deadline expiry, adversarial load,
SNI certificate selection, client authentication and installed packaging remain open.
No production-readiness or throughput claim is made.

## Source project build

The source builder accepts `--profile http --https-server` on macOS ARM64 and
`--profile windows-http --https-server` on Windows x64 in an MSVC developer shell. This
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
scenarios, two credential rejection cases and two Rust transport tests. Add
`--cleartext-build` for ordinary-host isolation. Windows also supports `--windows-jit`
with a project build; macOS runtime-async JIT retains its separate proof above.

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
under backpressure, expired-certificate fixtures, interpreter support and installed
TLS packaging remain open. Windows x64 qualification follows below.

## Windows source-host integration — 2026-10-12

The source builder now accepts `--profile windows-http --https-server`, builds the
x86_64 MSVC Rust archive with a static CRT, and links it into the existing shared
socket adapter. OS socket calls remain behind `socket-os.h`; TLS state, deadlines,
cancellation and EOF rules are shared with macOS. Windows project validation uses
the same client cases, plus an optional `--cleartext-build` isolation check.
The integration alone did not qualify execution; the passing CI evidence is recorded
below. Windows ARM64 and Windows runtime suspension
remain separate gaps. No packaged build-kit capability is added.

### Reusable secure streams direction

The author requests async stream operations followed by a secure-stream equivalent,
possibly `System.Networking.Security.SslStream`, then clarifies that these concepts
need not copy .NET when other platforms offer a better fit. Current InputStream and
OutputStream interfaces are synchronous. The next managed design should compare
.NET's stream decorator with capability-based async I/O and explicit TLS state
machines, addressing buffer ownership, cancellation, authentication, duplex use and
async shutdown. This is direction, not a shipped API or a settled class hierarchy.
The host-only switch remains provisional while common managed APIs are developed.

The `Windows native HTTPS project` GitHub workflow runs
`scripts/validate-windows-https-project.py` in an MSVC x64 environment. It rebuilds
source libraries with Raven `371d1d7078bffa05e5e2607b6d3cd3833d6e8404`, builds
HTTPS and cleartext versions of the same Raven project, and runs 15 host cases and
two Rust transport tests. Reports retain commands, compiler/artifact hashes and
failure logs; generated test keys live only in a temporary directory. A workflow
existing or starting is not evidence that Windows passed.

The [first Windows CI run](https://github.com/marinasundstrom/neoCLR/actions/runs/38239236379)
reached native host compilation and exposed MSVC's deprecated `getenv` diagnostic
under warnings-as-errors. Windows credential loading now uses owned `_dupenv_s`
snapshots, freed after configuration parsing, rather than disabling strict warnings.
This build failure is not Windows TLS execution evidence.

The parallel [ordinary HTTP regression](https://github.com/marinasundstrom/neoCLR/actions/runs/38239236215)
passed requests but failed its byte-for-byte callback-fault comparison: native
frames have readable names and extra entry frames; interpreter frames retain CLI
bridge encodings. Both reported the same UserFault, exit status and empty response.
The harness now compares those observable outcomes while retaining and checking
stack-trace structure. Interpreter bridge-name rendering remains a diagnostics gap;
this test correction does not fix or hide it in retained evidence.

Credential-path parity also requires Unicode preservation. Following Microsoft's
[wide environment snapshot contract](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/dupenv-s-wdupenv-s?view=msvc-170)
(reviewed 2026-10-12), the Windows adapter uses `_wdupenv_s` and explicit UTF-8
conversion for the private Rust ABI. The shared harness places ephemeral credentials
under a directory containing accented and non-BMP characters. Conversion failure
rejects configuration; credential paths and contents are not diagnostic output.

The [second Windows run](https://github.com/marinasundstrom/neoCLR/actions/runs/38240463962)
compiled and linked the host, then the dependency check correctly rejected two
unlisted system imports. The HTTPS-only allowlist now includes
`bcryptprimitives.dll` ([ProcessPrng](https://learn.microsoft.com/en-us/windows/win32/seccng/processprng))
and `api-ms-win-core-synch-l1-2-0.dll`
([WaitOnAddress](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitonaddress));
Microsoft documents both as Windows system components (reviewed 2026-10-12).
The executable uses the static CRT; arbitrary third-party DLLs remain rejected.
This does not change the ordinary cleartext host's dependency policy. The CI host
is Windows Server 2022 x64; this is not qualification of older Windows versions.

The [third Windows run](https://github.com/marinasundstrom/neoCLR/actions/runs/38241143604)
passed native compilation/link/dependency checks, then Rustls correctly rejected the
OpenSSL-generated test certificate as a CA used as an end entity. The fixture now
supplies its own minimal OpenSSL configuration and explicit CA:FALSE, key usage,
serverAuth and SAN extensions on every platform. It no longer inherits machine
certificate-generation defaults; trust and hostname verification are unchanged.

The [fourth Windows run](https://github.com/marinasundstrom/neoCLR/actions/runs/38242127298)
passed both Rust transport tests and served a verified TLS response, then failed
while the Python peer attempted reciprocal shutdown after server socket disposal.
Normal-response validation now reads authenticated TLS EOF with
`suppress_ragged_eofs=False`: absent close_notify raises an error, and trailing
application data fails. Client-initiated clean shutdown retains its separate unwrap
case. This checks the existing one-way notification contract without claiming the
unimplemented fully asynchronous bidirectional shutdown API.

The Windows gate now also requests `--windows-jit`. The validator derives its helper
DLL from the exact AOT project's TLS-enabled host recipe, uses the existing Windows
JIT export builder, checks its system dependencies against the AOT executable and
runs the twelve request/recovery cases again with `--jit-run`. Combined acceptance
is 27 host cases (12 AOT, 12 JIT, two invalid-credential startups, one cleartext
isolation check), plus two Rust transport tests. This uses compiler state machines
on Windows; Windows runtime suspension remains unsupported. This gate addition
alone is not a passing Windows JIT result or installed CLI TLS support.

## Windows/macOS parity milestone — 2026-10-12

The [combined Windows run](https://github.com/marinasundstrom/neoCLR/actions/runs/38243539279)
passes **27 host cases and two Rust transport tests**. Twelve scenarios run in both
AOT and JIT: TLS 1.2/1.3, fragmented requests, untrusted/wrong-host peers, ALPN
rejection, plaintext/truncated handshakes, abrupt/clean EOF, timeout and cancellation.
Each network case verifies a subsequent successful request in the same runtime.
The other three cases reject missing/invalid credentials before listening and
verify that a host built without TLS still serves cleartext HTTP.

[Windows results](windows-jit-validation.json), [HTTPS build](windows-jit-project-build.json)
and [cleartext build](windows-jit-cleartext-build.json) retain commands, hashes and
observed system dependencies. The earlier [AOT-only run](https://github.com/marinasundstrom/neoCLR/actions/runs/38243158880)
also passes [all 15 cases](windows-aot-validation.json).
[macOS results](macos-parity-validation.json) pass the same 15 project cases and both
Rust tests with explicit leaf certificates in Unicode directories. Its
[HTTPS build](macos-parity-build.json) and [cleartext build](macos-parity-cleartext-build.json)
identify the exact inputs. [Machine-readable comparison](platform-parity.json)
confirms equal AOT case outcomes/negotiated protocols across hosts and equal results
for all 12 Windows AOT/JIT request scenarios. Large build output is represented by
hash/byte count; fullReportSha256 identifies the original CI/local report.

The [ordinary Windows HTTP regression](windows-http-regression.json) also passes
five cases, including callback faults, with the corrected diagnostic comparison.
Interpreter bridge-name rendering is tracked in [runtime diagnostics](../../runtime-diagnostics.md).

This closes Windows x64 source-host HTTPS qualification, using compiler-generated
state machines in Windows AOT/JIT. macOS runtime-async JIT/AOT retains the earlier
separate evidence. The debug-built Windows compiler is a correctness tool here;
repeated JIT startup in this matrix is not a performance benchmark. Windows ARM64,
Windows runtime suspension, interpreter/client TLS, managed per-connection APIs,
full async bidirectional shutdown and installed packaging remain open. Continue
through [common async transport](../../stream-design.md#common-async-transport-direction--2026-10-12)
toward the HTTP server/client JSON showcase.
