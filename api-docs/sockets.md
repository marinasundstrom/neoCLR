# TCP sockets and listeners

**Development after Preview 9.** `System.Networking.Sockets` starts with an IPv4 TCP
API. Connections can send and receive bytes; listeners accept new connections.
The two-sided POC runs separate neoCLR server and client processes on loopback.
This remains a provisional subset of a complete Socket API.

[Socket](xref:System.Networking.Sockets.Socket) has these public operations:

| Operation | Result | Behavior |
| --- | --- | --- |
| `Socket.Connect(address, port)` | `Task<Result<Socket, SocketError>>` | Connect to a numeric IPv4 address and port 1–65535. Resolve hostnames separately with Dns.GetHostAddresses. |
| `Socket.Connect(addresses, port)` | `Task<Result<Socket, SocketError>>` | Snapshot 1–16 numeric IPv4 addresses from `Sequence<string>` and try distinct addresses in order under one five-second deadline. |
| `socket.Receive(buffer, offset, count)` | `Task<Result<int, SocketError>>` | Receive up to count bytes; short reads are normal. |
| `socket.Send(buffer, offset, count)` | `Task<Result<int, SocketError>>` | Send from a snapshot of the range; loop on short writes. |
| `Socket.Listen(address, port, backlog)` | `Result<Socket, SocketError>` | Bind and listen; port zero selects an available local port. |
| `listener.Accept()` | `Task<Result<Socket, SocketError>>` | Accept one connection; close accepted sockets separately. |
| `socket.GetLocalPort()` | `Result<int, SocketError>` | Query a live listener or connection port. |
| `socket.Close()` | unit | Close both directions; repeated calls are harmless. |

The [tested echo client](/samples/socket-client/Main.rvn) uses both await and `?`:

```raven
let socket = await Socket.Connect(addresses, 19090)?
let result = await Exchange(socket)
socket.Close()
return result
```

The enclosing function returns `Task<Result<int, SocketError>>`. `?` propagates a
connection error. Here addresses is a sequence of numeric addresses returned by
[Dns.GetHostAddresses](xref:System.Networking.Dns). Exchange sends the greeting and handles short writes and reads and returns its outcome before
the caller closes the socket, so a recoverable receive error also reaches Close.
The [complete sample and verifier](/samples/socket-client.zip) run against a
small host echo server and check collection during pending connect, send and receive.

For a positive receive count, a successful zero count means the peer has finished
sending. A zero-size request returns zero without checking EOF. Close is separate
from EOF: close your connection even when the peer has finished. This slice does not
yet provide a send-half shutdown operation.

Keep the destination array unchanged while its receive is pending, including through
other aliases. Do not start overlapping operations on that storage. These are caller
obligations. One send and one receive may be pending at the same time; a second
operation in the same direction returns Busy.
Only the actual returned byte count is written; the rest of the array is preserved.

Send copies the selected source bytes before returning its Task. Reusing or collecting
that array afterwards does not alter the pending send. This prototype policy costs
an allocation/copy and shares the transfer buffer budget with receives. A successful
count means the local transport accepted that prefix, not that the peer processed it.
Loop until the intended payload has been sent; this operation is not SendAll.
Empty sends return zero; a positive request that makes no progress is an error.

Close preserves an outcome already committed by the native operation, even if the
Task callback has not run yet. Transfers still waiting for native progress settle
as Closed. Earlier successful sends are not undone by a later failure or Close.

[SocketError](xref:System.Networking.Sockets.SocketError) distinguishes invalid
addresses/ranges, Busy, Closed, LimitExceeded and common transport failures such as
ConnectionRefused and ConnectionReset. Unexpected native failures use IoFailure.
Errors are Result values, separate from runtime Faults. No public native error-number
contract or individual cancellation method is available yet.

Connections belong to their invocation. Close promptly; teardown releases anything
left open. The current backend allows 64 open-or-connecting sockets, 64 operations
including completed results awaiting delivery/consumption, and 64 KiB of aggregate
receive storage plus send snapshots. These provisional limits are implementation budgets.

