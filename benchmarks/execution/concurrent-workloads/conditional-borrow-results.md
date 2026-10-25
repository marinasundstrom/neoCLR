# Conditional candidate-state borrowing

2026-10-25. Follow-up to [caller attribution](acquisition-sites-results.md).
The archived prototype uses a test-only `NEOCLR_LIVE_CONDITIONAL_BORROW` switch
to refine `NEOCLR_LIVE_BORROW_STORES`. It passes correctness checks but still loses
to checked stores, so the prototype is not retained in the working compiler.

## Placement proof and boundaries

Before lowering stores, group guarded stores by their original basic block.
Delay acquisition to the first store in each block only when no store-bearing
block is the entry and no store-bearing block can reach any other store-bearing
block, including itself. Reachability follows the complete CFG, rather than a
reducible-loop or branch-probability assumption. Otherwise retain the existing
single post-frame-entry acquisition. This ensures at most one dynamic acquisition
per activation, with no acquisition on eligible paths that execute no guarded store.
Multiple stores in one block reuse its handle across the inserted failure checks.

Every store still requires frame-linkage dominance. Acquisition remains within
the same-thread activation; no handle is exported or retained by another service.
Store ownership/context/width/extent validation, mutation logging, overflow handling
and production fault/return cleanup are unchanged. The getter returns a stable
candidate-state address, not an epoch/heap proof or a newly acquired resource.
No root/candidate association, TLS layout change or new native capability is needed.
Borrow capability bit 32 and the existing exclusion of migrating/shared-worker
activations remain in force. This is compiler placement only: Runtime Contract,
Raven semantics/emission and temporary CLI representation are unchanged. neoCLR
owns lowering validation and the native host continues to own lifetime/compatibility.
Existing .NET/Go ownership research linked by the native prototype remains applicable;
there is no .NET-facing semantic divergence in this internal experiment.

## Validation and measurement

Two focused tests cover a store-free path, successful and first/second failing
stores with exact acquisition/frame cleanup counts, and conservative CFG fallback
for sequential stores, self-loops and indirect cycles. The first runs both eager
and conditional policies. The mutually exclusive CFG is accepted. Together with
existing checks, 52 mutation/admission tests, 29 managed-selection tests and the
16-combination JIT width/alias test pass.

The measurement comparator uses the faster checked-store predecessor, not the
regressed eager-borrowed objects. Existing synchronous and guarded-sync controls
remain. Build/qualification processes finish before uninstrumented timing begins.

Native entry/record fixtures pass 1,440 AOT/sanitized-AOT/JIT cycles, including
legacy-host rejection. Full managed JIT serves 1,024 plaintext/JSON responses and
runs six standalone batches (stack elision off/on), completing 2,559 concurrent
epochs with released index metadata. HTTP JIT uses paced hosts here. Full AOT
selections compile with changed objects for all five workloads.

Uninstrumented controls pass 150 balanced standalone executions and 82,560 HTTP
responses. Paired conditional/checked ratios are:

| Workload | Live CPU change | Live elapsed/throughput change |
| --- | ---: | ---: |
| Tasks | +1.9% | elapsed +1.9% |
| Collections | +0.4% | elapsed +0.5% |
| Arrays | +1.0% | elapsed +0.8% |
| Plaintext HTTP | +3.1% | throughput −2.7% |
| JSON HTTP | +2.6% | throughput −2.8% |

Current live plaintext medians are 2,979 requests/s and 0.730 CPU seconds, versus
checked live 3,071 and 0.707; sync reaches 5,126 with 0.403 seconds. JSON is 1,889
requests/s and 1.138 CPU seconds, versus checked live 1,949 and 1.110; sync reaches
3,038 with 0.670 seconds. These are retained harness observations, not a server
scalability ceiling or a cross-framework capacity comparison. Small standalone
changes are not claimed statistically significant; no confidence intervals are
estimated. Do not infer a precise eager-versus-conditional performance difference
from separate historical cohorts.

