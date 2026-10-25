# Bounded native mutation gate — 2026-10-24

This implements the first primitive from the [mutation audit](../concurrent-mutation-audit/README.md).
It is private experimental C code, not an enabled GC or a new managed API. It is now integrated with the admitted live-generated/native paths; the initial
primitive evidence below predates that integration. The full-copy candidate
and synchronous collector remain the correctness/performance references.

## Implemented protocol

An owner thread creates a gate and begins an epoch. A marker presents that epoch to
copy at most 256 payload bytes into private scratch under the gate, then traces only
the scratch after unlocking. Writes contain up to eight patches in one transaction,
so array payload and marker updates can remain coherent. Before applying patches,
the gate records complete overwritten conservative words; after applying all patches,
it records final inserted words. Narrow/unaligned writes therefore preserve whole
candidate lanes. Identity lanes can be excluded by an aligned scan-start offset;
trailing incomplete words are protected but not logged, matching the current scanner.
Overlapping patches have sequential memmove semantics and may log duplicate lanes.

The original measurements below used 128 words. The current experimental default is
4,096 words (32 KiB), configurable from 8 to 16,384 with a consistent `GATE_WORDS`
definition across translation units; see the [matched HTTP evidence](../concurrent-store/plaintext-performance.md).
The wider bounded log avoids observed cancellation churn at a native-space cost. Writes reserve space for both old and new words before
changing anything. Exhaustion cancels the epoch and leaves the payload unchanged;
subsequent reads/drains/writes reject that cancelled epoch. The integrating owner
must stop/join the marker, discard its detached marks, and retry only after the cycle
is inactive. Synchronous recovery belongs at a root-published boundary, not inside
an arbitrary store with unpublished generated temporaries. Stopping prevents new
marker payload reads; the caller must join before reclaiming storage or destroying
the gate. Epochs never wrap, and stale tokens cannot access a later cycle.

Only the owner starts/stops epochs and mutates payloads. The marker may copy/drain.
Ownership failures return errors. Begin/stop serialize with payload reads; inactive
writes need no mutex because only the owner changes epoch state and no marker can
read inactive payloads. The active path uses a mutex and fixed inline buffers with
no explicit allocation or callbacks. This protects scanner/writer access; it does
not serialize multiple application writers or solve shared-heap ownership.

## Caller obligations still needing integration

All live payload reads by a marker must use this gate, and every admitted write to
scanned storage must participate. Sources/scratch/queue-output buffers are caller-owned
and must not alias gate internals or introduce unsynchronized live-heap writes.
Raw pointer extents/lifetimes remain trusted: bounds checks validate supplied spans,
not arbitrary C memory safety. Do not execute managed code, callbacks or collection
while holding the gate. The API executes copies internally instead of exposing a
lock token across generated calls.

The gate does not know type shapes or initialized-element semantics. A marker-only
transition that exposes an existing array payload must also enqueue/rescan that
payload; logging the marker alone is insufficient. The grouped payload/marker test
covers a complete transaction, not every publication path. New objects require an
explicit construction/publication contract. Large-object chunks, current/parked
roots, conditional exposure, detached mark queues, stable allocation identity,
future-address candidates and owner-thread sweep must be integrated separately.
The gate cannot authorize reclamation by itself.

## Validation and measured primitive costs

Optimized and UBSan/bounds builds pass:

- All 666 offset/length combinations in a 35-byte payload, independently checking
  changed bytes and exactly logged complete old/new lanes after the identity word.
- 100,000 grouped two-lane plus byte-marker transactions per build with a concurrent
  reader; 52,144/81,505 coherent snapshots in the retained runs. Every transaction's
  four logged candidate words are checked against its previous/final values.
- Queue exhaustion with unchanged payload, partial/zero-capacity drains, cancellation,
  inactive retry, stale epochs, nonwrapping epoch exhaustion, bounds and wrong-thread
  mutation/lifecycle rejection.

The previous host race-detector startup failure remains unresolved. UBSan and these
interleavings do not prove race freedom or qualify production integration.

Six sequential diagnostic trials execute ten million writes each, in forward/reverse
mode order. CPU per operation is 1.02–1.28 ns for a noinline direct store, 8.74–8.79 ns
for an inactive gate write, and 30.61–31.23 ns for an active write plus a queue drain
after every store. There is no background marker in this cost fixture; it isolates
protocol overhead, not contention, tracing or HTTP throughput. The direct store is
not the actual generated-store baseline. These two observations per mode establish
an overhead to measure during integration, not a performance gain or confidence bound.
Batching drains may reduce overhead, while real contention may increase it.

Run `validate.py --output <fresh-directory>` and `measure.py --output <fresh-directory>`.
Reports retain exact compiler commands and source hashes. No application performance
matrix was rerun for this isolated primitive because no production behavior changed.
Next integrate the primitive into an opt-in owned-heap marker and all admitted writer
paths, reject unsupported providers explicitly, then run existing differential GC
fixtures and batched standalone/HTTP consumers before considering default admission.

[Registered worker protocol](workers-results.md) documents the opt-in multiwriter prerequisite, validation and remaining shared-heap integration boundaries.
