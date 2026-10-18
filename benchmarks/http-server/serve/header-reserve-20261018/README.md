# HTTP header reservation — 2026-10-18

The encoding-cache allocation profile attributes the largest allocation groups to
array reservation. The request decoder starts its header ArrayList empty, then
reallocates through 4, 8, 16, 32, 64 and 128 elements for ordinary headers. Reserve
min(128, configured header limit) once in the decoder constructor. Larger headers
retain normal geometric growth and byte-by-byte limit validation.

This applies the existing capacity contract, comparable to [.NET List<T>(capacity)](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.-ctor?view=net-10.0),
without changing ArrayList defaults or adding pooling. It trades up to 128 bytes of
initial payload per decoder for fewer allocations/copies on ordinary requests.
Tiny requests may retain more capacity; idle/pending decoders also reserve it.
No transport buffer escapes, ownership changes, shared mutable arrays or GC policy
changes. Streaming and buffered requests use the same header parser.

The regression fixture tests exact and one-byte-over header limits at 127, 128,
129, 256 and 512 bytes, plus a one-byte limit. Larger inputs arrive in separate
sends crossing the reserved prefix. These are correctness tests with 60-second
read/write budgets, not performance qualifications. Public API signatures remain
unchanged.

## Diagnostic counters

Same 200 measured requests, 20 warm-up requests and one stop exchange as the
encoding-cache baseline, including startup/shutdown. Separate allocation-site and
GC runs agree on 61,601 allocations. Diagnostics overlapped build/tooling work;
elapsed GC time is not comparative evidence.

| Counter | Encoding-cache baseline | Header reservation |
| --- | ---: | ---: |
| Allocations | 62,706 | 61,601 |
| Allocated payload bytes | 3,343,768 | 3,327,856 |
| Collections | 18,405 | 19,062 |
| Root publications | 713,322 | 710,228 |
| Peak live payload bytes | 46,528 | 46,528 |
| Peak arena bytes | 69,072 | 68,768 |

Allocations fall 1.76%, bytes 0.48% and root publications 0.43%, but collections
increase 3.57%. All allocations reclaim at shutdown with zero GC failures.
No throughput gain follows from these counters.

## Validation

All 12 cases pass in interpreter, JIT and AOT (36 outcomes). The fixture uses
buffered Accept to isolate the shared parser; the unchanged Serve load fixture
exercises server-managed concurrency separately. [Contracts](evidence/contracts.json.gz).
The matching API snapshot passes its reference/source fingerprint check; no public
signature changes. Compressed build manifests retain exact compiler, backend,
library, native host and executable provenance.

Reproduce by building the four-library cohort with build-native-class-library.py,
then task-completion-20261018/build.py with --source Probe.rvn. Run verify.py with
--build, --library, --runtime, --backend, --host and --output. Build the unchanged
Serve/Server.rvn using the same helper without --source. Run allocation-sites.py
and the profile-native.c diagnostic separately from root-validation/compare.py.

## Sustained Serve/Go comparison

400 offered requests/sec for 60 seconds, 16 server lanes, 64 client slots,
ten-byte response, HTTP/1.1 connection-close and queued handler work. Baseline is
the encoding-cache cohort. Both neoCLR builds use experimental paced GC; Go uses
GOMAXPROCS=1/GOGC=100. Baseline/candidate/Go then Go/candidate/baseline order.
No builds or diagnostics overlap timing. The generator shares the host.

Every offered arrival counts; drain completions do not inflate timed throughput.
Provisional qualification requires every arrival valid, offered p99 ≤250 ms and
generator-lag p99 ≤5 ms. Each compact report links its lossless compressed report
with per-request and sampled resource records, build provenance and hashes.

| Arm | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/baseline-1.json) | 399.98 | 0 | 2.0393 | 21.50 | 0.65 | pass |
| [candidate-1](evidence/candidate-1.json) | 399.98 | 0 | 2.0184 | 8.82 | 0.63 | pass |
| [go-1](evidence/go-1.json) | 400.00 | 0 | 0.2621 | 6.19 | 2.37 | pass |
| [go-2](evidence/go-2.json) | 400.00 | 0 | 0.2733 | 6.31 | 2.63 | pass |
| [candidate-2](evidence/candidate-2.json) | 400.00 | 0 | 2.0158 | 10.19 | 0.63 | pass |
| [baseline-2](evidence/baseline-2.json) | 399.98 | 0 | 2.0347 | 11.91 | 0.64 | pass |

All trials validate 24,000 arrivals. Candidate CPU/completion is 1.02% lower in the
first pair and 0.93% lower in the second. Offered p99 is lower in both pairs, but
baseline latency itself varies substantially. Two short pairs do not establish a
statistical guarantee, a general latency reduction or a higher sustained rate.

Candidate CPU remains about 7.4–7.7 times its paired Go controls. Go CPU varies
between benchmark sessions; do not compare this ratio directly with the earlier
encoding-cache session's 9.7–9.9 ratio as an improvement. The relevant neoCLR gain
is the small within-session difference above. No higher offered rate is tested.

Reproduce timing with root-validation/compare.py --baseline pointing to the
encoding-cache candidate-final build, --candidate to this candidate, --go to the
same Go build, --rates 400 --duration 60 --output a fresh directory. These are local
sustained test points, not production capacity or maximum throughput.
