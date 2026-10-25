# Complete managed borrowed-store qualification

2026-10-25. Follow-up to [explicit lowering](borrowed-lowering-results.md) and
[native borrowing](store-handle-results.md). The integration passes correctness
checks, but the full-workload experiment regresses CPU and throughput. Keep it
opt-in; synchronous GC remains faster and the production default.

## Contract and boundaries

Test builds alone recognize `NEOCLR_LIVE_BORROW_STORES`. Framed value/replacement
bodies acquire candidate state once after frame linkage and retain existing fault
cleanup. Admission requires owned state handles and excludes native worker
safepoints. Unframed entry/adapter paths keep checked stores. Complete closure
analysis recognizes both exact store signatures and preserves the same bounds,
escape and caller proofs when removing stack guards. No managed API changes.

The private borrow bit moves from 16 to 32: full HTTP integration exposed that
native scopes already used bit 16. The host generator now uses a named scopes
capability instead of rewriting the query expression. Base/scopes/borrow/both
masks are 15/31/47/63; unknown versions return zero. Rebuild previous experimental
borrow hosts. Existing ordinary hosts remain compatible with ordinary stores.
The initial HTTP generation assertion exposed this collision before compilation.

Borrowing saves only candidate TLS-address resolution. Heap/context/destination
validation, epoch state and ownership remain live checks in their owning service.
Handles cannot cross native threads. This does not qualify shared-worker heaps,
migration, arbitrary native pointers or concurrent-default admission. The native
ABI adds an opaque handle to the store hook; Runtime Contract, Raven semantics,
CLI representation and existing temporary bridge restrictions are unchanged.
neoCLR owns lowering and the experimental host owns lifetime/compatibility.
This reuses the .NET/Go ownership research linked by the native prototype; it is
not a change to .NET-facing behavior or a claim to reproduce CLR internals.

## Validation

- 50 mutation/admission regressions, 29 managed selection tests and the JIT
  width/alias elision test pass (16 combinations in the last test).
- Entry/record consumers pass 1,440 native AOT/sanitized-AOT/JIT cycles.
- Four host capability configurations pass. A further 300 explicit borrowed-store
  cycles pass, including rejection of a scopes-only host (mask 31) before binding.
- Managed JIT serves 1,024 checked plaintext/JSON responses and runs six standalone
  batches of 4,096 iterations. These complete 2,561 concurrent epochs and release
  index metadata. HTTP JIT here uses paced hosts; this is not forced HTTP evidence.
- Five AOT workload pairs retain import/hash audits: previous objects import only
  checked stores, current objects import borrowed/acquire plus checked adapters.
  Stack elision counts match: tasks 64, collections 159, arrays 86, plaintext
  13,807, JSON 20,543. AOT objects predate the capability-only correction; final
  JIT/native validation uses the corrected runner. Emission was unchanged.

## Matched performance

Uninstrumented, sequentially run macOS ARM64 cohorts compare sync, guarded-sync,
live, and the previous guarded/live objects. Current borrowed modes alone add
`NEOCLR_LIVE_STORE_HANDLE`; other native options match. The retained command
manifest records reproduction arguments, sources, retained bundle and object
provenance. Existing artifacts named there are required, not distribution inputs.
150 balanced standalone executions and 82,560 checked HTTP responses pass.

| Workload | Paired live CPU change | Paired elapsed/throughput change |
| --- | ---: | ---: |
| Tasks | +2.2% | elapsed +2.2% |
| Collections | +0.7% | elapsed +0.5% |
| Arrays | +1.5% | elapsed +1.5% |
| Plaintext HTTP | +3.0% | throughput −1.6% |
| JSON HTTP | +3.3% | throughput −3.6% |

Current live plaintext medians are 3,009 requests/s and 0.727 process CPU seconds,
against sync 5,115 requests/s and 0.404 seconds. JSON is 1,887 requests/s and
1.138 seconds, against sync 3,082 and 0.669. These are this owned-heap experimental
harness's results, not modern-server capacity comparisons or scaling limits.
Guarded controls also show small CPU increases (0.3–1.5%). No confidence intervals
or population-level significance are claimed for these small changes.

Managed allocations match exactly across standalone cohorts and current/previous
live HTTP. Guarded HTTP contains up to two additional allocations/1,056 bytes;
retain ranges rather than claim equality across every mode. There is no allocation
reduction. Peak live/arena bytes are unchanged in standalone samples; HTTP RSS is
reported separately. This cohort does not measure complete pause distributions.

The prior microbenchmark excluded acquisition. This experiment includes acquisition
per activation and the extra store argument; either acquisition or generated code
cost may explain the reversal, but these timings do not isolate causality. Do not
promote borrowing as an optimization. Next investigate whether existing root-state
acquisition can safely carry an opaque candidate association, preserving separate
ownership and reset/park/thread lifetimes, and measure before accepting such a design.
This is a proposed next investigation, not an implemented lifetime contract.

## Retained evidence

`borrowed-managed-summary.json` contains paired ratios and allocation ranges;
`borrowed-managed-{ordinary-measure,http-plain,http-json}.json.gz` retain raw trials.
`borrowed-managed-measure-commands.json.gz` retains commands. The summarizer checks
native flag matching, pair membership, outputs, completed epochs and cleanup;
`borrowed-managed-negative.json` rejects missing/extra flags, a missing trial and
failed response. Run `summarize-borrowed-stores.py --root <results-root> --output
<summary.json>` after restoring raw reports to their named `results.json` folders.

The matching `borrowed-managed-*-build`, `*-jit`, `*-final-entry`, `*-final-record`,
capabilities, imports and scopes-negative compressed reports plus final test logs
retain correctness/provenance evidence. Production switches remain unchanged.
