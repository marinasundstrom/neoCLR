# Native collection pacing hypothesis

**Rejected for production.** Development experiment, 2026-10-13, macOS ARM64 AOT.
The isolated pacing policy improves these HTTP measurements but fails an allocation
admission contract. Nothing here changes the shipping collector or published scores.
`passed` in the reports means the experiment completed its expected checks;
`promotionEligible: false` records that the candidate must not be promoted.

## Hypothesis and scope

The collector-phase profiles show many full collections without new allocations.
Test whether collecting less often materially helps before choosing a different
collector. This is a pacing experiment, not generational or concurrent collection.
It complements the [.NET/Go/V8/ZGC research](../http-scheduling/README.md#generations-and-zgc-research--2026-10-13).

`policy.h` is included only in a generated copy of the root-boundary observer. It
collects on context switches, with less than 1 MiB remaining arena extent, after
16 allocations since the last observed collection, or after 64 boundaries. It
retains every root snapshot and the existing frame validation. Explicit collection
and final teardown remain unconditional; allocations themselves still cannot collect.
The counters are private per-thread experimental state, not a context ABI proposal.

The 1 MiB reserve is deliberately a hypothesis to test, not an allocation guarantee.
Small arenas continue to collect at every boundary. This design has no wall-clock
idle deadline; counting boundaries cannot establish timely cleanup while idle.

## Admission counterexample

`pressure-test.c` runs the same rooted sequence against both policies:

1. Allocate and root a 1 MiB object in a 4 MiB arena; publish a boundary.
2. Drop the root without allocating; publish another boundary.
3. Request 3.5 MiB.

Stress collection reclaims the first object at step 2 and succeeds. The paced policy
skips that collection because the apparent headroom exceeds its reserve, then faults
with NativeMemoryLimit. Explicit collection followed by retry succeeds. Sanitized
executions verify this expected difference in both reports. No production fallback
is changed to hide it. Increasing a reserve does not solve arbitrary-size allocation
admission or fragmentation in the general case.

This is why a throughput result cannot qualify the policy. A future pressure policy
needs an allocation-size-aware, fully rooted admission/retry contract. Before adding
collection inside allocation, audit native helper temporaries and borrowed arguments:
helpers currently rely on allocation not collecting, and an operation can allocate
multiple blocks. Retrying a whole operation can duplicate side effects. Keep explicit
and stress modes and test fragmentation, large requests, nested contexts, fault roots,
conditional owners and native-resource limits. Add a defined idle/quiescent cleanup
path; these measurements do not qualify one.

## Measured HTTP potential

Identical guest objects and all other native host sources are used within each pair.
Only the boundary policy is substituted. Each workload discards one warmup process
per variant, then runs four alternating rounds. Each process validates 32 responses,
discards the first eight request timings and computes throughput from the remaining
24. Builds finish before measurement. No diagnostic counters are enabled in timing
runs. These are short, sequential desktop-host measurements, not sustained capacity
or a controlled cross-platform comparison. Applications use state-machine lowering.

| Workload | Stress median requests/s (range) | Paced median requests/s (range) |
| --- | ---: | ---: |
| Plain text | 78.43 (75.80–79.04) | 176.57 (174.39–179.48) |
| JSON | 35.16 (32.40–39.80) | 84.27 (79.88–89.28) |

[Plain evidence](plain-20261013.json) and [JSON evidence](json-20261013.json) retain
all 640 validated responses (384 timed requests), exact commands and input hashes.
The observed ratios are about 2.25× and 2.40×; they describe this rejected experiment,
not neoCLR release performance. They support investing in correct collection pacing.
They do not establish the benefit of generations or imply equivalence to Go/.NET.

## Reclamation and limits

Separate diagnostic runs validate another 128 responses:

| Case | Stress collections | Paced collections | Allocations/reclaimed, each policy | Stress/paced peak arena extent |
| --- | ---: | ---: | ---: | ---: |
| Plain text | 177,098 | 2,820 | 16,340 / 16,340 | 13,216 / 14,248 bytes |
| JSON | 271,945 | 4,339 | 38,358 / 38,358 | 19,672 / 20,136 bytes |

[Plain validation](plain-validation-20261013.json) and
[JSON validation](json-validation-20261013.json) have zero collector failures and
complete final reclamation. Peak arena extent rises slightly; peak live payload is
sampled only at collections and is not directly comparable between frequencies.
The retained-body and idle-drain sanitizer tests also pass with the substituted
observer. Their small arenas use the conservative stress fallback; this verifies
that fallback and explicit-collection contracts, not general paced abandonment or
idle cleanup. Those remain qualification gaps alongside admission.

## Reproduce

Use passing plain/JSON native build records and fresh output directories:

```sh
python3 benchmarks/http-gc-pacing/evaluate.py --build PLAIN_BUILD/build.json --workload plain --output NEW_PLAIN_DIRECTORY
python3 benchmarks/http-gc-pacing/evaluate.py --build JSON_BUILD/build.json --workload json --output NEW_JSON_DIRECTORY
python3 benchmarks/http-gc-pacing/validate-policy.py --evaluation EVALUATION_JSON --ownership-recipe OWNERSHIP_VALIDATION_JSON --output NEW_VALIDATION_DIRECTORY
```

The ownership recipe is a passing `validate_conditional_roots.py --owned-bodies`
report including the idle-drain test. Evaluation emits an isolated observer copy;
no SDK, public API, production source or website publication changes.

## Rooted admission foundation — 2026-10-13

The private `neoclr_gc_allocate_rooted_v1` entry point now admits an allocation,
collects once on capacity pressure, and retries before reporting a language fault.
It requires a complete current root boundary (or a quiescent host), including all
native helper temporaries. Boundary identity checks do not prove completeness.
The existing arena-only allocator remains non-collecting; no helper or compiler
path opts in yet, and the rejected pacing policy remains disabled.

`native-gc-admission-test.c` covers preserved live roots, stale/wrong boundaries,
impossible requests, genuine exhaustion and the retained 4 MiB pressure example.
The conditional-roots validator passes all eight focused checks with undefined
behavior/bounds sanitizers, including owned-body and idle-drain checks
(`target/http-gc-rooted-admission-20261013/validation.json`). This is an admission
primitive, not a throughput improvement or completed helper-lifetime audit.
