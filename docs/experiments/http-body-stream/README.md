# Headers-first HTTP body streams

Development, 2026-10-13. The same managed reader handles client response bodies and
server request bodies over HttpConnection. It owns one 256-byte transport block,
including read-ahead received with headers. It performs no further receive until a
consumer reads. Fixed-length, chunked and client EOF framing enforce termination
and cumulative quotas without accumulating the decoded body.

## Public contract

- `HttpClient.Send(request, HttpCompletionOption.ResponseHeadersRead[, token])`
  returns at final headers. Default overloads retain ResponseContentRead buffering.
- `HttpServer.AcceptHeaders([token])` transfers an accepted context at valid headers.
  Existing Accept/ServeOne remain buffered. This explicit opt-in preserves working
  JSON consumers until their current buffered content helpers can accept streams.
- `HttpContent.TryGetStream()` returns the one-shot InputStream for streamed content;
  TryGetBytes is its buffered counterpart. Neither causes materialization. Length
  is -1 for unknown incoming length, otherwise the declared count, not remaining.
- `HttpResponse.Dispose` closes owned content and unread client connections. Server
  context completion/disposal closes incoming content. Body Close aborts the exchange.
- Read accepts caller cancellation, short reads and zero-count reads. It rejects
  overlapping reads as WrongAccess. Malformed/truncated framing becomes InvalidData,
  quotas LimitExceeded, and transport/deadline errors IoFailure. A cancellation that
  interrupts an active read closes the connection; pre-cancellation does not.

See the [API guide](../../../api-docs/http-body-streams.md) for lifetimes and bounds.
There is no runtime ABI, metadata format, collector or platform-provider change.
Known-length streaming output was committed separately; unknown-length output is
still rejected before writes, including forwarding chunked incoming content.

## Comparison and tradeoffs

.NET [HttpCompletionOption](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcompletionoption?view=net-10.0)
provides the client completion model: timeout/cancellation for Send ends at the
chosen completion point, and later reads require their own policy. Custom handlers
can still return buffered content. The explicit server headers entry point keeps
the existing callback examples working; ASP.NET Core's
[request body stream](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/middleware/request-response?view=aspnetcore-10.0)
is the target consumption model. Moving that default and making JSON helpers consume
streams are follow-ups, not additional buffering in this reader.

neoCLR deliberately exposes its existing InputStream capability and typed Result
errors rather than requiring .NET Stream or exceptions. TryGetStream, alongside
TryGetBytes, avoids implicit copying but requires the consumer to handle both
shapes. Length=-1 is a provisional sentinel because the existing Length is an int;
a future nullable/Optional length contract should be reviewed before stability.
The five-second transport ceiling, original server request deadline and 1 MiB
client decoded-body quota remain. This is bounded streaming, not unlimited bodies
or HTTP/1.1 keep-alive. TLS shares the transport implementation, but new TLS and
Windows streaming qualification remains open.

## Evidence and reproduction

[54 independent-peer cases](validation.json) pass on macOS ARM64: client and server,
nine cases each, interpreter/JIT/AOT. The peer withholds body bytes until the Raven
consumer prints HEADERS after receiving its stream. Cases cover read-ahead,
fixed/chunked/EOF bodies, truncation, malformed framing, oversize chunks, zero and
invalid reads, pre-cancellation, pending cancellation, overlapping reads and closing
a pending read, plus nonempty chunked input for an empty-only response/request. Destination sentinels check range preservation. Unknown-length
content forwarding is rejected without consuming the original stream.

[Buffered controls](buffered-validation.json), run before the final streaming-only
empty-body parity correction, pass the prior 16-response concurrency
fixture in all three modes, including fragmented fixed/chunked bodies, admission,
queued cancellation and shutdown. [Diagnostic AOT](gc.json) reclaims all 12,165
client allocations and all 9,891 server allocations, with zero failures. Peak live
payload is 12,969/client and 10,682/server bytes. These are whole-process correctness
checks, not allocation-per-request or throughput results.

Build both `client/Probe.rvnproj` and `server/Probe.rvnproj` with
`scripts/build-native-project.py --profile http`, the same freshly prepared HTTP
bundle, a 4 MiB byte-array limit and 32 MiB heap. Then:

```sh
python3 docs/experiments/http-body-stream/validate.py --bundle BUNDLE --client CLIENT_BUILD --server SERVER_BUILD --output validation.json
python3 docs/experiments/http-body-stream/profile.py --client CLIENT_BUILD --server SERVER_BUILD --output NEW_GC_DIRECTORY
```

[Library build identity](library-build.json) records every System.Web source hash
and the exact bundled compiler hash. [Qualification inputs](qualification-inputs.json)
identify fixture sources and the native documentation artifacts. The reference-only
CLI signatures and native API snapshot were refreshed; no exclusion was required.
No installed SDK, release or published website is changed by this development slice.

## Author direction: protocol first

On 2026-10-13 the author clarified “HTTP specification”: follow the protocol and
adopt established, obvious implementation optimizations. This is the correctness
baseline, not permission to relax framing for benchmark gains. Standards anchors
are [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) for semantics and
[RFC 9112](https://www.rfc-editor.org/rfc/rfc9112.html) for HTTP/1.1 framing.
Existing rejection of chunk extensions/trailers, method-body restrictions, missing
Expect handling/error responses and connection reuse are gaps to review against
the applicable requirements, not permanent neoCLR-specific semantics.

Next bounded conformance slice: accept/ignore syntactically valid bounded chunk
extensions, parse a bounded trailer section with appropriate field handling, and
exercise fragmented/interoperable cases. Keep resource quotas and invalid framing
checks. Only then broaden defaults or use keep-alive in comparative benchmarks.
Bounded byte scans, direct framing writes, demand-driven reads and removal of
whole-body copies are established implementation techniques; pooling, bulk copies
and larger tunable blocks should follow measured allocation and throughput evidence.
