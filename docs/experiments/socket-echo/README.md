# Two neoCLR processes exchange a TCP echo

**Development after Preview 9.** Server.rvn creates a loopback listener using
Socket.Listen("127.0.0.1", 0, 8), reports its OS-selected port, and awaits Accept.
The verifier compiles the existing hostname client with that port, then runs it in
a separate neoCLR process. Both sides transfer Hi using short-read/write loops.
The server closes the listener immediately after accepting; the accepted connection
continues independently. Closing the connection gives the client EOF.

```sh
python3 docs/experiments/socket-echo/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner /path/to/measure_async
```

The download includes Client.rvn/Client.rvnproj copied from the tested hostname
client; in the repository the verifier reads the adjacent socket-client source.
No fixed port, public DNS server or host echo implementation is used. Watchdogs
bound compiler and process waits. The verifier checks exact output, native EOF,
normal process termination and zero retained managed objects. The client uses a
256-object heap; the server uses the CLI's default limits with --gc-stats.

Listen binds synchronously and returns Result. Accept returns Task<Result<Socket,
SocketError>>. A pending accept permits unrelated TaskQueue work. Expected output
includes Other work runs while accept is pending, Echoed Hi and Server closed on
the server, and the existing hostname/client echo output on the client.

This is a one-connection, two-byte fixture, not a general echo daemon or HTTP server.
No accept timeout, per-operation cancellation, address reuse option, peer endpoint
API or send-half shutdown is exposed. Host invocation cancellation and teardown
remain the interruption boundary. The verifier's process watchdog is not a public
Socket deadline. IPAddress and HostEntry remain later value-object work.

Validation on 2026-09-24: the server reclaims 104 objects in three collections; the
client reclaims 1,428 in 31 collections, both with zero live objects. Native tests
cover bind conflicts, wrong-role operations, reserved capacity, pending/committed
close ordering, independent accepted ownership and teardown. A scheduler test
collects a captured accept callback while pending and staged for delivery. These
are local macOS results, not a release or evidence for every host platform.
