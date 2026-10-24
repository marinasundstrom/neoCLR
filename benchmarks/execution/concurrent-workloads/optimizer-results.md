# Generated-code optimization qualification (2026-10-25)

The previous stack-transfer measurements used Cranelift's default `none` level.
The pinned Cranelift 0.121.2 setting declares `none` first and describes it as
minimizing compilation time by disabling most optimizations; `speed` enables
execution-oriented optimization, including its default alias analysis.
Source: `cranelift-codegen-meta-0.121.2/src/shared/settings.rs`, `opt_level` and
`enable_alias_analysis`. C host `-O2` does not optimize the managed object.

For comparison, [.NET's compilation configuration](https://learn.microsoft.com/en-us/dotnet/core/runtime-config/compilation)
distinguishes quick compilation from optimized code and uses tiering to balance
startup and execution. This experiment does not implement tiering or change .NET
semantics: it tests an existing backend optimization level, with compilation
excluded from execution timing. Compile-time cost remains a separate tradeoff.

A common settings factory now covers scalar/value AOT, native libraries, managed
and scalar JIT and the relevant native test fixtures. Only test builds read
`NEOCLR_TEST_OPT_LEVEL=none|speed|speed_and_size`; ordinary builds retain defaults
and expose no new CLI/API contract. An invalid test setting fails closed. Use a
separate runner process per setting, never mutate the environment during compilation.

The retained-assembly recompiler accepts `--both` to rebuild synchronous and live
objects. It refuses to reuse a synchronous object from a different recorded
optimization level. Build reports record the setting, and standalone measurement
reports carry it forward. Both collectors must receive the same setting when
comparing them; a live/speed versus sync/none result cannot establish collector
superiority. Existing ABI, runtime contract, root descriptors, store barriers,
ownership and helper-admission rules are unchanged.

## Correctness qualification

The optimized backend passes all 183 nonignored codegen tests (33 require external
fixtures and remain ignored in that invocation). The first broad run exposed a
pre-existing diagnostic mismatch that also failed with `none`: reflection errors
still said `Option<TypeInfo>`, despite the existing contract and test requiring
`Optional<TypeInfo>`. Correct both declaring/base-type diagnostics; leave selection
and runtime behavior unchanged. The original test remains intact.

Forced/paced native fixtures pass 600 entry cycles, 840 record-array cycles and
720 primitive-box cycles across JIT and AOT with O2 and UBSan/bounds C hosts.
These exercise changing roots, multi-lane references, heap result aliases,
initialization, faults, pressure/exhaustion, retention and clean teardown.
The first runner differs from the final runner only in the diagnostic correction.

An explicit `none` rebuild of the retained task live object is byte-identical to
the preceding stack-transfer object. Invalid test optimization values reject.
All three retained standalone assemblies compile for both collectors at `speed`.
Production settings and concurrent-default acceptance remain unchanged.

Execution measurements follow below.

Fresh optimized plaintext and JSON JIT consumers each pass 128 checked responses,
134 completed epochs and clean shutdown (256 additional responses total).
Compilation and selection times in those reports come from the debug qualification
runner and are not request CPU measurements.

## Matched results

Five alternating rounds of 16 warmup plus 256 measured HTTP requests compare
`speed` sync, guarded-sync and live with `none` guarded-sync/live. All **13,600**
responses pass, with completed concurrent epochs and clean owner teardown. The
previous-code controls use the same native collector and providers. The ordinary
sync comparison uses `speed`, matching the new live setting.

| Handler | Live none → speed requests/s | Live CPU none → speed | Guarded-sync speed CPU | Ordinary sync speed CPU |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 1,484 → 1,561 | 0.194572 → 0.190640 s | 0.170875 s | 0.055900 s |
| JSON | 899 → 934 | 0.310451 → 0.298326 s | 0.267503 s | 0.091412 s |

Observed live throughput gains are 5.2%/3.9%, CPU reductions 2.0%/3.9%. Ordinary
sync reaches 4,420/2,400 requests/s in this sequential harness and remains much
faster. These are short loopback fixed-work measurements, not saturated capacity,
worker scaling, Kestrel parity or isolated GC pause distributions. Background
system activity limits small timing conclusions; no confidence interval is claimed.

Allocation counts/bytes match across all five controls: plaintext 57,120 /
4,394,837 and JSON 124,873 / 7,163,755 per 272 responses. Out-of-arena store calls
are unchanged at 2,710,373 and 4,582,821. The optimizer cannot remove calls whose
observable effects include checked publication and mutation logging. Median live
peak process RSS falls from 11,927,552 to 11,386,880 bytes for plaintext and from
14,172,160 to 13,221,888 for JSON; process RSS includes code/native memory and must
not be described as fewer managed allocations.

Forty-five standalone runs (five rounds, three modes, three samples) pass exact
output, allocation equality and lifecycle checks at 4,096 iterations each:

| Sample | Sync CPU | Guarded-sync CPU | Live CPU | Managed allocations / bytes |
| --- | ---: | ---: | ---: | ---: |
| Tasks | 0.015219 s | 0.049574 s | 0.082322 s | 65,548 / 2,261,361 |
| Collections | 0.089643 s | 0.257219 s | 0.421685 s | 237,580 / 7,864,689 |
| Arrays | 0.050844 s | 0.072714 s | 0.095401 s | 86,028 / 1,892,721 |

Previous live CPU medians were 0.083976/0.433332/0.097986 seconds. Those were a
separate cohort, so the roughly 2–3% reduction is suggestive, not a statistically
established effect. Counts and bytes are unchanged. Other retained samples' helper
admission gaps and cancellation batching limits remain as documented in the
[workload inventory](README.md); optimization does not relax their contracts.

## Costs and next target

`speed` reduces standalone object file sizes by about 11–18%, but this debug
qualification driver takes 81/110 seconds to compile plaintext sync/live and
118/156 seconds for JSON. These compilations overlapped independent qualification
work and are not release-compiler startup benchmarks. A short macOS `sample`
capture of JSON compilation places most samples in Cranelift code generation,
including optimization, rather than a deadlock. Keep compilation outside request
CPU timing and qualify release-driver startup before considering a default change.

The central result is the remaining guarded-sync overhead even without a marker:
0.171 versus 0.056 seconds for plaintext, and 0.268 versus 0.091 for JSON. Combined
with unchanged checked-store counts and prior site attribution, this makes the
native store adapter and repeated per-store validation/serialization a concrete
next profiling target. The control difference is not a precise attribution of all
that time to one function. Preserve ownership, cross-heap restrictions, bounds,
snapshot identity, mutation logging and fault semantics when testing improvements.
Do not infer that all out-of-arena writes are stack writes or bypass safety based
on the measured address distribution.

Production compilation and GC defaults remain unchanged. Continue with guarded
store costs, outer mutator pauses and broader worker/scaling acceptance; compiler
optimization alone does not close the concurrent/synchronous gap.

Raw build, correctness and timing reports are retained as `optimizer-*.json.gz`,
with `optimizer-summary.json`, test logs and the diagnostic compilation sample.

## Reproduction

Use an ad-hoc-signed copy of the debug `neoclr-codegen` test executable, as described
in [the previous evidence](transfer-results.md), and the retained input artifacts.
Run compiler qualification with `NEOCLR_TEST_OPT_LEVEL=speed`. For ordinary samples:

```sh
NEOCLR_TEST_OPT_LEVEL=speed python3 benchmarks/execution/concurrent-workloads/recompile-live.py \
  --both --build target/concurrent-workloads/transfer-build \
  --runner target/concurrent-workloads/test-runner-optimizer-final --output NEW_BUILD
python3 benchmarks/execution/concurrent-workloads/measure.py \
  --build NEW_BUILD --output NEW_MEASURE --rounds 5 --guarded-control
```

Build both HTTP selections with `qualify-http-selection.py` under the same setting;
run `measure-plaintext.py` with the resulting sync/live objects and the previous
live object, `--pairs 5 --include-guarded-sync`, once per handler. JIT execution also
requires the environment setting; AOT measurement runs the already compiled object.
Finish all compilation before collecting request CPU/throughput. The two native
array/boxing reports were generated before their scripts recorded the setting
explicitly; both invocations used `NEOCLR_TEST_OPT_LEVEL=speed`, as did the entry
fixture and the broad compiler test run. No `speed_and_size` execution is qualified.