Standalone allocations match exactly. Plaintext current/previous live ranges each
vary by one object/528 bytes; the previous guarded JSON cohort varies by four
objects/2,112 bytes. Other JSON modes match. No allocation reduction is observed.
The raw reports retain peak arena/live bytes and RSS; no pause advantage is tested
or claimed by this cohort.

A subsequent diagnostic-only run executes three standalone batches and 1,024 HTTP
responses, completing 1,793 epochs and releasing metadata. It measures acquisitions:

| Workload | Eager acquisitions | Conditional acquisitions | Reduction |
| --- | ---: | ---: | ---: |
| Tasks | 319,536 | 311,343 | 2.6% |
| Collections | 2,023,472 | 1,691,695 | 16.4% |
| Arrays | 475,184 | 356,399 | 25.0% |
| Plaintext | 1,987,205 | 1,827,456 | 8.0% |
| JSON | 2,989,189 | 2,746,496 | 8.1% |

Store-hook counts remain exactly equal to the earlier eager diagnostic in every
sample. This shows the intended acquisition reduction, not reduced mutation work.
Diagnostics run after timing, so their hot-path counters do not contaminate the
performance cohort. The placement decision precedes whole-closure stack elision;
a subsequently removed store can conservatively keep entry acquisition. The
prototype does not claim globally optimal placement.

## Decision

Reject promotion and remove this prototype from the working compiler. Retain its
patch and complete evidence, plus reusable prefix support in the profiling and
comparison tools. Correctness and reduced acquisition counts do not establish an
application improvement: checked stores remain faster here. Further acquisition
micro-optimization is a lower priority than reducing executed guard calls using
explicit compiler provenance, preserving ownership and tracing semantics. Earlier
stack-guard elimination remains the positive control for that direction. No new
runtime/default switch is shipped; shared-worker, pause and default acceptance
remain open.

## Retained evidence and reproduction

`conditional-borrow-experiment.patch.gz` contains lowering and its two tests,
relative to `760f12231`; it is archived and **not applied**. To reproduce, apply
that patch, build the codegen lib test runner with `cargo test -p neoclr-codegen
--lib --no-run --message-format=json`, copy/sign the executable on macOS, and use
`NEOCLR_LIVE_BORROW_STORES=1 NEOCLR_LIVE_CONDITIONAL_BORROW=1` together with the
retained selection recipes. `conditional-borrow-build-driver.py.gz` records the
exact reconstruction used here. The retained SDK/bundle/object paths are required.

`conditional-borrow-{ordinary,http}-build.json.gz` records selection commands,
environment and source/runner/object hashes. Entry/record and plain/json/ordinary
JIT reports retain execution/cleanup evidence; `conditional-borrow-invocations.json`
records the outer qualification settings, including the inherited prototype flag.
Regression/managed/elision logs retain focused test output. The build driver and
native reports retain the experiment's source hashes before reverting it.

`conditional-borrow-{ordinary-measure,http-plain,http-json}.json.gz` and
`conditional-borrow-measure-commands.json.gz` retain controls and raw trials.
`conditional-borrow-summary.json` contains paired ratios. Recreate it with
`summarize-borrowed-stores.py --prefix conditional-borrow --root <results-root>
--output <summary.json>`. Four negative summary controls reject missing/extra
native flags, missing trials and failed responses. Prefix-generalized analyzers
also reproduce their original retained summaries byte-for-byte.

`conditional-borrow-counts.json.gz` records acquisition diagnostics. Its driver is
`profile-store-acquisitions.py --prefix conditional-borrow --root <results-root>
--output <new-directory>`. Timings in diagnostic output are not performance evidence.
No Raven semantic/emission, CLI bridge, Runtime Contract or managed API changes
survive this rejected experiment; there is no compiler integration to carry to Raven.
