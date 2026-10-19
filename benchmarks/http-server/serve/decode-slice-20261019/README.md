# Decode borrowed UTF-8 without temporary byte arrays — 2026-10-19

Encoding.Decode now accepts ReadOnlySlice<byte>, replacing Sequence<byte>. The
UTF-8 and ASCII providers preserve strict Result errors, the 65,536-byte limit,
BOM/NUL and empty input. The native service reads only the borrowed window and
allocates owned result text. No intermediate managed byte array or HttpByteWindow
object is needed when HTTP decodes its contiguous headers. Custom encodings must
implement the new signature; arrays borrow implicitly. Noncontiguous builders
still explicitly materialize a window. Encode's return contract is unchanged.

This implements the author's direction to remove avoidable temporary arrays, not
a new string representation. [Encoding.GetString(ReadOnlySpan<byte>)](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.getstring?view=net-10.0)
is the .NET comparison: borrowed decoding avoids an intermediate input array while
still producing a string. neoCLR retains its existing UTF-8 representation and
strict Result-based contract rather than adding compatibility overloads.

## Runtime ownership and limits

The private Utf8DecodeSlice service requires exact ReadOnlySlice<Byte> metadata.
Native lowering passes owner/offset/length, validates the backing storage and every
selected initialization marker, validates UTF-8 and copies into owned text. The
rooted allocator retains the input owner when memory pressure triggers collection.
The current collector never relocates live payloads; a moving implementation must
revise this privileged pointer lifetime. No provider pointer or view escapes the
synchronous call, and this is not a stable Native API.

Interpreter arrays contain tagged Value slots rather than contiguous bytes. That
path builds owned UTF-8 host storage from the selected slots; it avoids a managed
byte-array snapshot but is not host-allocation-free. The old array InternalCall
remains for low-level fixtures. Public Encoding has one borrowed decoding contract.

Lowercasing/transformation buffers, incremental carry, unknown Sequence providers
and owned results can still require allocations. This change does not remove every
HTTP allocation or the remaining request target/host encoding arrays.

## Allocation evidence

Same 200 measured plain requests, 20 warmups and one shutdown exchange, including
startup/shutdown. Compare the preceding method-parser commit with this decoder.
Both native traces reconcile all counts and bytes, with zero dropped records.
The summarizer is ../request-token-20261019/text-allocations.py.

| Counter | Method-parser baseline | Borrowed decoder |
| --- | ---: | ---: |
| All allocations | 50,922 | 49,154 |
| All payload bytes | 3,021,397 | 2,961,729 |
| Actual strings | 889 | 889 |
| Actual string payload bytes | 12,209 | 12,209 |
| UTF-8 decode snapshot arrays | 884 | 0 |
| Decode snapshot payload bytes | 24,308 | 0 |
| UTF-8 encode arrays | 442 | 442 |

Exactly four snapshot arrays and four window wrapper objects disappear per
exchange: 1,768 fewer allocations (3.47%) and 59,668 fewer payload bytes (1.97%).
Combined with method parsing, six temporary arrays and six helper objects per
exchange disappear: 2,652 fewer allocations (5.12%) than the original array-slot
Promise baseline. Strings remain owned and unchanged. Payload totals exclude
allocator headers; these are not typed live-retention or GC-pause measurements.

## Qualification and reproduction

The 46-case HTTP wire oracle passes in interpreter, JIT and AOT, including invalid
UTF-8/token precedence. Native memory-view tests cover nonzero windows, source
mutation, default empty input, initialization outside/inside the window, invalid
bounds and collection during rooted decoding. Exact binding tests reject mutable,
persistent, array and non-InternalCall substitutes. The matching five Raven
libraries and native API snapshot build successfully; the legacy CLI reference
explicitly excludes the borrowed signature.

The first debug-interpreter codec run timed out on quota-heavy tests. Qualification
uses a freshly built optimized interpreter. An initial stream fixture migration
mistakenly used Length on ArrayList; its corrected Count access passes the separate rerun. Across those runs, all 36
codec/stream/boundary cases pass in interpreter, JIT and AOT; HTTP text adds three
cases per mode. The final remaining-contracts report covers 66 passing executions.
JSON DOM passes 21 tests in interpreter and JIT. AOT JSON qualification remains
blocked by the existing synchronous fixture/HTTP host link dependency on
neoclr_drain_default_queue_v1; retain that failed result, not an AOT success claim.

The final allocation cohort uses target/http-decode-qualified-libraries,
target/http-decode-qualified-bundle and target/http-decode-qualified-build. The
preceding method-parser baseline is target/http-method-final-build. Raven compiler
revision dcf7ac41f is unchanged. Native source changes implement the private decoder;
this is not a comparison of identical runtime binaries. Build using Xcode 26.2 and
its MacOSX26.2 SDK, as recorded by the build manifests.

```sh
python3 benchmarks/http-server/serve/request-token-20261019/text-allocations.py \
  target/http-decode-qualified-callers/summary.json \
  --output target/http-decode-qualified-text-allocations.json
python3 benchmarks/http-server/serve/decode-slice-20261019/compare.py \
  --baseline-build target/http-method-final-build \
  --candidate-build target/http-decode-qualified-build \
  --rates 400 --duration 30 --output target/http-decode-comparison
```

Compressed evidence retains source paths and SHA-256 hashes in evidence/manifest.json.
The preceding baseline trace is retained in ../request-token-20261019/evidence.
[JSON conversion work](../../../../docs/design/json-byte-processing.md) remains a
separate measured migration; this plain HTTP workload does not exercise JSON.

## Matched HTTP load result

Four 30-second windows at 400 requests/sec, synchronous GC, 1 MiB heap, 16 server
lanes, 64 client slots, connection-close; order baseline/candidate/candidate/baseline.
All 48,000 offered requests were admitted and validated within their windows, and
all servers shut down cleanly. No build or profiler ran during timing.

| Round | Baseline CPU ms/completion | Candidate CPU ms/completion | Baseline offered p99 ms | Candidate offered p99 ms |
| --- | ---: | ---: | ---: | ---: |
| 1 | 1.8883 | 1.9083 | 71.27 | 41.75 |
| 2 | 1.8983 | 1.9083 | 46.03 | 13.33 |

Mean CPU/completion is 0.79% higher in the candidate; no CPU improvement is shown.
Candidate p99 is lower in both pairs but varies substantially, so these four windows
do not establish a stable latency improvement or increased maximum throughput.
Sampled peak RSS is 12,848 KiB baseline and 12,752/12,768 KiB candidate. The supported
conclusion is fewer temporary allocations with preserved fixed-rate completion.
This bounded change does not repeat the earlier experimental concurrent-GC matrix
or establish reduced GC pause time.
