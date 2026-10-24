# Owned ancestor proofs — 2026-10-24

This implements the first native execution-state step from the
[consolidated bottleneck investigation](../../../docs/design/runtime-bottleneck-investigation.md).
The prior profile and source inspection identified repeated ancestor walks at every
root publication, even when a paced collection was not due. The change is retained
on the strength of paired CPU/throughput results and focused lifetime/GC validation.
It does not implement the proposed full metadata graph or execution-state redesign.

## Invariant and behavior

`neoclr_gc_enter_owned_v1` already admits immutable compiler-owned local descriptors.
It now records an `OWNED_CHAIN` proof in an unused bit of the existing frame flags:

1. An owned frame with no ancestor has a valid chain proof.
2. An owned child inherits a proof only when its parent has a proof, uses the same
   context, and currently has no transient descriptors or an owned transient table.
3. Suspended ancestors cannot republish descriptors. Runtime-owned frame linkage and
   context remain fixed until leave; parked frames retain their lifetime obligations.
4. A frame's own transient table is checked when published and tested before that
   frame becomes an ancestor. Replacing it with a checked native table prevents a
   subsequent child from inheriting the proof. Its changing values are never cached.

On a proved chain, publication observes the one shared fault context and skips the
ancestor descriptor walk. Frame/current-head checks, plan/header checks, publication,
pacing and worker rendezvous remain. The collector still walks every relevant frame
and reads current storage/stack/transient root values. Non-owned ancestors, mutable
transients and context transitions retain the full checked publication walk.

Leave clears the proof with the frame. Park/resume preserves intact frame chains;
new roots while a chain is parked start independent proofs. Worker entry masks preserve
this runtime bit and exclude it from exclusive/shared boundary-state validation.
The bit is not a source token, image identifier or cached reachability result.

Frames remain 104 bytes. No new allocation, per-object header, global cache or
compiler-generated import is introduced. Existing generated objects use the same
hooks; only native runtime source changes. The private owned-frame contract explicitly
forbids caller mutation of runtime frame linkage/context. This is not protection
against arbitrary native memory corruption. Ordinary native callers keep their checked
contract. No public managed API, Raven emission or CLI bridge mapping changes.

This follows the existing .NET comparison's separation of immutable execution metadata,
polling and live root values. neoCLR still uses explicit shadow frames, not CoreCLR
PC/register GC maps. The benefit is amortized ancestry validation; costs are a parent
proof check at owned entry and an extra branch at publication. General mixed-context
chains do not receive this fast path. Full execution-state/TLS consolidation remains
separate work.

## Paired results

Baseline is `77de160ee` root-probe.c; candidate is the new runtime, with the same
application object, other native sources, compiler, collector policy, owned 1 MiB
heaps and client. All builds precede timed runs. Plaintext GET, checked status/body,
ten-byte response, HTTP/1.1 connection close, 32 connections, client GOMAXPROCS=2,
two-second warmup and ten-second load. Every capacity response and server exit passes.

| Configuration | Baseline req/s | Candidate req/s | Throughput | Baseline CPU µs/response | Candidate CPU µs/response | CPU reduction |
|---|---:|---:|---:|---:|---:|---:|
| AOT, one isolated owner | 2,969.5 | 3,511.6 | +18.3% | 335.1 | 282.9 | 15.6% |
| AOT, four isolated owners | 8,016.9 | 8,989.6 | +12.1% | 449.1 | 394.4 | 12.2% |
| JIT, one owned entry host | 1,846.5 | 1,989.2 | +7.7% | 538.4 | 497.2 | 7.7% |

There are eight AOT trials, configurations forward/reverse, and four separate JIT
trials in baseline/candidate/candidate/baseline order. Both candidate observations
beat both baseline observations on throughput and CPU for each configuration.
AOT CPU includes server startup/warmup/drain/shutdown and subtracts client CPU. JIT
uses `ps time` after READY through warmup/load, divided by all completed/drained
responses; startup/teardown are excluded and clock resolution is about 10 ms.
Do not pool these CPU scopes or interpret this as a new JIT/AOT architecture ranking.

One-owner AOT p99 improves in these trials. Four-owner AOT p99 is worse overall
(baseline 4.49/4.67 ms, candidate 4.67/5.11 ms); JIT tails are mixed. The change is
retained for lower CPU and higher throughput, not uniform latency improvement.
These short macOS ARM64 runs are not confidence intervals, sustained capacity,
JSON capacity or Kestrel parity. No JIT arena placement experiment is enabled.

