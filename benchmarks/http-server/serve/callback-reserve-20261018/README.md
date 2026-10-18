# Reserve the known first Promise callback — 2026-10-18

The [caller profile](../caller-profile-20261018/README.md) identifies 2,687 empty
callback arrays immediately replaced by first-add growth. Promise.Register now
creates its lazily allocated callback list with capacity one, since the first
callback is already known. Promises without callbacks still allocate no callback
list. Additional registrations retain geometric growth and deferred FIFO posting.

This applies the existing capacity contract, comparable to
[.NET List<T>(capacity)](https://learn.microsoft.com/en-us/dotnet/api/system.collections.generic.list-1.-ctor?view=net-10.0),
without changing public Task/Promise semantics or replacing reference-type tasks.
For multiple callbacks, starting at one can add growth steps versus the former
first-add capacity of four; the measured Serve workload predominantly benefits
from avoiding the zero-capacity array. The tradeoff must remain explicit.

The shared Task completion fixture tests callbacks before/after settlement,
duplicate completion/cancellation, deferred ordering, reentrant posting and queue
reuse. It now registers nine callbacks before both completion and cancellation to
exercise growth beyond the initial reservation. Public API signatures stay unchanged.

## Validation and diagnostics

The expanded shared Task fixture passes interpreter, JIT and AOT.
[Contracts and tool hashes](evidence/contracts.json.gz). The unchanged API reference
passes its matching source/reference fingerprint check.

Same 200 measured +20 warm-up +1 stop exchanges, including startup/shutdown:

| Counter | Header-reservation baseline | Callback reservation |
| --- | ---: | ---: |
| Allocations | 61,601 | 58,914 |
| Allocated payload bytes | 3,327,856 | 3,212,315 |
| Collections | 19,062 | 19,493 |
| Root publications | 710,228 | 702,167 |
| Peak live payload bytes | 46,528 | 45,632 |
| Peak arena bytes | 68,768 | 67,776 |

Allocations fall 4.36%, bytes 3.47%, root publications 1.13%. The independent caller
capture agrees exactly with the allocation count, with no dropped records. This
workload removes exactly one allocation per first registration; no extra callback
growth offsets that saving here. Multi-callback applications may behave differently.
All allocations reclaim at shutdown and GC failures are zero. Collections rise
2.26%; idle/collection counts are timing-sensitive, as the caller-profile slice
explains. Diagnostic runs overlap tooling/other diagnostics, so elapsed GC time is
not comparative. [Counters](evidence/gc-counts.json),
[callers](evidence/callers-summary.json.gz), [requests](evidence/diagnostic.json).

Build with build-native-class-library.py and the shared task-completion build.py;
run its verify.py with the resulting cohort. Existing compiler/backend/native
host hashes and complete commands are retained in the compressed manifests.
## Sustained-load protocol and conditions

Each trial is 60 seconds with 16 server lanes, 64 client slots, ten-byte responses,
HTTP/1.1 connection-close and queued handler work. Baseline is header reservation;
both neoCLR builds use experimental paced GC, Go uses GOMAXPROCS=1/GOGC=100.
Order is baseline/candidate/Go then Go/candidate/baseline. No builds, diagnostic
captures or archive compression overlap timing. The generator shares the host.

Every offered arrival counts; throughput excludes drain completions. Qualification
requires all arrivals valid, offered p99 ≤250 ms and generator-lag p99 ≤5 ms.
All listed misses are generator_inflight_limit arrivals, retained as failures;
they must not be dropped from success denominators or reclassified as responses.
CPU/completion under losses is descriptive, not a qualified sustained-rate result.

The first 400/sec comparison fails for both neoCLR builds. A fixed 300/sec fallback
also fails qualification, including generator lag in both lossless Go controls.
Recorded process snapshots show competing CPU activity; these snapshots are not
proof that it caused the misses. During candidate-2 of that 300/sec sequence, the
author reports stopping competing activities. The remainder is retained as a
mixed-condition sequence. A new 400/sec comparison then runs from the beginning
with unchanged executables; its results are separate, not replacements for failures.
[Condition-change record](evidence/conditions-change.json).


### Initial 400/sec

| Arm | Valid/offered | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/initial-400-baseline-1.json) | 23851/24000 | 397.50 | 149 | 2.1199 | 147.90 | 3.80 | fail |
| [candidate-1](evidence/initial-400-candidate-1.json) | 23768/24000 | 396.12 | 232 | 2.0772 | 213.60 | 5.55 | fail |
| [go-1](evidence/initial-400-go-1.json) | 24000/24000 | 400.00 | 0 | 0.1917 | 9.99 | 4.32 | pass |
| [go-2](evidence/initial-400-go-2.json) | 24000/24000 | 400.00 | 0 | 0.2154 | 22.50 | 9.81 | fail |
| [candidate-2](evidence/initial-400-candidate-2.json) | 23936/24000 | 398.92 | 64 | 2.1504 | 174.58 | 2.75 | fail |
| [baseline-2](evidence/initial-400-baseline-2.json) | 23747/24000 | 395.68 | 253 | 2.1625 | 218.09 | 16.06 | fail |

