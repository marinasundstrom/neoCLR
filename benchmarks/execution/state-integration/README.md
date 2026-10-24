# Generated execution-state handles — 2026-10-24

Retained native backend/runtime integration of the [scoped prototype](../state-handle/README.md).
Baseline `a45a08c05`. This is a modest bookkeeping improvement, not concurrent GC.

## Implemented contract

For native GC without shared-worker safepoint lowering, generated entry uses
`neoclr_gc_state_enter_v1`, returning the state pointer already resolved during
frame admission. A compiler-owned eight-byte slot immediately after the 104-byte
collector-visible frame stores it. The combined stack reservation is 112 bytes;
stack-budget accounting includes the extra slot. It is never a managed root.
Typed stack/transient publication and every normal/fault return pass that handle
to the matching state imports. The collector's frame structure/traversal stays intact.

The handle is private to the runtime instance and owner native thread. It expires
with the activation. Same-thread cooperative suspension and nested host reentry
preserve its identity, but head/depth, descriptors, fault context and changing root
values remain checked/read on use. It must not escape into managed objects, queues,
thread transfers or migrating continuations. Forged/stale/wrong-thread handles are
outside this trusted generated-code contract, not dynamically validated capabilities.

Ordinary native callers, diagnostic probes and shared-worker lowering retain the
checked imports. Shared-worker safepoints still coordinate collection; no poll,
barrier, root update or GC budget is removed. The native collector remains blocking
for a heap's managed execution, and the interpreter is unchanged. Independent heaps
may execute/collect on different threads; that is not concurrent tracing of one heap.
New native objects require matching state imports in their host/native kit. A checked
old-host JIT attempt fails on the missing import before guest execution.

No Raven source semantics, managed API, Runtime Contract option or CLI representation
changes. This is a neoCLR backend contract; Raven's companion branch records it
without changing its compiler binary. Windows tests qualify object emission only.
Installed/released packages are not refreshed by this development change.

## Measurements and costs

Same managed inputs; old/new objects and matched native helpers. Eight AOT trials
(one/four isolated owners) and four one-owner JIT trials in forward/reverse order,
32 connection-close clients, GOMAXPROCS=2, two-second warmup, ten-second load.
No concurrent builds/profiling during capacity. AOT CPU includes server lifetime;
JIT CPU begins at READY and excludes compilation. Do not pool those scopes.

| Configuration | Throughput change | CPU/response reduction |
|---|---:|---:|
| AOT one owner | +1.13% | 1.56% |
| AOT four owners | +0.34% | 1.40% |
| Owned JIT one owner | +0.88% | 1.14% |

CPU ranges separate in this small cohort; JIT throughput ranges overlap. Latency
is mixed (four-owner AOT p99 is worse, JIT ranges overlap). These are modest local
results, not confidence intervals or peer parity. Costs: eight more native stack
bytes per affected activation, additional generated handle loads/arguments, and
1.80%/1.91% larger plain/JSON native objects. Managed allocation/publication counts
remain identical: 208 allocations/3,676 publications per plain request, 457/5,535
for JSON (32→64 deltas). JSON capacity is not measured here.

The initial entry adapter called the existing entry helper and then resolved TLS
again to return the handle. Its capacity regressed. Disassembly exposed the extra
resolver; returning the already-resolved state through the common admission helper
removes it. `capacity-initial.json.gz`, initial source and assembly preserve that
failed intermediate. The retained entry adapter tail-calls admission; cached
publication/leave use the supplied state. Slow collection and other services still
resolve their own TLS. This does not eliminate runtime TLS overhead.

## Validation and provenance

- 14 compiler/generated checks: six root-publication, six Windows object-generation,
  two forced-GC consumers covering borrowed owners, erased results, fault messages
  and references beyond inline lane 64. The final fixed-array argument lowering
  avoids a new temporary compiler Vec; 12 relevant compiler checks rerun and both
  HTTP objects are byte-identical to the measured objects.
- Four native lifetime configurations, main plus two fresh threads, and 28 expected
  aborts; the normal fixture is registered in the standard native validator.
- Existing 230 checked-root/slot-map runs, including expected rejection paths.
- Newly generated shared-worker graph/fault stress and 20 optimized/sanitized async
  configurations pass. Shared-worker objects continue using checked imports.
- Forced-GC plain/JSON requests pass in AOT and JIT, normal 32/64-request runs of the initial adapter
  verify bodies, completions and work counts. The fused adapter repeats forced-GC checks; its entry change performs no allocation or publication. Diagnostic timer values are not used
  for CPU shares: independent short runs and concurrent qualification can distort
  subtraction. Separate long collector measurements follow in the concurrent-GC study.
- New JIT code against an old host rejects the missing state import before READY.

Raw reports, logs, source identities and initial/fused assembly are retained here.
`summary.json` comes from `../owned-root-chain/summarize.py`. `build.py` rebuilds
plain/JSON objects; its first invocation assumed the wrong recipe shape and failed
before compilation (`objects-recipe-failure.json.gz`); the corrected recipe passes.
`measure.py` takes baseline/candidate objects, and `jit.py` uses the preserved baseline
codegen executable for baseline trials. `counts.py` binds the rebuilt objects.
Reproduction requires matching baseline/current compilers and native sources;
see recorded commands and hashes. No new managed API snapshot or website build is
needed for these private hooks.
