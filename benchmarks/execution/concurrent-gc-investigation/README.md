# Concurrent GC development baseline — 2026-10-24

**Author-directed priority:** develop/evaluate concurrent GC toward the default;
retain synchronous GC as a correctness/performance comparison. Reuse existing
heap lookup, metadata, root validation and execution-state improvements. This
report establishes measurements and design requirements, not a working concurrent
collector or a measured concurrent speedup.

Plain text is the maximum-throughput baseline against matched language/framework
samples, not a 10,000 requests/s target. The author's 10,000 example concerned basic
JSON and was illustrative, not a fixed acceptance threshold. Evaluation must also
cover work inside requests and standalone programs of varying complexity.

## First measured opportunity

Current synchronous native collector, matching generated state-handle hosts, owned
1 MiB heap, three independent 1,024-request processes per handler and execution mode.
All response bodies/completion counts pass: 12,288 responses in the ordinary profile
cohort and another 12,288 in the phase cohort. No concurrent builds or other profiling
ran during these sequential diagnostic cohorts. These are not capacity tests: the
client drives requests sequentially, and counters include startup and shutdown.

| Handler | AOT collector wall µs/request, range | JIT collector wall µs/request, range |
|---|---:|---:|
| Plain text | 18.88–20.48 | 17.28–17.58 |
| JSON | 40.15–42.71 | 39.10–39.51 |

A diagnostic-only collector copy splits successful cycles into index construction,
root validation/seeding, transitive tracing (including conditional fixed points),
cleanup/expiry and sweeping/coalescing. Mean phase wall µs per request:

| Handler/mode | Index | Roots | Trace | Cleanup | Sweep |
|---|---:|---:|---:|---:|---:|
| Plain/AOT | 2.81 | 0.24 | 13.59 | 0.05 | 3.25 |
| Plain/JIT | 2.53 | 0.24 | 12.50 | 0.07 | 3.12 |
| JSON/AOT | 6.19 | 2.54 | 28.90 | 0.12 | 7.02 |
| JSON/JIT | 6.01 | 2.55 | 29.01 | 0.11 | 6.90 |

Tracing contributes 64.6–68.1% of summed instrumented phase time; index/sweep
contribute most of the remainder. This supports concurrent tracing as the first
candidate, while preserving fast index/sweep work. It does **not** imply 65% of
application CPU is GC, a 65% throughput improvement, or that all tracing can overlap.
Do not divide these wall timers by CPU measurements from another cohort to derive
an Amdahl bound. Added clocks/code placement perturb timing, notably JSON; compare
phase shares and repeated ranges, not instrumented versus ordinary timing as a win.

Existing logarithmic pause buckets are captured, including startup/shutdown. p99
bucket upper bounds are 32.768 or 65.536 µs in these runs; observed individual maxima
range 29–111 µs across all configurations. These are collector timings, not HTTP
latency percentiles or full safepoint rendezvous delays. Phase cycle totals and pause
bucket counts agree with successful collector attempts. Four aggregation tests reject
failed/incomplete/mismatched inputs. Richer, loaded multi-owner pause/rendezvous and
thread-CPU measurements are still required.

## Design comparison and implementation requirements

