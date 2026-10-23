# Compact allocation descriptors — 2026-10-23

The author requested minimal object headers and avoiding unnecessary metadata
loading/resolution. The tested candidate used a 24-byte private native GC descriptor instead
of 32. **It was reverted after a throughput/CPU regression.** Active runtime
descriptors remain 32 bytes; the full candidate and fixtures are in rejected.patch. Its three words were aligned span plus kind, exact payload byte count, and
mark/worklist state. Allocation kinds 1–6 fit in the span's three unused alignment
bits. Sizes remain full-width; no smaller maximum allocation or global cache is
introduced. The candidate saved eight descriptor bytes per allocation before fragmentation.

The ordinary managed object's eight-byte identity word is unchanged, including the
static-storage initialization guard in its high word. This is a private allocator
layout change, not a compiler, managed API or serialized-heap format change.
Existing allocation addresses do not move during their lifetime. Old private
descriptors cannot be mixed into a running new collector; hosts must rebuild and
start with fresh arenas.

## CLR comparison and tradeoffs

[CoreCLR object.h](https://github.com/dotnet/runtime/blob/main/src/coreclr/vm/object.h)
describes objects using a MethodTable pointer, a preceding synchronization header,
and quickly computed size from type-level constants and an optional component count.
It also masks GC flags from aligned MethodTable pointers. The relevant principles
are compact per-object state, cheap size access and shared type-level information.
neoCLR uses alignment bits for allocation kind, but does not claim CLR's object
layout, typed tracing or allocator behavior.

Deriving all sizes/tracing from shared type descriptors could reduce overhead
further, but this collector also owns raw text/buffers, free extents and exact
interior-pointer bounds. Removing those fields requires a separate allocation and
type-metadata design. State cannot simply become a byte: it temporarily holds an
intrusive GC worklist pointer. This slice avoids introducing a metadata lookup,
side table, external worklist allocation or narrower size contract. The cost is
masking on span/kind access, which must be measured rather than assumed free.

## Validation and reproduction

The candidate passed the native GC suite, covering corrupt trailing headers, first-fit/split/coalescing,
interior roots and padding, dense indexes, all six kind tags, exact unaligned sizes,
invalid packed tags with unchanged heap/output, and rooted exhaustion/retry.
Exhaustion fixtures were resized to preserve their intended pressure and failure
paths after header savings. The compiled shared-worker test and forced-GC HTTP
consumer exercise collection with real emitted roots and objects.

Use the matching SDK and bundle from the parent experiment. Finish tests/builds
before capacity measurement. Apply rejected.patch to baseline 502cf8745 in an
isolated checkout to reproduce the candidate; reverse the patch afterwards:

```sh
python3 scripts/validate-native-gc.py --output target/compact-header-validation
python3 benchmarks/http-server/primitive-comparison/compact-header/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline 502cf8745 --output target/plain-compact-header
python3 benchmarks/http-server/primitive-comparison/compact-header/profile.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --experiment target/plain-compact-header
```

Both variants use the same compact application object, 1 MiB heap and 16 Serve
lanes per isolated owner, 256-allocation/1,024-publication pacing, ten-byte plain
response and EOF-aware client. Thirty-two connections run two seconds warmup plus
six seconds measured, forward/reverse at one/four owners. CPU/response includes
startup, warmup, drain and shutdown. These short local comparisons do not establish
sustained capacity, production latency, peer-server parity or Windows qualification.

Separate instrumented main-owner runs report peak arena extent; thread-local
statistics cannot represent the worker processes from an atexit handler. These
diagnostics are not throughput measurements or whole-process RSS.


## Results and decision

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 764.8, 768.0 | 1,307.2, 1,302.0 | 47.53, 49.45 |
| candidate | 1 | 751.0, 745.7 | 1,333.0, 1,343.1 | 44.84, 55.23 |
| baseline | 4 | 2,608.0, 2,664.8 | 1,472.2, 1,454.3 | 19.95, 20.05 |
| candidate | 4 | 2,533.8, 2,469.8 | 1,516.1, 1,554.0 | 20.33, 21.96 |

Mean throughput fell 2.4%/5.1% at one/four owners; CPU per response rose
2.6%/4.9%. Main-owner instrumentation measured peak arena extent falling
82,224 → 77,048 bytes (6.3%), with identical peak logical live payload of
49,657 bytes. Different request totals mean cumulative allocated bytes must not
be compared as per-request savings. No profile failures; all allocations reclaimed.

The candidate passed all 23 native GC kernel groups plus root validation,
profile, owned-body/idle-drain and memory-view checks in the validation script.
Forced-GC HTTP completed four requests with clean shutdown. Optimized/UBSan
compiled workers each completed 32 healthy and 32 fault invocations, 24,544 graph
collections and 448 fault collections. These reports qualify the archived patch,
not an integrated layout change. Earlier failed runs exposed old-layout assumptions
in pressure and corruption fixtures; the archived patch includes their corrections.

The memory saving is real, but the throughput/CPU penalty conflicts with the
author's primary HTTP objective. Retain the experiment as evidence, not a default
or additional runtime mode. The result does not prove bit packing is inherently
slow: this collector repeatedly walks descriptors, so extra decoding may be
amplified. Isolating that cause would require further profiling. The next structural
candidate is maintained allocation metadata and an owned allocation fast path;
see the [Java/Go/.NET comparison](../../../../docs/design/allocation-metadata-runtime-comparison.md).
