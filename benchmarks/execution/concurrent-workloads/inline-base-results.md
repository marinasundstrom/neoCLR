# Exact-base inlining and interleaved measurement (2026-10-25)

## Question and implementation

The [exact-base experiment](exact-index-results.md) improved HTTP but appeared to
increase ordinary CPU. Those ordinary baseline/candidate cohorts ran separately.
Add `--compare-exact-allocation-index` to measure all variants in one cohort,
rotating mode order through forward/reverse cycles. The first 294 executions
cover three samples, seven modes and fourteen rounds; each mode occupies every
ordinal twice per sample. Outputs, managed allocation equality and cleanup pass.

Live CPU medians (predecessor index / hybrid exact / direct exact), seconds:

| Sample | Predecessor | Hybrid | Direct |
| --- | ---: | ---: | ---: |
| Tasks | 0.024828 | 0.0262125 | 0.026228 |
| Collections | 0.1443885 | 0.1483065 | 0.147828 |
| Arrays | 0.0591035 | 0.0598755 | 0.0601355 |

This confirms the observed regression under interleaving. ARM64 disassembly shows
no out-of-line `known_base` symbol in the predecessor variant, but both exact
variants have one. Their scalar-store function contains five call sites to it,
versus zero in the predecessor variant. Static call sites are not calls executed
on every store. The assembly is retained separately from benchmark timings.

Add opt-in `NEOCLR_LIVE_INLINE_BASE` to request inlining of the existing predicate
with the compiler's `always_inline` attribute. The predicate and ownership checks
are unchanged. This tests whether expanded lookup logic crossed the compiler's
inlining threshold; it does not assume fewer calls necessarily reduce CPU.
`--compare-inline-base` adds hybrid-inline guarded/live controls to the ordinary
and HTTP tools. Production GC and compiler lowering are unchanged. The same
.NET/Go indexed-location research and safety boundaries from the linked experiment
apply; no managed API semantics diverge.

The native optimized and undefined/bounds safety suites, stale-hint/deletion
negative controls, 144,000 model comparisons and 1,440 JIT/AOT fixture cycles pass.
Existing ASan host-initialization limitations remain; these are not ASan results.

## Reproduction and analysis

Use `concurrent-workloads/measure.py --allocation-index --guarded-control
--compare-exact-allocation-index --compare-inline-base --rounds 18` with the
retained optimizer-build. Without inline controls, fourteen rounds balance seven
modes. `summarize-index-interleaving.py --measurement .../results.json --output ...`
rejects failed/instrumented cohorts, duplicate or missing round/mode rows,
unbalanced positions and allocation differences. It reports CPU medians and
paired within-round ratios with descriptive quartiles, not confidence intervals.

Native safety and generated entry/record-array validators take `--inline-base`
alongside `--allocation-index --exact-allocation-index`. The independent model
always enables the allocation index. HTTP takes `--compare-allocation-index
--compare-exact-allocation-index --compare-inline-base --include-guarded-sync`
with four clients, 2,048 measured requests plus sixteen warmup and three rounds.
All hosts compile before timing; profiled assembly is not used as timed evidence.

## Inlining results and decision

A second balanced cohort validates 486 executions across three samples, nine modes
and eighteen rounds. All modes occupy every ordinal twice for each sample. Exact
outputs, allocation equality, collector completion and native metadata cleanup
pass. Live CPU medians in seconds:

| Sample | Predecessor index | Hybrid exact | Inlined hybrid |
| --- | ---: | ---: | ---: |
| Tasks | 0.023637 | 0.024657 | 0.023678 |
| Collections | 0.134371 | 0.138359 | 0.1346475 |
| Arrays | 0.056858 | 0.057352 | 0.0567205 |

Inlining lowers CPU about 4.0%/2.7%/1.1% relative to the out-of-line hybrid and
returns the medians to within about 0.25% of the predecessor-index variant. This
supports the inlining explanation for the ordinary regression. It does not prove
zero regression for every workload or architecture. Counts and paired descriptive
ratios are in `inline-base-interleaved-summary.json`. No ordinary managed allocation
counts or bytes change. The initial and follow-up cohorts together cover 780 runs.

Disassembly now confirms zero `known_base` call sites in the scalar-store function.
The inlined hybrid has 453 store instructions versus 379 for the out-of-line
hybrid and 403 for the predecessor-index variant. That is 74 additional ARM64
instructions (296 bytes) in this function versus the hybrid, not a measurement of
total executable growth. Other callers may also grow. This is a code-size versus
call-cost tradeoff; memory layout and object headers remain unchanged.

Another 111,456 checked HTTP responses cover both handlers and nine modes, three
alternating rounds, four clients and 2,064 responses including warmup per trial:

| Handler | Predecessor req/s | Hybrid req/s | Inlined req/s | Predecessor CPU | Hybrid CPU | Inlined CPU |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plaintext | 2,468 | 2,656 | 2,699 | 0.864866 s | 0.809739 s | 0.792798 s |
| JSON | 1,582 | 1,682 | 1,690 | 1.350928 s | 1.272772 s | 1.251355 s |

Relative to the predecessor-index control, inlined exact lookup raises throughput
about 9.3%/6.8% and lowers CPU 8.3%/7.4%. The incremental inlining-only throughput
effect is smaller, about 1.6%/0.5%, with 2.1%/1.7% lower CPU. Do not overstate the
small JSON throughput effect from three trials. Client p99 medians for the hybrid
versus inlined variant are 1.687/1.603 ms plaintext and 2.765/2.760 ms JSON. These
are response latencies, not GC-pause distributions or saturation measurements.
Sync still leads at about 5,176/3,124 requests/s.

HTTP allocation equality is not exact across the cohorts. Plaintext sync,
predecessor-live and inlined-live each have a trial with one extra 528-byte object;
JSON sync and inlined-live also do. The raw reports retain every control's counts.
Typical totals are 429,856 objects / 33,057,881 bytes for plaintext and 943,817 /
54,065,775 for JSON. This is not an allocation reduction. Every indexed trial still
reports one native allocation, 16,640-byte peak index storage, zero retained at
shutdown. No new persistent GC metadata is introduced by inlining.

Retain inlining as an explicit candidate control, with the inlined hybrid the next
configuration to profile for remaining HTTP costs. This removes the observed
ordinary CPU penalty while retaining HTTP gains; it does not qualify concurrent GC
as the default. Shared-worker admission, larger heaps, sustained JIT performance
and pause distributions remain open. Native safety and the existing JIT/AOT fixtures
cover this change; unrelated platform suites and website builds were not rerun.

The summary tool passes both valid cohorts and rejects deliberately duplicated
rows, an unbalanced within-round order, and an instrumented timing report. Commands
and failures are retained in `inline-base-summary-validation.json`; the assertion
failures are expected negative controls. Raw reports retain source hashes and
compiler commands. The source-only inlining change uses the already qualified
Clang/GCC native experiment boundary, without changing Raven compiler semantics.
