---
title: Networking
---
# Networking

**Development after Preview 9.** The current POC resolves a host name, connects a
TCP client, sends bytes and reads the reply from a separate neoCLR server. It is available with matching development
artifacts; the published Preview 9 SDK does not include these APIs.

## Resolve, connect, exchange

[Dns.GetHostAddresses](xref:System.Networking.Dns) returns a
`Task<Result<Sequence<string>, DnsError>>`. Resolution uses the host configuration,
including local host entries. The numeric IPv4 strings can be passed to Socket.Connect.
Lookup errors and connection errors are separate outcomes.

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
timeout. Delivery still requires scheduler progress. There is no public operation
cancellation, automatic address fallback or shared DNS/connection deadline. The example
controls its host and uses a verifier watchdog.
Socket completion uses nonblocking polling; host lookup runs on bounded host threads.
Generated Raven state machines still implement async execution. Runtime suspension
remains future work.

## Where we’re heading

Two neoCLR programs can now exchange bytes. Next, address fallback under one shared
connection deadline will prepare the HTTP case. A small HTTP client and server will
exercise requests, headers, responses and bodies, using
Socket directly where useful. TLS is a separate requirement for HTTPS. TcpClient and
UdpClient may follow when a working case needs them.

Web will have a separate feature page and homepage entry when its HTTP POC runs.
The networking POC does not imply that HTTP or a web server is implemented.

## Reference and participation

Browse [networking types](/docs/namespaces.html) and the [socket reference](/docs/sockets.html).
A useful contribution is a reproducible transfer, lifetime or error-handling case,
with expected behavior and the development toolchain used.
