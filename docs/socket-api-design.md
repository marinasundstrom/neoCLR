# Socket API: first application-driven slice

**2026-09-24 · Development TCP client slice; full echo and broader Socket API remain open.**

The author selects APIs needed to build a web application running on neoCLR,
starting with sockets. Keep the [networking proposal](proposals/network-api.md)
as the direction. Establish interfaces and behavior incrementally through working
cases. This supersedes the earlier instruction to keep networking behind a general
Object/String consistency review; unfinished foundation work remains recorded and
is addressed when the application requires it.

## First case and layers

Start with a bounded loopback TCP echo client/server, then reuse its byte transport
for the HTTP application. Preserve the proposal's portable Socket boundary,
asynchronous waiting, recoverable Result failures, common streams and explicit
ownership. HTTP and a future web framework sit above these layers. DNS, TLS, UDP,
multicast, general socket options and web routing are not prerequisites for TCP echo.

The [first executable transport probe](experiments/socket-api/README.md) implements
nonblocking IPv4 TCP creation, bind/listen/accept/connect, endpoint queries, byte
send/receive, send shutdown and close on the host. It is an isolated experiment,
not a guest API or completion of S4. Neither runtime nor compiler artifacts change.

## Candidate guest interface

Use `System.Networking.Sockets` as the working socket namespace, following the
author's earlier namespace direction. Address/endpoint placement and the exact
Raven declarations remain provisional. These are contract sketches, **not compiled
signatures or available APIs**:

| Operation | Candidate result and behavior |
| --- | --- |
| Create Socket | Family/type/protocol as in the proposal; start with IPv4/Stream/Tcp. Native allocation can fail, so resolve fallible construction before exposing a constructor or factory. Unsupported combinations must be explicit errors. |
| Bind(endpoint) | `Result<(), SocketError>`; bind once, allow port zero, report the assigned local endpoint. |
| Listen(backlog) | `Result<(), SocketError>`; require a bound socket and positive backlog. Backlog is an OS hint, not an application connection quota. |
| Accept() | `Task<Result<Socket, SocketError>>`; wait without blocking unrelated work; accepted socket owns its connection independently of the listener. |
| Connect(endpoint) | `Task<Result<(), SocketError>>`; failed connect closes this socket in the probe; retry with a new instance. Explicit local bind before connect remains a follow-up. |
| Receive(buffer, offset, count) | `Task<Result<int, SocketError>>`; checked byte range, possibly short result; zero for a nonempty request means peer EOF. A valid empty request completes with zero without touching the transport. |
| Send(buffer, offset, count) | `Task<Result<int, SocketError>>`; possibly short result, caller advances by the count. Empty sends complete with zero on an open write direction. |
| LocalEndpoint / RemoteEndpoint | Local endpoint after bind/connect; remote endpoint on a connected socket. Exact property/error shape needs the addressing slice. |
| Shutdown send direction | Stop further sends, retain receives, and allow the peer to consume queued bytes before EOF. Probe repeats are harmless. General shutdown enum remains open. |
| Close / disposal | Idempotent resource release; later I/O fails Closed, including empty I/O. Terminal invocation teardown must also reclaim resources. Guest close/disposal spelling remains open. |

Keep `WouldBlock` inside the backend: it means the operation is still pending, not
a recoverable guest failure or EOF. Expected failures need stable cases, initially
closed/wrong state, invalid range, resource limit, address in use, access denied,
connection refused/reset and a fallback I/O category. Exact error representation and
diagnostic details are not frozen by the probe's Rust enum.

Cancellation requests and terminal Task cancellation remain distinct. Closing a
socket must settle its pending operations once, but whether they complete with a
Closed error or become Cancelled needs the guest completion integration case.
The transport probe retains no pending task or borrowed buffer across calls; it
does not decide that race by returning Closed on a later attempt.

## Comparison and tradeoffs

Primary documentation checked 2026-09-24:

- [.NET 10 Socket.ReceiveAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.receiveasync?view=net-10.0)
  supplies the familiar partial byte-count, EOF and cancellation baseline. Its
  documented zero-byte receive may wait for data. The proposed immediate empty
  receive is a deliberate simplification aligned with our stream direction, losing
  that readiness-probe behavior. Normal network errors use Result in neoCLR rather
  than .NET's exception-bearing tasks; this requires new adapters and error mapping.
- [.NET Socket.Shutdown](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.shutdown?view=net-10.0)
  separates directional shutdown from disposal. Retain that useful distinction for
  protocols that finish sending before receiving the reply. The checked .NET probe
  demonstrates the reverse direction remains usable after send shutdown.
