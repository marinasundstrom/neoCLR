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
The existing arena-only allocator remains non-collecting. StackTrace capture and
GC-generated concat/join now opt in through audited paths; the rejected general
pacing policy remains disabled.

`native-gc-admission-test.c` covers preserved live roots, stale/wrong boundaries,
impossible requests, genuine exhaustion and the retained 4 MiB pressure example.
The conditional-roots validator passes all eight focused checks with undefined
behavior/bounds sanitizers, including owned-body and idle-drain checks
(`target/http-gc-rooted-admission-20261013/validation.json`). This is an admission
primitive, not a throughput improvement or completed helper-lifetime audit.

## Rooted closed-workload pacing follow-up — 2026-10-13

`evaluate-rooted.py --output NEW_DIRECTORY` evaluates a separate native workload
where **every allocation uses rooted admission**, including the new concat helper.
It does not change generated-code boundary policy, HTTP hosts or installed SDKs.
Its report deliberately retains `promotionEligible: false`: unaudited helpers and
real scheduler idle integration still prevent general promotion.

The candidate collects after 16 allocations or 64 progressing root boundaries;
it always collects on an explicit idle/shutdown call. It does not guess a free-space
reserve. An allocation that cannot fit collects with published roots and retries
once, preserving large-request admission. Unexpected collection epochs force
collection rather than resetting the cleanup budget indefinitely across contexts.
The threshold is an experiment, not a managed API contract or wall-clock guarantee.

Contracts cover the former 4 MiB pressure counterexample, true live-heap exhaustion,
root-only reachability changes, exactly-once resource cleanup, explicit idle
cleanup and owned suspended-stack retention/abandonment. They supplement helper
pressure tests that preserve borrowed strings and array elements during retry.

The workload keeps 64 live objects or strings while performing 20,000 allocations
in a 64 KiB arena. One binary executes both policies; five alternating measured
pairs follow warmups. Timings include allocation and boundary collection, excluding
final idle cleanup. Separate profiled runs verify bounded arena extent, counts
and complete reclamation. This is neither a managed JIT/AOT application comparison
nor an HTTP throughput result. Keep the original rejected experiment above intact.

The existing [.NET/Go/V8 research](../http-scheduling/README.md) motivates measuring
allocation pressure and cleanup separately. This experiment remains a nonmoving
full collector; it implements no generations, barriers or concurrent marking.
Acceptance for a future managed policy additionally requires complete allocating
helper coverage, explicit interpreter/JIT/AOT policy behavior, real scheduler idle
cleanup and sustained/concurrent workload qualification.

The [isolated local run](rooted-macos-20261013.json), taken after other local
compilation/validation completed, improves the object workload 3.98× and the string
workload 3.57×. Each worker reclaims all 20,000 allocations. Separate diagnostic
runs reduce collections from 20,001 to 1,251, with peak arena extent rising from
4,160 to 5,120 bytes for objects and 3,120 to 3,840 for strings. The roughly 23%
space increase is a real tradeoff. Results cover this explicit rooted native
harness only; there is no claim that general helper coverage is complete.

The [Windows x64 run](rooted-windows-38422680168.json) at `4ba0a60d` improves
objects 4.25× and strings 3.98×, with the same allocation/reclamation counts,
collection counts and peak-extent tradeoff. Its focused native GC/root and socket
contracts pass. These are within-machine comparisons, not a ranking of operating
systems. The Windows job in [run 38422680168](https://github.com/marinasundstrom/neoCLR/actions/runs/38422680168)
passed. The subsequently completed [hosted macOS ARM64 run](rooted-macos-latest-38422680168.json)
also passes, improving objects 3.91× and strings 3.83×. Both jobs and their socket
contracts pass; the hosted macOS job does not run the separate GC validator, whose
20 local macOS executions are retained in the allocation evidence.

## Integrated native policy qualification — 2026-10-13

`evaluate-rooted.py --integrated --output NEW_DIRECTORY` now exercises the
runtime-owned policy used by `NEOCLR_GC_PACED` builds, preserving the historical
experiment when the option is omitted. Actual scheduler acquisition forces cleanup
before returning Idle/Pending; root-observer and scheduler tests run both variants.
Stress collection remains the installed-host default. The closed workload evidence
does not establish managed HTTP or whole-runtime throughput or justify promotion
by itself. Source hashes and separate diagnostic results accompany timing reports.

The integrated policy passes admission and ownership contracts in
[run 38455945430](https://github.com/marinasundstrom/neoCLR/actions/runs/38455945430)
at `f547c875b`. Five alternating measured pairs follow warmup for each workload;
diagnostic runs are separate. These are within-host comparisons, not an OS ranking.

| Environment | Object workload speedup | String workload speedup | Evidence |
| --- | ---: | ---: | --- |
| Local macOS ARM64 | 3.36× | 3.43× | [Report](integrated-local-20261013.json) |
| Hosted macOS ARM64 | 4.14× | 3.61× | [Report](integrated-macos-latest-38455945430.json) |
| Hosted Windows x64 | 4.10× | 3.81× | [Report](integrated-windows-2022-38455945430.json) |

Every worker reclaims all 20,000 allocations. Collections fall from 20,001 to
1,251. Peak arena extent grows from 4,160 to 5,120 bytes for objects and 3,120 to
3,840 bytes for strings (about 23%). Timings exclude final idle cleanup; these are
not scheduler-poll latency or sustained/concurrent application measurements. See
[qualification and limitations](../../docs/validation/runtime-product/native-scheduler-pacing/README.md).

## Managed HTTP qualification

`qualify-managed.py --bundle BUNDLE --aot BACKEND --runtime RUNTIME --output NEW_DIRECTORY`
freshly compiles the plain-text and JSON Raven servers, then links identical guest
objects into stress and paced AOT hosts. JSON uses explicit discovered reflection
roots, matching the existing benchmark contract. Both variants process the same
32 requests per process with independent framing/body validation. A warmup pair
precedes at least three alternating measured pairs; the first eight request times
in each process are excluded. Separate instrumented processes check allocation
counts, reclamation and peak arena extent after host teardown. Build commands and
input hashes are retained. The Windows HTTP project workflow offers `paced_gc` to
run the same qualification after preparing a matching toolchain.

This measures short sequential, connection-close HTTP, not sustained saturation,
concurrency, HTTPS or generic JIT performance. Installed defaults are unchanged.
The .NET comparison remains allocation-pressure budgeting at reported root
boundaries; this does not add generations or concurrent collection. The expected
tradeoff is fewer full sweeps versus more retained bytes and synchronous pauses.