.NET background GC overlaps generation-2 work with application execution; young
generations still use foreground collections. Concurrent and generational policies
are complementary, not synonyms. [Official .NET background GC](https://learn.microsoft.com/en-us/dotnet/standard/garbage-collection/background-gc).
Go's GC guide explicitly treats CPU and memory as a tradeoff. Its runtime implements
pointer-write barriers, including bulk operations; moving marking to a thread is not
sufficient to maintain reachability. [GC guide](https://go.dev/doc/gc-guide),
[barrier implementation](https://go.dev/src/runtime/mbarrier.go).

neoCLR currently relies on quiescent mutation: native object scanning reads payload
words with memcpy; pending work links occupy descriptor state bits; allocation/index
and cleanup/sweep share owner-local TLS. Concurrent reads/writes cannot simply reuse
those assumptions. Reusable algorithms need a per-cycle representation with explicit
ownership and memory ordering.

The primary path is concurrent nonmoving tracing/reclamation, initially qualifying
owned heaps before shared heaps. Nonmoving addresses preserve existing borrow/base
lookup contracts; moving/compacting collection is separate work. Keep synchronous
collection selectable for differential testing and bounded fallback, not as the main
optimization track. Required components:

1. An owned cycle context and stable allocation/index view, separate from mutator TLS.
   Do not hand a worker the cached generated state handle. Define cycle epochs,
   startup/cancellation/shutdown and behavior under allocation pressure.
2. A short root handshake captures values from active/parked stacks, faults, statics,
   native handles and conditional-owner groups. Immutable descriptor validation can
   be reused, but live slot values cannot be read later after a mutator resumes.
3. A mutation protocol (SATB/incremental-update or an explicitly measured snapshot
   alternative), covering generated field/array/byref stores, reference-bearing bulk
   copies, native helpers and conditional/weak-handle exposure. New allocations and
   barrier-buffer overflow need defined marking rules. Audit every path.
4. A read-safe payload protocol. Aligned loads alone are not a portable race argument:
   the current conservative scan also reads scalar words while writers may mutate
   them. Consider precise reference layouts with appropriate atomic access, or an
   immutable snapshot prototype. Snapshot copying trades pause time and memory for
   simpler read safety; it is an alternative to measure, not an adopted default.
5. Completion/remark verifies all roots and mutation queues are accounted for before
   reclaiming. Cache proofs must be revoked before reuse; free lists, conditional
   roots, parked-stack expiry, native resources and cleanup affinity must remain sound.
6. Bounded background work/CPU scheduling, backpressure and synchronous fallback for
   exhaustion. More GC threads do not inherently reduce CPU/request. Shared-heap
   participant coordination needs separate admission and mutation qualification.

Start with a correct concurrent mark candidate, including mutation-during-mark,
root handoff, new allocations, conditional cycles, faults, cancellation and shutdown
stress. A shadow/differential mode may compare decisions against the synchronous
collector before enabling reclamation. Such a shadow mode is not evidence of a
production concurrent speedup. Adopt the concurrent path as default after correctness
and measured workload results support it; do not merely rename the blocking worker
collector or suppress collection to simulate concurrency.

## Required workload matrix

| Workload | Existing evidence/entry point | Evaluation needs |
|---|---|---|
| Plain-text HTTP | state-integration objects and primitive-comparison peer harness | Matched peers, sustained load, varying owners/concurrency; headline throughput and CPU/request |
| JSON HTTP | checked reflection-JSON object and long cohorts here | Serialize/parse work, allocations, live heap, CPU and tails; same payload/response semantics |
| More complex HTTP | CPU/JSON worker recipes in bottleneck-review | Rebuild/admit current inputs; vary computation, allocation and async/I/O independently |
| Standalone collections/tasks | transient-handoff corpus and gc-mode-comparison | Batch enough work to separate steady execution from startup; output equivalence |
| Standalone JSON/strings/arrays/object graphs | runtime/compiler consumers and JSON corpus | Vary mutation, survival, graph depth, bulk writes and retained heap sizes |
| Standalone async/faults | rebuilt state-integration async and shared-worker consumers | Suspension, handoff, expiry, cancellation and cleanup correctness plus steady CPU/memory |
| Metadata/reflection/startup | bottleneck-review startup/lookup recipes | Keep load/selection/compilation separate from execution and GC |

Use JIT/AOT for the common native collector comparison; assess interpreter services
separately, since its heap is different. The older ordinary CPU/JSON interpreter
package failed current admission in the existing investigation; repair/rebuild it
coherently instead of weakening validation or reusing its timing as current evidence.
Existing tiny task/collection timings are dominated by startup and are not suitable
steady-state GC results. The matrix is **required work**, not a claim all these
performance cohorts were refreshed here. Rebuilt non-HTTP async/worker correctness
checks already pass in the state-handle integration; ordinary performance remains
to be batched and qualified alongside the concurrent candidate.

For every meaningful comparison report output correctness, wall execution time,
process and collector CPU, allocations/bytes, live/peak/reserved memory, collection
frequency and pause/rendezvous distributions. HTTP adds throughput, latency tails,
errors and scaling. Keep warm/cold runs, profiling/capacity, owner counts and client
CPU separate. Evaluate requests per core as well as requests/s. No single HTTP or
microbenchmark result can establish overall runtime improvement.

## Reproduction and scope

`measure.py` runs the normal timer cohort; `phases.py` generates an out-of-tree
instrumented collector; `measure-phases.py --gc-source <generated-file>` captures
phases and pause buckets in stderr. Both default to three 1,024-request runs per
mode and take `--sample plain|json` plus a fresh `--output`. Required application,
compiler and bundle artifacts are identified in reports; this is not a clean-checkout
installer. `inputs.json` explicitly hashes the actual phase collector (the reused
runner's input list also includes the production source). `phases.c.gz` preserves it.
The five phase counters are successful-cycle instrumentation; analysis rejects any
failed cycles. They are not production diagnostics APIs.

`summarize.py plain.json.gz json.json.gz phases-plain.json.gz phases-json.json.gz
--output summary.json` reproduces the tables. `test_summary.py` validates aggregation.
No production collector code changes for these measurements; synchronous collection
remains the current implementation. Concurrent default is the author-directed goal,
not a shipped capability or a measured outcome yet.
