# Native object lookup and survivor cache retention — 2026-10-24

This slice implements opt-in work counters in the shared JIT/AOT native collector
and uses them to choose a bounded improvement. Existing pause counters remain
unchanged. The new counters are private C diagnostics, not a managed API or an
interpreter heap measurement. See the [counter definitions](../../../docs/experiments/runtime-async/native-profiling.md#native-object-lookup-counters--2026-10-24).

## What was measured

Fixed two/four-request plaintext and reflection-JSON consumers use the existing
owned 1 MiB paced entry host. Response status/body and server completion totals are
checked. Counts include startup, all requests and teardown; they are not per-request
allocation totals. Idle collections can vary, especially in two-request runs.

For the original four-request policy, JIT and AOT match:

| Counter | Plain | JSON |
| --- | ---: | ---: |
| Exact-base calls | 431 | 955 |
| Cache hits | 403 | 896 |
| Validation walks | 28 | 59 |
| Descriptors visited by base validation | 12,423 | 31,393 |
| GC index queries | 50,759 | 76,002 |
| Queries rejected outside the arena | 33,879 | 51,300 |
| GC index block probes | 57,080 | 87,655 |
| Marking candidates | 50,579 | 75,741 |
| Ordinary-object words scanned | 45,214 | 65,692 |

The high hit rate hides costly misses, each validating many descriptors. Meanwhile,
ordinary-object scanning supplies most marking candidates, and about two-thirds
of index queries reject words outside the heap. This distinguishes proof/cache
work from candidate scanning; it does not measure metadata loading or cache misses
in the hardware. `corpus.json` adds checked task/collection AOT/JIT consumers.

## Implemented change

Previously every successful collection cleared the whole 64-entry exact-base
cache. It now revokes a matching entry immediately before reclaiming that allocation,
retaining proofs for surviving allocations. Pointer equality is required before
clearing a cache slot: reclaiming a colliding allocation must not erase another
live entry. Release still clears everything; allocation reuse establishes a fresh
proof. No additional cache, object header or managed allocation is introduced.

The cache remains restricted to an exclusively owned arena with runtime-controlled
descriptors. Exact-base misses and collection/release validation remain checked;
external heaps are unchanged. Interior/forged addresses are never inserted.
The change relies on the existing nonmoving allocation identity, not merely the
address remaining within the heap. Reclamation invalidates before coalescing,
trimming or reuse. Cleanup and failed-collection behavior remain unchanged.

Four plaintext requests now need 15 walks and 7,193 visited descriptors in both
modes: 42.1% fewer descriptor visits. JSON needs 38 walks, with 21,790 AOT and
21,380 JIT descriptor visits (30.6%/31.9% fewer); AOT has a differing idle-collection
schedule, so its total tracing work is not identical. The identical JIT tracing
counts isolate the admission improvement from tracing changes.

## Capacity and rejected experiment

Eight uninstrumented runs compare cleared/retained caches, configurations forward
then reverse. Each has one-second warmup, five-second measurement, 32 connections,
client GOMAXPROCS=2 and checked ten-byte plaintext responses over new connections.
CPU is server `ps time` after READY through warmup/load divided by completed/drained
responses, excluding compilation/teardown and with approximately 10 ms resolution.

| Mode / cache | Mean requests/s | CPU µs/response |
| --- | ---: | ---: |
| aot-cleared | 2,946.1 | 336.9 |
| aot-retained | 2,949.6 | 335.7 |
| jit-cleared | 1,901.9 | 525.4 |
| jit-retained | 1,905.6 | 522.9 |


All mean differences are below 1%. These short local trials establish no convincing
throughput improvement. Retain the change as reduced repeated validation work with
unchanged cache size, not a claim that this closes the HTTP throughput gap.

An earlier experiment doubled the temporary collection index from 256 to 512
buckets. It reduced four-request plaintext probes from 57,080 to 38,821 (32.0%),
but its uninstrumented AOT/JIT results were mixed: AOT improved modestly while JIT
regressed. It would also add 2 KiB of native stack during collection. The production
index remains 256 buckets, with its existing dense-heap path. Retained patch,
counter and capacity evidence document why fewer probes alone were insufficient.

## Validation and remaining work

Four optimized/sanitized enabled/disabled diagnostic runs cover arithmetic, reset
without heap changes, survivor cache hits, reclaimed-pointer rejection and independent
thread counters. Two further optimized/sanitized enabled checks verify survivor
proof retention while reclaiming colliding cache entries. Sixteen optimized/sanitized collector regressions cover exact bases,
forged/interior pointers, trailing corruption, eviction, split/reuse, release/reacquire,
sparse/dense indexes and reclamation. Four forced/paced optimized/sanitized host
lifecycle runs cover faults, admission, re-entry and independent owners. Forced-GC
plaintext/JSON checks exercise JIT/AOT; ordinary task/collection consumers check
outputs. Five analysis tests reject malformed counters and incomplete capacity data.

The next substantive tracing candidate is a compact per-type reference layout
shared by objects, separated from cold names/signatures/reflection data. It needs
complete producer coverage and a conservative fallback. Moving GC additionally
requires writable roots, relocation or pinning of interior/native references, and
worker coordination. This change does not implement either capability.

## Reproduction

`counts.py --sample plain|json --output NEW_DIRECTORY` collects fixed requests;
`--forced --requests 1` exercises forced collection. `--gc-source` permits a retained
source counterfactual. `validate.py` runs focused diagnostics using the known-good
compiler/SDK prefix in the earlier call-layout recipe. An initial default-SDK linker
failure is retained separately; it is not a collector test failure.

`measure-retained.py` compares the saved `target/lookup-profile/baseline-gc.c`
against current runtime source. Restore that file from `baseline-gc.c.gz` when needed.
`measure.py` and `larger-index.patch` retain the rejected index experiment; it must
be reconstructed from that baseline, not silently compared to a later runtime.
The scripts depend on earlier application/native artifacts and preserve exact build
commands. Raw JSON reports, source snapshots, patches and input manifests retain
provenance. No globally installed bundle or compiler default is changed.
