# Compiled worker read-phase live epochs

2026-10-25. Private `NEOCLR_WORKER_READ_EPOCHS` connects the experimental live
marker to actual compiled worker execution. It requires paced collection-owner
mode and copied participant roots. This is an opt-in AOT integration checkpoint,
not general concurrent mutator or HTTP admission, and does not change defaults.

## Ownership and admission

When an exclusive boundary exhausts a pacing budget, it records an epoch request.
Before that participant next releases its exclusive lease into a reviewed read
phase, the pool copies initial roots and starts marking on the persistent service
context. Readers can then execute while the marker traces the unchanged heap.
Before granting any next exclusive lease, the pool rendezvous pauses all readers,
copies current roots, waits/remarks/reclaims on the same service, and only then
resumes the exclusive operation. No allocations or heap mutations are admitted
between begin and finish. Stack/local root values may change and are recaptured.

All source frames must carry the read-phase capability from the new native map
import. Legacy worker images are rejected in this mode; their synchronous path is
unchanged. Invocation attachment and return pass through exclusive boundaries.
Host heap access and native read/mutation adapters wait for compiled participants
to leave and additionally reject an active epoch. Allocation dispatch rejects an
active epoch. Close drains compiled invocations before stopping the service, which
joins its persistent marker before pool teardown. Service state and diagnostic
counters remain under the pool gate. Epochs may end before their initiating reader
resumes if a waiting writer wins the lease; there is no minimum overlap guarantee.

The external arena remains checked storage, without acquiring owned-heap cache or
index authority. Detached marker metadata survives until finish; source root
buffers survive each service request only. This adds no Raven semantics, CLI
metadata, Runtime Contract configuration or compiler emission change. It consumes
the existing native read-phase capability. Full provider admission, shared native
resources, async suspension and JIT worker execution need separate qualification.

The existing [.NET/service comparison](allocation-owner.md) applies. The benefit
of this restricted protocol is a clear payload lifetime without generated write
barriers. The cost is that every exclusive transition ends the overlap opportunity,
and allocations still cross the service. It is a qualification step, not a claim
that this scheduling policy matches CLR background GC or is an efficient final
architecture. Wider mutation overlap needs registered barriers and tested provider
boundaries before admission.

## Evidence

`read-epochs-validation.json.gz` retains optimized and UB-sanitized compiled
runs. Each graph run completes 32 invocations, 20 live epochs and 3,904 dispatched
allocations. The marker reads 7,600/7,936 payload bytes, with 25/23 slow-path reader
admissions respectively. Valid completed snapshots match finished epochs, with no
invalid snapshots. Fault isolation completes 32 invocations and 128 allocations;
this small workload triggers no epochs. Closing with four running invocations
drains 488 allocation requests and supports repeated close. Missing roots,
missing allocation dispatch and an actual retained legacy object are rejected.

`read-epochs-pressure.json.gz` passes ordinary and UB/bounds-sanitized native
adapter runs: 32 explicit and 32 rooted-pressure requests, 96 allocation attempts
and cleanup. These exercise the synchronous fallback of the live-linked runtime;
they do not claim pressure allocation during an active marker epoch.

`read-epochs-sync.json.gz` retains the paced synchronous control. The final paired
measurement uses byte-identical managed objects, one warmup and five alternating
measured runs per variant, including graph/fault/shutdown checks in every process:

| Mode | Median elapsed | Median total child CPU |
| --- | ---: | ---: |
| Paced synchronous service | 107.36 ms | 237.67 ms |
| Read-phase live service | 111.43 ms | 241.54 ms |

Live is 3.8% slower elapsed and uses 1.6% more CPU here. An earlier cohort without
payload diagnostics was 3.1% slower / 6.3% more CPU. These short multithreaded
processes are diagnostic, not HTTP throughput or tail-latency evidence. Managed
allocation request counts match; native snapshot/root/marker allocations, memory
retention and pauses have not been quantified by this driver. No allocation
reduction or performance win is claimed. Reader admissions establish lifecycle
overlap, not simultaneous CPU execution or how much useful marking overlaps it.

A repeated run exposed a harness mistake requiring at least one reader admission
per epoch. A waiting writer can legitimately finish an epoch before that reader
resumes. The retained `read-epochs-reader-assertion.json.gz` shows this failure;
the corrected harness requires completed epochs, some reader admission and real
payload tracing across the graph run. `read-epochs-header-error.json.gz` preserves
an initial diagnostic-signature build mismatch, fixed before qualification.
Earlier TSan startup failures remain unresolved; these tests do not prove all
thread interleavings.

## Reproduction and next work

```sh
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --read-epochs --legacy-object target/compiled-collection-allocation-control/code.o \
  --output target/read-epochs-new
python3 benchmarks/execution/concurrent-participant-roots/validate-pressure.py \
  --read-epochs --output target/read-pressure-new
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --collection-owner --paced --output target/read-sync-new
python3 benchmarks/execution/concurrent-participant-roots/measure-owner.py \
  --control target/read-sync-new --owner target/read-epochs-new \
  --output target/read-cost-new.json
```

Use fresh output paths; manifests retain toolchain and input hashes. The legacy
object is an explicit retained fixture, not rebuilt by the validator. Without it,
the optional legacy-image rejection check is omitted.

Next measure epochs ended before useful reader progress, root copying/snapshot
allocation cost and service wakeups. Avoid assuming more marker threads help.
Before broad HTTP comparison, connect only provider paths whose mutation and
resource boundaries are qualified. Then use plaintext as the baseline, JSON,
tasks/collections/arrays as contrasting workloads, and synchronous controls with
matched pacing. Report requests/work per second, total CPU per unit, allocations
and memory separately; add pause distributions rather than substituting lower
GC wall time for application progress. Default promotion remains open.
