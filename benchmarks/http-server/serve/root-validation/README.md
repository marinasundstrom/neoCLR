# Root metadata search optimization — 2026-10-18

Author direction: optimize repeated root metadata validation, preserve GC safety,
measure Serve against the Go baseline, and commit each slice.

The preceding [native profile](../capacity-20261018/README.md) attributes 5.2% of
main-thread wall samples to `strstr` and 6.9% to `strlen`. These are top-of-stack wall
sampling observations, not predicted speedups. This slice targets substring search
only; it keeps string-length checks, all live frame descriptor checks (including
ancestors from other contexts), fault validation and collection policy unchanged.

A 256-entry thread-local table holds marker offsets, costing 1 KiB per native
thread. Each publication first verifies the current string length, bounds-checks
the hint, and compares the current marker bytes. A miss falls back to the original
substring search. The table stores no pointers and never trusts a prior validation.
Hash collisions, changed contents and reused image addresses therefore affect only
search efficiency. This is shared native GC infrastructure, so the potential benefit
extends beyond HTTP; the measured magnitude is specific to Serve. Interpreter-side
GC is unchanged. No frame layout, codegen ABI, public API or GC root lifetime
changes. Plans remain NUL-terminated readable strings under the existing private
contract; the substring test remains a marker check, not a JSON parser.

This follows the existing [.NET/collector comparison](../../../../docs/experiments/aot-console/native-gc.md#comparison-and-provisional-choices): root reachability
must survive every collecting boundary. The optimization is specific to neoCLR's
private diagnostic JSON representation, not a proposed .NET API divergence.
Moving all metadata validation to image admission would require an explicit immutable
image/lifetime contract; pointer-only memoization is deliberately not used here.

## Reproduction and validation

`relink.py --build QUALIFIED/native --output NEW_BUILD` reuses the exact guest object
and link settings, recording current native source hashes and both stress/paced
executables. It does not rebuild managed code. `compare.py --baseline OLD/native
--candidate NEW_BUILD --go GO_BUILD --output NEW_RESULTS` runs two reversed-order
rounds at 400/500 offered requests/sec, 30 seconds each. Settings match the
[Go baseline](../go/README.md): 16 server lanes, 64 client slots, ten bytes, connection
close, queued handler work; Go GOMAXPROCS=1/GOGC=100, neoCLR experimental paced GC.
All losses and generator-lag failures remain visible. Tests/builds run separately.

## Green Tea research supplied by the author

The author supplied [Go's Green Tea article](https://go.dev/blog/greenteagc) during
this work. It batches tracing by page to improve memory locality, retaining separate
seen/scanned state. This is relevant to later collector tracing work, distinct from
this root-publication search. It requires changes to heap organization/worklists,
with bookkeeping and correctness costs; no benefit to neoCLR is established yet.
The article distinguishes collection frequency from cost per collection, which
supports keeping our policy fixed while measuring an implementation change.

[Go 1.26 release notes](https://go.dev/doc/go1.26#runtime) confirm Green Tea is now
the default. Our retained Go 1.26.3 binary's `go version -m` lists no experiment
opt-out. The article's GC-overhead reductions are workload-specific, not HTTP
throughput predictions; its newer-amd64 vectorization results do not describe this
Apple M1 run. Collector replacement remains exploratory.

## Observations and limits

Two 30-second rounds, with entry and rate order reversed. Each row gives first /
second round values; successes after the window remain in the raw report and are
excluded from timed completions/sec.

| Runtime | Offered/sec | Timed completions/sec | Admission misses | CPU ms/completion | Offered p99 ms | Generator lag p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Baseline | 400 | 399.30 / 362.27 | 0 / 1074 | 2.469 / 2.683 | 79.07 / 246.87 | 0.67 / 1.60 |
| Candidate | 400 | 398.00 / 397.77 | 0 / 66 | 2.310 / 2.335 | 74.92 / 176.59 | 1.45 / 3.63 |
| Go | 400 | 400 / 400 | 0 / 0 | 0.212 / 0.212 | 13.02 / 10.73 | 5.25 / 4.43 |
| Baseline | 500 | 405.80 / 362.53 | 2762 / 4062 | 2.449 / 2.655 | 188.28 / 273.02 | 0.59 / 3.60 |
| Candidate | 500 | 432.47 / 421.47 | 1962 / 2293 | 2.303 / 2.339 | 180.82 / 255.74 | 0.50 / 1.45 |
| Go | 500 | 500 / 500 | 0 / 0 | 0.177 / 0.196 | 30.50 / 15.13 | 20.15 / 7.28 |

The first 400/sec pair validates all 12,000 responses in each run, with **6.5% less
CPU per timed completion**. Candidate CPU remains 2.30–2.34 ms across all trials.
The second baseline deteriorates to 2.65–2.68 ms; its larger apparent improvement
is not attributed entirely to the patch. The clean pair's timed throughput does not
increase at fixed offered rate; 21 baseline and 60 candidate responses finish in
drain. Under overload the first pair improves 405.8→432.5 timed completions/sec
(6.6%), but both lose arrivals. RSS peaks overlap: baseline 12,880–12,896 KiB,
candidate 11,200–12,912 KiB. No RSS improvement is claimed.

**No higher reliable sustained rate is established.** The candidate's second
400/sec run misses 66 arrivals; both 500/sec candidate trials fail qualification.
All admitted requests validate, server failures remain zero and shutdown accounting
passes. Go validates all 54,000 requests but three of four controls exceed the 5 ms
generator-lag criterion. Only its second 400/sec run qualifies in this set. Candidate
CPU/request is still about 10.9–11.0× Go at matched 400/sec, an observed same-host
ratio with scheduling/load limitations, not a universal runtime ranking. The prior
[quiet-host qualification](../capacity-20261018/README.md) remains historical evidence;
these counterexamples show why it cannot be treated as a guarantee.

Fresh separate 300/sec profiles show `strstr` top-of-stack counts falling from
161 to 75, consistent with less search work. Allocation, collection, root publication
and `strlen` remain major costs. Sampling perturbs execution: the baseline/candidate
profiled runs themselves miss 104/29 arrivals, and are not capacity tests. Counts
are wall samples, not exact CPU costs or a cache hit rate.

### Retained evidence

- [Run ordering/commands](evidence/comparison.json), with full lossless reports linked
  from [baseline 1](evidence/baseline-1.json), [baseline 2](evidence/baseline-2.json),
  [candidate 1](evidence/candidate-1.json), [candidate 2](evidence/candidate-2.json),
  [Go 1](evidence/go-1.json) and [Go 2](evidence/go-2.json).
- [Candidate build and source hashes](evidence/candidate-build.json),
  [probe build](evidence/probe-build.json), and [Go build metadata](evidence/go-build-info.txt).
- [45 native GC/root test executions](evidence/gc.json) pass with UBSan/bounds,
  including malformed ancestor descriptors, mutable plan hints, reclamation,
  owned bodies and idle drain. [Nine Serve lifecycle cases](evidence/lifecycle.json)
  pass across AOT/JIT/interpreter. [Exact Go/neoCLR response bytes](evidence/wire.json)
  match. Six load-controller tests pass. This is macOS ARM64 validation; Windows was
  not rerun, and no public API or website generation change is needed.
- [Baseline profile](evidence/sample-baseline-summary.json) and
  [candidate profile](evidence/sample-candidate-summary.json), each linking the raw
  compressed sample with its hash. Diagnostic request outcomes are retained in
  [baseline](evidence/sample-baseline.json) and [candidate](evidence/sample-candidate.json).
