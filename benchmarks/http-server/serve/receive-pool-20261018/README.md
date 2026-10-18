# Bounded HTTP receive-buffer reuse — 2026-10-18

Author-directed continuation of the [queue/capacity investigation](../task-queue-20261018/README.md)
and [pooling/Span ownership discussion](../../../../docs/http-server-design.md#capacity-reservation-and-pooling-investigation--2026-10-18).

## Scope and ownership

The candidate caches one 256-byte and one 4 KiB receive array per HttpServer.
It is private to the cooperative dispatcher, with at most 4,352 idle payload bytes
retained; excess returned buffers are dropped, and Close clears cached references.
This is an internal two-size scratch cache, not a public ArrayPool or Span API.
It does not pool response storage or alter native transfer copying.

Buffered-request decoders copy every consumed byte into owned storage. An exchange
returns its block only after the receive task has settled, or before another receive
when growing after a partial request. Cleanup detaches the exchange reference first.
Cancellation requests cancel I/O but do not directly return an outstanding buffer;
its completion callback owns cleanup. Queued-accept cancellation has no buffer.
The completed request can outlive the scratch lease. The nullable exchange field
represents no active buffer without allocating an empty array.

AcceptHeaders transfers storage to HttpBodyInput, so streaming exchanges retain
fresh owned arrays and bypass this cache. TLS uses the same completed-read boundary;
this slice does not change TLS transport internals. Uncleared scratch tails remain
private and are never parsed beyond the returned count. A future public pool must
specify clearing and borrowing independently, rather than inheriting this internal
contract. This follows .NET's separation of storage ownership from views; see the
linked design research for ArrayPool, Memory and Span comparisons.

## Validation and experimental history

The new Probe holds a 4,097-byte request while later exchanges receive different
payloads, including fragmented 4,097/5,000-byte bodies and short tails. It verifies
that the retained request is unchanged, malformed headers and premature EOF fail,
cancellation completes, and subsequent requests still echo exact bytes. Explicit
OnCompleted is an outcome adapter for cancellation observation. The fixture uses
60-second read/write budgets to accommodate interpreter execution; the separate
cancellation case uses a 500 ms timer. It is a correctness test, not a load benchmark.

The existing headers-first server fixture checks stream ownership, fragmentation,
invalid framing, EOF, cancellation and close. No public pool test surface or
friend-assembly access is introduced.

An initial candidate replaced returned scratch with Array.Empty. Since this runtime
currently allocates that empty array, its diagnostic allocation count increased
from 65,150 to 65,167 despite fewer payload bytes. The final candidate uses null
instead. Initial interpreter validation also failed while builds were active;
those failures are retained and not counted as passes. Final validation is run
without competing builds. Compiler admission rejected a nullable-array local in
an intermediate version; supported nullable-field patterns avoid that unrelated
temporary backend gap without changing public contracts.

The unchanged baseline also fails the original 15-second streaming fixture in
interpreter case 2. With a fixture-only 60-second read/write budget, all nine
streaming cases pass in interpreter/JIT/AOT. Production timeout defaults are
unchanged. [Buffered ownership results](evidence/contracts-final.json),
[streaming results](evidence/streams-long.json), [baseline failure](evidence/streams-baseline.log),
[candidate default-budget failure](evidence/streams-default-budget.log). The larger
fixture budget qualifies ownership behavior, not interpreter performance within
the default timeout. Earlier failed reports/logs remain in evidence.

## Allocation diagnostic

Same 200 measured +20 warm-up +one stop exchanges as the queue baseline, including
startup/shutdown. Diagnostics ran separately from sustained-load timing; elapsed
GC/profile time is not a matched speed comparison.

| Counter | ArrayQueue baseline | Receive-cache candidate | Change |
| --- | ---: | ---: | ---: |
| Allocations | 65,150 | 64,693 | −0.70% |
| Allocated payload bytes | 3,495,480 | 3,375,544 | −3.43% |
| Collections | 18,578 | 19,129 | +2.97% |
| Root publications | 718,247 | 717,297 | −0.13% |
| Peak live payload bytes | 49,283 | 46,480 | −2,803 bytes |
| Peak arena bytes | 74,416 | 68,744 | −5,672 bytes |

All candidate allocations reclaim at shutdown; GC failures are zero. Nullable
exchange storage removes empty-array allocations as well as enabling payload reuse,
so the reduction must not be attributed entirely to the cache. More collections
despite fewer allocated bytes show why allocation counts alone do not establish a
speedup. [GC counters](evidence/gc-counts.json), [diagnostic requests](evidence/diagnostic.json).

Reproduce with the shared task-completion build.py against the rebuilt four-library
cohort, separately building the normal Serve source and Probe.rvn. verify.py accepts
--build, --library, --runtime, --backend, --host and --output. For streaming, build
the retained evidence/StreamingProbe.rvn and run verify-streaming.py with
--contracts pointing at the successful buffered report, --build and --output.
Build manifests retain commands/hashes. No new public API or native GC policy is
introduced; the unchanged reference is regenerated and its source snapshot refreshed.

## Sustained-load comparison

Unchanged 400 offered requests/sec, 60 seconds each, 16 server lanes and 64 client
slots. Order baseline/candidate/Go, then Go/candidate/baseline. Baseline is the
ArrayQueue candidate. Both neoCLR builds use experimental paced GC; Go uses
GOMAXPROCS=1 and GOGC=100. All builds/diagnostics finished before timing. The load
generator shares the host; this is not production capacity or a maximum-rate test.

Every offered arrival remains in the success denominator. Timed throughput excludes
drain completions. The provisional target requires all arrivals valid, offered p99
≤250 ms and generator-lag p99 ≤5 ms. Raw compressed reports retain every request
and resource sample; no failed trial was discarded.

| Arm | Timed req/s | Misses | CPU ms/completion | Offered p99 ms | Lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline-1](evidence/baseline-1.json) | 388.03 | 717 | 2.0995 | 242.70 | 464.44 | fail |
| [candidate-1](evidence/candidate-1.json) | 399.98 | 0 | 2.0601 | 44.34 | 0.65 | pass |
| [go-1](evidence/go-1.json) | 400.00 | 0 | 0.2842 | 5.65 | 1.96 | pass |
| [go-2](evidence/go-2.json) | 400.00 | 0 | 0.2717 | 6.24 | 2.37 | pass |
| [candidate-2](evidence/candidate-2.json) | 399.98 | 0 | 2.0538 | 16.79 | 0.66 | pass |
| [baseline-2](evidence/baseline-2.json) | 399.98 | 0 | 2.0601 | 13.48 | 0.68 | pass |

Both candidate trials validate all 24,000 requests. Sampled CPU/completion is about
1.9% lower in the first pair, whose baseline suffered severe generator lag and
717 admission misses. The cleaner reverse-order pair differs by only about 0.3%,
with candidate offered p99 slightly higher. This does not establish a convincing
CPU speedup or higher sustained capacity. Candidate CPU remains roughly 7.2–7.6
times the corresponding Go control under these explicit settings.

Keep the allocation/byte reduction separate from timing: modest allocation savings
are demonstrated, but additional pooling is not justified by a throughput claim
from this experiment. The 4 KiB receive cache is covered by fragmented-body tests;
this tiny-response capacity workload primarily exercises the 256-byte class.
