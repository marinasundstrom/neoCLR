# Benchmark history

Recorded runs remain available as neoCLR evolves. Compare results only when the
workload, machine and measurement settings match; a newer date alone does not make
two runs comparable. These first Preview 14 measurements are exploratory baselines,
not established performance regression gates.

## 2026-10-13 — Preview 14, macOS ARM64

Apple M1, macOS 27.0, with desktop and indexing activity. Every recorded case passed
its output checks. Reports retain all rounds, startup timings and input hashes.

| Workload series | Measurement | Results and evidence |
| --- | --- | --- |
| HTTP plain text v1 | GET, 10-byte UTF-8 response, one client, connection close | [Report](/benchmarks/http/#plain-text-baseline) · [Raw run](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce1/benchmarks/http-server/results-20261013.json) |
| HTTP object JSON v1 | Construct and serialize a Person per request | [Report](/benchmarks/http/) · [Raw run](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce1/benchmarks/http-server/json-results-20261013.json) |
| HTTP PostgreSQL + JSON v1 | Parameterized SELECT, object mapping and serialization; reused connection | [Report](/benchmarks/http/) · [Raw run](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce1/benchmarks/http-server/database-results-20261013.json) |
| CPU sum v1 | Sum integers 0 through 9,999 per operation | [Report](/benchmarks/execution/) · [Interpreter run](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce1/benchmarks/execution/results-20261013.json) · [Native run](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce1/benchmarks/execution/cpu-native-results-20261013.json) |
| JSON nodes serialize/deserialize v1 | Nested Unicode document, separate serialization and parsing cases | [Report](/benchmarks/execution/) · [Raw run](https://github.com/marinasundstrom/neoCLR/blob/c713d0ce1/benchmarks/execution/results-20261013.json) |

These are separate series, not interchangeable measures of runtime speed. The SQL
AOT case links the shared native host; other HTTP AOT cases use the standalone HTTP
profile. General execution uses the console profile. Host and GC policies matter.

The initial native CPU batches were shorter than one millisecond. They remain in
the initial raw run as pilot data; the report uses the subsequent 10,000-operation
native batches. Interpreter CPU and JSON cases use 16-operation batches. This
measurement-setting difference is explicit and must be retained in later comparisons.

## 2026-10-13 — development GC comparisons

The [GC benchmark report](/benchmarks/gc/) keeps distinct paired series:
fixed-fixture macOS JIT/AOT, newer streaming-library macOS AOT, and Windows x64
JIT/AOT. Each links immutable raw runs, including slower candidate samples. Only
the root observer changes within each pair. These are development experiments,
not new Preview 14 release measurements or cross-platform rankings.

The later [Windows GC-versus-wait series](/benchmarks/gc/#windows-gc-versus-waits-2026-10-13)
compares legacy sleep and readiness waits on the same VM, with separate diagnostics.
It retains all rounds and the earlier independent profiling run. Its absolute rates
must not be compared against a different hosted VM's root-observation results.

## How history will grow

Keep raw runs immutable, including slower results. Corrections and replacement runs
will link back to the original evidence. Changes to payloads, queries, operation
semantics or timing protocols start a new comparable series. Runtime implementation
changes retain the workload identity so their effect can be measured.

Our goal is stable or improving performance with correct behavior. Establishing a
regression gate requires repeated measurements on a controlled host and thresholds
based on observed noise. A small change in one short run is not proof of an improvement
or regression. No automatic statistical acceptance gate is active yet.

See the [benchmark history policy](https://github.com/marinasundstrom/neoCLR/blob/main/docs/benchmark-history.md)
for provenance and comparison requirements. Windows x64 GC comparisons are now recorded separately above; Windows ARM64 and
sustained cross-runtime measurements remain future work. A [local development comparison](/benchmarks/http-comparison/)
now retains Kestrel, Go and Node pilot results; publication is on hold pending
neoCLR HTTP improvements.

The [dense object-graph series](/benchmarks/gc/#dense-object-graphs-2026-10-13)
is a separate collection-only native runtime benchmark. It includes chain, wide,
free-hole and sparse-buffer controls; it must not be combined with HTTP throughput
or cross-platform ratios.

The [allocation-throughput series](/benchmarks/gc/#allocation-throughput-2026-10-13)
compares complete validation plus separate search against a combined walk. Its
allocation-only timings exclude final collection; the linked pacing experiment
remains separate and is not a shipping or HTTP performance claim.

The [host-handle admission series](/benchmarks/gc/#host-handle-admission-2026-10-13)
compares separate validation/membership walks with a combined walk. Early/late-base
and small-heap controls remain visible. It measures root creation/read/release,
not collection time; its separate AOT HTTP comparison shows no material gain.
