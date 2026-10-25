# Reusing participant-root storage

2026-10-25. Opt-in `NEOCLR_WORKER_ROOT_REUSE` retains native root-buffer storage
per pool and removes a redundant descriptor traversal. It works with the copied
synchronous collector and the restricted read-phase live collector; no default,
managed API, Raven/CLI emission or provider admission changes.

## Cost and ownership

The previous worker path counted roots, allocated an exact buffer, then called a
capture API that counted/validated again before copying. Every rendezvous walked
participant descriptors three times and allocated/freed once. Live begin and finish
each performed that sequence. The new private helper validates/counts once,
grows storage if needed, and copies current values on its second walk. It does not
cache descriptor validity or root values across rendezvous. Both passes require
quiescent contexts, frame descriptors and values, with nonaliasing output.

A zero-initialized `neoclr_gc_root_buffer` belongs exclusively to the pool under
its gate and sole execution lease. Capacity grows geometrically from 64 words,
with overflow and native allocation-failure checks. Invalid inputs and allocation
failure preserve the prior buffer/result. A second-pass validation failure aborts
because it contradicts the required rendezvous. Only the current count is sent
to the collector; unused retained words are never registered roots. The service
has finished consuming a snapshot before the requester can reuse the buffer.
The live marker's snapshot owns its detached data independently. Pool destruction
frees the buffer after compiled workers/service have joined and heap cleanup has
completed. Direct caller-owned buffers have explicit idempotent disposal.

This follows the existing [root capture and .NET-oriented ownership comparison](README.md)
and [service tradeoffs](allocation-owner.md): native storage reuse is independent
of managed object semantics. Benefit: fewer native allocation requests and one
less descriptor walk. Cost: a four-word buffer record and retained high-water
capacity until pool destruction, potentially larger for deep stacks. It does not
reduce managed allocations or establish lower peak memory. No native allocation
failure injection or broad memory-retention acceptance is claimed.

## Validation and measurements

`root-reuse-unit.json.gz` passes ordinary and UB/bounds-sanitized runs. Tests compare
fresh captures against reused buffers while values and root counts change, grow
and shrink, reject malformed frames without replacing storage/results, and dispose
twice. Each build also passes 400 worker-stack captures and 6,400 liveness checks;
the actual collector consumes copied roots after source worker stacks return.
Removing the new helper's value-copy walk fails deliberately, demonstrating that
storage reuse cannot be mistaken for permission to retain stale root values.

`root-reuse-live.json.gz` and `root-reuse-sync.json.gz` pass optimized/UB-sanitized
compiled graph, fault and shutdown cases plus missing-dispatch/root negatives.
For the live graph, 40 captures (20 begin/finish pairs) use one growth allocation,
retaining 1,024 bytes instead of making 40 allocation/free pairs. The synchronous
graph uses 20 captures and one/two growth allocations, retaining 1,024/2,048 bytes
in these runs. The fault cohort captures no roots under pacing. Both retain the
same 3,904 graph / 128 fault / 488 shutdown managed allocation requests.
`root-reuse-pressure.json.gz` passes explicit and rooted-pressure fallback with
ordinary and UB/bounds-sanitized live-linked builds and buffer reuse enabled.

The rotating three-way cost measurement uses matching managed objects, one warmup
and five measured process runs per variant, checking graph/fault/shutdown outputs:

| Final cohort | Elapsed median | Total child CPU median |
| --- | ---: | ---: |
| Original live root capture | 108.30 ms | 236.59 ms |
| Reused live root buffer | 110.04 ms | 241.44 ms |
| Reused synchronous root buffer | 104.82 ms | 231.10 ms |

The tool's legacy `broadcast` label denotes the third (reused synchronous) variant
here; its manifest/command identifies the actual mode. Live reuse is 1.6% slower
and uses 2.1% more CPU in the final cohort. The first cohort was essentially flat
elapsed with 0.4% less CPU. Thus fewer native allocation requests do **not** establish
a throughput/CPU improvement. These short runs do not measure HTTP requests,
marker overlap duration, pause tails or all native allocation bytes. Preserve the
option and controls, but do not promote it as a performance default. The larger
remaining integration targets are service/lease coordination and provider coverage;
further root micro-optimization is not justified by these timing results alone.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-participant-roots/validate.py \
  --output target/root-reuse-unit-new
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --read-epochs --reuse-roots --output target/root-reuse-live-new
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --collection-owner --paced --reuse-roots --output target/root-reuse-sync-new
python3 benchmarks/execution/concurrent-participant-roots/validate-pressure.py \
  --read-epochs --reuse-roots --output target/root-reuse-pressure-new
python3 benchmarks/execution/concurrent-participant-roots/measure-owner.py \
  --control target/compiled-read-epochs-validated --owner target/root-reuse-live-new \
  --broadcast target/root-reuse-sync-new --output target/root-reuse-cost-new.json
```

Use fresh output paths and rebuild the control using `validate-pool.py --read-epochs`
if its retained executable is unavailable. Commands, input hashes and outputs are
archived beside this document. Overall concurrent-default and HTTP/JIT acceptance
remain open, as do the previously recorded TSan startup limitations.
