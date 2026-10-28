# Bounded worker queue comparison — 2026-10-28

The opt-in local/global/stealing policy has **no demonstrated general advantage**.
Keep the FIFO default. This macOS ARM64 screen tests native copied work, not
managed HTTP, async I/O, UI, RabbitMQ or sync-versus-concurrent GC performance.

## Qualification

The existing eleven native-pool groups pass optimized and fail-fast UBSan with
queues, compiled-worker coordination and worker tracing enabled together. Both
policies pass the new deterministic ordering, eight-local-dispatch fairness,
cancellation, stale-ticket and slot-generation reuse test optimized and UBSan.
Without the queue flag, default and trace-only ARM64 assembly are byte-identical
to commit `75260d4e7`. See archived commands and hashes.

## Timing

Each initial matrix has three interleaved pairs for 1/2/4 workers and four shapes.
Tiny shapes use 65,536 jobs per invocation; skew/blocking shapes use 1,024.
Fixed batches contain sixteen jobs. Blocking means a 500 µs native sleep, not
async suspension. CPU skew uses three long computations per nested batch.
Timing excludes tracing and sanitizers; per-job timestamp/result checks are
included equally. Candidate queue counters are included in its cost.
Latency is submit-call-to-callback-entry, including admission and gate contention.
Values below are ratios of median elapsed times (positive means more throughput),
and medians of per-run p99 latencies, not pooled p99. Desktop load/core placement
were uncontrolled. Neither matrix establishes a default-enablement threshold.

### First screen

| Workers | Shape | Throughput change | FIFO / local p99 µs |
| --- | --- | --- | --- |
| 1 | External tiny | +6.3% | 6 / 6 |
| 1 | Nested tiny | +0.4% | 3 / 4 |
| 1 | Nested CPU skew | -0.4% | 1176 / 1177 |
| 1 | Nested blocking | +2.1% | 2019 / 1942 |
| 2 | External tiny | -4.6% | 11 / 12 |
| 2 | Nested tiny | -1.6% | 4 / 5 |
| 2 | Nested CPU skew | -0.2% | 415 / 422 |
| 2 | Nested blocking | -0.9% | 671 / 666 |
| 4 | External tiny | +1.9% | 40 / 38 |
| 4 | Nested tiny | -4.6% | 10 / 11 |
| 4 | Nested CPU skew | -0.5% | 64 / 67 |
| 4 | Nested blocking | +0.6% | 30 / 27 |

### Repeat screen

The only intervening benchmark change enlarged the allowed round count for longer
runs; these matrix workloads stayed unchanged. Retain both screens: short tiny-job
results vary substantially, including direction reversals.

| Workers | Shape | Throughput change | FIFO / local p99 µs |
| --- | --- | --- | --- |
| 1 | External tiny | -8.9% | 6 / 6 |
| 1 | Nested tiny | +0.0% | 3 / 4 |
| 1 | Nested CPU skew | -0.4% | 1188 / 1186 |
| 1 | Nested blocking | +0.1% | 1946 / 1950 |
| 2 | External tiny | +44.6% | 21 / 12 |
| 2 | Nested tiny | -39.8% | 9 / 14 |
| 2 | Nested CPU skew | -6.0% | 448 / 583 |
| 2 | Nested blocking | -1.2% | 658 / 665 |
| 4 | External tiny | +2.5% | 39 / 41 |
| 4 | Nested tiny | +4.9% | 11 / 12 |
| 4 | Nested CPU skew | +0.1% | 64 / 64 |
| 4 | Nested blocking | +0.4% | 31 / 45 |

### Longer two-worker control

Five interleaved pairs per shape, 1,048,576 jobs per invocation. This follow-up was
selected after observing variation, not a predeclared independent confirmation.

| Workers | Shape | Throughput change | FIFO / local p99 µs |
| --- | --- | --- | --- |
| 2 | External tiny | -1.0% | 11 / 11 |
| 2 | Nested tiny | -1.2% | 5 / 6 |

The longer runs do not reproduce the extreme short-run changes. They also provide
no reason to adopt the candidate. Full CPU times, maxima, worker distributions,
steals and individual samples are archived, including losing cases.

## Separate profiling

Four-worker nested-tiny processes ran 4,194,304 jobs while macOS `sample` captured
two seconds of thread stacks. FIFO/local top-of-stack counts include condition
waits (2678/2700), mutex waits (363/337) and condition signals (103/109). These
include off-CPU waits: they are not CPU percentages or recoverable-speedup estimates.
Both policies retain the shared pool/heap gate. Coordination and handoff warrant
investigation, but these samples do not prove that removing a lock improves an
application. Profiled process timings are excluded from the timing tables.

## Next decision

Measure real I/O completion-to-resume latency and idle wakeups before changing
coordination. Include responsiveness at light load, mixed CPU/I/O pressure,
bounded admission, cancellation and saturation; then compare HTTP/message-style
consumers in JIT and AOT with both collectors. Future UI controls need owner-thread
affinity, frame/input latency and idle energy measurements on desktop/mobile.
No RabbitMQ binding or mobile/UI qualification is claimed. A future broker
comparison must pin protocol/client versions, payloads, acknowledgement/durability,
prefetch/backpressure, connections, resources and tail/error budgets.

[Reproduction and policy](../../experiments/queue-policy/README.md).
Compressed raw evidence is indexed by decompressed SHA-256 in `artifacts.json`.
