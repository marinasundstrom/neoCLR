# Native allocator accounting — 2026-10-25

Follow the [JIT lifetime evidence](../jit-compilation-lifetime-20261025/README.md)
with process-wide native allocation accounting. This is diagnostic instrumentation,
not a change to GC, allocation policy, or production hosting.

## Contract and boundaries

On macOS test runners, `NEOCLR_TEST_NATIVE_MEMORY=1` samples
`malloc_zone_statistics(NULL, ...)` before JIT preparation, before and after
compilation-input release, and after invocation (including executable cleanup).
The installed Xcode MacOSX26.2.sdk `usr/include/malloc/malloc.h` defines the C layout:
unsigned blocks-in-use followed by three size_t counters. Its contract explicitly
sums all zones for a null zone. `size_in_use` is live malloc storage;
`size_allocated` is allocator-reserved storage; `max_size_in_use` is the allocator's
high-water mark of touched memory. These are not RSS, physical footprint, direct
mmap mappings, isolated managed-heap use, or per-owner attribution.
The host reports zero for the high-water field despite nonzero live storage; retain
that raw value but do not interpret it as a supported peak measurement. In particular,
subtracting live malloc bytes from RSS cannot identify all of the remainder.

This follows the existing CLR-style separation of managed heap diagnostics from
whole-process native memory. It adds no managed API or runtime ABI, and production
builds exclude it. Fixed-size snapshots are taken before JSON serialization so
report construction is outside the measured snapshots. The OS may take allocator
locks; instrumented timings are diagnostic-only. No pressure-relief API is invoked.

`http.py --process-memory` requests these counters from current JIT test runners
in addition to external RSS snapshots. It rejects missing stages. Ordinary timing
runs clear the diagnostic switch. The harness requires a current runner; older
runners without these counters no longer satisfy JIT process-memory qualification.

## Reproduction

Build and pin the current release codegen test runner. Run
`native_memory_profile::observes_live_native_allocation_and_release --exact
--ignored --test-threads=1`: process-wide deltas require isolation from other tests.
Then use `prepare-jit-lifetime.py` to generate matched retained/released sync/live
recipes, and run `http.py --process-memory --protocols pipeline --requests 1024
--rounds 1` with those four modes. Memory profiles must run separately from timing
benchmarks. Record source/runner hashes and retain raw reports.

## Results

All eight cases pass 8,320 checked responses. The isolated 8 MiB counter test and
three process-memory helper tests pass. Release test/production compilation succeeds.
The C SDK layout probe reports size 32 and offsets 0/8/16/24 on macOS ARM64.

| Handler | Mode | Live before release MiB | Live after release MiB | Live after cleanup MiB | Reserved after release MiB | RSS after traffic MiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| plain | retained-sync-jit | 200.43 | 200.43 | 193.13 | 812.00 | 863.22 |
| plain | released-sync-jit | 200.43 | 10.82 | 3.52 | 904.00 | 914.61 |
| plain | retained-live-jit | 203.37 | 203.37 | 193.13 | 1024.00 | 1091.17 |
| plain | released-live-jit | 203.37 | 13.77 | 3.52 | 1052.00 | 1064.97 |
| json | retained-sync-jit | 200.43 | 200.43 | 193.13 | 920.00 | 958.84 |
| json | released-sync-jit | 200.43 | 10.82 | 3.52 | 916.00 | 919.02 |
| json | retained-live-jit | 203.37 | 203.37 | 193.13 | 984.00 | 1056.50 |
| json | released-live-jit | 203.37 | 13.77 | 3.52 | 960.00 | 975.83 |

Early release removes about 189.6 MiB of live malloc storage in each released case;
retained controls remain unchanged at that boundary. Reservation and RSS remain
large and variable. Released sync plaintext even has higher RSS than its retained
control in this cohort. These single diagnostic runs provide no timing advantage,
consistent RSS reduction, leak diagnosis, or collector comparison claim.

The native allocator reservation substantially exceeds its live allocation count;
it accounts for a large address-space reservation beyond live input state. Whether
those pages are resident requires separate mapping evidence. Direct JIT mappings
are outside the malloc counters, and the samples occur at different lifecycle
points from the external RSS snapshots. Do not subtract them as exact attribution.

Raw reports: `http.json.gz`, `recipes.json.gz`; test/build output:
`counter-test.txt.gz`, `build.txt.gz`; SDK provenance: `sdk-layout.json`.

## Interpretation and next work

The retained/released comparison isolates the lifetime of compilation inputs; it
does not change the managed object graph or collector settings. The allocator can
retain much more storage than is currently live. Large RSS therefore does not
justify changing GC heap limits or claiming a concurrent-GC leak.

Keep allocator pressure relief and platform-specific trimming out of production
until separately measured. First examine transient selection/metadata duplication
if reducing JIT startup residency is the target. For concurrent HTTP throughput,
continue with the existing matched CPU/stack evidence for root/store/index work;
allocator reservation alone does not establish an execution bottleneck. Do not
substitute memory profiles for timing or pause acceptance.
