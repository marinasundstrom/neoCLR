# Root-publication attribution — 2026-10-23

Kestrel remains the author's comparison target. This slice identifies work to
reduce; it does not improve or remeasure capacity. The latest uninstrumented
four-owner means remain 4,182 neoCLR versus 14,685 Kestrel requests/s on the common
connection-close fixture. See [peer comparison and limits](../peer-refresh/README.md).

## Findings

An instrumented main-owner run checks 6,530 responses and records about 3,677 root
publications per response. These counts are exclusive publication locations, not
inclusive call-stack costs, samples of CPU time, or evidence that every publication
can be removed. Startup/shutdown are included. No collection fails.

| Publication owner | Publications/response | Share |
| --- | ---: | ---: |
| HttpRequestDecoder.PushCore | 500 | 13.60% |
| HttpServer.EncodeResponse | 360 | 9.79% |
| HttpRequestDecoder.Parse | 232 | 6.31% |
| ReadOnlySlice.get_Length | 127 | 3.45% |
| ReadOnlySlice.get_Item | 113 | 3.07% |
| HttpServer.AppendWire | 113 | 3.07% |

The top three methods account for 29.70% of publications. Source inspection shows
PushCore reads slice length/item and several header-buffer properties in its
byte loop. Response encoding and AppendWire also iterate over bytes. The next
bounded optimization should reduce repeated work in these loops, or establish
compiler-verified non-collecting leaf calls where appropriate. A count does not
prove a call is safe to elide: fault handling, initialization, native callbacks and
worker coordination must remain covered. Rebuilding managed code requires a fresh
matching bundle and contract/forced-GC checks before paired capacity measurements.

The existing [CoreCLR/Java/Go comparison](../../../../docs/design/allocation-metadata-runtime-comparison.md)
remains the design basis. CLR compiler GC maps provide guidance for moving knowledge
out of runtime scans; this diagnostic does not implement CLR stack maps or a new
safepoint contract. Changing the current explicit-frame design needs measured
benefits and collection/worker validation. More worker threads alone do not resolve
the CPU-per-request gap. Persistent connections remain a separate support gap.

## Attribution and validation

`NEOCLR_GC_SITE_PROFILE` adds a diagnostic sink at each native-GC root publication.
The benchmark reporter uses a bounded, thread-local 16,384-entry hash table
(256 KiB per participating thread). It rejects unsupported function/instruction
indices, exhausted capacity and counter overflow instead of losing counts. At
process exit it sorts entries and checks their sum against the existing root
counter. This reporter reads only the exiting main thread, so the runner removes
HTTP worker defines. It is deliberately unsuitable for reporting aggregate worker
counts or capacity. The instrumented server is never used for throughput claims.

`profile.json` retains every function/instruction count, exact input hashes, commands,
wire-checked responses and clean shutdown. `summary.json` maps functions using a
fresh replay of the recorded codegen command; the regenerated object must match the
profiled object byte for byte. It includes generated class-dispatch thunks separately,
retains raw identities, and rejects unmapped functions. The object SHA-256 is
`4fdc93183f8be7f5d5fd2c143c9847875d0e5923f6ccde68cc0161f0ced6ad8c`.

Twelve optimized/UBSan-bounds cases cover repeated keys, a full table with collisions,
TLS isolation, sorted output, invalid indices, overflow, exhausted capacity and a
mismatched aggregate. Both ordinary GC and probe-only root objects are byte-identical
to the pre-hook sources when the macro is absent. This is a private diagnostic-only
change, so existing GC/worker qualification remains applicable; no public API or
website claim changes. UBSan is not a race detector.

## Reproduce

Use the SDK/compiler/bundle recorded in the peer comparison. Run diagnostics without
concurrent capacity trials. Each output directory must be new.

```sh
python3 benchmarks/http-server/primitive-comparison/site-profile/profile.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client \
  --output target/http-site-profile
python3 benchmarks/http-server/primitive-comparison/site-profile/summarize.py \
  --compilation target/plain-owned-root/measurement.json \
  --object target/plain-owned-root/candidate.o \
  --profile target/http-site-profile/profile.json \
  --output target/http-site-attribution
python3 benchmarks/http-server/primitive-comparison/site-profile/validate.py \
  --output target/http-site-validation
```

The object-identity check intentionally fails if runtime compiler/bundle changes
produce different code. Rebuild and profile the new object rather than borrowing
an older method mapping. Disabled-hook comparison uses HEAD as the pre-edit source
when testing a pending hook change; the retained validation records this change's
pre-commit comparison.
