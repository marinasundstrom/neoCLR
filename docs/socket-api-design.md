# Socket API: first application-driven slice

**2026-09-24 · Development TCP echo POC; HTTP and broader Socket API remain open.**

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


## Public Send and bidirectional client exchange — 2026-09-24

Socket.Send(buffer, offset, count) returns Task<Result<int, SocketError>>. The
[client sample](experiments/socket-client/README.md) now sends Hi and receives the
host server's echo. This is progress toward HTTP request/response bytes, not a neoCLR
listener or HTTP implementation. Host-backed DNS and listener/accept remain next work.

Compare [.NET 10 Socket.SendAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.sendasync?view=net-10.0)
(retrieved 2026-09-24): retain asynchronous byte-count completion, partial sends and
explicit close; expected errors use Result. Unlike a borrowed-memory contract, this
prototype snapshots the selected range before returning. That allows immediate source
reuse and avoids retaining guest storage for a pending send, at an allocation/copy
cost. The 64 KiB aggregate native buffer budget now covers both receive buffers and
send snapshots. No throughput/allocation improvement is claimed. Borrowing/pinning
remains a later alternative if a measured case warrants the extra ownership contract.

A shared transfer registry checks socket ownership, signed ranges, byte array shape,
callback shape and quotas before sending anything. It permits one pending transfer
per direction per socket. A stalled receive does not stop its send, nor does a stalled
send stop later ready operations. WouldBlock/Interrupted remain pending observations.
Each operation completes after one successful native write, consuming only its
reported prefix; unsent snapshot bytes are released, so callers resubmit the remainder.
An empty send completes with zero on an open socket; zero native progress for nonempty
data is IoFailure. Success is local acceptance, not a peer-delivery acknowledgement.

Close/cancel before native completion settles outstanding transfers once, frees buffer
budget and preserves the connection policy of the existing private cancellation path.
Already committed outcomes are retained even if Close precedes callback execution.
Callbacks remain traced roots. Send snapshots contain only host bytes; collection can
reclaim the original array before completion. Completed results retain their operation
slots until consumed. Private result/completion names are generalized from receive to
transfer; the public Receive signature is unchanged. No runtime suspension, public
scheduler or per-operation cancellation API is added.

Validation includes a real TCP send-buffer pressure test that observes short writes,
retains a blocked send until the peer drains, and verifies that received bytes equal
exactly the sum of reported counts. Focused tests cover source mutation/collection,
shared budgets, direction-specific Busy, simultaneous send/receive, invalid ranges,
foreign handles, empty sends, cancellation/close and exactly-once outcomes. The Raven
sample checks the full source/reference/importer/library/runtime chain, including
collection while pending and early source reuse. These are local transport tests,
not a benchmark or proof of every target platform's networking behavior.

Author clarification, 2026-09-24: character/string encoding belongs in a later
text/HTTP slice. Socket continues to transfer bytes. Exercise existing UTF-8
conversion with non-ASCII text and characters split across reads; protocol framing
must count encoded bytes rather than characters. Additional encoding contracts
remain provisional until that case requires them.

Validation on 2026-09-24: 35 focused backend, VM, scheduler and service tests pass.
The compiled send/receive sample uses `await Foo()?`, reclaims all 1,380 allocated
objects over 30 collections, and passes both negative visibility checks. The matching
API snapshot and combined 555-page website build pass. Evidence is local macOS.

## Private resolver lifetime checkpoint — 2026-09-24

The next hostname-client slice now has a private host-resolution source in the
scheduler. It is not yet a public Dns API or a compiled Raven hostname sample.
Keep the public Task/Result and address/error shapes for the following bridge slice.
No Socket.Connect behavior changes: its address remains numeric IPv4.

Compare [.NET 10 Dns.GetHostAddressesAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.dns.gethostaddressesasync?view=net-10.0):
retain asynchronous address-list lookup, separate from connecting. The prototype
uses the host resolver via Rust's [ToSocketAddrs](https://doc.rust-lang.org/std/net/trait.ToSocketAddrs.html),
which may block the calling thread (sources retrieved 2026-09-24). Blocking work
runs on bounded host threads, not the VM owner and not an isolated guest Task.Run
worker. A platform asynchronous resolver or reusable host pool remains an alternative;
the current implementation pays thread-start cost for a small controlled demo.

