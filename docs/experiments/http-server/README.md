# HTTP server POC

Development after Preview 9: requires matching neoCLR runtime, Raven compiler,
reference, bridge and library artifacts. The published Preview 9 SDK is insufficient.

`Server.rvn` listens on IPv4 loopback with an OS-selected port, prints the port,
serves one `/greeting` request with `Café 🌍`, then closes its listener. The response
uses UTF-8 bytes and a Content-Type header; HttpServer supplies Content-Length and
Connection: close. Async Main awaits serving and cleanup; queued work runs while
accept is pending. The response handler creates an already-completed Task to satisfy
the handler contract; it performs no asynchronous operation.

From the repository root:

```sh
python3 docs/experiments/http-server/verify.py \
  --toolchain-root /path/to/matching/development/toolchain \
  --runner target/release/examples/measure_async
```

The verifier needs .NET 10 for its independent client, Python 3, and the trusted
`measure_async` fixture runner built with `cargo build --release --example measure_async`.
The archive's verifier can also run directly with those two absolute tool paths.
It builds the Raven sample through the development toolchain. The neoCLR client
sources come from the adjacent http-client experiment or the same downloaded archive.

Validation covers a separate neoCLR client/server pair, .NET HttpClient, a fragmented
raw request, eleven malformed/request-limit cases and six invalid application responses.
It uses a 256-object heap, checks collection and zero final live objects, and prints
per-case diagnostics. `--responses-only` reruns the six response checks. Its process
watchdogs and explicit instruction limit are test guards, not HTTP request deadlines.

The public contract is intentionally small: Listen/GetLocalPort/ServeOne/Close and
received HttpRequest.Headers. Only HTTP/1.1 GET with exactly one Host, no body and
optional Content-Length: 0 is accepted. Response status must be 200. Headers have a
2,048-byte bound; requests admit 16 fields, responses 14 application fields plus
server framing. Response bodies admit 1,024 bytes. Requests and responses use ASCII
header syntax; content bytes can contain UTF-8 text.

Parse/handler/transfer errors close the accepted connection and return a provisional
string error, without sending an HTTP error response. No request deadline, public
cancellation, routing, streaming, TLS or permanent server loop is claimed. Closing
only the listener leaves an accepted exchange active. Cancelled handler tasks and
runtime Faults retain existing Task/runtime behavior.

See ../../http-server-design.md for .NET comparisons and tradeoffs, and the on-site
Web guide and API reference for browsable public contracts.

The native transfer deadline now also covers a peer that connects but leaves request
headers incomplete. Select `--requests-only --case 'stalled request'` for this case.
It must close the accepted connection and finish with the existing receive error,
without depending on peer EOF. Accept and handler waiting still have no deadline.

## Native compilation follow-up (2026-10-08)

This exact Server source now runs through Raven neoCLR CIL to a standalone macOS
ARM64 image. The [native-web verifier](../../../benchmarks/native-web/README.md#first-native-http-execution)
compares one compiled artifact across interpreter/native execution for four request
cases, with response/output parity and native cleanup checks. This narrower evidence
complements the interpreter verifier above; it does not carry over its entire case
matrix to native mode or establish server throughput. The POC remains work in progress.

## Native project showcase (2026-10-09 development)

`Native.rvnproj` builds this same source with the selected native compiler/library
bundle. Use `--profile http` on macOS ARM64 or `--profile windows-http` from a
Windows x64 MSVC developer shell; see the [project instructions](../../native-poc-bundle.md#windows-http-project-profile-development).
The five-case [project evidence](../../windows-native-http-project-validation.json)
passes on both platforms, including executable-only deployment and callback faults.
This qualification is for the bounded server; native HttpClient and Windows ARM64
remain separate work.
