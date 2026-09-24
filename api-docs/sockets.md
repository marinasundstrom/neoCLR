# TCP socket clients

**Development after Preview 9.** `System.Networking.Sockets` starts with an IPv4 TCP
client. It can connect, receive bytes and close. Send and listener operations are the
next steps toward a two-sided echo application; this is not a complete Socket API.

[Socket](xref:System.Networking.Sockets.Socket) has three public operations:

| Operation | Result | Behavior |
| --- | --- | --- |
| `Socket.Connect(address, port)` | `Task<Result<Socket, SocketError>>` | Connect to a numeric IPv4 address and port 1–65535. No hostname resolution. |
| `socket.Receive(buffer, offset, count)` | `Task<Result<int, SocketError>>` | Receive up to count bytes; short reads are normal. |
| `socket.Close()` | unit | Close both directions; repeated calls are harmless. |

The [tested greeting client](/samples/socket-client/Main.rvn) uses both await and `?`:

```raven
let socket = (await Socket.Connect("127.0.0.1", 19090))?
let result = await ReadGreeting(socket)
socket.Close()
return result
```

The enclosing function returns `Task<Result<int, SocketError>>`. `?` propagates a
connection error. ReadGreeting handles short reads and returns its outcome before
the caller closes the socket, so a recoverable receive error also reaches Close.
The [complete sample and verifier](/samples/socket-client.zip) run against a
small host greeting server and check collection while a receive is pending.

For a positive requested count, a successful zero count means the peer has finished
sending. A zero-size request returns zero without checking EOF. Close is separate
from EOF: close your connection even when the peer has finished. This slice does not
yet provide a send-half shutdown operation.

Keep the destination array unchanged while its receive is pending, including through
other aliases. Do not start overlapping operations on that storage. These are caller
obligations; only the one-pending-receive-per-connection restriction is enforced.
Only the actual returned byte count is written; the rest of the array is preserved.

[SocketError](xref:System.Networking.Sockets.SocketError) distinguishes invalid
addresses/ranges, Busy, Closed, LimitExceeded and common transport failures such as
ConnectionRefused and ConnectionReset. Unexpected native failures use IoFailure.
Errors are Result values, separate from runtime Faults. No public native error-number
contract or individual cancellation method is available yet.

Connections belong to their invocation. Close promptly; teardown releases anything
left open. The current backend allows 64 open-or-connecting sockets, 64 operations
including completed results awaiting delivery/consumption, and 64 KiB of aggregate
receive storage. These provisional limits are implementation budgets. There is no
configured connect deadline; host invocation cancellation remains available.

Connect and Receive use nonblocking sockets polled by the private scheduler. No
thread is created for each operation. Pending buffers and completion objects remain
GC roots until delivery; the Task result bridge consumes the native outcome once.
Generated async state machines and current TaskQueue affinity still apply. Explicit
custom queues must still be pumped by their owner. Runtime suspension is future work.

Compared with .NET Socket.ConnectAsync/ReceiveAsync, this first slice retains Task
completion, short reads, EOF and explicit ownership, but returns typed Result values
for recoverable errors. The connect factory avoids exposing an unusable half-created
connection, at the cost of no pre-connect options or local binding. It is provisional;
address value objects, DNS, IPv6, listener/accept, send, stream adaptation and individual
cancellation need their own working cases. No performance advantage is claimed.

The demo goal is a web application that receives and sends HTTP messages. Broader
Socket coverage and TcpClient/UdpClient convenience layers are evaluated only when
a working case needs them; a complete networking stack is not a prerequisite.


Current frontend limits: the sample tests empty error cases with IsClosed because
this importer does not yet admit direct value-type case tests. Keeping a Result
local across an additional await can also produce an uninitialized carrier field
in a generated heap state machine. The sample checks its already-completed Closed
result without another suspension; the general hoisted-Result case remains open.