Only owned hostname text and IPv4 address results cross threads. Guest callbacks
stay in the invocation's traced registry and then the existing scheduler ready slot.
A worker sends a durable outcome before signalling the coalesced wake. Resolver
completion rotates alongside socket and worker sources; generated state machines
remain unchanged, and this adds no public scheduler or runtime suspension contract.

Provisional limits are four simultaneous blocking lookups across the process, eight
operations per invocation including delivered-but-unconsumed outcomes, 253 ASCII
hostname bytes, and 16 unique IPv4 results. The host adapter inspects at most 256
returned endpoints; exceeding the bounds returns a limit error rather than silently
truncating the answer. Hostname admission excludes whitespace, ports, NUL and Unicode;
it is a narrow input guard, not a full DNS-name validator or IDNA implementation.
IPv6 results are filtered explicitly, with a distinct no-usable-address outcome.
Native resolver failure is currently generic; platform-specific error mapping is open.
The host's resolver allocations and its initial address-list allocation are outside
these result-retention budgets; they are not a whole-process memory bound.

Every private submission supplies a deadline. If the owner has not observed a
completion before expiry, timeout wins; cancellation similarly commits one outcome.
Results are consumed once after notification. No guest callback runs on a host thread.
Dropping an invocation drops its receivers and guest roots without joining a blocked
host call. Crucially, that call retains a process-wide permit until it exits: repeated
cancellation/teardown cannot evade the host concurrency bound. Late results cannot
reach a destroyed invocation. An indefinitely stuck host resolver may therefore
exhaust capacity; a deadline cannot force libc to stop. This is an explicit availability
tradeoff, not a claim of interruptible native DNS or an overall connection deadline.

Focused tests use injected blocked, failed and panicking resolvers for lifecycle and
accounting, plus numeric/localhost host lookups without public DNS. The scheduler
integration test collects captured callback graphs before completion and while staged,
then checks their release. Public API, API-reference entries, compiled hostname
exchange and end-to-end connection deadlines remain the next integration gate.

Validation on 2026-09-24: seven resolver tests, eight scheduler tests and five
existing socket/VM integration tests pass locally (20 total). The combined website
build checks 555 pages and the unchanged API snapshot. No public API or compiler
contract changes in this checkpoint; no SDK artifacts are regenerated.


## Public hostname lookup and networking POC — 2026-09-24

System.Networking.Dns.GetHostAddresses(hostName) now returns
Task<Result<Sequence<string>, DnsError>>. Numeric IPv4 strings work with the existing
Socket.Connect contract; lookup does not connect, pick an address or replace the
original hostname needed for HTTP/TLS. The five-second lookup deadline, host-work
bounds and cancellation/late-result lifetime rules from the private checkpoint apply.
No public cancellation overload is added; Cancelled represents the private backend
outcome and is not currently requested by an application. No overall connect deadline
or automatic multi-address fallback is claimed by this slice.

Compared with the .NET address array above, a read-only Sequence states the consumer
capability needed by this POC without committing to an IPAddress-style value object.
Strings defer address equality/family/normalization APIs, at the cost of no static
address validation at subsequent call sites. IPv4-only output is explicit. The
underlying sequence is an ordinary managed array view, not an immutable collection.
The native result carries owned strings; generated library code copies them into a
managed array on the VM owner before completing the Promise. Worker threads never
allocate in the guest heap. Native service bindings distinguish NameResolution from
SocketIo and IsolatedWorkers; only submission additionally requires TaskDispatch.

The tested product is the existing socket client extended to resolve localhost,
then exchange Hi with a controlled host server. Its first-address selection is a
local fixture policy, not a general robust connection algorithm. Before broader HTTP
client use, define bounded address fallback and an overall connection deadline.
The Networking feature page and homepage box are added for this working POC; Web
gets its own page and box when the HTTP POC is executable, per author direction.

