# HTTP client/server capability tracker and POC finish line

**Updated 2026-09-27.** The author asks to reach a point where this POC is “done for
now” and to track HTTP work on both client and server. This is the shared status
record, subordinate to the [platform roadmap](platform-roadmap.md).

**POC scope:** two neoCLR programs exchange typed JSON using bounded HTTP/1.1;
client/server cancellation, explicit resource lifetime, independent interoperability
and the current known-length upload increment. The bounded POC is complete on macOS arm64 as of 2026-09-27; feature scope is frozen.
[Preview 10](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.10)
is published with the qualified runtime, SDK and editor packages.
The author intends to release it after separate release qualification. Modern HTTP capabilities are tracked below without becoming automatic
completion requirements. POC completion is distinct from release qualification.

Statuses: **Implemented** means development source with linked focused evidence;
**Complete** records a satisfied POC gate; **Deferred** is outside this POC.
It does not imply inclusion in published Preview 9 or production readiness.

## Capability matrix

| Capability | Client | Server | Evidence / next boundary |
| --- | --- | --- | --- |
| HTTP/1.1 GET/HEAD/POST/PUT/PATCH/DELETE | Implemented | Implemented | [Verbs](experiments/http-verbs/README.md), [framing](http-client-design.md#bounded-response-framing-and-head--2026-09-25); bounded subset, not full conformance |
| Statuses and headers | Implemented: named/numeric status, defaults, request fields, case-insensitive lookup | Implemented: request lookup, response status/fields, no-body rules | [Statuses](experiments/http-status/README.md), [headers](experiments/http-headers/README.md), [request fields](experiments/http-request-headers/README.md) |
| Buffered bodies | Implemented, at most 1,024 bytes | Implemented, at most 1,024 bytes for request/response bodies | [POST](experiments/http-post/README.md) |
| Known-length source uploads | Implemented in current increment, at most 65,536 bytes, synchronous source and bounded chunks | Receives only within its existing 1,024-byte buffered limit | [Upload fixture](experiments/http-stream-upload/README.md); larger upload evidence uses an independent peer, not a claim of larger neoCLR server admission |
| Unknown-length/chunked request bodies | Deferred | Deferred; rejects request transfer coding | Define async producer and optional length before wire support |
| Response framing | Implemented: fixed-length, bounded chunked and close-delimited reception | Implemented: fixed-length buffered emission | [Framing contract](http-client-design.md#bounded-response-framing-and-head--2026-09-25); incoming chunked responses still fully buffered |
| Live response/request body streams | Deferred response reader | Deferred request reader and response writer | Separate body lifetime from connection lifetime; trailers and completion semantics |
| JSON DOM and flat typed mapping | Implemented: shared conversion and generic client helpers | Implemented: shared conversion through handler/context | [Mapped application](experiments/json-object-mapping/README.md), [helpers](experiments/http-json-client/README.md); current JSON size/shape limits remain |
| Cancellation and cleanup | Implemented through lookup/connect/transfer; upload ownership and one-shot rules | Implemented accept/context/complete/shutdown lifetime | [Network cancellation](experiments/network-cancellation/README.md), [context](experiments/http-context/README.md); synchronous source reads cannot be interrupted mid-read |
| Application pipeline | Implemented custom/forwarding HttpHandler | Implemented Accept/HttpContext and bounded callback hosting | No routing or application framework implied |
| HTTPS/TLS | Deferred for this cleartext POC | Deferred for this cleartext POC | [TLS feasibility](http-client-design.md#https-feasibility-checkpoint--2026-09-25); wider release scope remains an explicit decision, never an implicit public-network readiness claim |
| Pooling/persistent reuse | Deferred; currently one connection per exchange | Deferred; closes each exchange | Separate request and connection ownership |
| HTTP/2 | Deferred | Deferred | Multiplexing, flow control, HPACK, negotiation and per-stream errors |
| HTTP/3 | Deferred | Deferred | QUIC provider, QPACK, negotiation and stream lifecycle |
| Trailers, informational responses, duplex | Deferred | Deferred | Model events and terminal metadata independently of complete buffered bodies |
| Redirects, retries, compression and higher-level protocols | Deferred | Deferred as applicable | Separate policy/features; never replay a one-shot source implicitly |

See [modern HTTP direction](#modern-http-direction) for standards, .NET comparisons,
alternatives and protocol-specific acceptance cases. Future rows are tracked work,
not promises to implement all capabilities in the next release.

## Finish this POC

**Complete on macOS arm64 — 2026-09-27.** The finite HTTP POC is done for now.
No additional HTTP feature or optional optimization is required to close it.

- **Complete:** upload implementation, API/reference documentation and focused
  client/server evidence; see [source validation](experiments/http-stream-upload/validation.json).
- **Complete:** a fresh matching SDK and standard runtime bundle were archived,
  hash-verified and extracted outside the source checkouts. Typed JSON client/server
  and independent peers, all 11 upload cases, and the website download's mapped pair
  pass with zero final live objects. See [package acceptance](experiments/http-poc-package/README.md)
  and [exact revisions/hashes](experiments/http-poc-package/validation.json).
- **Complete:** supported POC host is macOS arm64, using the recorded matching .NET
  11 SDK/compiler for builds. Cleartext HTTP/1.1, fixed bounds/deadlines, synchronous
  upload sources and buffered server/response bodies remain the supported scope.
  Modern HTTP rows below remain deferred. No broader host/load/editor claim follows.

The recorded intermittent timeout and compiler diagnostic debt remain visible in
[acceptance findings](experiments/http-json/repeatability-20260926.md). Assess them
against supported sample execution and release quality; do not silently mark them
fixed, or require indefinite optimization under arbitrary competing machine load.
Compiler status and the upload getter workaround are owned by the
[toolchain tracker](tracking/toolchain-release.md#integration-and-correctness).

A runtime release still needs its own supported-target/package checks, known-defect
assessment and release notes. Publication is separate. HTTP feature work stops here until the author selects another capability. The
author intends to release this POC; [toolchain/release](tracking/toolchain-release.md)
owns efficient CI, selected-candidate packages and remaining release decisions.

## Modern HTTP direction

Recorded 2026-09-26 after the author clarified that “newer features of HTTP” means
more than progressively streaming HTTP/1.1 bodies. This is design guidance and a
future backlog, not implemented HTTP/2/3 support or a next-release commitment.
The [client/server tracker and POC finish line](http-capabilities.md) govern scope.

### Common message model, distinct protocol providers

Keep methods, URI, status, fields, body and exchange cancellation above wire framing.
[HTTP semantics (RFC 9110)](https://www.rfc-editor.org/rfc/rfc9110.html) is the shared
baseline. HttpHandler already separates the caller pipeline from HttpSocketHandler,
but current HttpRequest.Encode and socket-owned HttpContext internals still couple
implementation to HTTP/1.1. Those need separation when another provider is selected;
the current seam alone does not establish protocol independence.

[HTTP/2 (RFC 9113)](https://www.rfc-editor.org/rfc/rfc9113.html) introduces binary
frames, multiplexed streams, HPACK and stream/connection flow control over TCP.
[HTTP/3 (RFC 9114)](https://datatracker.ietf.org/doc/html/rfc9114) maps HTTP to QUIC,
including independent request streams and QPACK. Neither is a parser mode on the
current HTTP/1.1 byte stream. HTTP/2 over HTTPS also needs TLS/ALPN; HTTP/3 needs a
QUIC provider rather than assuming every connection is a Socket-backed TCP stream.

Keep these future capabilities explicit on both sides:

| Capability | Shared contract needed | Provider responsibility |
| --- | --- | --- |
| Streaming and long-lived bodies | Async production/consumption, optional known length, completion and bounded buffering | HTTP/1.1 lengths/chunks versus HTTP/2/3 DATA and end-of-stream |
| Multiplexed exchanges | Exchange ownership separate from connection ownership; distinguish request and connection errors | Admission limits, flow-control accounting, stream reset and connection shutdown |
| Duplex exchanges | Sending and receiving may progress independently; early responses can stop an upload | Scheduling reads/writes without unbounded queues or deadlocks |
| Trailers and informational responses | Distinct initial headers, interim events and terminal metadata | Encode/decode legal version-specific field sequences |
| Version negotiation | Requested policy, fallback constraints and actual negotiated version | TLS/ALPN, HTTP/3 discovery and supported-version selection |
| Reuse and replay | Explicit source replayability and request retry policy | Connection pooling does not imply replaying a consumed body |

Closing/cancelling an exchange must eventually mean release or reset that exchange;
it must not universally imply closing the underlying connection. Reading demand
must constrain production, including receive buffers, so flow control cannot be
implemented merely by accumulating a complete body. An InputStream adapter remains
useful for files and memory, but synchronous Read is not the eventual async body
contract. The first FromStream factory supports a declared range only; its Length
property is not a decision to require known lengths of every future body.

### Comparison and alternatives

.NET keeps version selection and response metadata on its message APIs:
[HttpVersionPolicy](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpversionpolicy?view=net-10.0)
and [TrailingHeaders](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpresponsemessage.trailingheaders?view=net-10.0)
are useful comparison points. neoCLR should preserve familiar semantics while
expressing expected failures through Result and acknowledged cancellation through
Task. No equivalent new public signatures are selected here.

A host HTTP provider could reuse mature protocol/TLS implementations, at the cost
of adapter/platform dependencies and less control over behavior. A managed protocol
implementation offers more control but adds substantial framing, compression,
flow-control and interoperability work. Evaluate those alternatives when selecting
a protocol milestone; the current POC does not choose or require a new backend.

Before claiming readiness for multiplexing, demonstrate two concurrent exchanges,
cancel one without aborting the other, and bound retained buffers under a slow
consumer. Trailer completion, early responses, duplex progress, negotiation failure
and connection loss need independent-peer tests. HTTPS must validate certificates
and hostnames. Higher-level RPC, event streams and WebSockets remain separate
consumer cases; streamed uploads alone implement none of them.

### POC boundary

Finish the current known-length upload slice and its client/server regression checks.
Do not extend this POC with unknown-length uploads, live response/server bodies,
HTTP/2, HTTP/3, trailers, pooling, duplex or a general body abstraction. Keep those
in the capability tracker with status and next evidence. This makes future design
visible while preserving a finite completion target.

## Issue and dependency ownership

This is the current HTTP/networking theme tracker, including neoCLR issue
[#18](https://github.com/marinasundstrom/neoCLR/issues/18). Its socket/DNS foundation
has typed addresses, bounded IPv4 connect/listen/accept/transfers and operation tokens;
see [networking evidence](experiments/network-cancellation/README.md) and
[socket contracts](socket-api-design.md). IPv6 transport and broader resolver policy
remain deferred. Existing design notes explain contracts; they do not add POC gates.

Shared text, streams and JSON work belongs to [library/data](tracking/library-data.md).
Task/scheduler and ownership foundations belong to [runtime/language](tracking/runtime-language.md).
Compiler debt, packaging procedures and release qualification belong to
[tooling/release](tracking/toolchain-release.md). The HTTP package acceptance result
and POC completion decision are recorded here so they have one owner.
