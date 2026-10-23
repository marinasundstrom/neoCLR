# Initialize GC-backed storage once — 2026-10-23

A fresh sample of the owned-base-cache HTTP server found root publication still
dominant (1,341 of 3,603 worker self samples, about 37%). Zeroing accounted for
267 self samples (about 7.4%); this includes necessary allocator/frame work, not
just duplicate initialization. The main thread's 3,603 condition-wait samples are
not worker CPU consumption. See sample-summary.json for commands, artifact hashes
and local raw-sample location. Sampling overhead makes that run diagnostic only.

Inspection found that the native GC allocator zeroes the full payload, while the
object/array/static constructors zero it again. The new private zero_new_storage
helper keeps explicit initialization for non-GC arena reservations but is a no-op
for fresh GC allocations. It is used only after successful reservation, before
publication. Object identity, array headers, nonzero initialization flags and
ordinary mutation/copy operations still write their required values.

This covers object fields, new static tables, byte and String arrays, record
reservations, reference-array slots and scalar-array slots/reserved flags.
The allocator still clears reused blocks and padding. Non-GC hosts need not supply
pre-zeroed buffers. No allocator API, object layout, metadata schema, validation,
allocation count, GC pacing or worker ownership contract changes.

The comparison with .NET remains the allocator's zero-initialization guarantee:
constructors can rely on already zeroed storage rather than repeat the same work.
[CoreCLR GC design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
describes zeroed allocation memory; neoCLR retains its own bounded nonmoving
allocator. This does not adopt CLR allocation quanta or Go's deferred zeroing policy.

## Validation and reproduction

A new storage-initialization fixture runs under both GC and non-GC builds with
dirty backing memory, then reuses dirty/poisoned storage. It checks ten constructor
modes at lengths 0, 1, 9 and 33: object/static fields, ordinary/reserved byte and
String arrays, records, references and ordinary/reserved scalars. Zero values,
static-table empty slots and mandatory one-valued initialization flags are checked.
Existing GC tests cover rooted pressure retry and receiver/reference lifetime.

Native GC/root validation, eight forced-GC HTTP responses across two invocations,
and compiled shared-worker optimized/UBSan stress run before timed trials. Unlike
the preceding ownership/cache changes, this initialization change applies to all
native GC allocations, including the checked shared-worker path. The same consumer
contracts also pass in non-GC arena mode.

Use the same SDK and bundle as the parent experiment:

```sh
python3 scripts/validate-native-gc.py --output target/storage-zero-validation
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/storage-zero-forced
python3 benchmarks/http-server/primitive-comparison/storage-zero/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline 9b8cfac86 --output target/plain-storage-zero
```

Only text-arena.c differs between variants. Both use the same compact application
object, owned allocation cursor/base cache, 1 MiB per owner, 16 Serve lanes and
256/1,024 pacing. The ten-byte plain HTTP response is checked through EOF.
Thirty-two connections run two seconds warmup plus six seconds measured, at one/
four isolated owners, forward/reverse. CPU/response includes the full server
lifetime and all warmup/window/drain completions. No builds/tests overlap capacity
trials. Short local measurements do not establish sustained limits or peer parity.


## Results and remaining work

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 949.2, 1,005.8 | 1,009.5, 979.9 | 56.77, 55.90 |
| candidate | 1 | 1,011.8, 951.0 | 970.3, 1,009.0 | 39.89, 55.60 |
| baseline | 4 | 3,535.5, 3,255.2 | 1,096.9, 1,130.2 | 17.41, 20.97 |
| candidate | 4 | 3,433.7, 3,506.3 | 1,094.2, 1,074.7 | 16.14, 16.33 |

Mean throughput changes +0.4%/+2.2% at one/four owners; estimated CPU/response
changes -0.5%/-2.6%. One-owner differences are within the observed run variation;
four-owner results are modest and overlap individual baseline trials. Do not
interpret this as a proven universal throughput improvement. Keep the simpler
single-initialization contract and its tests, with measured limits explicit.

All native GC/root checks and the new dual-mode initialization test pass. Forced
HTTP completes eight responses over two invocations, and optimized/UBSan shared
workers each complete 32 healthy and 32 fault invocations, 24,544 graph collections
and 448 fault collections. Reports retain commands and matching source hashes.

Root publication remains the larger CPU target. Removing redundant constructor
clears does not solve repeated ancestor descriptor validation or publication
frequency. Any future change there must preserve mutable/foreign descriptor
checks or introduce an explicit image-metadata ownership contract; no validation
cache or compiler metadata ABI change is part of this slice.
