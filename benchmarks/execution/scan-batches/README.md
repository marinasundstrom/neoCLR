# Rejected conservative scan batching

Development experiment, 2026-10-24. **Production scalar scanning is restored.**
Baseline is `82fad1e00`. The retained code change is a portable differential/liveness
fixture registered in the standard native-GC validator, plus reproducible evidence.
No new compiler, managed API, object format or collector policy ships here.

## Question and implementation

The latest [execution-state profile](../state-access/README.md) still places collection
first among worker self samples. Earlier [lookup counters](../lookup-profile/README.md)
show that ordinary-object fields supply most candidates, with roughly two-thirds of
queries outside the heap. Could filtering four contiguous words before full lookup
reduce that work without loading more metadata or increasing allocation headers?

Both experiments keep conservative semantics: skip the first object identity word,
scan only complete payload words, reject only outside-arena values, and send every
remaining value through the existing allocation/payload-bound checks. Interior and
unaligned pointers, marking order and fixed-point processing remain unchanged.
Reference arrays, records and root publication are unaffected. Trace-time payloads
are immutable, so reading a small batch ahead does not cross a mutation/callback.

The initial implementation creates a four-bit range mask. The adjusted variant
uses bitwise comparisons, skips empty masks, and reloads selected words instead of
retaining the temporary array through marking. Both preserve six measured tracing
counters, including the logical outside-arena queries filtered before `mark()`.

Optimized ARM64 disassembly explains why this is not an automatic win: the initial
filter uses scalar comparisons and a stack copy; the adjusted filter still does not
produce SIMD range comparisons. `collect_indexed` grows from 1,262 disassembly lines
to 1,357 initially and 1,336 after adjustment. These counts describe the inspected
build, not a portable instruction-cost model. The filter adds mask/control work and
qualifying values still encounter the normal lookup checks.

## Paired plaintext result

Same managed object, paced owned heaps, checked ten-byte response, connection close,
32 connections, Go GOMAXPROCS=2. Two-second warmup plus ten-second measurement;
two trials/configuration in forward/reverse order, 16 trials across two cohorts.
No builds or inspection overlap capacity. CPU covers server lifetime less client CPU.
Four owners use isolated heaps; this is not shared-heap capacity.

| Variant | Owners | Requests/s baseline → candidate | Throughput change | CPU µs/response baseline → candidate |
| --- | ---: | ---: | ---: | ---: |
| Initial | 1 | 3,730 → 3,660 | −1.9% | 264.1 → 268.4 |
| Initial | 4 | 10,850 → 10,637 | −2.0% | 335.1 → 340.1 |
| Adjusted | 1 | 3,700 → 3,678 | −0.6% | 267.1 → 267.7 |
| Adjusted | 4 | 10,629 → 10,682 | +0.5% | 338.5 → 337.6 |

The initial version loses in both configurations. Adjusted ranges overlap and its
sub-percent differences do not establish a convincing gain. Reject both rather than
add code to a hot loop without demonstrated benefit. These short local runs do not
rule out other batching techniques, compilers or targets. No JIT/interpreter capacity,
JSON performance or peer-parity claim is made. AOT rejection was sufficient to stop
this experiment before broader integration qualification.

## Retained safety tests

Each execution checks 142,272 configurations: payload sizes 1–144 bytes, every full
or partial word position, zero/all-ones/header/padding candidates, and every byte of
a nine-byte child payload. Each runs with external and exclusively owned heaps.
Placing a pointer in the identity word or a partial trailing word must not retain the
child; all valid interior offsets in actual fields must retain it. Replacing the root
must reclaim the whole graph. Partial-word cases deliberately fill allocation padding
so accidental over-scanning cannot pass just because padding contains zeros.

Both variants match the scalar baseline in optimized/UBSan+bounds builds, with
profiling enabled/disabled: eight executions per variant, 1,138,176 configurations.
The six printed tracing counters match exactly in enabled builds and remain zero
when disabled. A further replay verifies the adjusted snapshot out of tree after
restoring production source. The explicit-source capacity runner also rebuilds all
four hosts successfully without rerunning capacity. These reports preserve commands and outcomes;
no throughput inference is drawn from the functional test runtime.

## Architectural implication

This advances the [coherent investigation](../../../docs/design/runtime-bottleneck-investigation.md)
by rejecting one cheaper-looking scan path. It does not reduce the number of fields
visited. Reuse the [.NET/Java/Go design comparison](../../../docs/design/allocation-metadata-runtime-comparison.md):
authoritative shared physical tracing layouts could avoid scalar fields entirely,
but require complete producer coverage and safe image/layout lifetime handling.
An inspection-only mask or type token alone is not sufficient evidence for skipping
references. The earlier image-specific object-tracing prototype remains separate
and unintegrated. Collection-trigger reasons (allocation, publication, idle and
pressure) are also worth separating before changing GC budgets; this experiment
changes neither frequency nor pacing policy.

## Reproduction

The snapshots are experiments, never installed runtime sources. Headers and root
helpers use baseline `82fad1e00`; qualified inputs/host recipes must already exist.
The runners now accept an explicit candidate source so reproduction does not mutate
the checkout. Original measurement/validation scripts are archived as well, because
the recorded runs used the experimental source temporarily in its runtime path.

```sh
mkdir -p target/scan-replay
gzip -dc benchmarks/execution/scan-batches/adjusted-native-gc.c.gz > target/scan-replay/adjusted.c
python3 benchmarks/execution/scan-batches/validate.py \
  --candidate-source target/scan-replay/adjusted.c --output target/scan-replay/tests
python3 benchmarks/execution/scan-batches/measure.py \
  --candidate-gc target/scan-replay/adjusted.c \
  --build target/primitive-comparison-build/build.json \
  --object target/slot-maps/plain.o --candidate-object target/slot-maps/plain.o \
  --client target/primitive-comparison-refresh/client --output target/scan-replay/capacity
python3 benchmarks/execution/scan-batches/summarize.py \
  benchmarks/execution/scan-batches/initial-capacity.json.gz \
  benchmarks/execution/scan-batches/adjusted-capacity.json.gz target/scan-replay/summary.json
```

`summary.json` uses the established paired arithmetic. Source hashes verify both
snapshots against the raw capacity reports and the restored production collector
against Git. Existing website performance claims and API documentation remain valid;
no production feature changed, and no website build or compiler rebuild was needed.
