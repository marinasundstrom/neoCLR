# Private concurrent tracing through real Serve execution

This diagnostic adapter brings the bounded snapshot worker into the actual
asynchronous HTTP Serve host. **The existing paced synchronous collector remains
solely responsible for live reclamation.** There is no mark import, concurrent
sweep, collector-selector change or HTTP speedup claim.

## Integration

`build.py` verifies a qualified native Serve build and the existing walker
qualification, then relinks the retained guest with generated copies:

- Rename the existing native `boundary` and `idle` functions inside `shadow.c`;
  wrappers poll for completed private traces and then call the original functions.
  All collection behavior, root publication and pressure fallback remain enabled.
- Sample one in 64 successful boundary/idle opportunities. Capture at most one
  private snapshot using the existing 1 MiB / root / parked-frame limits; never
  queue another job while one is outstanding. This is diagnostic sampling, not a
  proposed collection cadence. It can still produce many copies per request.
- Preserve all POC invalidation hooks on host-root changes, park/resume/discard,
  collection and entry/teardown. Compare the owner epoch only after the worker has
  completed. Record unchanged versus invalidated, then free the copy without
  touching live marks. No result can reach the POC reclamation coordinator because
  `reclaim.inc` is not linked here.
- Inject worker cancellation/join before the HTTP host starts async/socket/task/TLS
  teardown. Native host checks still require all roots gone, the heap empty and
  the guard intact before freeing the arena. Diagnostics are written at exit after
  these checks; this adapter supports the qualified single-owner, single-invocation
  host, not arbitrary thread/context reuse.

The production GC, scheduler and host source files are unchanged. The generated
C adapters are built with UBSan/bounds; generated guest machine code and existing
static archives are not sanitizer-instrumented. The original native qualification
supplies guarded private-copy, reclamation and pending-worker cancellation tests;
this HTTP slice adds actual scheduler traffic, not a replacement for those oracles.

## Observed real requests

The [five-second diagnostic load](evidence/load.json) offers 50 req/s to Serve,
using the existing 16 server lanes / 64 client slots and a queued asynchronous
handler. All 250 offered requests validate, plus separate warm-up and shutdown.
Client/server counts match and shutdown is clean. The
[observer report](evidence/load-shadow.json) includes warm-up/control work too:

| Measure | Observed |
| --- | ---: |
| Successful GC boundary/idle opportunities | 908,975 |
| Snapshots started/completed | 14,153 / 14,153 |
| Completed with unchanged epoch | 5,989 (about 42%) |
| Invalidated before observation | 8,164 |
| Capture rejections / trace errors | 0 / 0 |
| Largest copied arena | 69,768 bytes |
| Native allocations / reclaimed allocations | 105,088 / 105,088 |
| Background results applied | **0** |

An [independent peer test](evidence/recovery.json) sends a malformed request,
then 16 concurrent valid clients, a recovery request and explicit stop/control
signals. It verifies exact response framing/body, 17 ordinary successes, one failed
request and clean exit. That run completes 956 traces (434 unchanged / 522 invalidated)
and reclaims all 7,752 allocations, with zero capture rejections or trace errors.
Neither run happened to cancel a still-running trace at shutdown; cancellation/join
is retained from the separately qualified worker infrastructure, not newly forced
by these HTTP cases.

An unchanged epoch is **not proof of safe live application**: this observer does
not execute the coordinator's identity/heap validation or import/sweep path. Likewise,
the trace count is not a count of extra live collections. The synchronous collector
already did its own work, so all copied tracing is additional overhead. These
instrumented runs qualify execution/ownership only; their throughput and latency
must not be compared with the uninstrumented baseline as a performance candidate.

## Reproduce and next work

```sh
python3 docs/experiments/concurrent-gc-http/build.py \
  --native-build target/http-concurrent-readiness/native \
  --walker-build target/gc-lifecycle-qualified-v2 \
  --output target/http-shadow-new
NEOCLR_GC_SHADOW_REPORT="$PWD/target/http-shadow-new/shadow.json" \
  python3 benchmarks/http-server/load/run.py \
  --runtime neoclr --build target/http-shadow-new --executable app-paced \
  --server-source benchmarks/http-server/serve/Server.rvn --control-stop \
  --workers 16 --concurrency 64 --rates 50 --duration 5 --rounds 1 \
  --output target/http-shadow-load-new
python3 docs/experiments/concurrent-gc-http/validate.py \
  --build target/http-shadow-new --output target/http-shadow-recovery-new
```

Use fresh directories and matching qualifications; the [build report](evidence/build.json)
retains all hashes and the exact link command. Runtime ownership remains unchanged,
consistent with the .NET expectation that live async state survives suspension;
this experimental snapshot tracing is an internal technique, not a new managed API.

Next, identify productive capture/application boundaries using this real scheduler
traffic, avoiding both repeated copying and unnecessary stale work. Qualify result
application with the existing coordinator, synchronous pressure fallback and host
shutdown/error/cancellation cases before producing an uninstrumented HTTP candidate.
Compare that candidate with the [refreshed Serve/Go baseline](../../../benchmarks/http-server/serve/concurrent-readiness-20261018/README.md)
using the same offered-load contract. Larger heaps, saturation, TLS, terminal guest
faults and cancellation with suspended handlers remain unqualified for this adapter.

## Subsequent live-application experiment

[Owner-side application in Serve](APPLY.md) now qualifies the separate `--mode apply`
adapter. The shadow mode and the historical results above remain tracing-only.
The current recovery script additionally holds a partial request across other
clients and checks 18 ordinary successes.


### Fresh source cohorts and two-collector comparisons

`build.py` also accepts a fresh source build's successful `host-compile-link` command,
retaining source-hash validation and explicit paced GC. `compare.py` checks identical
guest assemblies, objects and heap limits, supports `--workload plain|bulk`, and makes
`--go-build` optional. Omitting Go compares the two collectors in reverse order on
round two. Summaries retain p50/p95/p99 and cohort identity. See the
[2026-10-19 results](../../../benchmarks/http-server/serve/memory-gc-20261019/README.md)
for fresh-library qualification, reproduction and complete evidence.
