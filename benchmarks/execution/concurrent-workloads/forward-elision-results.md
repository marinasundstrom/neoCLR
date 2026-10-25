# Forwarded stack-store guard removal

2026-10-25. The [forwarding audit](forward-stack-results.md) is now separately
qualified for test-only guard removal. Keep it opt-in: it improves this native
concurrent candidate modestly, while synchronous GC remains faster.

## Contract and implementation

`NEOCLR_LIVE_ELIDE_FORWARDED_STACK_ARGUMENTS=1`, together with the existing
`NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS=1`, selects the forwarding proof for removal.
The audit flag alone cannot enable it. The audit driver clears the new removal
flag explicitly; qualifier drivers record their selection and exact expected
counts. Production binaries do not read these test-only switches.

The proof is unchanged: every direct caller must establish a native stack extent,
or forward an already established extent with a checked constant offset. Use the
minimum bound across callers. Unknown/loaded pointers, unsupported joins, exported
or address-taken parameters and circular assumptions stay unproven. Full generated
declarations and provider/call-closure checks remain mandatory. No native/async
lifetime or ownership exemption is introduced. Caller/ancestor stack storage must
remain alive for the direct call. No class/struct/record special case is used.

The existing lowering replaces an admitted guard with a direct width-preserving
store and a zero status, retaining SSA results and cleanup/control flow. Heap
barriers, tracing, root publication, native ownership and object lookup algorithms
are unchanged. This extends compiler-known native storage proofs, consistent with
the CLR comparison recorded in the earlier audit; it is not a new .NET API semantic.
The intended native representation is unchanged scalar writes into bounded stack
storage. Runtime Contract configuration, Raven semantics/emission and temporary
CLI encoding/information limits are unchanged; neoCLR owns this backend proof and
qualification, with existing native metadata/backend replacement work still open.

## Correctness evidence and explicit gaps

- All 52 focused mutation/admission tests pass. The executable width/alias fixture
  now runs 32 combinations: direct/forwarded calls, widths 1/2/4/8, stack-only/mixed
  callers and removal off/on. It checks exact bits, adjacent bytes and guard-call
  counts. The same 32 combinations pass with the borrowed ABI as a compatibility
  check; all measured workload objects use checked stores and no borrow imports.
- 28 complete managed-selection cases pass with removal enabled. The isolated
  async-wrapper fixture is rejected by the complete-declaration gate under both
  old direct-only and new forwarding removal; it passes without removal. Preserve
  that existing admission gap rather than weakening the proof. The initial broad
  selection-run failure and matched rejection/control logs are retained.
- Entry/record native AOT/sanitized-AOT/JIT fixtures pass 1,440 cycles. They contain
  no eligible elision sites and serve as regression evidence, not transformed-chain
  coverage. Complete managed consumers below exercise the transformed code.
- Full managed JIT serves 1,024 plaintext/JSON responses and runs six standalone
  4,096-iteration batches (removal disabled/enabled), completing 2,566 concurrent
  epochs with clean index release. HTTP uses paced hosts here. The HTTP JIT counts
  are exactly 13,883/20,727 removed sites. The first standalone driver run correctly
  rejected the stale prior expectation of 159 rather than 195 collection sites;
  output and GC checks had passed. The corrected driver uses matching hashed
  forwarding audits and all six final runs pass.
- All five AOT selections compile. Tasks remain byte-identical; collections,
  arrays, plaintext and JSON change as expected. Static removals are 64/195/103/
  13,883/20,727 respectively. Symbol/hash checks confirm checked-store imports.

Broader shared-worker/migration/default and complete pause acceptance remain open.
No ASan qualification is added here; the existing sanitized fixtures use UB/bounds
checks, and their lack of eligible sites is explicit above.

## Matched performance

After all compilation/qualification processes finished, run five interleaved modes:
sync, guarded-sync/current and previous, live/current and previous. Previous uses
the retained direct-stack-elision objects; native commands must match exactly after
normalizing only the executable/object paths. Borrowing is disabled. All 150
balanced standalone executions and 82,560 HTTP responses pass.

| Workload | Paired live CPU change | Paired elapsed/throughput change |
| --- | ---: | ---: |
| Tasks (unchanged object) | +0.2% | elapsed +0.1% |
| Collections | −1.6% | elapsed −1.5% |
| Arrays | −0.9% | elapsed −0.9% |
| Plaintext HTTP | −2.3% | throughput +2.6% |
| JSON HTTP | −1.9% | throughput +2.1% |

Guarded-sync HTTP also improves about 2% CPU and 2% throughput, supporting an
execution-check effect rather than a claimed change to marker efficiency. These
are measured paired medians, not confidence bounds; small standalone effects and
the unchanged-task variation should not be overstated.

Live plaintext median throughput is 3,146 requests/s (previous 3,067; sync 5,127),
with 0.694 process CPU seconds (previous 0.708; sync 0.406). JSON is 1,980 requests/s
(previous 1,952; sync 3,077), with 1.086 CPU seconds (previous 1.107; sync 0.670).
This fixed-work owned-heap harness is not a cross-framework capacity comparison.
Requests use the same four-client protocol/workload and counts in each mode.

Counters show the intended work reduction: first matched live trials remove
2,111,472 outside-arena guard calls in plaintext (20.1%) and 2,524,272 in JSON
(15.3%). Collections remove 290,816 such calls; arrays remove 57,344, leaving 13;
tasks are unchanged. These are workload guard counts, not percentages of CPU.
Heap-store counts can move between active/inactive epochs; those barriers remain.
The much smaller CPU change is reason to attribute remaining costs before assuming
further outside-arena guard removal dominates overall execution.

Standalone and JSON allocations match exactly across modes. Prior plaintext live
and guarded controls vary by one allocation/528 bytes; current live/guarded and sync
match the lower count. This is retained scheduling variation, not an allocation
optimization claim. Peak live/arena/RSS values remain in the raw reports. No new
pause-distribution improvement is established.

## Decision and evidence

Retain the opt-in transformation and the direct-only control. Next attribute the
remaining heap checks, root publication and metadata processing under the improved
code, while continuing toward shared-worker and default qualification. Do not
promote concurrent GC on these results: sync is still substantially faster.

`forward-elision-{ordinary,http}-build.json.gz`, `forward-elision-imports.json.gz`
and `forward-elision-build-driver.py.gz` retain build commands, environment and
source/runner/object hashes. The entry/record and plain/json/ordinary-JIT reports
plus focused logs retain qualification; the initial stale-count JIT report is
separately labeled `forward-elision-ordinary-jit-initial.json.gz`. Native and managed
invocations are recorded in `forward-elision-invocations.json`.

`forward-elision-{ordinary-measure,http-plain,http-json}.json.gz`,
`forward-elision-measure-commands.json.gz` and `forward-elision-summary.json` retain
raw controls and paired metrics. Recreate the summary with
`summarize-stack-arguments.py --prefix forward-elision --root <results-root>
--output <summary.json>`. Four negative controls reject unmatched native options,
missing trials, failed responses and metadata leaks. The generalized prefix parser
also reproduces its original retained summary byte-for-byte. SDK/bundle and retained
input paths named by these recipes are required; these are internal experiment tools.
