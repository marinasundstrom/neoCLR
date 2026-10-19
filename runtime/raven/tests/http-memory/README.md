# HTTP memory-view integration

Run against a matching native library/runtime bundle:

```sh
python3 runtime/raven/tests/http-memory/verify.py --bundle path/to/bundle --output target/http-memory-validation
```

The public consumer exercises the internal borrowed request-parser input and retained
body destination through `HttpServer.AcceptHeaders` and `HttpClient.Send` with
`ResponseHeadersRead`. It sends selected readonly socket windows, splits headers
across submissions, reads fixed-length and chunked bodies into nonzero destination
windows, collects during pending reads, checks sentinel bytes and EOF, and observes
acknowledged cancellation without cancelling its own entry point. The client case
also checks partial response reads and read-ahead storage.

The verifier checks exact stdout, empty stderr and successful exit in interpreter,
JIT and AOT modes. It records build/run commands and binary identities. This is a
correctness test, not a benchmark. Public InputStream signatures still use array
ranges pending their coordinated library migration; the HTTP implementation retains
Memory and borrows Slice only during synchronous consumption.
