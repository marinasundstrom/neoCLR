# Group root-publication thread state — 2026-10-23

Sampling the preceding fault-validation server still finds root publication as
the largest worker self-sample category: 1,677 of 3,852 samples (43.5%). Thread-local
address lookup accounts for 476 (12.4%), across the runtime. The main thread's
3,852 condition-wait samples are not worker CPU consumption. Sampling perturbs the
server; this run selects an investigation target rather than measures capacity.
See sample-summary.json for the artifact hashes and reproduction commands.

Group the private current-frame pointer, publication counter, frame depth and
spill-marker search hints into one C thread-local structure. This lets the native
compiler reuse the resolved thread-local address. Each field retains its existing
lifetime; parking clears the active chain/depth but retains the publication count
and search hints. Parked roots and the externally referenced worker hook remain
separate. There is no process-global state, managed allocation, descriptor cache,
header change, compiler ABI change or weaker validation.

An optimized Apple ARM64 assembly comparison confirms static thread-local lookup
sites fall from three to one in root publication and from two to one in each of
frame entry/exit. This is target/compiler evidence, not an ABI guarantee or a
claim about every operating system. The grouped state occupies 1,048 bytes on this
64-bit target, including four padding bytes; it replaces the same fields totaling
1,044 bytes before TLS placement/alignment. No memory-footprint reduction is claimed.
See assembly-comparison.json for exact commands and source/assembly hashes.

The existing [CLR/Java/Go research](../../../../docs/design/allocation-metadata-runtime-comparison.md)
motivates thread-owned execution metadata with cheap access. This representation
is specific to neoCLR's C runtime; it does not introduce CLR Thread objects, Java
TLAB allocation or Go scheduler ownership. Native thread isolation is unchanged.

## Validation and reproduction

The root fixture now checks nested entry/publication, parking, a different active
context, resume, restored depth/head, persistent publication counts and final
empty state, in both probe-only and native GC builds. Existing malformed/mutable
root and fault cases continue to run. The native GC suite, repeated forced-GC HTTP
and compiled shared-worker optimized/UBSan stress all pass before timing. UBSan
is not a race detector; the shared-worker checks exercise existing coordination.

```sh
python3 scripts/validate-native-gc.py --output target/root-thread-state-check
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/root-thread-state-forced
python3 benchmarks/http-server/primitive-comparison/root-thread-state/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline 87c442dad --output target/plain-root-thread-state
```

Use the preceding experiment's Xcode SDK and bundle. Only root-probe.c differs
between timed variants. Both use the same application object, owned allocation
scope/base cache, 1 MiB per owner, 16 Serve lanes and paced GC. One/four isolated
owners serve checked ten-byte plain HTTP responses through EOF at 32 connections,
two seconds warmup and six measured, forward/reverse. CPU/response covers server
lifetime and all warmup/window/drain completions after subtracting client CPU.
No builds/tests overlap timed trials. This is not shared-heap HTTP qualification
or a new comparison against other web servers.

All retained correctness reports pass and their input hashes match the tested
sources. Forced HTTP completes eight responses across two invocations. Each
optimized/UBSan worker run completes 32 healthy and 32 fault invocations, with
24,544 graph collections and 448 fault collections; graph participants reach four
concurrent workers. These checks cover collection across native threads without
changing the existing shared-heap synchronization contract.

## Results and remaining work

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 1,007.5, 1,063.7 | 962.5, 942.0 | 55.53, 47.64 |
| baseline | 4 | 3,626.2, 3,530.0 | 1,054.0, 1,066.8 | 14.85, 18.35 |
| candidate | 1 | 1,086.0, 1,098.8 | 921.3, 905.9 | 38.88, 46.02 |
| candidate | 4 | 3,591.7, 3,648.7 | 1,062.0, 1,031.9 | 16.93, 21.86 |

Mean throughput changes +5.5%/+1.2% at one/four owners; estimated CPU/response
changes -4.1%/-1.3%. Both one-owner candidate runs improve throughput and CPU cost
against both baselines, although baseline drift limits precision. Four-owner
samples overlap: there is no clear scaling improvement, and p99 latency is higher
in both corresponding candidate trials. Keep the local execution-cost reduction
and its modest storage padding cost, without claiming a universal throughput or
latency gain. Longer measurements are needed to characterize sustained capacity.

Root publication still scans all ancestor storage/transient descriptors. Skipping
those checks would require a separately validated ownership/lifetime contract for
execution metadata; this change introduces none. No managed allocations are saved,
and publication frequency, worker scheduling and collection budgets are unchanged.
