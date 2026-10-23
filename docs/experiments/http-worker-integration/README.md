# Experimental native HTTP application workers — 2026-10-23

Working source-only macOS ARM64 integration of the bounded native worker pool with
real compiled Raven `HttpServer.Serve`, async socket I/O and owner-local Task.Run.
This provides server/runtime primitives; it does not introduce an application
framework. The ordinary host remains the default.

## Ownership contract

`--profile http --http-workers N` selects 1–4 complete application invocations.
Each owns its heap (1 MiB by default), statics, Task queue, socket scope, GC roots
and pinned async stacks. The first valid listener call from each invocation shares
one kernel listening socket through independently owned descriptor duplicates.
IPv4 address, requested port (including zero) and backlog must agree. Later
listeners are private. Applications must account for startup running N times.

Managed objects never cross these heaps. This deliberately differs from .NET's
shared managed heap/thread pool: it enables concurrent request handling using the
current owner-local collector and scheduler, at the cost of replicated state,
heap capacity and connection lanes. It does not make Task.Run a shared-heap worker
dispatch API. See the [design comparison](../../design/native-http-worker-threads.md).
Retained bootstrap, packaged kits, TLS server mode and other platforms are rejected
by the build helper rather than silently acquiring these semantics.

Each owner closes its own resources. The group retains the base listener until all
workers join. A failed owner signals cooperative scheduler shutdown to its peers;
this does not preempt a CPU-bound loop. Normal shutdown remains application-owned:
the fixture stops every owner through its private control socket. There is no new
public global shutdown or channel API.

## GC correctness gate

The separate [shared-heap graph test](../native-worker-pool/README.md#shared-graph-gc-gate--2026-10-23)
passes optimized and UBSan builds. Four workers access the same compiled closure,
retain aliased cyclic objects, replace edges, keep displaced children alive across
allocation-triggered GC, observe previous mutations, and reclaim the entire heap
when roots are released. Managed access is serialized by the pool gate. This is a
shared-object correctness baseline, **not** qualification of concurrent mutators.
The isolated HTTP integration cannot establish shared-heap GC correctness.

## Reproduce

Use the matching source libraries and compiler bundle identified by the retained
build records. On this machine Xcode's macOS26.2 SDK was selected explicitly.

```sh
NeoClrLibraryRoot=/absolute/path/to/libraries python3 scripts/build-native-project.py \
  --project docs/experiments/http-worker-integration/Server.rvnproj \
  --profile http --http-workers 4 --bundle /absolute/path/to/bundle \
  --aot /absolute/path/to/bundle/bin/neoclr-codegen --output target/http-workers-new
python3 docs/experiments/http-worker-integration/validate.py \
  --executable target/http-workers-new/app --workers 4 --requests 160 \
  --output target/http-workers-check-new
python3 docs/experiments/http-worker-integration/native.py \
  --validation docs/experiments/native-worker-pool/delivery-validation.json \
  --output target/http-workers-native-new
```

Output directories must be fresh. `Server.rvn` exercises an awaited Task.Run in
each response and reports the owner identity. `validate.py` verifies three runs,
all owners serving one endpoint, eight incomplete request reads while complete
requests progress, shutdown with reads pending, and a separate guest-fault run
that must terminate the whole group. Four-owner validation covers 480 responses;
a separately built default host covers 240 responses and its existing exit status.
The native test covers 1–4 workers, repeated startup, independent duplicates,
listener argument mismatch and occupied-port failure in optimized/UBSan builds.
Build-helper/profile and GC layout tests also pass. No successful TSan qualification
is claimed; the earlier toolchain/control failure remains open.

## Preliminary throughput evidence

`measure.py --build target/http-workers-new/build.json --output target/http-workers-measure-new`
checks input hashes and relinks the same generated application object for 1, 2 and
4 owners with the existing **paced GC** option. It leaves the default GC policy
unchanged. Two trials per count, order 1/2/4/4/2/1, use 32 concurrent loopback clients,
one second warmup and a six-second measured window, Connection: close and the
same response/Task workload. Only responses completed inside the window count.

| Owners | Requests/second, two trials | Process CPU seconds per whole trial |
| --- | --- | --- |
| 1 | 458.8, 456.2 | 7.12, 7.10 |
| 2 | 896.3, 892.2 | 14.11, 14.08 |
| 4 | 1666.5, 1687.7 | 27.88, 27.88 |

This demonstrates multicore scaling of isolated owners on this M1 desktop, not
parity with other web servers or a shared-heap speedup. CPU time includes startup,
warmup, measurement, drain and shutdown. Heap capacity and Serve lanes increase
with owners (1 MiB and 16 lanes each), although client concurrency stays fixed.
Short local trials do not establish production capacity or latency guarantees.
These measurements were already underway when the author clarified the shared-GC
priority; they remain comparison evidence. Matched external-server throughput,
allocation profiling, shared async settlement and parallel-mutator safepoints
remain subsequent work after the relevant correctness gates.

Retained JSON files contain raw validation/measurement outputs and exact commands.
Build summaries preserve inputs, artifact identities and command metadata, replacing
large compiler stdout with its digest/byte count; full reports remain at the local
paths recorded in those summaries. No managed public API changed, so no API
reference signature refresh is required.