## Separate candidate profile

A further four-owner run samples for five seconds under checked load. The native
object is recompiled byte-identically before resolving generated symbols. There
are 10,768 worker wall-clock self samples; coordinator waiting stays separate.
`stack_roots` accounts for 366 samples (3.40%), versus 1,573/11,044 (14.24%) in the
prior investigation's separate baseline capture. This shift is consistent with the
removed walk; it is not an individual-helper CPU speedup calculation.

Remaining prominent leaves include `collect_indexed` (1,643), TLS resolution (1,453),
owned frame entry (965), base validation (380), and strlen (323). The next execution
work should address state access/frame-entry metadata with an explicit lifetime
contract. Collection and managed parser/async costs remain separate. Capacity
claims above exclude this profile and its perturbed load rate.

## Safety and regression evidence

- 126 focused executions pass: 28 normal completions and 98 expected aborts, across
  optimized/sanitized native/probe-only, forced/paced and worker-hook configurations.
  The new fixture keeps changing ancestor roots alive through a 16-frame chain,
  reclaims replaced roots, preserves parked roots, and checks same/foreign contexts,
  transient replacement, checked ancestry, fault validation and worker masks.
- Optimized and UBSan shared-worker graph/fault stress each complete 32 graph and
  32 fault invocations, with 24,544 graph and 448 fault collections. Peak simultaneous
  participants are four and three respectively. This exercises actual shared-heap
  coordination; it is not a shared-heap throughput measurement.
- Five qualified async consumers run in reuse/default and optimized/sanitized forms:
  20 configurations / 40 invocations, including cancellation/disposal, root retention,
  context reuse, cleanup and deliberate terminal faults.
- Forced-GC plaintext and reflection-JSON HTTP pass in JIT and AOT: four complete
  server invocations, two checked responses each. Ordinary capacity runs exercise
  production pacing independently.
- Two aggregation tests check paired arithmetic and reject missing, failed or
  profiled cohorts. Archived-data reanalysis reproduces summary.json.

The initial consumer replay runner incorrectly assumed every fixture must exit zero;
it stopped at the first deliberate terminal-fault case. The guest produced exactly
the expected fault. The corrected runner verifies recorded exit code, stdout hash and
stderr, rerunning just the four fault configurations. Both reports and a reconciled
validation record are retained; this was a harness error, not a suppressed runtime
regression. No unrelated broad suite or website build was run. Windows runtime
execution is not claimed; the portable C fixture is added to the standard validator.

## Evidence and reproduction

`tests.json.gz`, `consumer-initial.json.gz`, `consumer-faults.json.gz`,
`consumer-validation.json`, `forced-{plain,json}.json.gz`, `capacity.json.gz`,
`jit-capacity.json.gz`, `summary.json`, `profile/` and `inputs.json` preserve raw
commands, expected failures, source/object identities and profile mapping. Reuse the
qualified bundle/objects recorded in these reports; the runners are local experiment
harnesses, not clean-checkout installers. Use fresh output directories. Do not run
builds/diagnostics concurrently with capacity measurement.

```sh
python3 benchmarks/execution/owned-root-chain/validate.py --output target/chain-new/tests
python3 benchmarks/execution/owned-root-chain/consumers.py --output target/chain-new/consumers
python3 benchmarks/execution/lookup-profile/counts.py --sample plain --forced --requests 2 --output target/chain-new/forced-plain
python3 benchmarks/execution/lookup-profile/counts.py --sample json --forced --requests 2 --output target/chain-new/forced-json
python3 benchmarks/execution/owned-root-chain/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/root-map/candidate.o --candidate-object target/root-map/candidate.o \
  --client target/primitive-comparison-refresh/client --output target/chain-new/capacity
python3 benchmarks/execution/owned-root-chain/jit.py --output target/chain-new/jit-capacity
python3 benchmarks/execution/owned-root-chain/profile.py --directory target/chain-new
python3 benchmarks/execution/owned-root-chain/summarize.py \
  target/chain-new/capacity/measurement.json target/chain-new/jit-capacity/results.json target/chain-new/summary.json
python3 -m unittest discover -s benchmarks/execution/owned-root-chain -p 'test_*.py'
```

Native commands use the recorded explicit Xcode 26.2 SDK/compiler. No compiler or
library default is changed. The earlier object-tracing prototype remains separate
and unintegrated; this change does not depend on it.