Pending connects have a provisional five-second monotonic deadline from native
admission. If no outcome has been committed before expiry, the next owner poll closes
the native socket and returns TimedOut. Already committed success or failure survives
later delivery. The timeout releases socket capacity immediately; the outcome retains
its operation slot until consumed. This bounds native waiting, not arbitrary guest
work or callback delivery: the owner must continue scheduling. Host invocation
cancellation remains available. Accept has no operation deadline.
The timeout is not configurable and does not cover DNS lookup. The sequence overload
shares it across all connection attempts. While alternatives remain, a pending attempt
gets at most one second; the final address gets the remaining time. Failed attempts
release their native socket before trying another. If all fail, the last error is
returned. Overall expiry returns TimedOut. A committed success ends the search.

Each nonempty Send/Receive now has its own provisional five-second deadline from
native admission. Expiry is checked before the next I/O attempt and returns TimedOut.
It releases transfer storage and the receive destination root, without closing the
connection. No bytes are transferred by the expired operation. Already committed
results remain unchanged, and empty transfers still return zero. These bounds can
reject legitimately slow peers and are not yet configurable. The caller can start a
new operation or close the connection; HTTP closes its owned connection on this error.
Repeated short transfers get fresh bounds, so this is not a whole-request deadline.

The sequence must remain stable while Connect reads Count and its indexed values.
It is snapshotted before Connect returns; later mutation is safe. All entries are
validated before network activity. Empty input returns InvalidRange; more than 16
entries returns LimitExceeded even if duplicates would reduce the count. Invalid
numeric addresses return InvalidAddress. Valid duplicates are skipped in input order.
Custom sequence code is not bounded by the native deadline. These fixed time limits
are useful for the controlled demo but can reject slow connections; this is not a
configurable production connection policy.

Connect, Send and Receive use nonblocking sockets polled by the private scheduler. No
thread is created for each operation. Pending receive buffers and completion objects remain
GC roots until delivery; send snapshots contain owned native bytes; the Task result bridge consumes the native outcome once.
Generated async state machines and current TaskQueue affinity still apply. Explicit
custom queues must still be pumped by their owner. Runtime suspension is future work.

Compared with .NET Socket.ConnectAsync/SendAsync/ReceiveAsync, this first slice retains Task
completion, short reads, EOF and explicit ownership, but returns typed Result values
for recoverable errors. The connect factory avoids exposing an unusable half-created
connection, at the cost of no pre-connect options or local binding. It is provisional;
address value objects, IPv6, stream adaptation and individual
cancellation need their own working cases. No performance advantage is claimed.

The demo goal is a web application that receives and sends HTTP messages. Broader
Socket coverage and TcpClient/UdpClient convenience layers are evaluated only when
a working case needs them; a complete networking stack is not a prerequisite.


Current frontend limits: the sample tests empty error cases with IsClosed because
this importer does not yet admit direct value-type case tests. Keeping a Result
local across an additional await can also produce an uninitialized carrier field
in a generated heap state machine. The sample checks its already-completed Closed
result without another suspension; the general hoisted-Result case remains open.


Host-backed hostname resolution is available through Dns.GetHostAddresses.
Planned next: an HTTP request/response client using
Socket directly; TcpClient and UdpClient are not required for it. The HTTP
prototype will include headers, status and byte bodies with explicit message framing.
The first controlled demo uses plain HTTP; HTTPS needs a separate TLS implementation.
These planned additions are not available in the current Socket API.


## Listening and accepting

[The two-process echo sample](/samples/socket-echo.zip) binds to 127.0.0.1 with port
zero and reports GetLocalPort to its verifier. Listen accepts ports 0–65535 and
backlogs 1–128; the OS may cap the backlog. Bind conflicts return AddressInUse.
No address-reuse options are exposed. Listen is synchronous; Accept is asynchronous.
One accept may be pending per listener. Accept reserves capacity before waiting;
listeners, connections and pending connect/accept reservations share 64 socket slots.
Completed results retain their operation slot until consumed. Closing a listener
settles its waiting accept as Closed; an already committed accepted connection stays
open independently. Send/Receive on a listener and Accept on a connected socket
return InvalidOperation. Listener teardown closes remaining native resources.
No public accept deadline or individual cancellation API is provided yet.
