# Runtime snapshots and HTTP owners — 2026-10-23

This implements the first bounded part of the author's standard-monitoring proposal:
private versioned runtime snapshots, plus safe aggregation of completed isolated HTTP
application invocations. It supports cost investigation; it is not a throughput
optimization, live monitor, public managed API or a claim of Kestrel parity.

## Contract

`runtime/native/diagnostics/snapshot.h` defines version 1, byte size, an availability
mask and thirteen uint64 cumulative counters. Capture reads only the caller thread's
existing GC/root/async counters into caller-owned native storage. No allocation,
clock, formatting, I/O, lock or managed value object is needed by capture/arithmetic.

Counters cover collection/allocation/reclamation counts, reclaimed bytes, root
publications, GC attempts/nanoseconds/failures, allocated payload bytes, async fibers
and resumes, stack mappings and unmappings. Bytes are payload counts where supplied
by GC; GC nanoseconds are inclusive collection elapsed time, not sampled CPU time.
Fibers are runtime async bodies, not a count of managed Tasks. Allocation bytes and
GC timing/attempt/failure detail require the existing `NEOCLR_GC_PROFILE` build.
Unavailable fields export as JSON null, never as misleading zero measurements.
Sources and snapshot.c must use matching feature flags.

Delta requires two snapshots of the same owner with no intervening source reset or
counter wrap. Merge requires disjoint intervals/owners with identical availability.
Version, size, mask, subtraction underflow and addition overflow are checked;
failure leaves output unchanged. In-place output is supported. These functions do
not infer owner identity or make arbitrary shared-heap snapshots additive.

The optional `NEOCLR_RUNTIME_SNAPSHOT` HTTP adapter captures before/after each
application invocation. It reserves one of four group-owned slots under the existing
startup gate, then each invocation writes only its own slot. The coordinator reads
them only after the pool has joined/destroyed its workers. No concurrent reads of
live TLS, global per-request atomics or per-request locks are introduced. A host sink
runs once on the coordinator, synchronously while copies remain alive. It must not
retain pointers into the group. Disabled builds contain none of this code/storage.

The benchmark sink merges these **isolated application intervals**, including their
startup/shutdown, and preserves per-invocation rows. They are not whole-process
counts: coordinator/pool work is excluded. Slots identify invocation order, not
stable OS thread IDs; failed invoked jobs may also contribute. Shared-heap workers
are not integrated and must not be counted once per participant. Joined copies retain
counts after worker exit; subsequent runs create new group storage.

Heap occupancy gauges, summed worker peaks, queue/wait timing, pause histograms,
CPU samples and live worker counts are deliberately outside this initial snapshot.
They need distinct semantics and collection mechanisms. The existing detailed
publication-site profiler remains a separate benchmark diagnostic. Invalid snapshots
or failed export do not abort the application; this benchmark diagnoses them by a
missing/invalid output file. Production transport and reporting policy remain open.

## Validation and overhead

Eight optimized/UBSan-bounds configurations cover no sources, GC-only, async-only and
all sources. Tests check source-field mapping, unavailable masks, four-thread delta
capture and post-join merging, TLS isolation, version/size/mask mismatch, null inputs,
underflow, overflow, unchanged output on rejection and in-place results. The HTTP
worker object is byte-identical to HEAD before this change with the flag disabled.
Sanitizers do not establish race-detector qualification.

Six serial four-owner trials use off/basic/timed/timed/basic/off ordering, the same
validated application object and EOF-aware client, 32 connections and a four-second
window. There is no separate warmup. Builds happen before each run; no build or
other benchmark overlaps measurement. Every response, exit, available counter sum,
stack mapping balance and timed GC failure count is checked. CPU is child process
user+system time minus client process CPU, divided by all window/drain responses;
it includes startup/shutdown and export. This is a short overhead check, not the
longer peer-capacity comparison or a sustained-load qualification.

| Mode | Mean requests/s | Mean server CPU µs/response |
| --- | ---: | ---: |
| Off | 4,935.0 | 753.91 |
| Basic snapshot | 4,794.9 | 755.56 |
| Snapshot + existing GC timing | 4,847.6 | 763.41 |

Basic is -2.84% throughput and +0.22% CPU/response; timed is -1.77% and +1.26%.
The short trials have desktop variation and mixed latency. They do not establish
zero overhead or resolve a small repeatable cost; differences also include code
layout and startup/export. Keep detailed timers opt-in and use longer paired runs
if monitoring is moved onto the request path or enabled by default. No throughput
improvement is claimed. An additional one-owner timed run validates that boundary.

Basic four-owner results report about 208 allocations and 3,677 publications per
response. Timed runs report about 16,004 allocated payload bytes and 3.60 GC attempts
per response with zero failures. These are work counts across invocation intervals,
not attribution of CPU time to methods or evidence that GC alone explains the gap.

## Reproduce

Use the SDK and matching application/compiler bundle from the peer comparison.
Each output path must be new. `--mode off|basic|timed` and `--workers 1|4` select the
explicit diagnostic configuration. The runner records source/object/binary hashes.

```sh
python3 benchmarks/http-server/primitive-comparison/runtime-snapshot/validate.py \
  --output target/runtime-snapshot-validation
python3 benchmarks/http-server/primitive-comparison/runtime-snapshot/run.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-owned-root/candidate.o \
  --client target/primitive-comparison-refresh/client \
  --workers 4 --mode basic --output target/runtime-snapshot-basic
```

Disabled-object validation compares the working source to HEAD; the archived result
records the pre-commit comparison for this implementation. General .NET metrics and
EventPipe guidance, deliberate initial limitations and future timing/sampling work
remain in the [runtime diagnostics design](../../../../docs/design/allocation-metadata-runtime-comparison.md#standard-runtime-diagnostics-proposal--2026-10-23).
No compiler metadata/bridge or public Raven API changes; API snapshot/website builds
are not relevant to this private native diagnostic.