### 300/sec fallback; activity change during candidate-2

| Arm | Valid/offered | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/initial-300-baseline-1.json) | 17914/18000 | 298.55 | 86 | 2.4318 | 243.63 | 25.91 | fail |
| [candidate-1](evidence/initial-300-candidate-1.json) | 17968/18000 | 299.45 | 32 | 2.3176 | 175.47 | 9.76 | fail |
| [go-1](evidence/initial-300-go-1.json) | 18000/18000 | 300.00 | 0 | 0.2606 | 17.82 | 7.22 | fail |
| [go-2](evidence/initial-300-go-2.json) | 18000/18000 | 300.00 | 0 | 0.2594 | 17.07 | 6.73 | fail |
| [candidate-2](evidence/initial-300-candidate-2.json) | 17928/18000 | 298.78 | 72 | 2.2073 | 231.24 | 16.37 | fail |
| [baseline-2](evidence/initial-300-baseline-2.json) | 17707/18000 | 295.12 | 293 | 2.1048 | 321.11 | 32.92 | fail |

### 400/sec repeat after the author-reported activity change

| Arm | Valid/offered | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/repeat-400-baseline-1.json) | 23622/24000 | 392.98 | 378 | 2.0501 | 221.86 | 11.60 | fail |
| [candidate-1](evidence/repeat-400-candidate-1.json) | 23682/24000 | 394.70 | 318 | 2.1058 | 222.85 | 7.04 | fail |
| [go-1](evidence/repeat-400-go-1.json) | 24000/24000 | 400.00 | 0 | 0.2033 | 6.71 | 2.76 | pass |
| [go-2](evidence/repeat-400-go-2.json) | 24000/24000 | 400.00 | 0 | 0.2062 | 5.82 | 2.27 | pass |
| [candidate-2](evidence/repeat-400-candidate-2.json) | 23994/24000 | 399.88 | 6 | 1.9922 | 31.54 | 0.49 | fail |
| [baseline-2](evidence/repeat-400-baseline-2.json) | 23990/24000 | 399.23 | 10 | 2.0381 | 83.25 | 1.00 | fail |

## Interpretation and next work

The repeat candidate uses about 2.7% more CPU/completion in the first pair and
2.3% less in the second. All four neoCLR repeat trials miss arrivals; both Go
controls pass. There is no repeatable CPU improvement or qualified candidate
300 or 400 requests/sec point established in this session. Earlier successful header-reservation
trials remain historical evidence, not proof that this candidate sustains that rate.
Neither global generator failure nor external activity alone explains all results:
the fresh Go controls pass while neoCLR still loses arrivals.

Keep the small allocation change for its directly verified reduction on Serve,
with explicit multi-callback growth costs. It does not resolve latency/admission
variation. Next work should correlate stalls with scheduler/GC activity and isolate
load-generator interference, ideally with a separate generator machine. Promise
result storage remains a substantial allocation target requiring broader semantics
and retention tests; no representation change is included here.

Reproduce each sequence with root-validation/compare.py, the matching build
folders, --rates 400 (or 300), --duration 60 and a fresh --output directory.
All compact reports link lossless raw request/resource reports with hashes.
