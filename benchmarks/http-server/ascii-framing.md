# Fixed ASCII response framing — 2026-10-13

Follow-up to [bulk byte paths](byte-paths.md): the three fixed response fragments
(`HTTP/1.1 `, Content-Length prefix, Connection: close line) now write ASCII bytes
directly. Dynamic headers/content still use explicit encoding. There is no shared
mutable cache or new buffer lifetime. This follows the same byte-oriented protocol
principle as Kestrel, without adding a public Span/Memory contract or changing APIs.

[24 exact response-wire checks](ascii-framing-validation-20261013.json) pass in
interpreter, JIT and AOT, including HEAD, status-specific framing, UTF-8 content and
rejected headers. Earlier parser checks remain applicable: request parsing is unchanged.

## Allocation evidence

| Workload | Allocations/request before → after | Requested bytes/request before → after |
| --- | ---: | ---: |
| Plain | 519 → 510 | 20,913 → 20,673 |
| JSON | 1,210 → 1,198 | 37,294 → 36,982 |

[Plain capture](ascii-framing-allocation-plain-20261013.json) and
[JSON capture](ascii-framing-allocation-json-20261013.json) use the same diagnostic
AOT close-8-to-close-32 counter window as the preceding slice. Both validate 32
responses, report no allocation failures and reclaim every allocation by exit:
16,340 plain / 38,358 JSON. Peak live bytes remain 8,119 / 12,542. Matched compressed
stack captures are retained beside the reports. Root/collector work remains prominent;
this small allocation reduction does not address that dominant runtime overhead.

## Matched native timing

[Raw comparison](ascii-framing-comparison-20261013.json) uses the same host/compiler
and the previous slice as its baseline, with five alternating rounds, 48 processes
and 1,536 validated responses (960 measured). No concurrent builds or profiler ran.

| Case | Before median req/s (range) | After median req/s (range) |
| --- | ---: | ---: |
| Plain JIT | 73.45 (72.88–73.96) | 76.09 (70.06–76.31) |
| Plain AOT | 76.82 (73.69–77.59) | 78.40 (74.38–78.75) |
| JSON JIT | 38.52 (37.96–38.95) | 39.14 (36.21–39.21) |
| JSON AOT | 40.09 (36.85–40.68) | 40.80 (40.71–41.16) |

Median gains are 1.6–3.6%, with overlapping ranges except JSON AOT. Treat timing as
exploratory, not an established speedup. The allocation reduction is directly counted.
The unrolled constants increase guest object-file size by 46,800 bytes (plain) and
44,800 bytes (JSON); that is the code-size cost of bypassing codec calls in this
backend. A future constant byte-data facility may improve that tradeoff.

Installed SDK, published baselines and Windows qualification are unchanged. The
website's buffered-body description and existing public API reference remain valid.
