# Automatic concurrent GC candidate — 2026-10-24

This experiment connects the [snapshot/remark collector](../concurrent-remark/README.md)
to ordinary owned-heap JIT/AOT execution. It consumes background marks for real
reclamation, preserving synchronous explicit/pressure/idle cleanup semantics.
Production source and the default collector remain unchanged. The candidate still
copies/computes extra data and is not recommended as the performance default.

## Lookup comparison

The original detached tracer uses binary search. The optional
`NEOCLR_SHADOW_SPARSE_INDEX` now uses detached ordinal buckets with the production
collector's population threshold (256/4096 buckets) and minimum 64-byte bucket
width. It preserves exact allocated payload bounds, interior roots and free-block
rejection, and adds 2/32 KiB plus an allocation per snapshot. Original addresses are
lookup keys, never dereferenced by the worker.

Optimized and undefined/bounds qualification passes both the original shadow and
explicit-reclamation fixtures, including the 142,272 scan configurations. Fresh
forward/reverse native workload pairs reduce candidate process CPU by about 41%:
the ratio to synchronous collection falls from 3.70× to 2.21×. `lookup-summary.json`
and the clean raw reports preserve the values. Initial measurement runs overlapped
the end of validation and are retained but excluded. The previous 3.69× result was
for the complete prototype, not an isolated cost of copying. Sparse lookup makes
that distinction concrete; it does not establish that copying will become viable.

## Automatic lifecycle

- Owned allocation/publication-budget collections start or finish an epoch.
- Idle first finishes any pending epoch and synchronously rechecks the current
  roots, preserving cleanup semantics. It then captures survivors to begin work
  for the next interval. This also exercises plain requests below both thresholds.
- Explicit and allocation-pressure requests finish and synchronously recheck. This
  releases floating garbage before the existing allocator's single retry.
- A context switch cancels the old snapshot before collecting the requested context.
  Invocation completion cancels before retiring intern descriptors and static roots.
  Heap release cancels and joins the marker before relinquishing the arena.
- Shared-worker builds reject at compile time. This first integration requires
  profiling and uses unchanged generated objects and native root imports.

The cancellation paths discard copied state without reclaiming managed objects.
Ordinary collection supplies subsequent cleanup. The extra synchronous rechecks
are intentional compatibility work and a performance cost; this prototype has not
removed the need to account for them. Each owner has one persistent marker and at
most one in-flight snapshot, with bounded queue depth but potentially substantial
copied-heap memory. It is not a global background worker budget.

## Evidence and discovered lifecycle issues

Longer plain/JSON cohorts pass 1,024 responses in each of JIT/AOT: 4,096 checked
responses, 4,163 completed concurrent epochs, four invocation cancellations and
zero epochs left active at host exit. Counts include startup/shutdown. Forced-GC
fallback passes two responses per sample/mode, with zero concurrent starts.
Optimized/undefined-bounds lifecycle tests cover pressure recovery, idle resource
cleanup, foreign contexts, intern retirement and release. Existing synchronous GC
and intern-seed fixtures pass with their original forced-collection behavior.

The initial publication-only integration never activated on these samples; the
runner rejected it. The next version served all responses but aborted at shutdown.
Diagnostics found an intern descriptor legitimately changing from INTERN to BYTES
at invocation completion. Cancellation before that transition fixes the failure;
the normal host shutdown invariants remain intact. Preserve the failed runs and
diagnostic collector source in `evidence/`.

An initial existing-fixture run used paced flags although that fixture depends on
forced root-publication collection. The same synchronous baseline fails with exit
58 under those flags. The corrected runner keeps its original forced configuration
and tests automatic pacing in the dedicated lifecycle fixture. An initial missing
`stdint.h` compile failure is retained separately.

Race-detector and AddressSanitizer qualification remain unavailable on this host
as documented in the shadow study. No Windows or shared-heap qualification is
claimed. The current GC timer does not include all wrapper waiting/recheck work;
it must not be presented as an end-to-end pause distribution. HTTP latency observes
the complete server path. Full blocking-pause instrumentation remains required.

## Plaintext capacity result

Four one-owner AOT trials (forward/reverse, 2-second warmup, 10-second window,
32 connections, two client processors) pass response and shutdown checks.
Synchronous throughput is 4,146–4,341 responses/s; candidate throughput is
3,321–4,008. Process CPU/response rises from 223–231 to 251–264 microseconds.
The paired means are -13.7% throughput and +13.4% CPU/response.
Candidate p99 ranges from 9.34 to 29.12 ms versus 8.08–12.76 ms. These small, noisy
local cohorts are not confidence intervals or peer comparisons. They show a
working integration with a regression, not a case for changing the default.
`capacity-summary.json` and `evidence/capacity.json.gz` retain every pair.
Both variants are profiled; process CPU includes the background marker. Warmup and
shutdown are included in the process-CPU-per-completed-response calculation.

## Reproduction and remaining work

```sh
python3 benchmarks/execution/concurrent-integration/generate.py --output target/concurrent-new/native-gc.c
python3 benchmarks/execution/concurrent-integration/validate.py --output target/concurrent-new/validation
python3 benchmarks/execution/concurrent-integration/measure.py --gc-source target/concurrent-new/native-gc.c --sample plain --requests 1024 --output target/concurrent-new/plain
python3 benchmarks/execution/concurrent-integration/measure.py --gc-source target/concurrent-new/native-gc.c --sample json --requests 1024 --output target/concurrent-new/json
```

The runners reuse the qualified objects and native host recipe from state
integration. `capacity.py` compares the same plaintext managed object with matched
profiled synchronous/candidate hosts; its one-owner scope is deliberate and does
not establish worker scaling. Broader JIT/AOT standalone samples, complex handlers,
worker counts, full pause/RSS measurements and matched peer comparisons remain
under the original goal. The next optimization must reduce snapshot/remark work
and preserve these lifecycle checks before considering a default change.
