# HTTP memory-view cohort: synchronous and experimental concurrent GC

Author-directed performance comparison, 2026-10-19. A fresh source-built library/Serve cohort shows a workload-dependent result: the
experimental collector improves bulk overload throughput and bulk tail latency,
while the tiny-response overload is faster with paced synchronous GC. CPU cost is
a diagnostic for optimizing this prototype, not an acceptance veto or evidence
of its eventual optimized cost.

## Measurement contract

The same Raven `Server.rvn` uses `HttpServer.Serve`, 16 lanes and a queued await
per handler. Compare paced synchronous mark/sweep with the isolated applying
snapshot collector, using the identical AOT guest object and library cohort,
1 MiB heap, 64 load-generator slots, HTTP/1.1 connection close and loopback.
Plain responses contain ten bytes; bulk responses contain 8192 bytes. Both validate
exact bytes and framing. The comparator rejects mismatched guest assemblies,
objects or heap limits. No Go baseline is included in this two-collector study.

Each rate receives two 30-second trials per collector, reversing rate and collector
order in round two. Warmup, drain and control shutdown are outside timed throughput.
Unsent arrivals at the slot cap remain misses; response latency alone excludes them.
The provisional operating-point gate is all arrivals successful, response p99 at
most 250 ms and generator p99 lag at most 5 ms. It is not a product SLO or a maximum
capacity determination. CPU covers the process, including the concurrent worker;
RSS is sampled, not exact managed live bytes. No build or profiler overlaps timing.
Desktop activity and the same-machine generator remain uncontrolled limitations.

The concurrent implementation copies a bounded private snapshot, traces it on a
worker, and conditionally applies/reclaims on the owner. It also changes pacing,
invalidation and fallback behavior; this comparison cannot isolate concurrency
from those costs. It remains experimental and is not a selectable production GC.
TLS correctness is qualified separately; this load study is plain HTTP, not HTTPS.
Shared public stream signature migration remains incomplete.

## Qualification and reproduction

The current TLS library and memory-view paths pass interpreter/JIT/AOT checks in
[the TLS evidence](../../../../docs/validation/memory-views-20261019/tls-integration-validation.json).
Before timing, the fresh walker/oracle tests, sanitized 96 KiB heap recovery and
release recovery verify malformed input, suspended requests, 16 peers and joined
shutdown. The tests retain epoch/root invalidation, pressure fallback and final
reclamation; they are a bounded qualification, not a runtime-wide proof.

Build inputs use Raven `dcf7ac41f` and the memory/TLS integration through `7d9a8197b`.
The retained manifests identify exact binaries and source hashes; commit identity
alone is not the cohort definition. Build the matching library/bundle using the
normal native-library scripts, then:

```sh
python3 benchmarks/http-server/serve/task-completion-20261018/build.py \
  --library target/tls-memory-libraries \
  --compiler /path/to/rvnc.dll --backend target/debug/neoclr-codegen \
  --link-build target/http-memory-consumer-v4/build --output target/http-memory-gc-cohort
python3 docs/experiments/concurrent-gc-walker/run.py --output target/http-memory-gc-walker
python3 docs/experiments/concurrent-gc-http/build.py \
  --native-build target/http-memory-gc-cohort --walker-build target/http-memory-gc-walker \
  --mode apply --heap-bytes 98304 --output target/http-memory-gc-sanitized
python3 docs/experiments/concurrent-gc-http/validate.py \
  --build target/http-memory-gc-sanitized --apply --output target/http-memory-gc-safety-recovery
python3 docs/experiments/concurrent-gc-http/build.py \
  --native-build target/http-memory-gc-cohort --walker-build target/http-memory-gc-walker \
  --mode apply --release --output target/http-memory-gc-apply
python3 docs/experiments/concurrent-gc-http/validate.py \
  --build target/http-memory-gc-apply --apply --output target/http-memory-gc-release-recovery
python3 docs/experiments/concurrent-gc-http/compare.py \
  --sync-build target/http-memory-gc-cohort --apply-build target/http-memory-gc-apply \
  --rates 200 400 600 --duration 30 --output target/http-memory-gc-plain
python3 docs/experiments/concurrent-gc-http/compare.py \
  --sync-build target/http-memory-gc-cohort --apply-build target/http-memory-gc-apply \
  --workload bulk --rates 50 100 --duration 30 --output target/http-memory-gc-bulk
```

Use the matching Xcode 26.2 clang/SDK for native qualification on this host.
The existing build seed supplies host link settings; guest code and all five managed
libraries are freshly compiled. The synchronous cohort retains both stress and
paced executables; this comparison explicitly selects `app-paced`.


## Throughput and latency results

These are paired 30-second operating points, each repeated in reverse order.
Completed/sec is the mean of the two timed windows. p99 columns show the range of
the two trial p99s, not a percentile recomputed across combined samples. Misses sum
both trials and are unsent generator arrivals. All 161,056 admitted measurement
requests validate; 186,000 were offered across 24 trials. Server/client counts match
and every server shuts down cleanly. All generator p99 lags remain below 5 ms.

