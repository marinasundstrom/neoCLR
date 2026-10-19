# HTTP performance investigation — 2026-10-19

Fresh profiles identify repeated runtime work as the leading optimization target.
The best bounded candidates are single-result Promise storage and checked bulk
response copying. Larger runtime work should target root publication, heap lookup
and collector pacing. This investigation changes diagnostic tooling only; it does
not claim a server speedup or promote the experimental collector.

## Evidence and scope

Reuse the exact binaries/guest from the [24-trial collector comparison](../memory-gc-20261019/README.md).
The synchronous baseline is explicitly paced, with the same 1 MiB heap as the
experimental concurrent collector. Plain HTTP, AOT, macOS ARM64, 16 server lanes,
64 client slots and connection-close remain the contract. Source/binary identities
are retained in [inputs](evidence/inputs.json).

Four independent 12-second diagnostic windows sample each process for five seconds
at one-millisecond intervals, beginning three seconds into load. Plain offers
400 requests/sec; bulk offers 100 requests/sec with an 8192-byte response. These
are wall-clock stack samples, including waiting threads, not CPU durations.
Exclusive samples partition a thread; inclusive callers overlap. One capture per
case localizes work but does not establish stable percentages across machines.
Do not combine thread percentages or infer an achievable speedup from them.

| Exclusive main-thread symbol | Sync plain | Concurrent plain | Sync bulk | Concurrent bulk |
| --- | ---: | ---: | ---: | ---: |
| Root publication (`neoclr_gc_stack_roots_v1`) | 18.3% | 17.4% | 18.8% | 18.6% |
| String length checks (`strlen`) | 9.2% | 7.7% | 7.9% | 7.5% |
| Allocation (`neoclr_gc_allocate_v1`) | 17.4% | 18.1% | 6.0% | 7.7% |
| Exact heap-base validation (`validated_bases`) | 7.1% | 6.9% | 1.2% | 3.5% |
| Indexed collection (`collect_indexed`) | 19.9% | 6.6% | 26.8% | 4.0% |
| Concurrent result polling/apply (`cycle_poll`) | — | 5.4% | — | 9.1% |
| Memory copying (`memmove`) | <0.1% | 2.7% | <0.1% | 10.5% |

Main-thread sample counts are 3499, 2801, 3478 and 2823 respectively. Concurrent
worker condition-variable waits account for 81.9% of plain and 79.8% of bulk wall
samples. This does not imply that tracing costs nothing, or measure worker CPU
utilization. It makes additional tracing workers a weak first hypothesis. The
bulk call tree shows memory-copy samples beneath `cycle_start`, consistent with
snapshot capture copying the used arena; not every memmove sample is GC work.

`EncodeResponse` appears inclusively in 30.8% of synchronous bulk and 36.0% of
concurrent bulk main-thread samples, versus 5.2%/5.4% for plain. Those counts include
runtime/GC callees, not just HTTP byte copying. The handler also constructs its
body byte by byte, so this benchmark includes application payload construction.

All admitted responses validated, server totals matched and shutdown was clean.
Sampled plain windows admitted 4690/4800 synchronous and 4265/4800 concurrent;
bulk admitted 1200/1200 and 1168/1200 respectively. Remaining arrivals were unsent
slot-limit misses. Concurrent bulk p99 was approximately 803 ms under sampling,
unlike the uninstrumented comparison: retain this disturbance, do not reinterpret
it as a production regression or replace the earlier throughput results. Process
and host activity snapshots remain in the raw reports.

A separate allocation-caller diagnostic at 20 plain requests/sec measures 200
requests, 20 warmup requests and one shutdown exchange plus startup. It records
54,715 allocations and 3,113,755 payload bytes, with zero lost records. Promise
result backing arrays account for 2,909 allocations and 1,115,421 bytes (**35.8%**
of payload bytes). Callback backing arrays account for 2,687 allocations and
67,175 bytes. These are array-only counts, not whole Promise object cost; payload
bytes exclude native allocation headers and are not live-memory size. All caller
counts/bytes reconcile. Timing in this instrumented run is not capacity evidence.

## Ranked implementation candidates

1. **Store a Promise result without a list.** `Promise<T>` currently constructs
   `ArrayList<T>(1)` and uses its count as completion state. A direct result field
   and explicit settlement state could remove list/array allocations and accesses.
   Confirm generic initialization and precise retention for references, structs and
   large unions in interpreter/JIT/AOT before choosing the representation. Retain
   multiple reads/awaiters, cancellation, duplicate-completion behavior and queued
   callbacks. Pooling Promise instances is a separate and riskier lifetime change;
   do not combine it with this step. The result payload still needs storage, so
   35.8% is an opportunity indicator, not a predicted byte saving.

2. **Use checked bulk operations in buffered response encoding.** `EncodeResponse`
   allocates an exact snapshot, then copies every header/body byte in Raven loops.
   The native byte-view kernel already has an overlap-safe copy operation, but a
   callable managed copy API/backend integration remains open. Qualify that path
   and use it for contiguous owned body storage. Preserve the current snapshot
   isolation, HEAD/no-body status behavior, bounds and error-before-mutation rules.
   Do not replace a snapshot with an alias to mutable application content. Retaining
   a readonly memory view alone cannot prevent mutation through another alias.
   Later, explicit ownership transfer or segmented sends could remove the copy,
   at the cost of a different ownership contract and more complicated partial sends.

