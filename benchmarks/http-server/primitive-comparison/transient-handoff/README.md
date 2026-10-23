# Native GC transient handoff reuse — 2026-10-24

The native backend omits empty transient-root publications and identical
post-success republications, including their per-site JSON records. This removes
runtime calls, descriptor stores and repeated metadata validation without removing
any pre-operation collection or worker boundary. Diagnostic probes retain all
phase events and full inspection plans; the native runtime ABI is unchanged.

## Lifetime argument

Before each call/construction, the existing pre-operation publication clears the
caller's old transient table. For a result with traceable lanes, lowering zeros the
scratch result lanes and publishes their addresses before entering the callee
(phase 3). Constructor receivers are initialized and published before constructor
entry (phase 1). The callee uses a different frame and writes into the caller's
existing output storage. Returning changes values, not descriptor addresses, so
rebuilding the table after successful return (phase 2) adds no roots or lifetime.
The next boundary snapshots the evaluation stack and clears that transient table;
leaving/unwinding clears the frame. Interior pointers, reference-containing values
and erased discriminator lanes remain included by the existing tracing recipes.
A type with no seed lanes needs no transient table at either point.

The implementation retains an emission entry for skipped events so existing lowering
sites and stack-table reservation stay consistent. It emits no metadata/callback for
those entries. This is not general leaf-call effect analysis, immutable-transient
ownership or pointer-based validation caching. Published descriptor validation and
fault, static-initialization, async suspension and worker coordination remain active.
Native frame bookkeeping can retain phase 1/3 until the next boundary; phase-2 event
observation remains supported in diagnostic-probe mode, not native GC execution.

Compared with the [.NET execution-metadata guidance](../../../../docs/design/allocation-metadata-runtime-comparison.md),
this removes duplicate work over known storage lifetimes. It does not adopt CLR
stack maps or claim precise liveness. Shared binary image metadata, registration
and unload contracts remain future work. Raven CLI encoding and Runtime Contract
configuration are unchanged; paired bridge docs record ownership and limitations.

## Work and image size

The same HTTP fixture's generated object shrinks from 9,569,264 to 8,650,880 bytes
(9.6%). Transient metadata records fall from 5,934 to 1,554; all 3,982 pre-operation
boundary maps and 1,133 storage maps remain. The new emitted-object regression checks
that reference results retain their pending handoff while scalar results and duplicate
post-success events disappear only in native GC mode.

A separate atomic-counter diagnostic executes eight forced-GC plaintext responses
across two invocations. Transient publications fall from 63,124 to 10,132 (83.9%),
while both objects execute exactly 38,352 boundary publications. Counts include host
startup/shutdown and are not per-request steady-state estimates. The diagnostic uses
identical instrumented runtime source in both cases, native sanitizers and checked
responses; its execution time is not used as capacity evidence.

## Validation

Twenty focused native/codegen contract tests pass across the selected suites,
including the new emitted-map test and unchanged diagnostic probe tests. Initial
harness/setup failures and their repairs are recorded in `tests.json.gz`; a zero-test
filter invocation is explicitly excluded. The native runtime sources are unchanged.

- Forced-GC sanitized HTTP completes eight checked responses across two invocations.
- Recompiled optimized/UBSan shared-worker graph and fault cases each complete 32
  invocations; graph runs reach four participants and 24,544 collections, fault runs
  448 collections. Every worker boundary remains enabled.
- Five async consumers pass 40 invocations across context-reuse/default-fallback and
  optimized/sanitized builds, including nested awaits, retained results, faults,
  cancellation/disposal and cache overflow.
- The [cross-workload report](workloads.md) adds paced plain/JSON HTTP counts and
  ordinary task/collection samples, with the JSON DOM/stream admission gap explicit.

The native array fixture initially failed at its stale hard-coded 129-lane rejection:
the established runtime limit is 256. Fix the assertion to reject
`NEOCLR_NATIVE_LAYOUT_LANES + 1`, then rerun that test. No array/runtime semantics
change. The original failure and repair are retained with the test evidence.

## Capacity

| Trial | Requests/s | CPU µs/response | p99 ms |
|---|---:|---:|---:|
| baseline-w1 | 2,239.2 | 446.3 | 19.21 |
| baseline-w4 | 6,566.0 | 552.2 | 7.69 |
| candidate-w1 | 2,447.1 | 405.5 | 19.74 |
| candidate-w4 | 6,941.3 | 509.6 | 7.56 |
| candidate-w4 | 7,304.8 | 498.2 | 5.93 |
| candidate-w1 | 2,458.2 | 402.0 | 22.49 |
| baseline-w4 | 6,552.0 | 553.2 | 6.42 |
| baseline-w1 | 2,188.6 | 453.4 | 23.56 |

1 owner(s): mean throughput 2,213.9 → 2,452.6 requests/s (**+10.8%**);
CPU/response 449.9 → 403.8 µs (**−10.2%**).

4 owner(s): mean throughput 6,559.0 → 7,123.1 requests/s (**+8.6%**);
CPU/response 552.7 → 503.9 µs (**−8.8%**).

Both candidate trials outperform both respective baseline trials in throughput and
CPU cost. Tail latency is mixed; no uniform p99 improvement is claimed.

The same native runtime, application sources, heap ownership, context policy and GC
budgets apply. Only the generated object differs. Each uninstrumented trial uses two
seconds warmup, ten measured, 32 connections and checked ten-byte responses through
EOF; one/four owners run forward/reverse. CPU per response includes startup/shutdown
and warmup/window/drain responses, subtracting client CPU. No tests, compilation or
diagnostic runs overlap timed load. Short desktop trials do not establish sustained
capacity or Kestrel parity. Shared async HTTP remains separately unqualified.

## Cross-workload investigation

The author additionally requests a coherent profile across ordinary samples and
HTTP configurations of different complexity. See [workloads.md](workloads.md) for
normalized counts, configuration controls, preparation failures and coverage gaps.
No claim of JSON throughput improvement follows from the plain-text capacity trial.

## Reproduce

Use the recorded Xcode SDK environment, compiler/bundle inputs and fresh output paths.
`build.json.gz` records the current-backend compilation command; the baseline object
is the byte-verified object from the preceding free-index comparison. Retained reports
record commands/hashes; decompress `.gz` files to inspect them.

```sh
cargo test -p neoclr-codegen --test root_probes --test native_gc --test gc_selection
python3 benchmarks/http-server/primitive-comparison/transient-handoff/count.py \
  --build target/primitive-comparison-build/build.json \
  --object target/transient-handoff-build/candidate.o --output target/handoff-count
python3 benchmarks/http-server/primitive-comparison/transient-handoff/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --candidate-object target/transient-handoff-build/candidate.o \
  --client target/primitive-comparison-refresh/client --output target/handoff-capacity
```

Forced-GC HTTP, recompiled shared-worker checks and async consumers reuse the existing
`context-reuse/forced.py`, `free-index/validate.py` and `context-reuse/qualify.py` runners.
The archived reports identify exact commands and inputs. No public API or website
build is needed; the development benchmark description is updated separately.

Paired Raven documentation: `767383f58` on `codex/neoclr`; Raven main and the
Raven compiler binary are unchanged. Qualification here is local macOS ARM64,
not a Windows execution claim.
