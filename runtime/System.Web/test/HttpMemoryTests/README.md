# HTTP memory-view integration

Run against a matching native library/runtime bundle:

```sh
python3 runtime/System.Web/test/HttpMemoryTests/verify.py --bundle path/to/bundle --output target/http-memory-validation
```

One named async `NeoClr.Testing` case exercises the internal borrowed request-parser input and retained
body destination through `HttpServer.AcceptHeaders` and `HttpClient.Send` with
`ResponseHeadersRead`. It sends selected readonly socket windows, splits headers
across submissions, reads fixed-length and chunked bodies into nonzero destination
windows, collects during pending reads, checks sentinel bytes and EOF, and observes
acknowledged cancellation without cancelling its own entry point. The client case
also checks partial response reads and read-ahead storage.

The shared server and fixed-length/chunked/cancelled/client phase order remain
intact. Assertions return TestFailure with a phase prefix instead of numeric exit
codes. Main explicitly awaits the complete lifecycle and reports its result through
a bound reference adapter to TestSuite; this is not automatic async discovery.
OnCompleted is retained only to observe cancellation without cancelling Main.

The verifier checks the exact framework transcript, empty guest stderr and successful
exit in interpreter, JIT and AOT modes. Only the exact documented DAM2004 JIT
compiler warning is accepted and recorded separately. It records build/run commands and binary identities. This is a
correctness test, not a benchmark. Public InputStream signatures still use array
ranges pending their coordinated library migration; the HTTP implementation retains
Memory and borrows Slice only during synchronous consumption.

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#http-memory-lifecycle)
records the preserved assertions and validation. Production APIs, host profiles,
process/instruction budgets and historical reports remain unchanged.

Qualification now passes interpreter, JIT and AOT. The earlier wait-bridge failure
came from a compiler-generated borrowed UTF-8 receiver temporary retained in
SendText after ToArray. EncodeSendWindow performs that copy in a synchronous frame
that returns before any await; only the owned byte array reaches SendText.
The interpreter's conservative borrowed-view suspension check remains unchanged.
The linked evidence preserves the earlier failures and the passing follow-up.
