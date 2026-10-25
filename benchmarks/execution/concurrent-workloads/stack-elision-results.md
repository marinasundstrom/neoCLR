# Bounded stack-argument guard removal (2026-10-25)

## Implemented experiment and boundaries

The [read-only audit](stack-argument-audit-results.md) now supports opt-in guard
removal in the test compiler with `NEOCLR_LIVE_ELIDE_STACK_ARGUMENTS=1`. It runs
only after the existing live call-closure validation and checks that every
non-imported function declaration has a supplied body. The compiler driver emits
function addresses through explicit IR; it does not embed them in data relocations.
This is a documented precondition of the private pass, not a generic module-wide
escape-analysis API.

Only local, non-address-taken direct callees whose every caller supplies a bounded
stack address qualify. Stack slots remain alive during these direct calls. Width
and nonnegative offset must fit the minimum remaining extent across all callers.
Loads, ambiguous joins and unknown forwarding remain unproven. Callback targets,
exports, missing bodies and mixed stack/native-pointer callers remain guarded or
reject the pass. Native output aliases retain their existing contract.

At a qualified hook call, emit an unaligned native store of the original low
1/2/4/8 bytes and replace the I32 status with zero. Preserve its SSA identity,
branches and root cleanup; backend optimization can simplify the constant branch.
No allocation, safepoint, collector scheduling or root-publication operation is
removed. This extends the existing compiler-stack exemption; it does not let the
collector infer ownership from an arbitrary address outside its arena.

The .NET comparison remains compiler-known stack storage versus managed-heap
barriers, as in the existing provenance research. No public API, Raven emission,
Runtime Contract or temporary CLI encoding changes. neoCLR owns this test-only
native codegen experiment and its eventual qualified implementation. Production
code generation and the synchronous GC default remain unchanged.

## Safety and execution

Three focused tests pass. The audit/bounds tests now verify generated IR and
idempotence, and reject a missing function body without mutation. A native JIT test
runs sixteen baseline/candidate combinations over unaligned 1/2/4/8-byte stores
and mixed stack/external-pointer callers. Values and sentinels match; hook counts
confirm removal only for the all-stack cases and retention for mixed callers.

Existing entry and record-array fixtures pass 600 and 840 JIT/AOT cycles, including
optimized and UB/bounds-sanitized hosts. They contain **zero eligible stores**;
this is regression/exclusion evidence, not execution coverage of replaced stores.
Actual transformed execution is covered by the dedicated JIT test and the ordinary
and HTTP AOT cohorts below. Broader transformed managed-JIT/alias stress and pause
qualification remain required before promotion.

Complete HTTP compilation removes the audited 13,807 plaintext / 20,543 JSON
static sites. Standalone tasks/collections/arrays remove 64/159/86 respectively.
The hot header accessor and loaded Result receiver stores remain guarded.
All source/selection and object hashes are retained; no duplicate object binaries
are committed. The later final runner adds only a missing-body test and comments
to the pass used for these compiled workload objects.

## Matched measurements

Both generated-code variants use identical native inlined exact-index hosts.
The summary verifies normalized build-command equality beyond managed object and
executable names. Sync and guarded-sync controls are retained. Instrumented profiles
are excluded from timings. All native builds finish before timed trials.

The ordinary cohort has five modes and ten rotating forward/reverse rounds: each
mode occupies each position twice per sample, **150 executions**, exact expected
outputs and identical managed allocations/bytes across modes.

| Sample | Previous live CPU | Candidate live CPU | Paired median CPU change |
| --- | ---: | ---: | ---: |
| Tasks | 0.0237105 s | 0.0227695 s | −3.9% |
| Collections | 0.134414 s | 0.126997 s | −5.6% |
| Arrays | 0.0567415 s | 0.055508 s | −2.2% |

Each HTTP sample uses four connection-close clients, 2,048 measured requests plus
16 warmups, five modes and four alternating rounds: **82,560 checked responses**
total. Every live trial completes concurrent epochs; guarded-sync completes none.
Mutation/native-scope failures are zero and index metadata is released at shutdown.

| Sample | Previous live req/s | Candidate live req/s | Previous/candidate process CPU |
| --- | ---: | ---: | ---: |
| Plaintext | 2,661 | 3,071 | 0.808340 / 0.704171 s |
| JSON | 1,663 | 1,946 | 1.291390 / 1.109810 s |

Paired median throughput ratios are 1.156/1.165; paired CPU ratios 0.872/0.857.
Ratios of aggregate medians differ slightly. Sync still achieves about 5,161/3,032
req/s here. Guarded-sync also improves, supporting the interpretation that removing
redundant store checks helps independently of background collection.

No managed allocation reduction is claimed. Live plaintext retains 429,856
allocations / 33,057,881 bytes. Candidate JSON retains 943,817 / 54,065,775;
previous JSON varies by one additional 528-byte allocation. Whole HTTP cohorts do
not have exact cross-mode equality. Ordinary peak live/arena figures are unchanged;
HTTP median RSS falls slightly (about 57/156 KiB), which is not a reliable general
memory-saving claim. Native index capacity remains 16,640 bytes per 1-MiB arena.

These are local fixed-work comparisons, not saturation, cross-framework capacity
or GC pause distributions. Keep this test-only opt-in while qualifying broader
managed JIT and boundary stress. Next profile the remaining guarded stores/root
work; loaded receiver provenance requires independent alias/write reasoning.

## Evidence and reproduction

`stack-elision-summary.json` contains checked paired ratios, allocation ranges and
input hashes. `stack-elision-{ordinary-measure,http-plain,http-json}.json.gz`
retain complete trials; `stack-elision-measure-commands.json.gz` retains invocation
commands. Build reports and entry/record-array results use corresponding names.
`stack-elision-tests.log` records the final focused tests.

`audit-stack-arguments.py --elide` builds HTTP candidates; without that switch it
still requires byte-identical objects. `build-stack-arguments.py` rebuilds the
retained ordinary selections and reuses sync objects. `measure.py --previous-build`
and `measure-plaintext.py --previous-inline-object` compare old/new generated code
with identical native settings. `summarize-stack-arguments.py` checks successful
execution, ordinary position balance, metadata cleanup and matching native recipes
before summarizing. Three negative controls reject mismatched native flags,
unbalanced ordinary trials and failed HTTP execution; evidence is retained in
`stack-elision-summary-negative.json.gz`. Existing retained bundle/host recipes
are required.

The subsequent [managed-JIT qualification](stack-jit-results.md) closes the
bounded transformed HTTP/standalone JIT execution gap identified above. Broader
boundary stress and full pause/default qualification remain open.
