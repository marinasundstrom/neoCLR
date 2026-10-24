# Heap-gate granularity and separate timing scopes — 2026-10-24

The author supplied an external assessment asking whether the gate protects objects
or heap regions and emphasizing CPU versus application progress. The prototype has
**one mutex per owned heap**, shared by its single admitted mutator and marker. It
is not one mutex per object and does not admit arbitrary shared-heap mutators.
Unbatched tracing acquires it for each payload read, at most 256 bytes, while lookup
and tracing of the private scratch happen outside the lock.

## Bounded batch experiment: not selected

`NEOCLR_LIVE_BATCH` retains the same mutex while tracing a bounded amount of work,
including detached-index lookup. It releases after at most 32 objects, 4,096 copied
bytes or 64 payload reads. Large objects and initialized-array markers therefore
cannot create an unbounded single hold through their size alone. These are work
bounds, not time guarantees: scheduler preemption and mutex fairness still matter.
Barrier words are drained between batches, and cancellation is checked at reentry.
There are no marker callbacks or allocations inside the critical section.

Both unbatched and batched optimized/UBSan-bounds native suites pass, including the
negative deletion-barrier control. A 20 KB object and 1,024-element reserved array
exercise the byte/read limits; recorded maxima reach exactly 4,096 bytes and 64 reads.
This extends existing gate/marker evidence, not exhaustive interleaving qualification.
The race-detector platform gap remains.

Fresh non-instrumented native graph pairs use sync, unbatched, batched, batched,
unbatched, sync order, identical graph/output checks and 10,000 cycles each.

| Mode | Process CPU (seconds) | Marker thread CPU (seconds) | Wall (seconds) | Peak arena bytes |
| --- | ---: | ---: | ---: | ---: |
| Synchronous | 0.855–0.861 | — | 0.855–0.861 | 233,016 |
| Unbatched live | 2.271–2.274 | 0.748–0.749 | 1.836–1.837 | 236,600 |
| Batched live | 2.726–2.764 | 0.875–0.899 | 2.231–2.258 | 236,600 |

Batching uses **20.8% more process CPU** than the unbatched candidate in this cohort.
It performs 1,289,999 payload-gate acquisitions per trial, with 512-byte/32-read
maximum holds for this small-object graph. The former path reads one 16-byte payload
per graph object; the smaller lock count does not produce a speedup. All variants
perform the same checked graph workload; each concurrent trial drains 1,280,000
barrier words and completes 10,000 valid epochs without cancellation. These local
pairs are not confidence intervals. Batching remains an opt-in experiment.

## Separate instrumented observations

`NEOCLR_LIVE_GATE_PROFILE` adds clocks and is measured separately from the cost
cohort above. Instrumentation changes scheduling and adds overhead; do not combine
its absolute timings with the uninstrumented process-CPU results.

| Instrumented scope | Unbatched live | Batched live |
| --- | ---: | ---: |
| Marker thread CPU | 0.872–0.884 s | 1.051–1.053 s |
| Owner begin/finish thread CPU | 1.132–1.139 s | 1.128–1.129 s |
| Guarded-write thread CPU | 0.222–0.226 s | 0.663–0.672 s |
| Total guarded-write wall time (640,000 calls) | 0.332–0.336 s | 1.184–1.187 s |
| Maximum guarded-write wall time | 34 µs | 119–243 µs |
| Total begin wall time | 0.601–0.606 s | 0.607–0.608 s |
| Maximum begin wall time | 87–89 µs | 91–215 µs |
| Total finish wall time | 0.748–0.765 s | 0.525 s |
| Maximum finish wall time | 120–167 µs | 61–134 µs |

Batched observed maximum gate holds are 7–31 µs; total holds are about 0.604 s per
trial. Shorter finish waits coexist with longer guarded writes and more process CPU.
This supports rejecting this larger critical section; it does not isolate every
source of cost or prove that all alternative batch sizes are inferior.

The synchronous instrumented collection totals are 0.573–0.578 s wall, with observed
maxima of 525–824 µs and 0.570–0.574 s owner CPU. Concurrent maxima are smaller in
this fixture, but concurrent owner blocking occurs at begin, finish **and writes**.
Do not equate the largest observed collection call with an HTTP latency percentile
or promise. Finite samples do not establish a maximum over all schedules.

Counter scopes:

- `LIVE`: copied allocation-header bytes, candidate payload bytes read, drained
  barrier words, valid marker epochs, invalid epochs. Payload bytes exclude shape/
  marker and owner-remark reads; arena peaks exclude detached metadata and stacks.
- `TIMES`: marker thread CPU across all passes; guarded-write total wall, maximum
  wall, count and thread CPU. Write timing covers `gate_write`, including locking,
  copying/logging and clocks; it excludes the adapter's preceding allocation lookup.
- `PAUSES`: begin total/max wall; finish (or sync collect) total/max wall; owner CPU
  inside those lifecycle calls. Begin includes capture, finish includes waiting and
  owner reclamation. Allocation, ordinary generated work and unrelated scheduling
  are outside this scope.
- `BATCH`: payload-gate acquisitions, maximum bytes/reads under one hold, maximum
  hold wall and total hold wall. Hold timing starts after acquisition and ends before
  unlock. Zero timings without profiling mean disabled, not zero cost.

CPU scopes and wall scopes must not be mixed; overlapping wall intervals must not
be summed as process time. These counters expose important GC phases, not every
GC-related cost in the runtime. The reported earlier 2.59× was whole-workload process
CPU, not isolated collector CPU. Allocation throughput, application/HTTP throughput,
full memory costs and end-to-end latency still need qualified managed integration.

## Reproduce

Use `validate.py --sparse-index --batch --output <fresh>` for batched qualification
and omit `--batch` for the control. Use `measure.py --sparse-index --batch-comparison
--output <fresh>` for non-instrumented pairs; add `--gate-profile` in a separate run.
`summarize-batches.py --costs <report> --profile <report> --output <file>` verifies
counter/epoch bounds and keeps the scopes separate. Reports can be JSON or gzip JSON.
Retained source and raw reports are prefixed `batch-`; older reports remain unchanged.

Next: integrate admitted writer/publication paths while investigating a read protocol
that does not extend the heap mutex across lookup/tracing. The goal remains lower
interruptions at acceptable total CPU and memory cost, not winning raw collection
speed alone. No batching/default or public API change is enabled by this experiment.
