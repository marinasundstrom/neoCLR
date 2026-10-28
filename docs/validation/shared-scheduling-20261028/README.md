# Shared ready storage: correctness and native layout

Development qualification on macOS ARM64, 28 October 2026. Candidate remains opt-in
with `NEOCLR_SHARED_READY_QUEUE`. No HTTP speedup or collector preference is established.

The owner scheduler and native worker FIFO/local queues now use one bounded queue
primitive. Root ownership, execution affinity, I/O readiness and completion remain
with the existing callers; this is shared ready storage, not complete lifecycle
unification or parallel managed HTTP execution.

## Checks

- [Queue and owner contracts](contracts.json): model checks at capacities 0, 1, 2,
  16 and 256, each with 20,000 operations; wraparound, FIFO/LIFO, removal, full/empty
  behavior and cleared slots. Diagnostics off/on, optimized/UBSan. Existing owner
  scheduler lifetime, backlog and fairness checks pass under sync/concurrent GC,
  optimized/UBSan, with shared storage and diagnostics enabled.
- [Worker FIFO](worker-fifo.json) and [combined optional worker paths](worker-combined.json):
  optimized/UBSan checks pass, including the local/global/stealing variant combined
  with compiled coordination, trace hooks and bounded completion delivery. The
  combined worker check is a synchronous native fixture, not concurrent generated
  HTTP qualification.
- [Layout report](layout.json): actual private compiler definitions, source hashes,
  commands and output. These are fixed native sizes, not retained task memory.
- The shared-capture HTTP source compiles with the current generic-contract Raven
  bundle (`target/dynamic-access-20261028/bundle-generic`). An older bundle rejected
  current Web-source annotation syntax; the fixture compiles Web sources like the
  existing persistent HTTP benchmark, without widening internal APIs. Generated
  execution and HTTP load measurements remain open; source compilation is not an
  HTTP correctness or performance result.

## Native layout inventory

| Record / storage | Control | Shared FIFO | Shared FIFO + diagnostics |
| --- | ---: | ---: | ---: |
| Runtime context header | 1,080 B | 1,080 B | 1,080 B |
| Worker job | 1,208 B | 1,208 B | 1,208 B |
| Worker pool (16 slots, maximum 4 threads) | 21,184 B | 21,464 B | 21,504 B |
| Owner scope | 12,384 B | 16,488 B | 16,528 B |
| Parked-root record | 40 B | 40 B | 40 B |
| Probe frame | 104 B | 104 B | 104 B |
| Ready entry | — | 16 B | 16 B |
| Ready queue header | — | 24 B | 64 B |

The owner scope contains 4,096 bytes of fixed ready entries; its total grows by
4,104 bytes versus the linked-list header control. Pool size grows by 280 bytes.
Diagnostics add 40 bytes per queue. The local/global/stealing shared variant has
a 22,848-byte pool and 1,216-byte jobs under its own flags; do not compare it as
though its policy/configuration matched FIFO.

Nested records overlap: do not add these rows. Stack reservation/commit, TLS root
tables, heap capacity, managed Task/state-machine/capture graphs, native allocation
overhead and OS resources are excluded. In particular a 40-byte parked-root record
does not mean a suspended execution costs 40 bytes. Idle storage is a real candidate
cost even if removing per-submission allocation later proves useful.

## Open gate

[Required profiles](../../experiments/shared-scheduling/README.md) include retained
managed memory, actual suspension/resumption, worker/OS switches, queue/lock/wakeup
costs and correlated GC/JIT events. Run matched HTTP cohorts with many endpoints,
file/downstream-network I/O and mixed short/long handlers under both collectors,
with separate traced and untraced runs. Native layouts and correctness checks do
not close the HTTP goal or replace its website throughput/latency tables.
