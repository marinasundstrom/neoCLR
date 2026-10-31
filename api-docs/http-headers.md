# HTTP header collections (development)

Incoming `HttpServerRequest.Headers` and `HttpClientResponse.Headers` expose
`HttpHeaderCollection`, a read-only `Sequence<HttpHeader>` interface. Outgoing
`HttpClientRequest.Headers` and `HttpServerResponse.Headers` expose
`MutableHttpHeaderCollection`, which extends that read contract. The implementations
are internal and can specialize storage for each role.
`HttpHeaderNames` exposes the existing canonical lowercase name catalog, including
`ContentType`, `ContentLength`, `Host`, `Authorization`, `Cookie`, `SetCookie` and
`CacheControl`. Custom names remain ordinary strings.

```raven
let values = response.Headers[HttpHeaderNames.ContentType]
let contentType = response.Headers.GetContentType()?
let hasCredentials = request.Headers.Contains(HttpHeaderNames.Authorization)
```

Use these expressions inside functions returning the appropriate `Result` when
propagating `HttpError`. The header tests and WebDemo cache middleware compile and
exercise this API.

| Member | Contract |
| --- | --- |
| `static Empty: HttpHeaderCollection` | Reusable read-only empty collection, cached within the runtime invocation. |
| `static Snapshot(headers: Sequence<HttpHeader>): HttpHeaderCollection` | Shallow snapshot in entry order; subsequent source mutations are invisible. Does not validate outgoing framing or values. |
| `Count: int` | Number of field entries, including repeated names. |
| `self[index: int]: HttpHeader` | Positional entry; invalid bounds are terminal. Parsed entries are materialized on demand; reference identity is not a contract. |
| `self[name: string]: Sequence<string>` | Independent snapshot of matching raw values in order. Invalid or missing names give an empty sequence. No comma splitting or joining. |
| `Contains(name: string): bool` | Case-insensitive presence; invalid names return false. No matching list or entry materialization. |
| `GetSingle(name: string): Result<Optional<string>, HttpError>` | `None` when absent, `Some("")` for a present empty value, `Protocol` for an invalid name or more than one field. No matching list or entry materialization. |
| `GetContentType(): Result<Optional<MediaType>, HttpError>` | Parses a single Content-Type, returns None if absent, Protocol if malformed or duplicated. No content read or negotiation. |
| `GetIterator(): Iterator<HttpHeader>` | Entry order, including repeated names. Materializes parsed entries when Current is requested. |

Names use ASCII case-insensitive comparison and HTTP token grammar. Repeated
`Set-Cookie` values remain separate, including commas within an Expires attribute.
The string indexer is a multiple-value API even for fields normally used once.
Use `GetSingle` or `GetContentType` to enforce those expectations.

`MutableHttpHeaderCollection.Create()` creates an independent empty mutable
collection. `Set(name, value)` replaces all matching fields; `Append(name, value)`
adds a separate field; `Remove(name)` removes all matching fields and returns
whether any existed. Each returns `Result<..., HttpError>` and validates before
mutation. Client-owned collections reject transport/content-controlled fields,
matching `WithHeader`; server-owned collections validate names and values, with
framing checked when sent. Concurrent mutation is unsupported.

## Ownership and allocation

`Snapshot` copies the entry sequence. Message constructors retain supplied storage;
this does not imply deep immutability or thread safety. Incoming views do not expose
the mutable interface, even when supplied mutable storage. Outgoing views remain
attached across `SetHeader`, direct header mutation and `Respond`. Request
`WithHeader` still derives a separate message. `Empty` is never mutable.

Incoming parsers store alternating name/value strings in a private buffer. Their
validation and limits are unchanged. Named lookup and content-type inspection read
that buffer directly; positional entry access projects `HttpHeader` only as needed.
Names and values are still decoded strings: this is not deferred decoding or a
borrowed wire-buffer lifetime contract. No arbitrary input is interned.

Use `HttpHeaderCollection.Empty` when no fields are needed. It exposes no mutation;
setting a response header derives new message storage and leaves Empty unchanged.

The public view is created lazily and reused for the message. Lookup
compares borrowed UTF-8 views instead of encoding temporary byte arrays. The string
indexer still allocates a result snapshot; `Contains` and `GetSingle` avoid that list.
`GetContentType` on the collection parses on each call; message-level
`GetContentType` retains its existing parsed-value cache.

This changes the declared return type of `Headers`; rebuild consumers with the
matching System.Web library. Sequence-based consumers remain source-compatible.
`GetHeaderValues` remains supported with its existing snapshot behavior. Use `HttpServerResponse.SetHeader` or the mutable collection for response fields,
and `HttpClientRequest.WithHeader` to derive a client request.

## .NET comparison

ASP.NET Core's [IHeaderDictionary](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.iheaderdictionary)
returns multiple values and an empty result for a missing name. neoCLR retains that
lookup shape, with `Sequence<string>` snapshots and `Result<Optional<string>, HttpError>`
for checked single-value access instead of an out parameter. Incoming and outgoing contracts deliberately separate read and mutation access.
This prevents incoming-message APIs from implying outgoing configuration semantics. Compact storage saves eager entry objects
but positional iteration now materializes them. Neither a dictionary nor a parsed
header cache is imposed on every message; those tradeoffs can evolve behind this API.

A dedicated `HttpHeaderValues` type is a follow-up candidate, not implemented here.
The current named indexer returns `Sequence<string>`. Such a value view could
specialize zero/one/many values while keeping repeated fields distinct from
comma-separated syntax within a field. The author raised this during the collection
slice; it does not change the snapshot contract implemented here.
