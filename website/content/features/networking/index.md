# Networking

Resolve a host name, connect a TCP socket and exchange bytes. Use these APIs when
you need a protocol below [HTTP](/features/web/). Socket operations are asynchronous
and return typed results. The current transport backend supports IPv4.

## Resolve and connect

`Dns.GetHostAddresses` returns a Task containing either addresses or a `DnsError`.
Resolution uses the host configuration, including local host entries.

```raven
{{DNS_RESOLVE_SAMPLE}}
```

This function comes from the [socket client sample](/samples/socket-client/Main.rvn).
Its complete program connects to a controlled localhost server, sends `Hi`, reads
the echo and observes EOF. Lookup and connection errors have separate types.

[Download the client and verifier](/samples/socket-client.zip) ·
[Download the two-process echo example](/samples/socket-echo.zip)

## Exchange bytes

TCP is a byte stream: reads and writes can complete with fewer bytes than requested.
Loop until your protocol has transferred its complete message, and distinguish EOF
from an error. Stream operations represent work that may complete later; a returned
Task does not imply a separate thread.

Own the connection for the whole exchange and dispose it when finished. Cancellation
is cooperative; buffers must remain valid until the operation acknowledges completion
and cleanup. [Stream APIs](/docs/streams.html) describe the common contracts.

## Addresses and deadlines

`IPAddress.Parse` parses an address without DNS. Address values compare by family
and bytes. IPv4Address and IPv6Address have distinct concrete types; the current
socket backend rejects IPv6 before starting I/O. URI brackets and IPv6 scope
identifiers are not supported.

`NetworkDeadline` lets lookup, connection attempts and partial transfers share a
budget. A default deadline is expired. HTTP adds its own configurable total request
timeout, which does not remove shorter transport limits.

## Secure a connection

Development `SslStream` adds TLS over an owned byte transport, with explicit
certificate credentials and trust. Follow the [TLS guide](/docs/ssl-streams.html)
for supported hosts, setup and limitations. Each higher-level protocol still owns
its negotiation; HTTPS support does not enable PostgreSQL TLS automatically.

## Read further

- [Socket contracts and limits](/docs/sockets.html).
- [System.Networking API](xref:System.Networking).
- [HTTP applications](/features/web/) and [SQL databases](/features/sql/).

<a id="shared-budgets-development"></a>

### Development memory windows

Socket receives take `Memory<byte>` and sends take `ReadOnlyMemory<byte>`,
including cancellation and shared-deadline forms. Whole arrays convert implicitly;
explicit windows preserve offsets without copying. Receives retain the destination
until completion; sends preserve the existing snapshot behavior. See the
[migration and lifetime contracts](/docs/sockets.html#development-memory-window-migration-2026-10-19).


TLS plaintext handling now retains selected managed memory windows across awaits
and borrows slices for synchronous copy/control operations. The TLS staging buffers
remain owned arrays; public stream signatures still await coordinated migration.

### Default arguments (development)

Cancellation-aware operations now use `cancellationToken: CancellationToken = default`
in place of redundant forwarding overloads. Omitting it uses a non-cancellable token;
passing an explicit token keeps the existing cancellation behavior. Rebuild binary
consumers against matching libraries. See [parameter defaults and migration](https://github.com/marinasundstrom/neoCLR/blob/main/docs/parameter-defaults.md#runtime-api-consolidation-2026-10-30).
