# Acquisition callers and rejected unused-handle cleanup

2026-10-25. Follow-up to [acquisition density](store-acquisition-results.md).
This narrows the next compiler optimization using actual caller counts. It does
not enable borrowing, change GC ownership or establish a new throughput gain.

## Caller evidence

`profile-store-acquisitions.py --sites` adds a bounded, main-owner-thread native
histogram of acquisition return addresses. It removes the executable's ASLR slide
and resolves addresses against that executable's text symbols. The summarizer maps
`neoclr_value_N` through the matching selection's compiled index, checking input
and object hashes and exact count sums. This is diagnostic-only macOS evidence:
no timing or CPU claim is made from instrumented runs. Each table is 16,384 slots;
full-table insertion aborts rather than silently dropping events or looping forever.
It does not collect migrating/multiworker activation counts.

All three existing 4,096-iteration standalone batches and both 512-response HTTP
consumers pass exact output checks, completed-epoch accounting and metadata release.
The final run records executable hashes as well as command/source/object provenance.
Acquisition totals reproduce the earlier aggregate counts exactly. Two negative
summary controls reject a mismatched count and an invalid retained-input hash.

| Sample | Observed acquisition sites | Frequent callers |
| --- | ---: | --- |
| Tasks | 80 | Object constructor; Optional constructors; ArrayList count/add |
| Collections | 162 | Object constructor; Order.Pending; iterator operations; WriteLine |
| Arrays | 87 | WriteLine; Int32ToString; TypeShape; Console.WriteLine |
| Plaintext HTTP | 711 | Header buffer index/count; sequence/array index; ViewLength |
| JSON HTTP | 887 | Header buffer index/count; Object constructor; sequence/array index |

Arrays execute 61,440 acquisitions in `neoCLR.Runtime.WriteLine` alone (12.9% of
475,184). HTTP header-buffer index/count accessors together account for 230,912
acquisitions: 11.6% of plaintext and 7.7% of JSON totals. These are invocation
shares, not CPU shares. Calls in synthetic functions absent from the selection
remain unmapped: 512 plaintext acquisitions and 5,120 JSON acquisitions; all
standalone acquisitions map. Their symbols/counts remain in the report.

## Why dropping wholly unused handles is insufficient

An experimental post-elision cleanup removes only calls to the private address
getter whose SSA result has no remaining use. It does not remove root entry or
resource-acquiring calls. The existing width/alias JIT fixture, augmented with
acquisition counters, passes all sixteen combinations: safe elision removes the
unused acquisition, while mixed pointers retain acquisition and store checks.

However, retained audits have zero functions whose guarded stores all disappear
in **each of the five workloads**. Rebuilding complete selections with this cleanup
produces byte-identical objects against the prior borrowed-store objects. Therefore
there is no reason to remeasure execution or retain this runtime change. The patch
and test output are archived for review; the working compiler is restored to the
committed implementation. This is a rejected optimization, not shipped behavior.

The pre-lowering IR for arrays' `neoclr_value_21` (`neoCLR.Runtime.WriteLine`)
explains one concrete source of low density. Stores write fault code/message only
on error branches; its successful branch returns without a store. The current
borrowed lowering nevertheless acquires at frame entry. A diagnostic emission
first hit an existing-object output rejection after writing the IR; rerunning with
a fresh object destination passes. This does not count as a runtime failure.

## Next implementation target

Investigate acquisition placement or checked-store fallback for conditional fault
paths. Preserve frame-linkage dominance, same-thread lifetime, store validation and
fault cleanup. A single common dominator can still be the entry block when several
separate error paths contain stores; moving acquisition without checking loops can
also increase its frequency. Use these concrete cases, not a blanket static store
threshold, to design and qualify the next lowering. Caller counts alone do not
prove that every listed getter/constructor has the same control-flow behavior.

This avoids introducing a root/candidate association before showing it is needed.
It does not authorize bypassing context/heap ownership checks or fault-root tracing.
Production remains synchronous; concurrent worker/pause/default acceptance and the
broader HTTP CPU/throughput gap remain open.

## Evidence and reproduction

- `store-acquisition-sites.json.gz`: final diagnostic builds, executable/input
  hashes, raw counts, native symbol tables, output and cleanup evidence.
- `store-acquisition-sites-summary.json.gz`: selection-linked callers and shares.
- `acquisition-sites-negative.json`: invalid-evidence controls.
- `unused-borrow-objects.json.gz`: all five recompilations, environments and hashes.
- `unused-borrow-experiment.patch.gz`, `unused-borrow-unit.log`: rejected cleanup and
  its focused test. The patch is relative to `f7054cb6e` and is not applied.
- `unused-borrow-arrays-console.clif.gz`, `unused-borrow-arrays-ir-final.log.gz`:
  concrete pre-lowering fault-path evidence and successful emission.

Run the profiler with `--root target/concurrent-workloads --output <new-directory>
--sites`, then `summarize-acquisition-sites.py --profile <new-directory>/results.json
--root target/concurrent-workloads --output <summary.json>`. Retained inputs and the
macOS SDK are required. No public API, Raven emission, CLI bridge, Runtime Contract
or production compiler behavior changes are included.
