# Generated closure completion delivery — 2026-10-28

Development prerequisite, macOS ARM64. No HTTP throughput, JIT startup, parallel
managed execution or production integration claim. Runtime implementation and
collector defaults are unchanged. See the [fixture and reproduction](../../experiments/worker-delivery/README.md#generated-closure-delivery-qualification).

## Accepted qualification

`qualified.json.gz` records commands, compiler/runtime/source hashes and all four
successful builds/runs. Same generated Native AOT object; C runtime/adapter checked
in optimized and UBSan builds. UBSan does not instrument generated code. Two worker
threads, one outstanding ticket and dispatch budget one; guest callbacks serialize
through the existing pool heap gate. Sixty-four owner suspension/resumption cycles
per run; each callback allocates 2,500 boxed values and mutates the same cyclic,
aliased captured graph and shared static counter. Live displaced references survive
collection. Successful submissions release the caller closure root, leaving the
ticket root until adoption; the deliberately faulting job retains a recovery root
because failed results are discarded. Forced pool collection precedes diagnostic
copy/take. Job 33 faults with checked text; subsequent jobs recover.

| Build | Deliveries | Worker allocations | Worker collection calls | Worker epochs started | Owner epochs started/finished/cancelled |
| --- | ---: | ---: | ---: | ---: | --- |
| Sync optimized | 64 | 160,196 | 482,442 | 0 | N/A |
| Sync UBSan | 64 | 160,196 | 482,442 | 0 | N/A |
| Concurrent optimized | 64 | 160,196 | 2,048 | 0 | 123 / 59 / 64 |
| Concurrent UBSan | 64 | 160,196 | 2,048 | 0 | 128 / 64 / 64 |

Counts are qualification diagnostics, **not comparable performance units**. Collector
pacing differs. Worker deltas sum thread-local counters around generated invocation;
owner epoch counts include pump/idle operations and depend on scheduling. No owner
epoch remains active at the checked teardown boundary. These tests add no new timing
measurements; the previous native delivery latency/CPU table remains unchanged.

## Finding: concurrent build does not imply concurrent worker collection

The serialized callback bridge transfers the arena cursor and static storage to a
fresh invocation while holding the pool gate. It does not register that arena with
`neoclr_gc_heap_acquire_v1`. Automatic concurrent epochs require the context arena
to match the thread-local owned heap (`runtime/native/gc/concurrent/automatic.py`).
Consequently generated pool callbacks collect synchronously in this configuration,
although the separately owned owner heap runs background epochs. Raising callback
allocation pressure from 500 to 2,500 objects still starts zero worker epochs.

This narrows the qualification: shared captures and owner delivery work with both
collector builds, but overlapping marking and generated worker mutation is not
established. Do not attach a live pool/HTTP arena using the existing empty-heap
acquisition API or copy captures to bypass this boundary. Next design owner/worker
heap participation and parked-root/service admission, then qualify actual worker
marker activity before the [HTTP integration and new benchmark](../../experiments/worker-delivery/http-plan.md).

## Retained investigation

- `initial`: fixture abort caused by assuming take returns a raw reference. The
  contract returns an opaque root and clears the pointer; failed jobs discard
  results. Corrected test adopts roots and retains one fault-recovery root.
- `corrected`, `allocation-check`: four passing builds each at 500 allocations per
  callback; the latter verifies worker allocation/collection deltas. These did not
  prove background worker activity.
- `epoch-check`: expected-positive worker epoch assertion fails in concurrent
  optimized after sync cases pass.
- `pressure-epoch-check`: the same assertion fails at 2,500 allocations; establishes
  that increasing pressure alone is insufficient.
- `qualified`: final explicit worker-fallback/active-owner assertions, all four
  builds pass. No runtime change was made to turn an unsupported ownership path on.

`index.json` hashes the decompressed retained reports, final object/native metadata,
generated collector source and codegen output. Earlier reports identify their input
hashes; intermediate fixture source revisions are not separately archived. TSan
remains unavailable as established by the previous delivery slice. Cancellation,
capacity pressure, socket I/O and fairness are not newly qualified here; reuse
existing native evidence within its scope. The compiled callback itself is
synchronous; suspension occurs in the native owner adapter, not in a generated
HTTP handler or a general managed Task.Run implementation.

Website content checks: `python3 scripts/test_build_website.py`, 26 passing tests.
No website build or publication was performed for this prose-only update.
