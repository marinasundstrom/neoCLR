# Reusing unchanged slot payload counts — 2026-09-26

Follow-up to the [array walker optimization](array-budget-cost.md). Five
[traced exchanges](slot-budget-trace-runs.txt) using that optimization all passed
(client execution 6.3–8.4 seconds);
the earlier timeout was not captured. A POST sampling profile still places 472 of
689 main-thread samples under quota accounting, including 257 in heap accounting.
This supports reducing remaining scan cost, not a diagnosis of the timeout.
The temporary [trace patch](io-trace.patch) now also records true deadline-expiry
results, omitting false polls. Runtime source and comparison runners contain no
trace instrumentation. The profile itself remains a local diagnostic artifact.

## Contract and implementation

Each storage slot retains its last complete logical array element/byte counts.
A successful whole-slot assignment, interior field/element write or reset clears
those counts. Aliased writes share the same owning slot. Failed writes preserve
both storage and its counts; failed measurements never publish partial counts.
Only numeric counts are retained, so the cache cannot keep managed objects alive.

Every instruction boundary still sums all frame/heap slots and scans evaluation
stack values against the current limits. Saturating arithmetic, logical UTF-8
charging per occurrence and collection-before-rejection are unchanged. A cached
count is not cached permission: tighter limits and cumulative roots are checked
again. References are not traversed by the payload walker; their target slots are
counted separately, so this does not need cross-slot invalidation. Any future
mutable payload mechanism must preserve this invariant.

This reuses the [.NET/CLR array comparison and neoCLR quota contract](../../managed-arrays.md)
and [Raven array-shape research](../../raven-array-shapes.md). It changes an internal
neoCLR interpreter cost, not array identity, assignment semantics or public APIs.
It makes no claim that CLR performs these scans or that neoCLR is faster than CLR.
The benefit is avoiding repeated payload traversal for unchanged slots; costs are
two counters plus validity storage per slot and an invalidation obligation on all
writes. This metadata remains outside the logical payload quota.

Keeping full rescans is simpler but retains the sampled cost. A global running
counter would also avoid scanning slots, but requires accounting across ownership,
copies, limits and collection. Per-slot summaries are the bounded choice here:
no scheduling, GC policy, global accounting or deadline changes. A failed quota
pass may populate complete summaries before rejecting the aggregate; collection
retries still start with a fresh aggregate and only count surviving slots.

## Validation

Six slot and three array accounting unit tests pass. The slot cases include
replacement, reset, failed writes, aliased nested field/element mutation, changed
limits, cumulative roots, native-style whole-array replacement and collection.
Seventy-nine focused array/reference/class-field/GC integration checks pass.
The public API and website behavior/limitations are unchanged; no full suite,
compiler rebuild, API snapshot refresh or website build was needed for this slice.

The comparison below uses the same driver and release settings as the earlier
investigation, with baseline `24c342db` and a candidate containing only the runtime
summary change. Both execute identical compiled mapped GET/POST peers and a copied
System library; runner/input hashes and every observation are retained in the
[result snapshot](slot-budget-summary-results.json). Runner order alternates.
Server execution includes waiting for client preparation. This is a small local
sample with background machine activity, not controlled load qualification.

Reproduce the comparison with:

```sh
python3 docs/experiments/http-json/compare-runners.py \
  --toolchain-root /absolute/path/to/matching/development/bundle \
  --baseline /absolute/path/to/24c342db/measure_async \
  --candidate /absolute/path/to/summary/measure_async \
  --repeat 3 --output /absolute/path/to/comparison.json
```

Focused Rust checks:

```sh
cargo test --locked --lib slots::tests
cargo test --locked --lib arrays::tests
cargo test --locked --test managed_arrays --test reference_arrays \
  --test object_reference_arrays --test reserved_arrays --test reference_slots \
  --test class_field_references --test gc_diagnostics
```

| Iteration | Baseline client / server | Candidate client / server |
| --- | --- | --- |
| 1 | 5,948 / 7,662 ms | 3,128 / 5,504 ms |
| 2 | 7,334 / 8,587 ms | 2,260 / 4,613 ms |
| 3 | 8,094 / 9,802 ms | 5,239 / 10,118 ms |

All six observations pass. Both runtimes retain 761 client / 626 server managed
allocations, 13 / 11 collections and zero final live objects. The candidate client
is faster in each observed iteration; server timing is mixed and includes startup
waiting. No improved pass-rate claim follows: both variants pass this sample, and
the earlier intermittent failure was not reproduced. Preserve the failure evidence
from the prior comparison. Next capture any recurrence at native-operation
boundaries before choosing timeout or scheduler policy. The independent compiler
unresolved-call defect and packaged acceptance remain open; M1 is not declared done.
