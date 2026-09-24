# First socket transport contract probe

**2026-09-24 · S4 started, host-side evidence only.** Direction:
[networking proposal](../../proposals/network-api.md). Maintained candidate API:
[socket design](../../socket-api-design.md).

[probe.rs](probe.rs) implements a test-only socket lifecycle around socket2 0.6.1.
Sockets are nonblocking; a call returns Ready(Result) or Pending. The test driver
retries Pending with a five-second watchdog and a 1 ms sleep. That driver is not a
runtime scheduler, wakeup registration, Task adapter or proposed timeout API.

The probe separates Created, Bound, Listening, Connecting, Connected and Closed.
Connect failures release the socket. Accepted sockets independently own their native
resource and explicitly enable nonblocking mode. Close drops the owner; ordinary
Rust drop also releases it. It has no integer guest handles, borrowed guest buffers,
operation registry or GC integration. Pending calls retain nothing after returning.
The 64 KiB per-call transfer ceiling is an experimental bound, not a public limit or
aggregate memory quota. Only IPv4 TCP is tested; all buffers are host-owned Rust data.

## Reproduce

From the repository root on Unix with Rust and .NET 10 runtime/SDK support:

```sh
cargo test --locked --manifest-path docs/experiments/socket-api/Cargo.toml --target-dir target/experiments/socket-api
dotnet run --project docs/experiments/socket-api/dotnet/Baseline.csproj
```

Checked locally on **Darwin arm64**, Rust **1.95.0 (59807616e 2026-04-14)**,
.NET SDK **11.0.100-rc.1.26425.128**, targeting `net10.0`:
**8 Rust tests passed**, and the .NET baseline printed:

```text
PASS: endpoints, pending receive cancellation, short reads, EOF, half-close
```

The eight transport tests cover:

- UTF-8 bytes with two-byte receives, endpoint reporting, peer EOF after queued
  data, repeated send shutdown and continued reverse-direction traffic.
- Pending receive preserves the buffer; empty receive/send do not consume data.
- An idle connection does not prevent another socket from transferring data.
- Closing a pending listener/receiver makes subsequent attempts terminal; close
  repeats are harmless. This is **not Task cancellation evidence**.
- Wrong states, invalid backlog and transfer limits fail without consuming data.
- Duplicate bind is recoverable; closing the listener permits a fresh bind.
- Refused connect is a recoverable error and closes the failed socket.
- Sixteen connection/drop cycles deliver peer EOF.

The refused-connect test closes an unused ephemeral listener before connecting.
A different local process could acquire that port in between. Keeping the port
bound without listening was tried, but macOS left the connection pending until the
test watchdog, so that is not a portable refusal fixture.

The [.NET baseline](dotnet/Program.cs) checks corresponding stream/endpoint behavior
and real pending-receive cancellation. It does not claim the Rust probe already
supports operation cancellation. Empty .NET receives are deliberately not required
to complete immediately; see the documented comparison in the design.

## Limits

Unix-only probe; macOS is the only validated platform. Linux and Windows validation,
IPv6, DNS, UDP, TLS, socket options, exact OS error diagnostics, simultaneous guest
operations, guest range validation, resource admission, wakeups, GC and Fault teardown
remain open. Tests exercise short receives but do not force send backpressure or
peer resets. The implementation adds **no public neoCLR API**, production dependency
or compiler change. It does not complete S0/S4 or demonstrate a web app on neoCLR.
