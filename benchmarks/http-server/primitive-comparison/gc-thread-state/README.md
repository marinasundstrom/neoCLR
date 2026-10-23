# Rejected GC thread-state tuning and renewed peer comparison — 2026-10-23

The grouping candidate is **reverted**: this run did not establish a reliable
benefit. The active runtime remains at the generated-owned-storage implementation.
The new lifetime tests remain; candidate evidence and rejected.patch are archived.

The author reiterates that HttpServer should reach comparable request throughput
to modern frameworks. Improvements relative to neoCLR alone do not establish
that result. This slice checks a bounded runtime cost reduction, then refreshes
the same-host Go/Node/Kestrel comparison to expose the remaining gap.

Profiling the generated-owned-storage server shows allocation at 545 of 3,414
worker self samples (16%), root publication at 427 (12.5%), and TLS resolution at
412 (12.1%). Main-thread condition waits are excluded from worker attribution.
Sampling is diagnostic and perturbs execution; see sample-summary.json.

The candidate groups ownership/cache state, cumulative statistics, pacing and the cleanup guard
in one private thread-local object. Preserve each component's lifetime: releasing
ownership clears only its nested owner state; statistics and pacing persist.
Host roots, resource lists, worker hooks and optional profiling stay separate.
No allocation policy, collection budget, frame/header, managed API or compiler
contract changes. The existing CLR/Java/Go thread-owned metadata research remains
applicable; this is a C-runtime layout choice, not a new threading model.

Optimized Apple ARM64 TLS address sites change from 2 to 1 in allocation, 4 to 1
in the paced boundary, and 2 to 1 in idle collection. This is target/compiler
assembly evidence, not a guarantee for every platform. Grouped state occupies
624 bytes on this 64-bit target versus fields totaling 620 bytes before placement/
alignment (four padding bytes); no memory-footprint reduction is claimed.

New tests verify ownership release preserves all cumulative statistics and an
unrelated ownership scope does not reset a context's publication budget. Existing
resource-cleanup reentry, heap cache/reset, pacing/pressure and root checks pass
under native GC sanitizers. Forced-GC HTTP completes eight responses over two
invocations. Optimized/UBSan shared workers each complete 32 healthy and 32 fault
invocations, 24,544 graph collections and 448 fault collections. UBSan does not
establish race freedom. Only a comment-path typo changed during the GC suite;
validation-source-notes.json records that hash difference explicitly.

## Paired measurement (candidate reverted)

Both variants use the same generated-owned application object, heap ownership/base
cache, 1 MiB per owner, 16 Serve lanes and paced GC. Only native-gc.c differs.
One/four isolated owners serve checked ten-byte HTTP responses through EOF at
32 connections, two seconds warmup and six measured, forward/reverse. CPU/response
includes server lifetime and warmup/window/drain completions, subtracting client
CPU. Builds/tests do not overlap timed trials.

To reproduce the candidate, apply rejected.patch before running the paired
measurement. The final runtime has no grouping change.

```sh
python3 scripts/validate-native-gc.py --output target/gc-thread-state-check
python3 benchmarks/http-server/primitive-comparison/gc-thread-state/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-owned-root/candidate.o --baseline a67aa5a0a \
  --output target/plain-gc-thread-state
```

Use the same Xcode SDK and compiled bundle as the preceding experiment. The native
change needs no Raven compiler change, public API snapshot or website build.

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 1,660.7, 1,375.2 | 606.8, 669.4 | 21.43, 53.28 |
| baseline | 4 | 5,060.5, 3,623.0 | 738.0, 908.9 | 7.63, 22.93 |
| candidate | 1 | 1,709.3, 822.0 | 587.9, 759.9 | 21.77, 197.63 |
| candidate | 4 | 4,903.5, 4,697.2 | 739.0, 769.6 | 11.84, 20.93 |

The first one-owner pair modestly favors grouping, but later samples deteriorate
sharply: candidate one-owner throughput falls by over half, with p99 near 198 ms;
the final baseline trials also slow substantially. Four-owner baseline throughput
falls about 28% between trials. This run cannot distinguish a small implementation
effect from desktop scheduling/load variation; no positive performance claim is
supported. Revert the change rather than choose a favorable subset of samples.

The final restored runtime passes the new ownership-counter and pacing-lifetime
cases under the same sanitizer flags (restored-validation.json). The larger
candidate GC/forced-HTTP/shared-worker reports are experiment evidence, not new
runtime behavior. The peer refresh uses the restored established runtime, not the
rejected candidate; see [current peer comparison](../peer-refresh/README.md).
