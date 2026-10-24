# Concurrent immutable shadow tracing — 2026-10-24

This experiment runs native graph tracing on a persistent background thread using
an immutable heap capture. The synchronous collector still computes the reference
marks and performs all cleanup/reclamation. Before the next capture, the owner
waits for the previous job and compares every block's mark bit. This is a correctness
oracle and a concurrency/lifetime experiment, **not concurrent reclamation, a new
default or a throughput improvement**.

It follows the [author-directed concurrent GC baseline](../concurrent-gc-investigation/README.md).
.NET background GC and Go's concurrent collector provide the design comparison;
this diagnostic implementation deliberately copies the heap instead of reading
mutable payloads concurrently. That buys read safety without new write barriers,
but adds copying, allocations, memory and duplicate tracing. It is not proposed as
the final low-CPU architecture.

## Ownership and correctness

`generate.py` instruments an out-of-tree copy of `native-gc.c`. At the point where
production has seeded strong roots and interns, the owner copies the arena, block
offsets and seed marks. Conditional host/explicit edges and owned parked-frame
values are captured separately. Scalar 32-bit discriminators remain excluded.
The worker holds no live frame, payload, root-table or execution-state pointer.
Original heap addresses are numeric lookup keys; binary search resolves them into
copied block offsets, checking allocated payload bounds and interior pointers.

The payload scanner is extracted from the current production scanner, preserving
object, string-array, record-array, initialized-slot and intern semantics. The
worker owns its mark bitmap and bounded work queue and reaches a conditional-edge
fixed point. Reference marks are copied before synchronous cleanup/sweep. Mutex
publication/completion protects job ownership; the owner alone reads the reference
bitmap. Jobs cannot be dropped or overwritten. One job per owner bounds queued
memory, and the next capture waits when the worker falls behind. Owned-heap release
and the explicit experiment report drain and join the worker.

This shares production strong-root seeding and payload semantics: it independently
checks detached lookup/work queues, conditional capture, fixed points and lifetime,
not an independently implemented proof of every GC rule. Reference collections
that fail descriptor checks are counted as invalid and excluded from mark equality.
The existing malformed-input fixture exercises seven such captures per configuration.
No worker result can release a managed allocation or native resource.

## Validation

Optimized and undefined/bounds-sanitized runs both pass eight native fixtures:

- Delayed snapshot test: 600 comparisons across three fresh owner threads plus
  one empty/null-arena collection. The worker is held behind an atomic test gate
  while the owner completes collection, changes roots and a cyclic graph, and
  reuses reclaimed storage. Only then may the worker trace its copy.
- The existing scan fixture compares 284,544 collections per configuration across
  142,272 payload/lane/candidate combinations, including interior pointers and
  header/padding rejection.
- Existing conditional/parked-root/resource/fault, sparse/dense index, stale-mark,
  intern and binary slot-map fixtures retain their original assertions.
- Checked plaintext and JSON HTTP runs cover 1,024 responses per JIT/AOT mode:
  4,096 responses total. Every successful collection must have a matching shadow
  result, zero invalid captures and no HTTP failure.

`validation.json.gz` is the final native report. Initial and revised runs, short
HTTP trials and longer cohorts are retained separately. `native-gc.c.gz` is the
final generated collector; each HTTP report hashes its actual generated source.

ThreadSanitizer crashes in its dyld initialization before `main`; an empty program
reproduces the crash. AddressSanitizer hangs in recursive allocator initialization
before `main`; an empty program also times out. The failures, smoke-test commands
and relevant startup stacks are retained. **Neither sanitizer is qualified on this
host.** Undefined/bounds sanitization is passing evidence, not a substitute for a
race detector. Windows and shared-heap workers are not qualified by this experiment.

## Cost interpretation and next work

`summary.json` reports copied bytes, capture wall time and background tracing CPU
per response, including startup/shutdown. These are sequential-client diagnostics,
not capacity or latency trials. Capture time includes allocations and initial
worker creation but excludes waiting for the previous job; worker CPU excludes
waiting, owner comparison and freeing copies. Do not add these unlike measurements
or interpret them as whole-runtime CPU or a projected concurrent speedup.

The snapshot approach performs additional work and retains an extra heap copy plus
indexes, two bitmaps, a queue and conditional edges. Its binary-search lookup also
differs from the production sparse index. The resulting overhead makes it unsuitable
as evidence of a performance win; its value is the repeatable differential check
while the real heap changes underneath it.

Next implementation work must define the live-heap read and mutation protocol,
including native/bulk/byref writes, conditional/weak exposure, new allocations and
final reclamation. The conservative production scanner reads scalar words as well
as reference lanes, so atomically publishing references alone cannot establish C
read safety. Audit these paths before permitting concurrent live-heap scans. Keep
this snapshot oracle available to compare future implementations. Expand to the
standalone and complex-handler matrix required by the author; native fixtures and
plain/JSON HTTP are only this first qualification slice.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-shadow/generate.py --output target/shadow-new/native-gc.c
python3 benchmarks/execution/concurrent-shadow/validate.py --output target/shadow-new/validation
python3 benchmarks/execution/concurrent-shadow/measure.py --gc-source target/shadow-new/native-gc.c --sample plain --requests 1024 --output target/shadow-new/plain
python3 benchmarks/execution/concurrent-shadow/measure.py --gc-source target/shadow-new/native-gc.c --sample json --requests 1024 --output target/shadow-new/json
python3 benchmarks/execution/concurrent-shadow/summarize.py benchmarks/execution/concurrent-shadow/plain-final.json.gz benchmarks/execution/concurrent-shadow/json-final.json.gz --output target/shadow-new/summary.json
```

The HTTP runner reuses the qualified generated objects and native host recipe from
[state integration](../state-integration/README.md). Native qualification accepts
`--sanitizers thread` or `--sanitizers address` for a compatible host. No production
collector, public API, compiler import, Raven bridge encoding or runtime default
changes in this slice.
