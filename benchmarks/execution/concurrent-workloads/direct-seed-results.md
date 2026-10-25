# Reusing validated headers during concurrent seeding

2026-10-25. Follow-up to [seed phase attribution](seed-parts-results.md), on the
retained macOS ARM64 owned-heap experiment. Production GC remains synchronous.

`NEOCLR_LIVE_DIRECT_SEED` avoids converting an already validated new-object header
back to an address and resolving it through `indexed_block`. The current heap walk
has already established the exact header boundary and allocated state, following
`build_index` validation; the owner cannot change layout during final collection.
This private helper accepts only that local proof, not arbitrary pointers or roots.

The generator extracts the state transition from canonical native `mark` rather than
maintaining a second implementation. Preserve zero-byte exclusion, already-marked
objects, pending-list links, atomic TEXT/BYTES handling and `atomic_marks`, which
participates in fixed-point progress. All old snapshot identities are still validated
before marks are applied. No barrier, root, ownership, layout or allocation change.
The indexed path remains available and is the default; this change does not enable
the separate linear old-identity validation experiment.

This follows the existing [.NET/Go metadata research](allocation-index-results.md):
reuse a validated internal representation within its proven lifetime instead of
re-resolving it. It changes no public .NET-shaped contract or compiler/CLI bridge.
No Raven changes are required. No additional metadata or allocation is introduced.

Native safety and mutation/ownership negative controls pass. Optimized and
UB/bounds-sanitized comparisons test 24,768 combinations of payload size (including
zero), sixteen kind values, unmarked/queued/scanned state and existing pending tails
against the generated canonical mark function. A deliberately missing atomic count
must fail. These focused comparisons test transition equivalence, not every possible
thread interleaving; native lifetime/barrier tests provide separate coverage.

An initial generator assertion rejected a trailing-newline mismatch before compiling
any runtime; the corrected extraction retains the complete canonical transition.
The earlier ASan initialization gap remains unresolved; sanitizer evidence here is
UB/bounds only. Shared heaps, broader pause qualification and concurrent default
acceptance remain open.

## Uninstrumented comparison

150 ordinary executions and 82,560 checked HTTP responses pass with unchanged
managed object files. Five modes retain sync, guarded-sync and live controls with
and without direct seeding. Ten forward/reverse rotating standalone rounds balance
ordinal position; HTTP uses four paired rounds, four clients, 2,048 requests and
16 warmups per executable. Only the direct-seed native define differs within a pair.
Summary checks reject failed output, missing trials and unexpected compiler flags.

| Live workload | Paired CPU change | Paired throughput change |
| --- | ---: | ---: |
| Tasks | -0.04% | — |
| Collections | -1.19% | — |
| Arrays | -0.49% | — |
| Plaintext HTTP | -0.95% | +2.03% |
| JSON HTTP | +0.21% | +0.35% |

These small mixed effects do not justify promotion. In particular, JSON's ratio of
paired throughput differs from its aggregate medians: 1,981.5 requests/s indexed
versus 1,979.9 direct. Plaintext medians are 3,051.3 versus 3,111.3 requests/s; sync
remains faster at 5,083.0. JSON sync is 3,094.5 requests/s. Guarded controls, whose
seed path is inactive, also fluctuate; no broad speedup or pause advantage is claimed.

Standalone allocation totals match exactly (tasks 65,548; collections 237,580;
arrays 86,028). Live plaintext allocations match at 429,856. JSON ranges differ by
one scheduling-dependent object: indexed 943,817–943,818, direct 943,817. Retain the
range, not a claim of reduced allocations. Existing allocation-index metadata cost
and shutdown release remain unchanged.

Entry and record-array consumers pass 600 + 840 JIT/AOT cycles at `speed`, including
forced/paced and sanitized native hosts. These validate native direct-seed behavior;
they do not establish broader transformed application or shared-worker admission.

## Phase attribution and decision

Separate diagnostics pass 18 ordinary executions and 8,320 HTTP responses. Pooled
application CPU per finish (indexed → direct, nanoseconds): tasks 2,735 → 2,067;
collections 4,547 → 2,681; arrays 3,415 → 2,294; plaintext 8,971 → 5,499;
JSON 10,261 → 6,092. These local reductions of 24–41% support the mechanism, but
instrumentation/inlining perturb timings. They do not override the small mixed
uninstrumented application results or establish pause/latency improvement.

Keep the direct path opt-in. Profile whole-request execution again before investing
further in this phase: removing much of its local cost barely changes overall CPU.
Retain ownership/mark-state proof reuse as reusable work, while investigating the
larger remaining store/root/execution costs. No concurrent-default promotion.

Evidence: `direct-seed-summary.json`, `direct-seed-{ordinary,http-plain,http-json}.json.gz`,
`direct-seed-{ordinary,http-plain,http-json}-profile.json.gz`, and
`direct-seed-profile-summary.json.gz`. Safety reports are
`direct-seed-{safety-v2,entry,record-arrays}.json.gz`; model/negative output is in
`direct-seed-tests.log`; summarizer negatives in `direct-seed-summary-negative.json.gz`.
Commands are retained in `direct-seed-{commands,profile-commands}.json.gz`, with
source hashes in `direct-seed-inputs.json`. Retained bundles/objects are required.
Use measurement `--compare-direct-seed` alongside inline/index controls, native
validation `--direct-seed`, and `concurrent-live/test-direct-seed.py` for transition
comparisons. The initial rejected generator report remains under
`target/concurrent-workloads/direct-seed-safety`; successful evidence is `safety-v2`.
