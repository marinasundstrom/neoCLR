---
title: Networking
---
# Networking

**Development after Preview 9.** The current POC resolves a host name, connects a
TCP client, sends bytes and reads the reply from a separate neoCLR server. It is available with matching development
artifacts; the published Preview 9 SDK does not include these APIs.

## Address values

[IPAddress](xref:System.Networking.IPAddress) is a closed class hierarchy with
IPv4Address and IPv6Address implementations. `IPAddress.Parse(text)` returns
`Result<IPAddress, IPAddressError>` without DNS or network I/O. Addresses compare
by family and bytes, with matching hashes; two equal addresses can be separate
objects. An IPv4-mapped IPv6 address remains distinct from an IPv4 address.

IPv4 parsing accepts four decimal components and rejects abbreviated, hexadecimal,
octal and leading-zero forms. IPv6 accepts hexadecimal groups, `::` compression
and a final dotted IPv4 component. Formatting produces lowercase IPv6, compresses
the longest zero run (first on ties), and uses dotted decimal for mapped IPv4.
URI brackets and IPv6 scope identifiers are not supported in this iteration.

Unlike .NET's single IPAddress class, the families have distinct types. The current
socket backend and DNS lookup remain IPv4-only. Socket overloads accept IPAddress
values; IPv6 produces `SocketError.UnsupportedAddressFamily` before starting I/O.
String socket overloads remain available. Development callers of DNS should update
from `Sequence<string>` to `Sequence<IPAddress>` and rebuild with matching artifacts.

## Resolve, connect, exchange

[Dns.GetHostAddresses](xref:System.Networking.Dns) returns a
`Task<Result<Sequence<IPAddress>, DnsError>>`. Resolution uses the host configuration,
including local host entries. The immutable IPv4 address values can be passed to Socket.Connect.
Lookup errors and connection errors are separate outcomes. `DnsError` also uses
normal union syntax; match cases such as `DnsError.InvalidName` directly. Its former
per-case `Is*`/`Get*` helpers have been removed in the development API. `SocketError` is authored
with normal union syntax in the development library. Match its named cases directly;
per-case `Is*` properties and `Get*` methods are no longer part of that API. Rebuild
applications with the matching development SDK and runtime library.

SocketError is a basic union whose cases carry no data. Unions become more useful
when different variants carry different data. Enums remain an appropriate option
for models that only need named constants; this API does not prescribe unions
for every such model.

```raven
{{DNS_RESOLVE_SAMPLE}}
```

This function comes from the [tested client sample](/samples/socket-client/Main.rvn).
`await ...?` propagates a failed lookup; successful results provide a read-only
sequence of addresses. The complete program connects to the first address for its
controlled localhost server, sends `Hi`, and reads the echo in potentially short
transfers. It then observes EOF and closes the connection. It also verifies invalid
hostname handling and performs allocation churn while operations are pending.

[Download the sample and verifier](/samples/socket-client.zip). The verifier starts
a local echo server; it requires no external DNS or Internet service.

## A neoCLR listener

The [two-process echo sample](/samples/socket-echo.zip) runs both sides on neoCLR.
The server calls Socket.Listen with loopback and port zero, then reports GetLocalPort
so the client can connect. Accept returns a new connection through Task/Result.

```raven
{{SOCKET_SERVER_SAMPLE}}
```

Closing the listener stops new accepts; the accepted connection remains usable.
Echo loops over Receive and Send until the two-byte greeting is exchanged. The
sample checks EOF, cleanup and unrelated queued work while Accept is pending.
Listen binds synchronously. Bind conflicts return AddressInUse; another pending
accept returns Busy. Send/Receive on a listener or Accept on a connected socket
returns InvalidOperation.

## Bytes and ownership

`System.Networking.Sockets.Socket` supplies Connect, Listen, Accept, Send, Receive, GetLocalPort and Close.
Send and Receive complete with an actual byte count: callers loop to handle partial
transfers. A positive-size receive returning zero means EOF. One send and one receive
can be pending on a connection at the same time.

Send snapshots the selected bytes before returning its Task, so callers can reuse
the source array. That convenience costs a copy. A pending receive retains its
destination array; callers must not mutate or overlap that storage until completion.
Close releases the connection. Invocation teardown releases connections left open.
Read the [socket contract](/docs/sockets.html) for limits and completion ordering.

Sockets carry bytes. Character and string encoding belongs at an explicit text
boundary; future HTTP bodies must be framed by byte counts. Existing UTF-8 work is
the starting point for testing non-ASCII text split across network transfers.

## Relationship to .NET

Like .NET's Dns and Socket APIs, resolution is separate from connection and transfers
can be partial. neoCLR uses Task with typed Result errors. The lookup returns a
read-only sequence of numeric strings; an IPAddress-style value type is not required
yet. Compared with .NET's borrowed send buffers, the current snapshot policy reduces
caller lifetime obligations at an allocation and copying cost. There is no performance
advantage claimed for this prototype.

## Current limits

The client is IPv4-only. Hostnames use a narrow ASCII input guard; Unicode/IDNA,
IPv6, custom DNS records and resolver configuration are not implemented. Lookup has
a five-second deadline, four concurrent blocking host calls across the process and
bounded result sizes. A timed-out native lookup may continue until the host returns,
retaining its capacity slot. The VM remains able to dispatch other work.

A pending TCP connect has a five-second deadline and releases its native socket on
timeout. Each nonempty Send/Receive also has a separate five-second deadline.
Transfer timeout releases its buffers and returns TimedOut while leaving the socket
open. This bounds a stalled operation, not a full exchange; the limits are provisional
and cannot yet be configured. Delivery still requires scheduler progress. There is no public operation
cancellation or a shared DNS/connection deadline. The example
controls its host and uses a verifier watchdog.
Socket completion uses nonblocking polling; host lookup runs on bounded host threads.
Generated Raven state machines still implement async execution. Runtime suspension
remains future work.

## Where we’re heading

Two neoCLR programs can now exchange bytes. Connect can also snapshot a sequence of
up to 16 numeric IPv4 addresses, skip duplicates and try them in order within five
seconds. Pending attempts get at most one second while alternatives remain; the last
gets the remaining time. The echo demo checks fallback and caller-list reuse.
These provisional limits are not configurable and can reject slow connections.
A [development HTTP client](/features/web/) now exercises requests, headers,
responses, UTF-8 bodies and handlers over Socket. A bounded neoCLR HTTP responder also exchanges messages with independent clients. TLS is a separate requirement for HTTPS. TcpClient and
UdpClient may follow when a working case needs them.

The HTTP experiment provides bounded client/server exchanges; it is not a complete HTTP stack.

## Reference and participation

Browse [networking types](/docs/namespaces.html) and the [socket reference](/docs/sockets.html).
A useful contribution is a reproducible transfer, lifetime or error-handling case,
with expected behavior and the development toolchain used.

## Cancelling an operation

Development overloads on Dns.GetHostAddresses and Socket.Connect, Accept, Receive
and Send accept a CancellationToken. Existing overloads remain available. A request
that wins produces a cancelled Task after the provider has consumed native completion;
transport failures remain Result errors. A completed native result survives late
cancellation. Wait for task completion before reusing a receive buffer.

Cancelling an accept leaves the listener usable, and cancelling a transfer leaves
the connection open. A cancelled connect releases its pending connection. DNS lookup
may continue in the host after its guest Task is cancelled, retaining its bounded
host-capacity permit until it returns. Tokens are currently invocation-local.
[The API guide](/docs/sockets.html#per-operation-cancellation-development) explains
ownership and limits. The development HTTP client forwards these tokens across lookup, connect and transfers.