3. **Make root descriptors cheap to publish.** `root-probe.c` repeatedly checks
   string lengths and walks frame storage; the profiles show this in both modes.
   A registered immutable descriptor, validated at image load and tied to image
   lifetime, could move structural checks off the hot path. Keep current mutable
   descriptor checks until that contract exists. Validate malformed metadata,
   reused image addresses, unload/reload, spills and parked frames. Simply caching
   a pointer or deleting `strlen` would violate current corruption detection.
   This requires compiler/runtime contract work and separate Raven documentation.

4. **Avoid full heap walks on ordinary allocation and owner lookup.** The allocator
   validates every block while finding a first fit; `validated_bases` walks the
   entire heap even after finding a root. Consider maintained free-span and exact
   allocation-base indexes, with explicit mutation/invalidation ownership. Costs:
   extra metadata, updates on splitting/coalescing and snapshot coordination.
   Preserve the existing malformed-tail failure before heap/output mutation, or
   explicitly design and validate a new trusted-metadata boundary. No early-exit
   lookup is justified by this profile alone.

5. **Pace concurrent capture and synchronous work by useful progress.** Current
   concurrent idle calls bypass the 16-allocation/64-boundary admission gate;
   each capture allocates/copies the used arena. Instrument owner capture, worker
   tracing, validation/apply, fallback and rejected/discarded work separately.
   Then test coalescing idle triggers and allocation-byte/pressure-based pacing,
   possibly reusing private snapshot storage. Preserve allocation-pressure fallback,
   root/epoch invalidation, eventual reclamation while idle and joined shutdown.
   Tightening pacing may retain more memory or shift pauses; test both payloads
   and both collectors. CPU is diagnostic, not a veto on this unoptimized design.
   These samples do not give exact phase duration or identify an optimal threshold.

6. **Follow with protocol and transport work.** The response writer still caps
   direct sends at 4096 bytes. Test fewer/larger transfers with retained memory,
   partial-send, cancellation and fairness coverage rather than assuming fewer
   sends are faster. HTTP keep-alive would amortize accepts and per-connection state,
   but needs unread-body draining, read-ahead, timeouts and connection supervision.
   It changes the workload: keep the connection-close baseline and add a separate
   persistent-connection comparison. Parser materialization and byte-at-a-time
   access deserve a later focused profile. More Serve lanes alone will not remove
   work on the single owner thread; multicore request execution requires separate
   runtime/scheduler/GC ownership qualification.

## .NET comparison and choices

.NET separates Task's reusable-result semantics from lower-allocation ValueTask
paths; the latter can wrap reusable completion sources and carry stricter usage
constraints. That motivates efficient result storage here, not an automatic new
public type or loss of Task semantics. See Microsoft's
[ValueTask design explanation](https://devblogs.microsoft.com/dotnet/understanding-the-whys-whats-and-whens-of-valuetask/).

Kestrel's documented optimizations include reducing per-connection allocations and
combining socket operation state. Those support investigating ownership and
operation reuse after basic storage costs, rather than introducing compatibility
array overloads. See [ASP.NET Core 6 performance work](https://devblogs.microsoft.com/dotnet/performance-improvements-in-aspnet-core-6/).
These are primary-source design precedents, not performance comparisons against
this neoCLR binary. No .NET throughput number is inferred.

## Reproduction and next acceptance test

```sh
python3 benchmarks/http-server/serve/diagnose.py \
  --executable target/http-memory-gc-cohort/app-paced --sample \
  --rate 400 --duration 12 --output target/http-investigate-20261019/sync-plain
# Repeat with the apply executable, then --workload bulk --rate 100 for both.
python3 docs/experiments/concurrent-gc-http/sample-summary.py \
  target/http-investigate-20261019/sync-plain/sample.txt \
  --guest-build target/http-memory-gc-cohort/build.json
python3 benchmarks/http-server/serve/caller-profile-20261018/capture.py \
  --build target/http-memory-gc-cohort --output target/http-investigate-20261019/callers
```

Caller relinking uses the matching Xcode 26.2 toolchain/SDK. All four sampling
summaries reconcile with macOS sample's collapsed leaf totals. Compressed raw
samples, request reports and allocation attribution are in [evidence](evidence/manifest.json).
The sampling tool now accepts both workloads and validates their exact body sizes.
No production source, API or GC policy changed; no website capability update is needed.

Implement and commit candidates separately. First qualify semantics, then verify
allocation/copy reductions, then run reversed-order **uninstrumented** plain and
bulk comparisons at the existing admitted and overload rates. Report all misses,
p50/p95/p99, completed throughput, CPU and memory. A convincing result must preserve
correctness and improve useful throughput/latency or resource cost without hiding
regressions. Exact GC pause distributions, HTTPS, persistent connections and
cross-platform results remain open.
