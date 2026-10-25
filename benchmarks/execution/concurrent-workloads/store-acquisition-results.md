# Borrow acquisition cost and workload density

2026-10-25. The [complete managed experiment](borrowed-managed-results.md) regressed
HTTP CPU/throughput despite favorable amortized scalar-store measurements. This
follow-up isolates acquisition cost and counts its frequency in the same five
workload objects. No production/compiler/GC contract changes.

## Observations

A separate noninlined C activation executes 1, 2, 4, 8, 16, 64 or 1,024 stores.
Both variants pay that activation call; borrowing acquires once inside it. Each
cell performs exactly 1,048,576 stores, verifies the result and releases the heap.
Six alternating checked/borrowed pairs cover 176,160,768 stores. Native sources
are separate translation units, built at O2 without LTO. Thread CPU is measured;
there is no concurrent epoch in this inactive-heap/outside-heap microbenchmark.

| Stores per activation | Outside-heap borrowed/checked CPU | Inactive-heap ratio |
| ---: | ---: | ---: |
| 1 | 1.072 | 1.041 |
| 2 | 0.967 | 1.067 |
| 4 | 0.931 | 1.040 |
| 8 | 0.899 | 0.953 |
| 16 | 0.993 | 0.942 |
| 64 | 0.889 | 0.935 |
| 1,024 | 0.880 | 0.926 |

Low-density acquisition can erase the benefit; the threshold differs by path.
Noise (including the outside-heap 16-store cell) means these are not universal
break-even thresholds or reliable per-call cost estimates. This isolates the ABI
pattern, not generated managed register pressure or marking/barrier costs.

Diagnostic-only native source instrumentation counts acquisition and borrowed-hook
calls on the main owner thread. It leaves heap ownership, store checks and existing
objects unchanged. Each ordinary sample runs its existing 4,096-iteration batch;
plaintext/JSON each serve 512 sequential checked responses. Outputs, nonzero
completed epochs, candidate accounting and index release pass.

| Sample | Acquisitions | Total store-hook calls | Calls/acquisition |
| --- | ---: | ---: | ---: |
| Tasks | 319,536 | 331,819 | 1.038 |
| Collections | 2,023,472 | 2,035,755 | 1.006 |
| Arrays | 475,184 | 233,515 | 0.491 |
| Plaintext | 1,987,205 | 3,765,452 | 1.895 |
| JSON | 2,989,189 | 5,626,572 | 1.882 |

The checked wrapper also delegates to the borrowed hook, so these averages are
**upper bounds** on generated borrowed stores per acquisition. They are not
per-function histograms. Fewer stores than acquisitions can result from branches
that execute no guarded store or guards removed after acquisition emission; this
measurement does not distinguish those causes. Counts include startup/shutdown.
These diagnostics complete 1,793 concurrent epochs; they are not performance
trials, multiworker counters or pause-distribution measurements. Instrumentation
adds hot-path work; its timings must not replace the prior uninstrumented cohort.

## Decision and boundaries

Do not introduce a new execution-state association solely on the earlier amortized
microbenchmark. Current `neoclr_gc_state_enter_v1` returns a root/pacing state with
same-thread park/resume lifetime. The candidate borrow returns a separate TLS
address. Neither is a heap/epoch proof; services retain their own resets and
validation. Sharing an opaque association would require initialization, library
identity, thread lifetime and reset/park rules, plus ABI qualification.

First investigate acquisitions that survive after all relevant guards disappear,
and activation paths that execute too few stores to amortize acquisition. Static
store counts alone are not a safe profitability model: loops/branches, stack
elision and unframed adapters change dynamic density. Retain checked paths and
compare any selective emission with complete managed workloads. Eliminating a
proven unused acquisition could be a low-risk cleanup, but it is not implemented
or claimed as an HTTP improvement here. The larger concurrent/sync gap remains;
root publication, metadata, complete pauses and shared-worker acceptance stay open.

## Reproduction and evidence

`concurrent-live/measure-store-acquisition.py --output <new-directory>` records
build commands, hashes and all alternating trials in `store-acquisition.json.gz`.
`profile-store-acquisitions.py --root target/concurrent-workloads --output
<new-directory>` uses retained borrowed-managed manifests/objects and writes
`store-acquisition-counts.json.gz`. It requires the existing SDK and artifacts
named in those manifests. Instrumented sources and original command-input hashes
are recorded. `store-acquisition-summary.json` contains ratios and diagnostic
counts without large console output. Existing native-store safety qualification
is reused because no store/collector behavior changed. No .NET-facing API, Raven
bridge, native state layout or production default changes are made.
