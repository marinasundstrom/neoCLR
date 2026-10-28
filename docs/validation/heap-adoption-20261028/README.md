# Exclusive heap adoption prerequisite — 2026-10-28

Private `NEOCLR_EXCLUSIVE_HEAP_ADOPTION`, macOS ARM64. No generated-worker,
HTTP, JIT, or default scheduling integration. This is a checked sequential lease
for an existing native arena, not simultaneous mutator participation.

## Contract and checks

The caller must prove exclusive access and previous-owner release/marker join.
Adoption rejects current TLS heap ownership, active/parked frames, host roots,
resources and malformed allocation spans. It validates the whole arena before
mutation, creates fresh index storage, then reconstructs the free list, exact-base
cache and allocation-start index. Index allocation failure leaves the live arena
unchanged. Pacing context is invalidated; no statistics or unrelated TLS state is
transferred. Release must occur on the adopting thread while the invocation and
roots are still alive, before another owner touches the heap. Concurrent release
cancels/joins its marker using the existing release contract.

`qualified.json.gz`: sync/concurrent optimized and UBSan pass malformed leading/
trailing span rejection, allocation-index failure, host-root/active-frame/parked-frame
and existing-heap rejection, free-block reuse, exact live bases, reacquisition and
eight sequential pthread handoffs per run. Concurrent worker tests explicitly
start a marker, observe it active, then release and verify settled epoch counts
before roots/context can go away. They do not perform overlapping managed writes.
The gate is supplied by pthread join in this fixture; the API cannot discover
foreign-thread roots or enforce an arbitrary caller's exclusivity promise.

Default native GC assembly is byte-identical to HEAD before this change at `-O2`.
The opt-in implementation is included at the end of the native collector, sharing
its descriptor checks and private ownership/index layout. No existing API behavior
or ordinary host changes. Reproduce with:

    SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk python3 docs/experiments/worker-delivery/validate-adoption.py --output target/adoption-check

## Cost screen, not application throughput

Five alternating sync/concurrent trials, 128 KiB arena, 100 warmup and 2,000 timed
adopt/release pairs per size/trial; 8-byte atomic payloads, no collection during the
timed loops. Table reports medians of per-trial mean microseconds per pair. Compiler `-O2`,
collector policies and commands are retained. CPU is process `clock()`; these are
short microbenchmarks without independent-process contention controls.

| Existing blocks | Sync wall / CPU µs | Concurrent wall / CPU µs |
| --- | ---: | ---: |
| 0 | 0.039 / 0.040 | 0.108 / 0.109 |
| 128 | 0.964 / 0.964 | 1.035 / 1.036 |
| 1,024 | 7.407 / 7.395 | 7.608 / 7.584 |

The implementation validates/scans descriptors and rebuilds per-thread state;
this screen makes that cost visible. No collector speedup, HTTP gain or cross-thread
handoff latency is measured. A future hot path should consider a persistent heap
control block and transferable ownership metadata instead of rebuilding on every
callback; that is a design candidate, not an implemented optimization.

## Generated-worker admission remains closed

The temporary worker probe adopted the arena, but its 8 KiB heap still triggered
pressure collections before a background epoch could start; its expected-positive
epoch assertion failed. Code review then identified a more important gate:
`runtime/codegen/src/values.rs` rejects `native_host_bootstrap` with experimental
live stores. The ordinary generated callback image has no qualified concurrent
store-barrier profile. Heap registration alone would be unsafe if it did start
marking alongside those writes. That worker integration was withdrawn; only the
native primitive is committed. No positive concurrent-worker result is claimed.

The previous finding remains valid: an unregistered borrowed arena cannot start
owned epochs. Registration is necessary but not sufficient; pressure/pacing and
compiler write coverage also matter. Next qualify a restricted generated callback
profile with live-store barriers, parked pool-root coverage, static initialization,
fault cleanup and native-service admission before using this lease in the worker
bridge. Then address the HTTP owner's existing roots/resources and measure owner
blocking; the HTTP integration/new-benchmark gate is unchanged.

## Retained investigation and limitations

`initial-contract` passed the initial native handoff checks. `invalid-fixture` aborted
because the newly added root-frame test supplied NULL storage, which the root ABI
rejects even with zero slots; the final fixture supplies a valid slot. The
`withdrawn-worker-probe` retains the failed epoch assertion and input hashes;
intermediate source revisions are not separately archived. `default-assembly`
records matching assembly hashes/commands. `index.json` hashes decompressed files.
TSan remains unavailable on this host as documented in the earlier delivery slice.
No concurrent mutator, arbitrary heap migration, managed Task.Run or production
performance qualification is implied.

Website content checks: 26 tests pass; no website publication performed.
