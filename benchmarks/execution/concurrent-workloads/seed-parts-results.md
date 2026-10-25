# Snapshot seed phase attribution

2026-10-25 follow-up to [collection attribution](collection-seed-results.md).
Nine standalone executions and 4,160 checked HTTP responses pass on the retained
owned-heap macOS ARM64 host. This is diagnostic evidence, not a throughput trial.
Tasks, collections and arrays each run three times; plaintext and JSON each run
two pairs of 1,024 requests plus 16 warmups, with four concurrent clients. The
indexed live control is used throughout; the optional linear identity pass is off.

| Workload | Identity validation | Mark application / new-object queuing | Timing remainder |
| --- | ---: | ---: | ---: |
| Tasks | 34.15% | 54.92% | 10.92% |
| Collections | 34.49% | 58.22% | 7.28% |
| Arrays | 35.33% | 55.53% | 9.15% |
| Plaintext | 46.40% | 50.30% | 3.30% |
| JSON | 44.77% | 52.30% | 2.93% |

Shares are pooled owner CPU within snapshot seed, not application CPU or entire
collection CPU. Application includes the monotonic current/snapshot offset merge,
applying old marks and queuing new objects; tracing their closure occurs later.
The remainder includes timing/report bookkeeping and wrapper boundaries. Four
additional paired wall/CPU clock samples per successful active seed call perturb small measurements;
no clocks are added per object. Two extra owner TLS rows cost 1,104 bytes before
alignment, only in this diagnostic build. No production allocation or GC policy changes.

`SEED_PART` rows are nested inside `COLLECTION_PART 2`. Never add them to the four
disjoint collection parts. The parser checks both nested counts against the parent,
histogram bounds and total CPU/wall sums, and keeps a nonnegative remainder. Legacy
reports without seed rows remain readable. Both string and line-list stderr formats
are supported (ordinary and HTTP harnesses use different representations).

Deterministic UB/bounds-sanitized tests cover inactive/active execution, exact timing,
argument/results and preservation of work. Negative controls reject absent/duplicate
rows, wrong counts and sums exceeding the enclosing seed. Only successful complete
collections qualify; a failed identity check does not produce a complete pair.

The larger remaining component is application, not just old-identity validation.
Next inspect reuse of already validated block pointers when queuing new objects:
the current pass calls address-based `mark` after obtaining the exact block from a
validated heap walk. Preserve atomic-object handling, pending-list state and existing
root marks if removing that redundant lookup. This is a candidate, not an implemented
optimization. Shared-worker/default qualification and the sync performance gap remain.

Reproduction commands and status are retained in `seed-parts-commands.json.gz`;
raw results in `seed-parts-{ordinary,http-plain,http-json}.json.gz`; validated nested
rows/remainders in `seed-parts-summary.json.gz`; tests in `seed-parts-tests.log`.
The existing retained objects/bundle named in the command manifest are required.
The current diagnostic source is `collection-parts.py`; `--profile-collection`
enables both levels. No Raven/compiler or managed API contract changes.
