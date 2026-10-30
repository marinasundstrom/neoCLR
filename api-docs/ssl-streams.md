# TLS streams

<a id="tls-streams-development"></a>

`System.Networking.Security.SslStream` implements `InputStream`, `OutputStream`
and `Disposable` in System.Networking. It wraps separate async input/output
capabilities, commonly two views of the same transport. Authenticate before reading
or writing. Operations return Tasks; no `Async` suffix is used.

## Configuration and authentication

`SslClientAuthenticationOptions.Create(trustPem: byte[])` returns
`Result<SslClientAuthenticationOptions, SslError>`. The PEM bundle supplies explicit
trust anchors; the provider still validates the peer name, validity period and
certificate chain. No operating-system store or validation bypass is provided.

`SslServerAuthenticationOptions.Create(certificatePem: byte[], privateKeyPem: byte[])`
returns `Result<SslServerAuthenticationOptions, SslError>`. Supply a PEM chain and
matching unencrypted PEM private key. Inputs are copied into provider-owned
configuration and are not retained as managed arrays. Dispose configurations after
all intended sessions have been created; existing sessions retain their credentials.
The provider releases native resources on explicit disposal or collection of their
managed owners. This is not a general X509 inspection, PKCS#12, certificate-store,
revocation-policy or key-management API.

Construct `SslStream(input, output)` to own the underlying capabilities, or pass
`leaveOpen: true` to retain them. Both Close methods must be idempotent when the two
capabilities represent the same transport.

- `AuthenticateAsClient(options, targetHost[, cancellationToken])` authenticates the
  peer and uses the expected DNS name for SNI. Returns `Task<Result<unit, SslError>>`.
- `AuthenticateAsServer(options[, cancellationToken])` accepts a TLS client without
  requesting client certificates. Returns `Task<Result<unit, SslError>>`.
- `IsAuthenticated` is true only after authentication while the session remains usable.

The shared provider enables TLS 1.2/1.3. This first API does not expose protocol/cipher
selection, ALPN, remote-certificate inspection, session resumption policy or mutual
TLS. `SslError` has InvalidCertificate, InvalidPrivateKey, KeyMismatch, InvalidTrust,
Closed, AuthenticationFailed and InvalidConfiguration cases. Authentication transport
or verification failures currently collapse to AuthenticationFailed.

## Transfers and lifetime

Read and Write have the same buffer/offset/count overloads as the base stream
contracts, with `cancellationToken: CancellationToken = default`. Read returns authenticated
plaintext and zero for authenticated EOF; missing close-notify is InvalidData.
Write completes after the whole range is accepted by the underlying output.
Zero-count reads consume nothing. Invalid ranges return InvalidRange. Operations
before authentication, after failure or overlapping an active operation return
WrongAccess. Read and write are currently serialized together; full duplex is a gap,
not a different long-term concurrency contract.

Flush forwards to the underlying output. `Shutdown([cancellationToken])` returns
`Task<Result<unit, StreamError>>`, sends close-notify and flushes local writes while
leaving reads usable. Repeated successful shutdown is harmless. Writes afterwards
return WrongAccess. Close/Dispose are idempotent and abortive; they do not await
shutdown. They cancel pending wrapper work, including when leaveOpen is true.

Pre-cancellation preserves state. Cancellation or I/O failure after an operation
starts aborts the session, preventing unsafe retries of a partially consumed TLS
operation. Cancellation is represented by a cancelled Task. Underlying streams must
honor cancellation for prompt completion. Managed buffers remain rooted across awaits;
the synchronous provider pump borrows them only for the duration of each call.

## HTTP integration

`HttpSocketHandler(sslOptions)` borrows client trust configuration. Use it with
`HttpClient(handler)` for HTTPS. The default handler supports HTTP and rejects HTTPS
without explicit trust before opening a connection. `HttpRequest.IsSecure` retains
HTTPS through WithHeader and default-header copies and identifies authenticated incoming requests; the default HTTPS port is 443.

Set `HttpServerOptions.SslOptions` before Listen to enable HTTPS. Listen snapshots
the reference; retain its configuration until the server closes. The existing request
read budget includes TLS authentication; response write deadlines cover encryption
and graceful shutdown. TLS authentication reports `HttpError.Tls`, encrypted stream
errors report SecureTransport, and socket errors retain Transport. HttpClient.Timeout
and caller cancellation cover lookup, connect, authentication and all transfers.

The HTTP parser/content/JSON layer is shared with cleartext HTTP. The old native HTTPS
host remains a separate host adapter; managed SslStream does not force PostgreSQL to
adopt the same backend.

## Qualification

The local macOS consumer exchanges nested JSON using HttpClient and HttpServer,
with GC during request handling, in interpreter, default JIT and native AOT modes.
[HTTP failure checks](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/shared-tls/http-failures-validation.json) also
pass hostname rejection followed by recovery, client timeout/cancellation after
ClientHello, and the server handshake deadline in all three modes. This is a
development checkpoint, not broad protocol or performance certification.
The interpreter harness uses an optimized runtime and explicit aggregate array
budgets (1,048,576 elements and 64 MiB of charged payload); native modes use their
own bounded heap contracts. Runtime defaults are unchanged. Windows managed-stream
execution remains pending; prior Windows native HTTPS host qualification exercises
a different integration path.
