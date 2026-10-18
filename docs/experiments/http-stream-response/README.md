# Known-length server response streams

Development slice, 2026-10-13. **Update 2026-10-18:** [buffer refinement](../../../benchmarks/http-server/buffers-20261018/README.md) increases producer blocks to 4 KiB; the current fixture additionally checks an 8,193-byte borrowed source completes in three reads. Historical evidence below retains its original 256-byte limit. `HttpContent.FromStream` now works for server output,
as well as the existing client upload path. This is not incoming-body streaming.

The sender snapshots framing and source ownership, writes headers, then alternates
producer reads (at most 256 bytes) and complete writes of each borrowed block.
Short producer reads are valid. The next read cannot overwrite bytes still being
sent. No full response body is assembled for stream content. Owned sources close
once; borrowed sources remain open. A HEAD response declares the source length but
does not read it. The existing 1 MiB body limit and connection-close policy remain.

The response-write countdown reaches a waiting producer through cancellation.
Producers must acknowledge cancellation; the runtime cannot forcibly complete
arbitrary managed operations. Caller cancellation remains a cancelled task;
acknowledged deadline expiry becomes HttpError.TimedOut. Framing already written
cannot be retracted after a producer failure.

## Evidence

`Server.rvn` and the independent Python wire driver cover owned/borrowed sources,
71-byte deferred reads, source replacement after Complete starts, HEAD, empty
content, premature EOF, typed source errors, invalid read counts, disposed content,
caller cancellation and timeout while the producer waits. Each mode runs ten cases.
Use `scripts/build-native-project.py --profile http` with a freshly rebuilt matching
HTTP bundle, then run `validate.py --build BUILD --bundle BUNDLE --output REPORT`.
`profile.py --build BUILD/build.json --output NEW_DIRECTORY` checks diagnostic GC.
Reports retain input identities. These tests measure correctness, not throughput.

## Design comparison and remaining work

.NET supports stream-backed content through
[StreamContent](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.streamcontent?view=net-10.0).
neoCLR reuses its existing InputStream/HttpContent contract for both send directions.
Requiring a declared length keeps this first slice's framing deterministic, at the
cost of not supporting generators whose total output size is unknown. Chunked
output is a remaining gap, not a permanent restriction.

The next receiving slice should follow
[HttpCompletionOption.ResponseHeadersRead](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcompletionoption?view=net-10.0)
for explicit client completion at headers, with separate body-read cancellation and
timeout. ASP.NET Core exposes [request and response streams](https://learn.microsoft.com/en-us/aspnet/core/fundamentals/middleware/request-response?view=aspnetcore-10.0)
without requiring eager whole-body materialization. neoCLR must preserve bounded
read-ahead across the header boundary and enforce fixed/chunked/EOF framing as
consumers read. Current incoming requests and client responses remain buffered.
TLS and Windows response-stream qualification remain open; shared HttpConnection
provides the transport path but does not itself prove those executions.

The pinned compiler required a closure for a captured cancellation source's Cancel
method rather than its direct method group (NEOMETA001), and unique local names
where inner scopes precede same-named outer declarations. These are recorded
compiler follow-ups, not intended Raven restrictions.

Local macOS qualification: [all 30 cases pass](validation.json) across interpreter,
JIT and AOT. [Diagnostic AOT](gc.json) reclaims all 7,822 allocations with zero
failures and peak live payload of 9,591 bytes. No Windows result is inferred.
