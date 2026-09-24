# TCP socket clients

**Development after Preview 9.** `System.Networking.Sockets` starts with an IPv4 TCP
client. It can connect, send and receive bytes, and close. Listener operations are
the next server-side step toward a two-sided neoCLR echo application; this is not a complete Socket API.

[Socket](xref:System.Networking.Sockets.Socket) has four public operations:

| Operation | Result | Behavior |
| --- | --- | --- |
| `Socket.Connect(address, port)` | `Task<Result<Socket, SocketError>>` | Connect to a numeric IPv4 address and port 1–65535. No hostname resolution. |
| `socket.Receive(buffer, offset, count)` | `Task<Result<int, SocketError>>` | Receive up to count bytes; short reads are normal. |
| `socket.Send(buffer, offset, count)` | `Task<Result<int, SocketError>>` | Send from a snapshot of the range; loop on short writes. |
| `socket.Close()` | unit | Close both directions; repeated calls are harmless. |

The [tested echo client](/samples/socket-client/Main.rvn) uses both await and `?`:

```raven
let socket = await Socket.Connect("127.0.0.1", 19090)?
let result = await Exchange(socket)
socket.Close()
return result
```

The enclosing function returns `Task<Result<int, SocketError>>`. `?` propagates a
connection error. Exchange sends the greeting and handles short writes and reads and returns its outcome before
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
receive storage plus send snapshots. These provisional limits are implementation budgets. There is no
configured connect deadline; host invocation cancellation remains available.

Connect, Send and Receive use nonblocking sockets polled by the private scheduler. No
thread is created for each operation. Pending receive buffers and completion objects remain
GC roots until delivery; send snapshots contain owned native bytes; the Task result bridge consumes the native outcome once.
Generated async state machines and current TaskQueue affinity still apply. Explicit
custom queues must still be pumped by their owner. Runtime suspension is future work.

Compared with .NET Socket.ConnectAsync/SendAsync/ReceiveAsync, this first slice retains Task
completion, short reads, EOF and explicit ownership, but returns typed Result values
for recoverable errors. The connect factory avoids exposing an unusable half-created
connection, at the cost of no pre-connect options or local binding. It is provisional;
address value objects, DNS, IPv6, listener/accept, stream adaptation and individual
cancellation need their own working cases. No performance advantage is claimed.

The demo goal is a web application that receives and sends HTTP messages. Broader
Socket coverage and TcpClient/UdpClient convenience layers are evaluated only when
a working case needs them; a complete networking stack is not a prerequisite.


Current frontend limits: the sample tests empty error cases with IsClosed because
this importer does not yet admit direct value-type case tests. Keeping a Result
local across an additional await can also produce an uninitialized carrier field
in a generated heap state machine. The sample checks its already-completed Closed
result without another suspension; the general hoisted-Result case remains open.


Planned next: host-backed hostname resolution and listener/accept
and a bounded HTTP request/response client using Socket directly. DNS belongs in
near-term client work; TcpClient and UdpClient are not required for it. The HTTP
prototype will include headers, status and byte bodies with explicit message framing.
The first controlled demo uses plain HTTP; HTTPS needs a separate TLS implementation.
None of these planned additions is available in the current Socket API.
