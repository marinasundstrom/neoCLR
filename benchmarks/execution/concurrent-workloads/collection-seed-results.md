# Final-collection attribution and linear seed validation

2026-10-25, retained macOS ARM64 owned-heap experiment. Concurrent GC remains
opt-in; synchronous GC remains faster and the production default. This follows
[finish attribution](finish-phases-results.md), not a new integration or shared-heap
admission. No managed API, compiler emission, Runtime Contract or Raven bridge changes.

## Where final collection spends owner CPU

The diagnostic cohort passes 27 standalone executions (tasks, collections, arrays)
and 12,480 checked plaintext/JSON HTTP responses. Instrument only the final
collection inside concurrent finish, retaining paired wall/owner-CPU measurements.
The following shares are of final-collection owner CPU, not total application CPU:

| Sample | Build index | Frame roots | Snapshot seed | Sweep | Remainder |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tasks | 15.8% | 2.7% | 28.5% | 26.3% | 26.7% |
| Collections | 13.1% | 3.5% | 32.8% | 27.4% | 23.2% |
| Arrays | 16.0% | 3.5% | 33.6% | 30.8% | 16.1% |
| Plaintext | 10.2% | 5.5% | 34.9% | 20.0% | 29.4% |
| JSON | 10.1% | 8.3% | 37.3% | 20.5% | 23.8% |

Frame roots covers `mark_frames`, not every root source or validation. Remainder
includes worklist tracing, other roots, resources, cleanup and instrumentation.
The four parts are disjoint; the parser rejects totals exceeding the enclosing
collection and inconsistent completion counts. Deterministic UB/bounds-sanitized
fixtures verify active/inactive gating, argument/result preservation and accounting;
negative controls reject missing/duplicate rows, malformed counts and excess totals.
Diagnostics add 2,208 bytes of owner TLS counters plus an active flag before alignment,
on top of existing finish diagnostics, and no heap allocation. They perturb execution
and are not used to claim application throughput improvements.

## Bounded optimization

`NEOCLR_LIVE_LINEAR_SEED` replaces repeated indexed lookups in the old snapshot
identity-validation pass with a monotonic walk over current heap headers. Existing
`build_index` has already validated current spans; the owner cannot mutate layout
during this pass, and the marker has finished. Snapshot headers/offsets remain private
and immutable. Every old allocated identity must still match its exact boundary,
allocated state, span, payload bytes and kind before any snapshot marks are applied.
Offsets must be strictly ordered. Old free blocks may split, merge or gain allocations.
The second seed pass, new-allocation tracing, barriers and reclamation are unchanged.

This reuses existing lifetime/ownership proofs instead of adding caches, object header
fields or allocations. It costs O(snapshot entries + traversed current blocks), which
can lose to indexed lookup when sparse old allocations follow heavily subdivided free
regions. Keep the indexed implementation and the linear path opt-in. The approach
uses the existing [.NET/Go allocation metadata research](allocation-index-results.md):
metadata belongs to the owning collector and current address/layout evidence remains
explicit. It does not claim to reproduce CLR's collector or imply .NET API divergence.

## Validation and uninstrumented comparisons

Native safety and barrier negative controls pass. An independent exhaustive-boundary
oracle passes 36,000 fragmented-layout/identity comparisons across optimized and
UB/bounds-sanitized builds; disabling kind validation fails the negative control.
Entry and record-array JIT/AOT fixtures pass 600 + 840 cycles. Record arrays use their
existing class/struct layouts; the optimization adds no record-specific GC treatment.
These fixtures exercise native seed validation, not stack-argument guard elision.
The previously recorded ASan initialization gap remains open.

The uninstrumented cohort passes 150 ordinary executions and 82,560 HTTP responses.
Five modes retain sync, guarded-sync and live controls; ordinary order rotates forward
and reverse across ten rounds. Four-client HTTP uses four paired rounds. Objects are
identical between native variants, and checked compiler command differences contain
only the linear-seed define. Summarizer negative controls reject flag drift, unbalanced
rounds and failed responses. All outputs and shutdown checks pass.

| Live workload | Paired CPU change | Paired throughput change |
| --- | ---: | ---: |
| Tasks | -0.36% | — |
| Collections | -0.50% | — |
| Arrays | -0.10% | — |
| Plaintext HTTP | -0.55% | +0.76% |
| JSON HTTP | -0.70% | +1.62% |

These are small observed effects, not a demonstrated broad performance win. The
unchanged guarded-sync path also varies: plaintext median throughput moves from
3,448 to 3,431 requests/s. Live plaintext medians move from 3,078 to 3,099 requests/s,
while sync is about 5,122 requests/s in this cohort. No claim of Kestrel parity or
concurrent-default readiness follows.

Managed allocations are unchanged in standalone runs (65,548 tasks, 237,580
collections, 86,028 arrays). HTTP retains explicit scheduling-dependent allocation
ranges rather than claiming reduction. No additional persistent native metadata is
introduced; the existing allocation index remains 16,640 bytes per 1-MiB arena.

A separate diagnostic comparison passes 18 ordinary executions and 8,320 HTTP
responses. Snapshot-seed owner CPU per finish falls about 24% for plaintext and 27%
for JSON, but rises about 7% for tasks; collections/arrays fall about 7%/6%.
Short instrumented comparisons and altered inlining limit interpretation. Seed
validation is only one part of seed processing, itself one part of application cost.

## Evidence and next work

[Compact attribution and source hashes](collection-seed-evidence.json) and
[uninstrumented summary](linear-seed-summary.json) retain the numbers. Raw reports:
`collection-parts-{ordinary,http-plain,http-json}-v2.json.gz`,
`collection-parts-summary.json.gz`, `linear-seed-{ordinary,http-plain,http-json}.json.gz`,
`linear-seed-{ordinary,http-plain,http-json}-profile.json.gz`, and
`linear-seed-profile-summary.json.gz`. Safety evidence is in
`linear-seed-{model,safety,entry,record-arrays}.json.gz` and
`collection-parts-tests.log`. Command manifests are
`collection-parts-commands.json.gz`, `linear-seed-commands.json.gz` and
`linear-seed-profile-commands.json.gz`; retained target paths/bundles are required.

Measurement tools expose `--compare-linear-seed` alongside the existing inline/index
controls. Collection attribution requires `--profile-collection --profile-finish
--profile-owner-pauses`; these runs are marked diagnostic-only. Native validation
entry points accept `--linear-seed`; the independent model uses
`concurrent-live/validate-seed-linear.py`.

Next separate remaining seed application/tracing and the explicit remainder, then
compare common sweep/root work. Do not assume old-identity validation explains the
whole seed cost. Broader heap geometry, sustained JIT/pause qualification, shared-worker
safety and default acceptance remain open. Existing Unicode/reflection/JSON helper
admission gaps in the standalone matrix remain gaps, not measured zero costs.