| Payload | Offered req/s | Sync completed/s | Concurrent completed/s | Sync p99 ms | Concurrent p99 ms | Sync / concurrent misses |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10 B | 200 | 200.00 | 200.00 | 5.16–5.21 | 4.83–4.97 | 0 / 0 |
| 10 B | 400 | 400.00 | 400.00 | 8.89–32.81 | 8.53–11.08 | 0 / 0 |
| 10 B | 600 | 483.37 | 458.40 | 141.24–149.40 | 153.28–158.79 | 6871 / 8370 |
| 8 KiB | 50 | 50.00 | 50.00 | 10.94–11.44 | 11.67–12.28 | 0 / 0 |
| 8 KiB | 100 | 100.00 | 100.00 | 22.28–23.65 | 9.62–9.64 | 0 / 0 |
| 8 KiB | 200 | 111.48 | 122.58 | 591.26–628.53 | 546.69–548.70 | 5184 / 4519 |

Both collectors pass the provisional operating-point gate in both rounds at
200/400 req/s for 10-byte responses and 50/100 req/s for 8 KiB responses. Both
fail it at the overload points; completed throughput there describes overloaded
delivery, not a sustainable offered rate. The bulk overload also exceeds the
250 ms response-p99 target. No failures are discarded from the results.

For 8 KiB responses, concurrent GC delivers **10.0% more timed completions** at
200 offered req/s, with fewer misses. At 100 req/s, both complete all arrivals and
concurrent p99 is about **58% lower** in the paired results, while median latency is
about 6% higher (8.51 ms versus 8.03–8.04 ms). For ten-byte responses at 600 offered
req/s, concurrent delivery is **5.2% lower**. Tiny-response p99 at 400/sec varies
between rounds, so no uniform latency gain is inferred. This is evidence to continue
optimizing the concurrent design for representative workloads, not a single winner
across HTTP use cases or proof of a production capacity limit.

## Resource diagnostics and optimization follow-up

CPU is reported as a resource cost, not a pass/fail criterion. The author clarified
that the current collector is not optimized and CPU usage alone should not decide
its value. Current measurements do not predict the optimized collector's cost.
A latency/throughput gain can justify extra CPU where the deployment has headroom;
resource-constrained operation remains a separate question.

| Payload / offered req/s | Sync CPU ms/completion | Concurrent CPU ms/completion |
| --- | ---: | ---: |
| 10 B / 200 | 2.152 | 2.777 |
| 10 B / 400 | 2.007 | 2.678 |
| 10 B / 600 | 2.066 | 2.731 |
| 8 KiB / 50 | 7.513 | 10.137 |
| 8 KiB / 100 | 7.300 | 9.832 |
| 8 KiB / 200 | 8.948 | 10.740 |

Sampled peak RSS ranges from about 12.7–12.8 MiB for synchronous GC and 13.7–14.1 MiB
for the concurrent process. CPU includes the background worker; both modes have
the same 1 MiB managed heap. Per-trial p50/p95/p99, generator lag, sampled CPU peaks,
RSS and outcomes are retained in the summary files. Exact GC pause distributions
and a matched allocation-rate profile were not instrumented in these timed runs;
HTTP latency must not be reported as a GC pause measurement.

The experimental counters show substantial repeated work. For example, the first
400/sec tiny-response trial starts 482,976 snapshots, applies 265,143 and discards
217,833; the first 100/sec bulk trial starts 532,530, applies 470,270 and discards
62,260. These counts include warmup and teardown, and fallback counts are attempts,
not necessarily full collections. Every experimental run reports zero trace errors
and full final allocation reclamation. No resource leak is hidden by process exit.

Next, measure capture, tracing, validation/sweep and fallback phase costs in a
separate instrumented cohort, then test pacing/coalescing hypotheses against the
same workload pairs. Preserve root/epoch invalidation, pressure fallback and cleanup
invariants. Retest the bulk latency/throughput benefits and the tiny-response
regression after each optimization; do not assume all current overhead will vanish.
The data does not yet identify which phase dominates CPU or establish a general
concurrent-collector algorithmic limit.

## Retained evidence

- [Plain comparison](evidence/plain/comparison.json), [derived summary](evidence/plain/summary.json).
- [Bulk comparison](evidence/bulk/comparison.json), [derived summary](evidence/bulk/summary.json).
- [Bulk overload comparison](evidence/bulk-overload/comparison.json), [derived summary](evidence/bulk-overload/summary.json).
- [Synchronous build](evidence/sync-build.json), [experimental build](evidence/apply-build.json), [sanitized build](evidence/sanitized-build.json), [walker/oracle qualification](evidence/walker.json).
- [File hashes](evidence/manifest.json). Each trial has a compact browsable JSON and a
  lossless `.json.gz` original linked with hashes. Raw reports include build identities,
  all requests and resource samples. Collector totals and recovery reports are retained
  alongside them. No binaries or generated TLS private keys are published.

Derive the tables with `docs/experiments/concurrent-gc-http/summarize.py` on each
original comparison directory. Use `benchmarks/http-server/serve/archive-report.py`
to retain raw reports losslessly. The `bulk-overload` follow-up uses the same command
as bulk above with `--rates 200` and a new output directory.
