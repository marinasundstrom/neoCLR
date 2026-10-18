# Invocation-owned encoding providers — 2026-10-18

Follow-up profiling of the [receive-cache candidate](../receive-pool-20261018/README.md)
finds 1,989 allocations attributed to Encodings.get_Utf8 in the fixed diagnostic
workload. Both providers are stateless; only CreateEncoder/CreateDecoder own mutable
session state. Cache providers in invocation-owned static fields while continuing
to create independent sessions. Empty-array caching remains a separate investigation;
this slice changes no array representation, compiler or GC policy.

## Contract and comparison

.NET separates its [UTF-8 provider](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.utf8?view=net-10.0)
from [stateful decoders](https://learn.microsoft.com/en-us/dotnet/api/system.text.encoding.getdecoder?view=net-10.0).
neoCLR's existing strict errors, BOM-as-data behavior, UTF-8 strings and size limits
remain unchanged; this is not a change to .NET replacement-fallback or preamble
semantics. Providers may be shared; consumers must not depend on getting a new
identity on each access. Session instances remain independent.

The cache uses existing invocation-owned static storage. Native contexts remain
thread-affine. Interpreter first readers may create competing stateless providers;
this is not a once-only singleton identity or general Lazy<T> publication contract.
No partial object is stored, no session state is shared, and the normal static root
retains cached references until context teardown. The cache holds at most two
providers plus its static-storage bookkeeping. Existing static-storage tests cover
context isolation/reset; the new consumer exercises providers under allocation
pressure, without extending that hosting contract.

The first version used concrete provider types for both fields. The three-mode
consumer passed, but UTF-8-only Serve compilation failed with a missing/ambiguous
ASCII interface implementation. Shared static-owner layout reachability appears to
introduce the unused concrete implementation without its methods. The final fields
use their Encoding interface contract, and Serve compiles. The [failed build](evidence/concrete-field-failure.json.gz)
is retained; the broader native interface-selection gap is not fixed or redefined
as a language restriction by this change.

## Validation

[Contracts.rvn](Contracts.rvn) checks strict UTF-8/ASCII errors, BOM preservation,
independent interleaved decoders, finished-session isolation, independent encoder
output, ASCII rejection and repeated access after allocation pressure. Interpreter,
JIT and AOT pass. [Evidence](evidence/contracts.json.gz). The shared verify.py now
accepts an optional --source and --expected-line; defaults preserve its Task fixture.

Rebuild the four-library cohort, use the shared task-completion build.py for Serve,
and run verify.py with --source pointing at Contracts.rvn and --expected-line
'Encoding provider contracts passed'. Compressed build manifests retain compiler,
backend, library, object, native-host and executable hashes. The matching API
reference and XML summaries are refreshed; no public signature changes.

## Diagnostic allocations

Same 200 measured requests, 20 warm-up requests and one stop exchange, including
startup/shutdown. Separate caller-attribution runs agree with the allocation total.
Diagnostics overlapped tooling and are excluded from comparative elapsed-time claims.

| Counter | Receive-cache baseline | Encoding cache | Change |
| --- | ---: | ---: | ---: |
| Allocations | 64,693 | 62,706 | −3.07% |
| Allocated payload bytes | 3,375,544 | 3,343,768 | −0.94% |
| Collections | 19,129 | 18,405 | −3.78% |
| Root publications | 717,297 | 713,322 | −0.55% |
| Peak live payload bytes | 46,480 | 46,528 | +48 bytes |
| Peak arena bytes | 68,744 | 69,072 | +328 bytes |

All allocations reclaim at shutdown and GC failures are zero. get_Utf8 attribution
falls from 1,989 allocations to three initialization allocations (provider and static
storage), not three provider objects. Counts charge the current published frame,
not an allocated-type census. [Counters](evidence/gc-counts.json),
[baseline attribution](evidence/baseline-allocation-summary.json),
[candidate attribution](evidence/allocation-summary.json),
[request diagnostics](evidence/diagnostic.json). No throughput gain is inferred
from these counts.

## Sustained Serve/Go comparison

Same 400 offered requests/sec, 60-second windows, 16 server lanes and 64 client
slots; baseline/candidate/Go then Go/candidate/baseline. Baseline is the receive-cache
cohort. Both neoCLR builds use experimental paced GC; Go uses GOMAXPROCS=1 and
GOGC=100. Builds and diagnostics finish before timing. The load generator shares
the host; these are local tested points, not production or maximum capacity.

Every offered arrival counts in the success denominator. Timed throughput excludes
drain completions. Provisional qualification requires all arrivals valid, offered
p99 ≤250 ms and generator-lag p99 ≤5 ms. Full compressed reports retain every
request and sampled resource record.

| Arm | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/baseline-1.json) | 399.98 | 0 | 2.0605 | 14.56 | 0.63 | pass |
| [candidate-1](evidence/candidate-1.json) | 399.98 | 0 | 2.0376 | 16.46 | 0.64 | pass |
| [go-1](evidence/go-1.json) | 400.00 | 0 | 0.2092 | 4.67 | 1.44 | pass |
| [go-2](evidence/go-2.json) | 399.95 | 0 | 0.2054 | 4.60 | 1.31 | pass |
| [candidate-2](evidence/candidate-2.json) | 399.98 | 0 | 2.0338 | 9.78 | 0.63 | pass |
| [baseline-2](evidence/baseline-2.json) | 399.98 | 0 | 2.0588 | 18.91 | 0.64 | pass |

All six trials validate 24,000 arrivals each and pass the provisional target. The
candidate uses about 1.1% and 1.2% less sampled CPU/completion than its corresponding
baseline round. This small difference repeats in both orders, but two short pairs
do not provide a statistical guarantee or establish a higher sustainable request
rate. Latency is mixed: candidate p99 is slightly higher in the first pair and
lower in the second. No general latency improvement is claimed.

Candidate CPU is roughly 9.7–9.9 times the matching Go control here. Use these
paired results rather than treating Go CPU from earlier, noisier runs as a fixed
constant. Streaming/ASCII semantics are tested separately; this HTTP load workload
principally measures UTF-8 provider reuse. No higher rate is tested in this slice.