- [Tokio TcpStream](https://docs.rs/tokio/latest/tokio/net/struct.TcpStream.html)
  documents readiness plus try-read/write and the need to handle WouldBlock after
  readiness. This is an implementation alternative, not a selected dependency or
  scheduler. The probe uses socket2 0.6.1 with nonblocking native sockets instead.

This is a library/host boundary decision, not a new language feature, CLI metadata
contract or instruction. Keep current Raven async state machines for the first
integration. A bounded worker implementation could simplify wakeup integration but
adds threads and difficult cancellation of blocking calls. Readiness/completion
integration avoids blocking the invocation but adds registration, wakeup, fairness
and teardown work. This probe establishes transport behavior only; it selects no
production event backend and makes no throughput or allocation improvement claim.

Owned host buffers with invocation-thread delivery remain the first integration
candidate, reusing the [external-I/O ownership research](experiments/external-io-progress/README.md)
and [VM delayed-copy evidence](experiments/delayed-copy/README.md). This avoids foreign
threads mutating managed arrays but costs copies and requires explicit quotas and
GC roots. Direct native writes to guest memory remain unselected.

## Next implementation checkpoint

First establish the [internal scheduling boundary](runtime-scheduling-design.md),
following the author's runtime-async question. Existing queue behavior is a transitional
adapter; transport completion must not require a particular generated continuation.
The current TCP/GC/VM probes validate parts of that ownership boundary, not a portable
scheduler. Keep full runtime frame suspension separate from this first bridge.

1. Add the smallest Raven-facing addressing, Socket and error contracts needed by
   echo; implement matching runtime services and checked artifact metadata. Refresh
   the API reference from the matching bridge in the same change.
2. Integrate pending accept/connect/read/write with invocation-owned completion and
   the internal scheduler through the transitional queue adapter. Empty-queue wakeup
   and unrelated ready work must progress.
   Bound open sockets, admitted connections, operations and retained payload bytes.
3. Run Raven echo under forced GC with exact buffer ranges and native-owned payloads.
   Add direct-IL invalid-range/handle tests, completion/cancellation orderings,
   stalled-peer tests and terminal-Fault teardown. Test peer reset and forced short
   sends/backpressure; tiny loopback sends alone do not establish either behavior.
4. Reuse the established stream contracts for TCP before HTTP; do not introduce a
   competing stream hierarchy merely to reproduce names in proposal examples.

S4 closes only when real Raven programs run on neoCLR with these lifecycle checks.
The full web-app goal remains open after this initial transport implementation.

## Reusable private receive backend — 2026-09-24

[src/socket_io.rs](../src/socket_io.rs) now supplies the scheduler's TCP receive source.
It adopts an already-connected host TcpStream, retaining that resource independently
of pending reads and completed results. At this checkpoint the guest injection service remained test-only; the public
client bridge described below follows it.
This is internal implementation, not a public contract or release announcement.

The owner-thread registry uses nonblocking reads into owned native buffers and copies
only the reported bytes into checked managed arrays. It retains pending destination
and callback roots, then transfers callbacks to the scheduler ready slot. A completed
byte count or internal error stays in the registry until its result is taken exactly
once; it cannot be consumed before callback delivery to the scheduler. Transferring
into a ready slot is not a claim that the guest callback has already executed.

Cancelling before a read poll settles that operation without consuming transport bytes
or closing the socket. Cancellation after terminal completion returns false. There is
no concurrent OS operation needing acknowledgement in this backend. Close is idempotent,
settles outstanding reads as Closed and drops the connection; closing an unknown/foreign
ID has no effect. Later reads on a closed ID fail. These internal choices remain
provisional until their public error/cancellation mapping is documented and tested.

Socket and operation IDs are separate private types backed by process-unique, checked
monotonic allocation. IDs never wrap/recycle and do not alias another invocation's
registry. They are not OS descriptors or a public capability/security model. A retired
operation result cannot be read twice or cancel a later operation.

Current private budgets are 64 open sockets, 64 pending-or-unconsumed operations and
64 KiB of aggregate requested receive storage. One pending receive per socket is
allowed. Terminal completion releases receive storage; consuming the delivered result
releases its operation slot. These limits exclude table/allocator overhead and retained
managed graphs, which remain under existing heap accounting. There is no public Limits
extension or frozen quota contract. Admission failure keeps a registered socket usable;
failed adoption consumes/drops the supplied host stream owner.

Comparison reuses the .NET ReceiveAsync and owned-buffer research above. Connection
reuse after cancellation and independent asynchronous operation results support the
future Task<Result<...>> bridge without requiring runtime suspension. Costs include a
host buffer, managed-array replacement copy, retained outcome records and linear scans
of the bounded operation table. No performance advantage is claimed. I/O errors are
currently collapsed into an internal Io category; public diagnostic/error mapping,
send/accept/connect, OS readiness notification and per-operation guest cancellation
remain work for the application bridge.

Focused backend checks cover cancellation followed by a real second read, result quota
retention/consumption, foreign and stale handles, transactional range/type/byte-budget
admission, open-socket budget rejection/recovery, close/EOF, and later-ready progress
past an idle socket. The existing TCP VM
fixture now runs through this backend for actual collector and TaskQueue integration.

```sh
cargo test --lib socket_io::tests
cargo test --lib socket_vm_probe
cargo test --lib scheduler::tests
cargo test --test workers
```

Callback target binding remains the VM's responsibility; the registry checks the
callback's declared Func<Void> shape. It does not enforce exclusive aliases to a
pending destination: caller writes or overlapping reads on different sockets remain
an open public buffer-usage contract. Managed-array replacement keeps array identity
but currently copies the destination; this is the same conservative ownership path
used in the earlier GC probes.

Next introduce the smallest Raven-facing addressing/socket/operation-result bridge,
with matching reference/API documentation and a runnable echo case. Preserve current
queue affinity for this bridge; generated state machines remain the execution model.

Validation: all 35 targeted backend, VM, scheduler and worker checks pass; the
combined website builds with all 523 current API pages checked.

## First public TCP client — 2026-09-24

The next bounded product is a [Raven greeting client](experiments/socket-client/README.md)
running against a host loopback server. System.Networking.Sockets now exposes
Socket.Connect(string address, int port) as Task<Result<Socket, SocketError>>,
Receive(byte[] buffer, int offset, int count) as Task<Result<int, SocketError>>,
and idempotent Close(). This is development work, not a published SDK release.
The [on-site guide](../api-docs/sockets.md) and generated reference cover the exact surface.

Compared with [.NET 10 Socket.ConnectAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.connectasync?view=net-10.0)
(retrieved 2026-09-24), the provisional factory returns a connected owner and a typed
Result, avoiding public partially initialized state. The cost is no explicit family,
pre-connect options or local bind. Retaining .NET's separate create/bind/connect
shape remains the option for the lower-level API as listener/configuration cases are
added. Numeric IPv4 strings avoid freezing a broad IP/endpoint model in this case;
they do not imply hostname lookup. No automatic DNS, IPv6 or UDP fallback occurs.

Private socket2 0.6.1 creation/nonblocking connect joins the existing receive registry.
Native pending-connect errno mapping is isolated in socket_io.rs; it is not a guest
error number or compiler rule. Connect callbacks are traced; connected resources and
operation results have distinct lifetimes. Failed connects release native sockets,
while their results occupy the operation budget until consumed. Shared quotas include
pending connects. Internal completion classes translate erased private results into
Promise<Result<...>>; their types, handles and methods remain internal. Existing
TaskQueue affinity and generated state machines are retained. No runtime suspension,
worker-per-socket or public scheduler is introduced.

The factory cannot be individually closed while still pending. Host invocation
cancellation and teardown remain the escape path; native TCP timeout is not an
application deadline contract. Read aliasing remains a caller obligation. Error
classification is deliberately small and falls back to IoFailure. Public send,
listen/accept, endpoint queries, deadlines and stream adaptation remain open.

### Convenience layers to evaluate later

**Author follow-up:** noted the future need for TcpClient and UdpClient above sockets.
**Assistant response:** record them as later convenience-layer candidates, not new
prerequisites for this slice. [.NET TcpClient](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.tcpclient?view=net-10.0)
provides a connection-oriented API and NetworkStream access;
[UdpClient](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.udpclient?view=net-10.0)
provides datagram operations (both retrieved 2026-09-24). They simplify common cases
but add ownership and overlapping surface to maintain. Evaluate TcpClient or the
proposal's TcpConnection when adapting TCP to streams; evaluate UdpClient with an
actual datagram use case. Preserve message boundaries for UDP, which is not a byte
stream. Exact names and wrapping/ownership rules remain uncommitted design choices.
The current Socket.Connect factory may move to such a convenience layer when the
explicit socket lifecycle is in place; do not mistake it for the final full design.


**Author clarification:** the immediate purpose is an HTTP-capable web-app demo
showing that the platform could become something useful. Implement only the
provisional interfaces needed along that path; do not complete every networking
layer first. The immediate sequence is receive client, send plus listen/accept and
echo, then bounded HTTP request/response handling with the necessary stream/text
pieces. General-purpose client wrappers and broad protocol coverage remain later.

The first sample also exposes an existing frontend/runtime boundary limitation:
hoisting a Result across an additional await can leave its non-defaultable carrier
uninitialized in the generated heap state machine. The sample avoids that extra
suspension for its already-completed Closed result; no initialization checks were
weakened. Track a focused hoisted-union fix if the next HTTP/echo case needs it.

Validation: 31 targeted backend, VM, scheduler and service-analysis tests pass.
The Raven client and both negative visibility checks pass with 914 allocations,
22 collections and zero live objects; all 554 API/site pages and ten website
regression checks pass. Validation is local macOS evidence.

## HTTP prototype essentials and DNS — 2026-09-24

**Author direction:** build the HTTP client over Socket directly if that is the
simplest starting point; include the essential request, header and response model,
and consider DNS now. This refines the demo sequence without requiring a complete
networking stack. The following is a plan, not additional shipped APIs.

**.NET baseline:** .NET 10's [SocketsHttpHandler connection setup](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Net.Http/src/System/Net/Http/SocketsHttpHandler/ConnectionPool/HttpConnectionPool.cs)
creates/connects a Socket and wraps it in an owning NetworkStream; TcpClient is not
a prerequisite. Start neoCLR HTTP directly on Socket with a private byte reader/writer
boundary. A stream adapter can replace that boundary when TLS or another consumer
needs it. This avoids a public wrapper prerequisite but leaves buffering, partial
transfers and ownership to implement explicitly. Do not put HTTP parsing in Socket.

Promote hostname resolution into the near-term client work. Compare
[Dns.GetHostAddressesAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.dns.gethostaddressesasync?view=net-10.0):
asynchronous address lookup with address-family selection is useful; adapt failures
to Result. Use the host resolver, respecting host configuration, rather than writing
a DNS packet client or requiring UdpClient. Keep resolution and connection separate
internally; preserve the original hostname for HTTP Host and later TLS identity.
Try supported returned addresses with a bounded overall deadline, distinguishing
resolution failure, no usable address and connection failure. The current transport
is IPv4-only: make filtering explicit, not a claim of IPv6 support. Names, address
value types and exact signatures remain provisional.

Blocking host resolution must not run on the VM scheduler thread. Evaluate a bounded
host worker or platform async resolver. A worker completion should carry owned host
data, not guest references. Limit requests and result sizes; cancellation must prevent
late delivery and release accounting correctly even when the host lookup itself
cannot be interrupted. This is necessary operation-lifetime work, not a requirement
for runtime frame suspension. Test with an injectable resolver; public DNS is not a
reliable required test dependency.

### Concrete products and order

1. **Socket exchange:** send with partial-write handling, then bind/listen/accept;
   a bounded two-sided echo proves byte flow and resource cleanup. DNS work can follow
   send without waiting for listener completion.
2. **Hostname client:** resolve a controlled hostname, select/connect an address,
   and exchange bytes. Add connection/operation deadline behavior needed for stalled
   hosts. Show lookup and connection errors distinctly.
3. **HTTP client:** an HTTP/1.1 GET and small POST against a controlled local server.
   Model method, target, headers and byte body on requests; status, headers and byte
   body on responses. Keep text decoding separate from body framing. Expose typed
   transport/protocol failures and bounded headers/body sizes.
4. **Web-app exchange:** the neoCLR listener receives a request and returns a small
   HTTP response that the client and a conventional client can both consume.

[HTTP/1.1 RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html) requires parsing a
byte stream, independent of receive boundaries. The prototype must handle split and
coalesced messages, case-insensitive field names without losing repeated values,
Host and correct request-target construction, partial writes, and body framing.
Start with explicit connection close and Content-Length for the controlled demo.
Before broader HTTP/1.1 interoperability, include chunked decoding, informational
responses and method/status-specific body rules. Reject conflicting lengths,
unsupported transfer codings, malformed headers and truncated bodies; do not silently
interpret them as supported messages. Validate outgoing fields against CR/LF injection.
A restricted demo must state its accepted subset, not claim general HTTP/1.1 support.

TLS is required for HTTPS; it remains a separate explicit milestone before testing
arbitrary HTTPS services. Reject unsupported schemes rather than downgrade. Pooling,
redirects, cookies, compression, proxies, HTTP/2, HTTP/3, general DNS record APIs and
TcpClient/UdpClient wrappers are not initial demo gates. Source retrieval: 2026-09-24.

Validation should combine fragmented in-memory protocol fixtures, injected resolver
outcomes, loopback transport tests and one compiled Raven GET/POST sample. Compare
observable requests/responses with a conventional HTTP implementation. Keep OS-specific
resolver/socket tests separate from reusable protocol tests, following the author's
CI direction. Update the generated public API reference as each API is implemented.
