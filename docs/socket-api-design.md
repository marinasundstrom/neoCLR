# Socket API: first application-driven slice

**2026-09-24 · Active implementation direction; guest API not yet available.**

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

1. Add the smallest Raven-facing addressing, Socket and error contracts needed by
   echo; implement matching runtime services and checked artifact metadata. Refresh
   the API reference from the matching bridge in the same change.
2. Integrate pending accept/connect/read/write with invocation-owned completion and
   the default TaskQueue. Empty-queue wakeup and unrelated ready work must progress.
   Bound open sockets, admitted connections, operations and retained payload bytes.
3. Run Raven echo under forced GC with exact buffer ranges and native-owned payloads.
   Add direct-IL invalid-range/handle tests, completion/cancellation orderings,
   stalled-peer tests and terminal-Fault teardown. Test peer reset and forced short
   sends/backpressure; tiny loopback sends alone do not establish either behavior.
4. Reuse the established stream contracts for TCP before HTTP; do not introduce a
   competing stream hierarchy merely to reproduce names in proposal examples.

S4 closes only when real Raven programs run on neoCLR with these lifecycle checks.
The full web-app goal remains open after this initial transport implementation.
