# HTTP content copy foundation — 2026-10-19

The Raven HttpContent base exposes CopyTo(OutputStream) and the cancellation-token
overload. This is the first implemented transfer slice of the
[streaming content direction](../../design/http-content-streaming.md), not a
completed abstract hierarchy or custom producer contract.

.NET's [HttpContent.CopyToAsync](https://learn.microsoft.com/en-us/dotnet/api/system.net.http.httpcontent.copytoasync?view=net-10.0)
is the API comparison. neoCLR follows the destination-stream and cancellation
model, using its ordinary async method naming and typed HttpError.Content failures.
The existing concrete byte/stream representation remains transitional. The method
does not add framing or replace the existing socket upload/server-response adapters.

## Behavior

- Arrays are retained and written directly. General sequences and source streams
  allocate at most 1024 bytes of staging, filled only for the range passed to Write.
- A destination write must settle before the buffer is reused or the source is read
  again. OutputStream.Write already promises complete-range success; CopyTo does not
  reinterpret it as a short-write count API.
- Known lengths stop exactly without overreading and reject premature EOF; incoming
  unknown lengths stop at the framing stream's EOF. Invalid read counts are rejected.
- Read/write failures preserve StreamError; partial output is allowed on failure.
  Cancellation is a cancelled Task, not a new HttpError case. Pre-cancellation does
  not consume content. The only explicit Promise bridge is for cancellation; ordinary
  transfers use await. The existing Task implementation still allocates internally.
- A started stream content cannot be copied or uploaded again after completion,
  error or cancellation. Buffered content remains replayable until disposed.
- Neither source nor destination is flushed or closed by CopyTo. Content disposal
  retains its existing leaveOpen policy; disposal during a pending write stops the
  next read after that write settles. Caller-owned destinations remain open.

## Focused validation

`runtime/raven/tests/http-content-copy` uses an async entry point because the current
shared discovery runner only invokes synchronous tests. The six cases cover pending
backpressure with explicit GC, array/direct and sequence/staged output, exact lengths,
empty bodies, stream errors and invalid counts, cancellation and disposal/replay.
A test-only output stream retains CancellationToken in its owner before entering an
async helper: this compiler bridge rejects capturing that application value parameter
across await. No compiler change or public stream signature workaround is introduced.

The independent loopback peer in validate.py withholds the body until the guest
prints HEADERS, then tests fixed-length, chunked, connection-close, truncated and
malformed responses. Main.rvn copies into MemoryStream and verifies every byte,
length and sentinel, then rejects a second consumption. This directly exercises the
unknown-length receive path through the real HTTP framing stream. It is client-side
plain HTTP qualification; it does not establish new server/TLS transport behavior.

Run the managed suite with scripts/test-runtime-library.py, suite http-content-copy,
in native/interpreter/jit modes. Build Probe.rvnproj with build-native-project.py
(profile http), then run validate.py --bundle BUNDLE --build BUILD --output REPORT.
Use a bundle rebuilt from the matching source. The compiler is Raven codex/neoclr
`dcf7ac41f`; this change does not modify Raven or native runtime sources.

Evidence is retained in evidence.json.gz. API signatures/XML summaries are refreshed
in both native and legacy CLI references, with no new type-selection exclusion.
No throughput, allocation-count or collector performance improvement is claimed;
staging capacity and backpressure are correctness contracts in this slice.

Qualification result: all six lifecycle cases pass in interpreter, JIT and AOT
(18 executions), and all five loopback cases pass in interpreter and AOT
(10 exchanges). Both API snapshots validate. Initial rejected fixture builds are
retained separately from passing results; native compilation was not treated as
proof of execution.