Two integration cases remain open. Task<Result<string[], DnsError>> triggered a Raven
MetadataLoadContext mismatch for System.String[] during code generation; using a
Sequence avoids that path but is not a compiler fix. A nested OnCompleted callback
capturing a locally created task faulted with a null capture at Task.GetResult. The
sample separates callback registration into a named function; the original nested
capture case needs an independent .NET-target reproduction before assigning the bug
to Raven emission or neoCLR import. These are deferred general-fix candidates, not
neoCLR semantic changes or reasons to weaken runtime reference checks.

Validation on 2026-09-24: eight resolver tests, nine runtime-service tests and eight
scheduler tests pass (25 total). The compiled hostname/echo sample reclaims all
1,428 allocations across 31 collections, with zero live objects; all three negative
visibility checks pass. Ten website tests and the combined 572-page API/site build
pass. Networking was inspected in the local browser. This is local macOS evidence,
not cross-platform networking certification or a runtime release.


## Address value objects — future direction, 2026-09-24

The author proposes introducing IPAddress when the networking model is ready, and
possibly HostEntry, continuing the platform's value-object direction. This does not
replace the immediate listener/accept slice or require changing the POC's string
addresses now. Neither type is implemented or a prerequisite for the first HTTP demo.

Compare [.NET IPAddress](https://learn.microsoft.com/en-us/dotnet/api/system.net.ipaddress?view=net-10.0)
and [IPHostEntry](https://learn.microsoft.com/en-us/dotnet/api/system.net.iphostentry?view=net-10.0)
(primary sources reviewed 2026-09-24). IPAddress models an address; IPHostEntry groups
host information and addresses. Borrow that separation without committing to the
complete .NET surface or its mutability. Value-object semantics do not decide whether
the eventual neoCLR representation is a class or a struct.

Assistant proposal for evaluation: make IPAddress immutable, with validated parsing
returning Result, equality/hash based on the address representation, and deterministic
formatting. Keep hostnames and ports separate from an IP address. Define IPv4/IPv6
family, mapped-address and scope handling deliberately when those cases are needed;
do not infer identity from display text alone. Richer validation and stable equality
would reduce repeated parsing and ambiguous string comparisons, at the cost of new
contracts and migration from the current DNS/socket signatures. Decide string
convenience overloads from actual call sites rather than requiring typed addresses
everywhere in the platform.

HostEntry remains optional: introduce it if a resolver case needs more than an address
sequence. Select hostname/alias fields only when the host resolver can supply them;
keep the original requested hostname distinct from any reported canonical name.
A lookup result is a snapshot, not permanent host identity or peer authentication.
Its equality and collection semantics need a concrete use case before being fixed.


## Listener/accept and two-process echo — 2026-09-24

Socket.Listen(address, port, backlog) -> Result<Socket, SocketError> binds a numeric
IPv4 address and listens synchronously. Port zero requests an OS-selected port,
reported by GetLocalPort() -> Result<int, SocketError>. Accept() returns
Task<Result<Socket, SocketError>>. The [two-process POC](experiments/socket-echo/README.md)
now runs a neoCLR listener and a separate neoCLR hostname client.

Compare [.NET Socket.Listen](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.listen?view=net-10.0)
and [Socket.AcceptAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.acceptasync?view=net-10.0)
(primary sources reviewed 2026-09-24). Keep nonblocking accept and a separately owned
accepted socket. Combine bind/listen in a factory to avoid exposing partly initialized
resources; this removes pre-bind configuration flexibility. Typed Result errors
include AddressInUse and InvalidOperation. A single Socket class has connected and
listening roles; wrong-role calls are checked at runtime. Separate capability types
remain an alternative if real consumers justify the larger surface. No throughput
or portability advantage is claimed.

Listen admits ports 0–65535 and requested backlogs 1–128 (subject to OS limits),
without reuse-address options. One accept may be pending per listener. Listeners,
connections and pending connect/accept reservations share 64 native socket slots;
a pending accept reserves its future connection slot before touching the native queue.
Accepted outcomes retain ownership until the application consumes them or teardown
releases them. Operation slots are shared with transfers and connects. Nonblocking
accept is polled alongside transfers under the existing scheduler policy. WouldBlock
and Interrupted preserve the pending operation; callbacks remain traced guest roots.

Closing a listener settles its uncommitted accept once as Closed, frees its reservation
and closes the listening resource. It does not revoke an already accepted connection,
even if the completion callback has not run yet. No accept deadline or public
per-operation cancellation is added. Host cancellation and invocation teardown remain
available. The sample's watchdog does not substitute for a future connection policy.
GetLocalPort is a minimal fixture/application need; IPAddress and HostEntry remain
future direction rather than prerequisites for this POC.

Validation: 18 backend tests and nine scheduler tests include real loopback accept,
address-in-use, invalid ranges/roles, same-listener Busy, shared quota reservations,
close before/after acceptance, independent ownership, teardown and callback GC.
Nine runtime-service tests include Accept's SocketIo + TaskDispatch requirement.
The compiled two-process fixture completes with zero live objects: server 104
allocations/three collections, client 1,428 allocations/31 collections. This is
local macOS evidence. HTTP framing, overall connect deadlines and fallback remain open.


## Pending connect deadline — 2026-09-24

Pending numeric-address connects now use a fixed five-second monotonic deadline
from native admission. The owner checks expiry before observing transport completion.
An outcome committed earlier survives expiry and delayed callback delivery. On expiry,
the registry drops the native stream, releases socket capacity and queues the existing
TimedOut result exactly once. The operation record remains bounded and retained until
result consumption. Accept has no deadline; shared completion storage must not make
listeners expire. Send/Receive remain unchanged.

This is a provisional POC policy, not a general timeout API. No new public signature,
compiler service, thread or timer queue is introduced. The existing private scheduler
poll observes deadlines; arbitrary guest execution or an unpumped custom queue can
still delay delivery. Runtime suspension remains future work.

Comparison checked 2026-09-24: [.NET Socket.ConnectAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.sockets.socket.connectasync?view=net-10.0)
has address-list and cancellation-token overloads. The current neoCLR factory uses
one address and Result errors, so it cannot express that full policy. A fixed bound
is useful for the controlled demo but can reject slow legitimate connections and
cannot be tuned by applications. This is a limitation, not an improvement over .NET.
The existing nonblocking/owned-resource research above continues to apply.

**Next policy to implement and validate:** retain DNS host order, attempt a bounded
number of distinct addresses sequentially, and use one absolute deadline across the
operation rather than granting every retry a fresh five seconds. Preserve the original
hostname for HTTP Host (and later TLS identity). Keep this policy in a private client
adapter until its use establishes a public contract; IPAddress/HostEntry and parallel
IPv4/IPv6 racing are not prerequisites. DNS currently has a separate five-second bound;
combining lookup and connection into one deadline remains open. This paragraph is a
plan, not implemented fallback.

Deterministic backend tests inject owner time with real native streams: expiry wins
over unobserved success at the exact boundary, a committed success/failure survives
expiry, timed-out transport closes, results are single-use, socket/operation quotas
recover and listeners remain pending. They do not depend on public unreachable IPs
or wall-clock sleeps. Existing VM/scheduler and echo checks exercise the unchanged
Task/Result bridge and GC ownership.

Validation: 44 focused Rust checks pass (21 socket backend, nine scheduler, nine
runtime-service and five VM socket checks). The API snapshot validates, the combined
website builds with 955 RavenDoc pages checked, and all 15 website tests pass. No
compiler/library signature changed, so the existing matching reference assembly is
retained and its XML snapshot refreshed. No SDK or website publication was performed.


## Address-sequence connection POC — 2026-09-24

`Socket.Connect(addresses: Sequence<string>, port: int)` now snapshots 1–16 numeric
IPv4 entries synchronously, validates the complete input and skips duplicate
endpoints in input order. Empty input returns InvalidRange; an oversized sequence
returns LimitExceeded; malformed entries return InvalidAddress. A sequence must be
stable during its synchronous read; subsequent changes do not affect the operation.
Only parsed native IPv4 endpoints are retained, not guest strings or the source list.

One operation/socket reservation and one absolute five-second monotonic deadline
cover all attempts after native admission. While alternatives remain, each pending
attempt gets at most one second; the last may use the remaining budget. Refusal or
another native failure advances immediately on observation. The previous stream is
dropped before opening its replacement. Exhaustion returns the last attempt error;
overall expiry returns TimedOut without starting another address. Already committed
success wins and releases untried candidates. Accept and existing byte transfers
are unchanged. Owner scheduling is still required for progress and deadline delivery.

This refines the previous plan to hide fallback entirely in a client adapter: the
working case directly consumes the sequence returned by Dns.GetHostAddresses, so a
small Socket overload is useful before HTTP exists. Timing, retries and operation
handles remain private. No public scheduler, endpoint object or retry-policy type is
introduced. The runtime adapter remains compatible with future suspension ownership;
it does not require the public Task to expose its generated state machine.

Comparison reuses the .NET 10 ConnectAsync address-list/cancellation contract linked
above. Sequential address attempts and one connection result are familiar behavior;
neoCLR uses its read-only Sequence contract and typed Result errors. The provisional
fixed total/per-address bounds favor a controlled demo, can abandon a slow usable
address, and cannot be tuned or cancelled individually. Parallel address racing may
reduce latency but costs concurrent connections, cleanup races and more quota policy;
it remains unselected. The existing Tokio/nonblocking ownership comparison still
applies. No portability or performance improvement over .NET is claimed.

DNS remains a separate five-second operation. This completes the bounded
**connection-attempt** fallback case, not a single DNS/TCP/HTTP request deadline.
HTTP must define request/header/body limits and phase deadlines explicitly. IPv6,
TLS authentication, richer diagnostic errors and IPAddress/HostEntry stay separate.

The echo client deliberately prepends 127.0.0.2 to the localhost results, then
reuses its mutable source list after Connect returns. The verifier's listener is on
127.0.0.1. Success demonstrates fallback and snapshot ownership under forced GC.
Controlled-clock backend checks cover per-address expiry, no extension of the total
deadline, total expiry before another native connect, full preflight validation,
duplicate removal and failed-attempt cleanup.

Target integration adds a checked private SocketConnectAddresses array/callback
service and a Sequence overload in the compiler reference/importer. Runtime Contract
configuration, compiler semantics and state-machine emission are unchanged. Refresh
the reference, bridge, generated library and runtime together; this is not a new
compiler capability or an SDK release.


Validation: all 43 focused Rust checks pass (25 socket backend, nine scheduler,
nine runtime-service). The regenerated library snapshot matches its sources; the
combined website checks 955 RavenDoc pages and all 15 website tests pass. The
compiled separate-process echo passes on local macOS: server 104 allocations,
three collections, zero live objects; client 1,856 allocations, 43 collections,
zero live objects. These are lifecycle checks, not throughput measurements or
cross-platform release validation. No compiler semantics changed or SDK was released.


## Uri and address-family values — author direction, 2026-09-24

The author requests future Uri and IPAddress value objects, suggesting IPAddress as
an IPv4Address/IPv6Address union. This is a candidate, not a settled representation
or an instruction to interrupt HttpClient integration. Compare .NET 10's
[IPAddress](https://learn.microsoft.com/en-us/dotnet/api/system.net.ipaddress?view=net-10.0)
single-class model with explicit address-family variants. The union could make family
matching and family-specific validation clearer, at the cost of another wrapper and
exhaustive-case evolution. Decide immutable byte storage, equality/hash, mapped IPv4,
IPv6 scope IDs, parsing and formatting together. Host names and ports remain separate;
adding IPv6 values does not itself implement IPv6 sockets.

Compare [.NET Uri](https://learn.microsoft.com/en-us/dotnet/api/system.uri?view=net-10.0)
for parsing, components, comparison and relative/base resolution. A neoCLR value should
separate syntactic validation from scheme-specific HTTP policy and define normalization,
escaping and original-text preservation explicitly. Avoid imposing a typed value on
all string-taking APIs; choose overloads from real cases, as with Path. Primary sources
reviewed 2026-09-24. The current limited HTTP URL parser is not a general Uri contract.
