# HTTP byte paths — 2026-10-13

The request decoder now accepts a received byte block, returning one outer result
per block. It still validates framing byte by byte; chunk decoding remains byte
oriented. Receive storage is borrowed only during the synchronous call. Header and
body bytes are copied into owned storage before the receive buffer is reused.
Internal header windows avoid a temporary copy when decoding text. HTTP version
comparison uses bytes, and transfer-encoding comparison reuses the decoded value.

Response framing is written directly into the output byte collection. Status and
length digits no longer require strings; validated header bytes are reused instead
of concatenating and re-encoding a complete header string. Public contracts and
limits are unchanged. Responses and bodies remain buffered and copied; this is not
streaming or zero-copy transport.

Like [Kestrel's parser](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Core/src/Internal/Http/HttpParser.cs),
framing operates on bytes. [RFC 9112 section 2.2](https://www.rfc-editor.org/rfc/rfc9112.html#section-2.2)
requires parsing HTTP messages as octets. No public Span/Memory equivalent or new
buffer lifetime contract is introduced. Existing strict header acceptance remains
unchanged; these optimizations do not broaden protocol support.

## Validation

All checks passed on macOS ARM64 with the matching development System.Web library:

- [22 AOT protocol checks](protocol-validation.json): fixed/chunked bodies,
  fragmentation, framing ambiguity, header/body limits and truncation.
- [Five explicit interpreter checks](interpreter-validation.json).
- [Eight malformed-header checks](malformed-validation.json), interpreter and AOT.
- [Four JSON checks](json-validation.json), interpreter and AOT.
- [24 response-wire checks](responses-validation.json), eight each in interpreter,
  default JIT and AOT: GET/HEAD, 204/205/304, custom 299, UTF-8 content,
  mixed-case reserved framing headers and CRLF injection.

The response fixture is Server.rvn / Server.rvnproj. After compiling it with the
matching bundle and native host, reproduce its wire checks with:

```sh
python3 docs/experiments/http-byte-paths/validate.py \
  --build target/http-byte-response-cases-v2-20261013 \
  --bundle target/http-byte-candidate-v2-20261013/bundle \
  --output target/http-byte-response-cases-v2-20261013/validation.json
```

Reports retain exact commands and input hashes. See the
[allocation and throughput comparison](../../../benchmarks/http-server/byte-paths.md)
for additional workload and GC evidence. Windows has not been requalified by this
local slice. The installed SDK and published Preview 14 baseline are unchanged.
