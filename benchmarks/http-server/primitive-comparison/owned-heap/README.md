# Explicitly owned heap allocation cursor — 2026-10-23

This follows the [Java/Go/CoreCLR comparison](../../../../docs/design/allocation-metadata-runtime-comparison.md).
It implements a bounded ownership contract and maintained allocation cursor,
rather than reducing descriptor width or caching validation of mutable host data.

## Implemented contract

The private native APIs neoclr_gc_heap_acquire_v1 / neoclr_gc_heap_release_v1 lend
an empty arena exclusively to the runtime on one native thread. One scope may be
active per thread; nested acquisition and acquisition of a nonempty arena fail.
The arena address, buffer, capacity and cursor remain bound to the scope. Mutating
those fields is unsupported and shape mismatches are rejected. Only runtime
allocation/collection may write allocation descriptors. Native helpers can access
valid payloads but cannot overwrite headers. There is no cross-thread handoff or
shared-heap allocation support in this scope.

This is an explicit trusted-native contract, not a memory-protection boundary.
It cannot detect every illicit header overwrite at the next allocation. It is
therefore opt-in, not automatically enabled for arbitrary caller arenas. Outside
the scope, the allocator still validates the full heap before mutation, including
corrupt trailing headers after a fitting hole. Root admission remains fully checked in this initial slice; the later
[owned-base cache](../owned-bases/README.md) adds bounded positive proofs.

Within the scope, a maintained cursor skips the prefix before the earliest
potential free block. First-fit search stops at a fit; if no holes remain, append
needs no descriptor walk. Smaller holes remain candidates for later requests.
Allocation maintains the cursor and sweep rebuilds it while coalescing/trimming.
The cursor can conservatively precede live blocks; this is not a complete free-list
or constant-time allocation guarantee for fragmented heaps.

GC still validates the whole heap before marking. Release validates before returning
ownership and clears thread-local scope state even on corruption, without repairing
the heap. Release does not clear host roots or waive their separate lifetime rules.
Successful collection to empty permits reuse within the same scope. Release is
required before explicit reset/free or transfer to another native thread.

There are five machine words (40 bytes on the tested 64-bit target) of thread-local
bookkeeping, no new per-object fields and no side-table allocation. The 32-byte
descriptor and managed payload/context/compiler ABIs remain unchanged.

## Integration and limits

The source HTTP benchmark host acquires/releases the scope only with
-DNEOCLR_GC_OWNED_HEAP. Each isolated HTTP owner has its own native thread and arena.
The default host path is unchanged. Existing compiled shared-worker graph tests
exercise the checked allocator: **they do not qualify the owned path for a shared
heap**. No user-facing API, scheduler or Raven compiler change is involved.
This is an opt-in native integration, not a default deployment configuration.

The approach borrows the explicit local ownership principle from CLR allocation
contexts and HotSpot TLABs. It does not implement their region allocator or moving
collector. Go's size-class bitmap approach remains an alternative if fragmentation
and root membership scans warrant a more substantial index.

## Validation and reproduction

The native suite adds owned/checked first-fit equivalence, split/hole reuse,
reclamation-to-empty and reuse, nested/wrong-scope rejection, arena shape checks,
corrupt-descriptor GC/release and reacquisition checks. Existing corrupt-tail tests
continue on the checked allocator. The suite also covers tracing, lifetime,
pressure retry, dense indexing and root descriptors.

Forced-GC HTTP uses UBSan/bounds, two invocations on the same acquired arena, four
validated responses per invocation and clean shutdown. Shared-worker optimized/
UBSan regression tests retain the checked path. Finish tests before performance
measurement; use the same SDK and bundle as the parent experiment:

```sh
python3 scripts/validate-native-gc.py --output target/owned-heap-validation
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/owned-heap-forced
python3 benchmarks/http-server/primitive-comparison/owned-heap/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline a46987519 --output target/plain-owned-heap
```

Both variants use the same compact application object and current host source;
only the candidate enables owned-heap scope. Baseline GC source is taken from the
recorded parent revision. One/four isolated owners each have a 1 MiB heap and
16 Serve lanes, paced at 256 allocations/1,024 publications. The EOF-aware client
validates ten-byte HTTP responses at concurrency 32, two-second warmup and
six-second measurement, forward/reverse. CPU/response includes the full server
lifetime and all warmup/window/drain completions.

These are short local comparisons, not sustained limits, CPU-affinity tests,
cross-platform qualification or parity with reference servers. Tail latency
remains separately reported.

## Results

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| checked baseline | 1 | 763.8, 759.3 | 1,306.6, 1,322.2 | 42.56, 57.25 |
| owned candidate | 1 | 895.8, 901.5 | 1,118.2, 1,111.5 | 39.64, 48.52 |
| checked baseline | 4 | 2,696.5, 2,644.5 | 1,451.3, 1,465.3 | 14.41, 22.14 |
| owned candidate | 4 | 3,126.8, 3,051.5 | 1,236.9, 1,253.4 | 17.68, 20.84 |

Mean throughput improves 18.0% with one owner and 15.7% with four; estimated CPU
per response falls 15.2% and 14.6%. Four-owner tail latency is mixed and slightly
higher on average; this is not a uniform latency improvement. No timed trials
overlapped builds/tests. All responses and clean process exits were verified.

All 24 native GC kernel groups and the script's root-validation, profile,
owned-body/idle-drain and memory-view checks pass. Forced-GC HTTP passes eight
requests across two invocations with the same acquired arena. The separate
checked-path shared-worker regression passes optimized and UBSan builds: each
completes 32 healthy and 32 fault invocations, 24,544 graph collections and 448
fault collections. JSON files retain commands and input hashes.

Keep the integration opt-in. A full free-space index, exact-base membership index,
shared-heap ownership transfer and type-driven size/tracing descriptors remain
separate work. At this baseline, root admission still performs a complete heap walk. The later
[base-cache follow-up](../owned-bases/README.md) avoids walks on owned-scope hits,
adding 512 bytes of TLS bookkeeping on the tested 64-bit target.
