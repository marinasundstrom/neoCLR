# Managed-JIT stack-store qualification (2026-10-25)

This follows the [matched AOT measurements](stack-elision-results.md). It changes
qualification tooling, not lowering, collector behavior, Runtime Contract, CLI
representation or Raven emission. The guard-removal pass remains test-only and
opt-in; synchronous GC remains the production default.

## What is checked

`qualify-http-execution.py --elide-stack-arguments --inline-index` explicitly
selects transformed managed JIT code and the inlined exact-index native candidate.
The harness requires a positive reported replacement count, validates every HTTP
status/body, checks the guest exit code independently of the Rust test runner,
and reads collector/index counters on the invoking owner after cleanup. The new
switch cannot be used to claim JIT qualification from an AOT-only execution.
The referenced AOT object is retained provenance, not the code executed by JIT.

Forced and paced modes use separate hosts. Forced collection exercises the
synchronous fallback and records zero concurrent epochs; it is correctness stress,
not concurrent throughput evidence. Paced runs must complete concurrent epochs.
Both require released index metadata and no active epoch after shutdown.

`qualify-stack-jit.py` runs retained 4,096-iteration tasks, collections and arrays
with guard removal disabled and enabled, using the same paced host. It checks the
complete repeated output, guest status, exact expected replacement count,
completed epochs and index release. Each case uses a fresh process; this is not a
long-running multi-invocation or cross-worker sharing test.

The first standalone launch mistakenly selected the forced host. Guest execution
succeeded, but the harness rejected its zero completed epochs. Preserve this as
`stack-jit-zero-epoch-control.json.gz`; it does not qualify concurrent execution.
The corrected cohort uses the paced host. This negative control demonstrates why
success status alone is insufficient.

## Results

All four managed HTTP JIT invocations pass **2,048 responses**: 512 each for
plaintext/JSON, forced/paced. Both plaintext selections remove 13,807 static stores;
both JSON selections remove 20,543. Forced runs report zero concurrent epochs.
Paced plaintext completes 521 and JSON 516: **1,037 completed epochs**, with one
shutdown cancellation each, zero active epochs and all index metadata released.

All six ordinary managed-JIT invocations pass exact output over 4,096 iterations
each. Baseline/candidate completed epochs match: tasks 128/128, collections
464/464 and arrays 168/168, **1,520 completed epochs** total. Candidate replacement
counts are 64/159/86; baseline emits no removal diagnostic. Every invocation
releases its 16,640-byte index metadata allocation. Across both cohorts this is
**2,557 completed concurrent epochs**. These are owned-heap correctness/lifecycle
checks, not a proof for arbitrary thread interleavings or shared managed heaps.

No new throughput, CPU or managed-allocation improvement is inferred from these
runs. Compilations overlapped, clients are sequential, and forced collection has
a different pacing policy. Reuse the previous matched AOT performance evidence.
The JIT reports separately retain compilation and invocation durations without
calling their ratio a concurrent-GC speedup.

`stack-jit-{plain,json,ordinary}.json.gz` retain commands, outputs, runner/bundle
hashes and detailed JIT results. `stack-jit-summary.json` contains the compact
checked counts. The generated native collector and owner report host are archived
as `stack-jit-{native-gc,owned-host}.c.gz`; both HTTP cohorts used identical source.

Reproduction uses `qualify-http-execution.py --execution jit --pacing both
--requests 512 --elide-stack-arguments --inline-index` with the retained final test
runner and sample object. Use `qualify-stack-jit.py --build
target/concurrent-workloads/stack-elision-ordinary-build --host
<qualification-output>/jit-paced.dylib --runner <same-runner> --output <new-folder>`
for standalone coverage. Full commands/selection recipes are retained in the raw
reports; retained compiler bundle and native recipes are required.

## Pause measurement boundary found during this work

The existing `neoclr_gc_profile.pause_buckets`, total and maximum times surround
`collect_conditional` in `runtime/native/gc/native-gc.c`. The generated live wrapper
can return through `experiment_gc_begin` or `experiment_gc_finish` before that
outer timer starts. Those functions call the timed collector recursively for
phase work, so the counters do measure that work, but **not the complete owner
stall**:

| Operation | Existing timer coverage |
| --- | --- |
| Begin snapshot/collection phase | Inner collection timed; surrounding bookkeeping excluded |
| Finish marker wait and mutation-log drain | Before the inner timer; excluded |
| Finish remark/reclamation | Inner collection timed |
| Final disposal and cancellation/join | Outside that inner timer |
| Store gate contention | Separate optional gate counters, not collection histogram |

Do not treat the existing buckets as full concurrent-GC pause distributions.
Next add diagnostic owner-operation timing for begin, finish and cancel, plus
synchronous comparison and store wait attribution. Keep nested phase counters
separate to avoid double counting; preserve sums/counts/maxima and histogram
bounds. Measure instrumentation overhead separately and use uninstrumented cohorts
for throughput/CPU conclusions. This is a plan, not implemented profiling support.
Source evidence is the generated `native-gc.c` retained and hashed by both HTTP
qualification reports, together with `automatic.py`, `generate.py` and
`native-scopes.py`.

This slice does not establish cross-worker GC, new pause distributions, an
allocation improvement or concurrent-default acceptance. Existing broader sample
coverage gaps remain documented in the workload README.
