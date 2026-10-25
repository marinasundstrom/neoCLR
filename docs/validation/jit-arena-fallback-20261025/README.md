# Bounded JIT arena with system fallback — 2026-10-25

The [current placement measurements](../jit-arena-current-20261025/README.md)
establish a useful shared runtime improvement but leave a fixed 64 MiB failure
boundary. This slice adds a test-only fallback policy without promoting placement
or concurrent GC defaults. It reuses the pinned Cranelift providers and the existing
Apple instruction-cache wrapper, following the same implementation research.
No API, metadata, call-range or managed-GC semantic contract changes.

## Policy and ownership

`NEOCLR_TEST_JIT_ARENA=fallback` tries one 64 MiB reservation. If reservation fails,
all allocations use the system provider. On the first arena allocation error,
future requests permanently use the system provider. There is no repeated attempt
to allocate from an exhausted arena and no relocation or early freeing of existing
code/static data. The original fixed experiment remains selectable with `1`;
non-test compiler builds continue to use system allocation.

| State | New allocations | Existing storage |
| --- | --- | --- |
| Arena available | Arena | Owned until module teardown |
| Reservation unavailable or allocation failed | System | Existing arena storage remains owned |
| Freed | Rejected | Both providers freed once |

Both providers participate in finalization; errors propagate. The outer cache
wrapper tracks code allocated through either provider. The existing synchronous
module owner retains the host library through invocation and frees both providers
before unloading it. Explicit free is idempotent. This is a cap on the *initial
reservation*, not a cap on total JIT memory: spilled system allocations can grow.

The pinned arena allocation error used here leaves previous allocations intact.
This does not promise recovery from provider panics, protection failures or arbitrary
native faults; those underlying contracts remain unchanged. Large requests may
lose placement locality but retain the existing system allocation path. Native RSS,
reservation-size choice and platform-specific policy still require qualification.
Windows is not qualified by macOS tests and retains its production default.

## Safety evidence

System and fallback configurations each pass 203 active codegen tests; 34 retained
external-fixture tests remain ignored. Production compilation passes. The tests
include earlier repeated instruction publication and partial-allocation cleanup.

New injected-provider tests verify routing for executable/writable/read-only
allocations, no primary retries after spill, both finalizers remaining active,
unavailable reservation, secondary allocation failure propagation, exactly-once
free and rejection of allocation after free. A real Apple ARM64 test repeats 64
lifetimes: publish arena code and read-only data, force spill with a 128 KiB request
against a 64 KiB test reservation, publish system-allocated code, call both functions
and verify both data regions, then free both providers. This exercises mixed
provider lifetime and cache publication without allocating a huge compiled image.

These focused tests validate the new provider boundary; they do not establish
native-memory pressure behavior or a new throughput gain. The previous placement
cohort remains historical evidence, not a measurement of this fallback implementation.

## Reproduction

Build and pin the current release codegen test runner. Run it with no placement
flag and with `NEOCLR_TEST_JIT_ARENA=fallback`, retaining separate logs. Prepare
matched HTTP recipes with:

```sh
python3 benchmarks/gc-suite/matrix/prepare-jit-placement.py \
  --build target/gc-split-frame-build --runner target/jit-fallback-current/runner \
  --policy fallback --output target/jit-fallback-current/http-build
python3 benchmarks/gc-suite/matrix/http.py \
  --build target/jit-fallback-current/http-build --output target/jit-fallback-current/http-smoke \
  --modes sync-jit fallback-sync-jit live-jit fallback-live-jit \
  --protocols pipeline --requests 64 --rounds 1 --smoke
```

Source/runner hashes, test logs and host identities are retained alongside this
record. Retained native/toolchain inputs remain prerequisites. No default changes.

HTTP smoke qualification passes all eight plain/JSON system/fallback cases across
both native collectors: 640 responses including warmup, with clean response and
collector lifecycle checks. These runs do not force the normal 64 MiB reservation
to spill; the direct and injected tests above cover that boundary.
