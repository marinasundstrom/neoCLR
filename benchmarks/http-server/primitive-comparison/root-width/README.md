# Rejected invalid-bit root-width experiment — 2026-10-23

The candidate is **reverted**. The active runtime retains the ordinary two-width
comparison. Boundary regression tests and the reproducible rejected.patch remain.
Validation reports describe the candidate unless explicitly named restored.

The preceding [profile](../root-thread-state/README.md) identifies ancestor root
validation as the largest sampled worker cost. The tested candidate reduces
arithmetic inside that scan while retaining all current descriptor reads.

For a uint32 width, `(width - 4u) & ~4u` is zero exactly when width is 4 or 8.
Unsigned subtraction maps those widths to 0 and 4. Masking bit two rejects all
other results, including values below four that wrap around. OR the resulting
invalid bits directly into the existing failure accumulator instead of first
normalizing two comparisons to a Boolean. Address and storage-flag checks remain
unchanged. Transient flags remain ignored by their existing contract.

Every publication still validates every ancestor storage/transient descriptor,
including previously accepted descriptors that callers mutate. No validation
cache, immutable-metadata assumption, allocation, header, ABI, compiler encoding,
thread ownership or collection-budget change is introduced. Probe-only builds
retain the existing validation and volatile observation. The existing
[CLR comparison](../../serve/root-validation/README.md) remains applicable: this
is a portable-C tuning of neoCLR's checks, not adoption of CLR stack maps.

Optimized Apple ARM64 assembly contains 407 static instructions in the publication
function versus 433 before the change. Both remain vectorized. Static instruction
counts are not dynamic CPU counts or portable performance guarantees; the retained
assembly-comparison.json records commands and hashes for this compiler/target.

## Validation and reproduction

New fixtures warm a valid 33-slot ancestor and then mutate its width at vector
position 15 or tail position 32 before publishing a child in another context.
They cover 0, 1, 3, 4, 5, 7, 8, 9, 12, 16, 2^31, 2^32-4 and 2^32-1. Exactly 4 and
8 succeed; all others abort before collection/observation. Cases run in both
probe-only and native GC builds, with and without sanitizer instrumentation so
optimized vector code is directly exercised. Existing address/flags, mutable-plan,
fault/context, batch-length and park/resume tests also pass.

The native GC suite, eight forced-GC HTTP responses over two invocations, and
compiled shared-worker optimized/UBSan stress pass. Each worker build completes
32 healthy and 32 fault invocations, with 24,544 graph collections and 448 fault
collections. UBSan does not establish race freedom. Reports retain exact commands
and the source hashes tested for the candidate.

To reproduce the candidate comparison, apply the retained patch to the baseline
implementation before running these commands (in a separate checkout if needed).
The unchanged runtime and new tests can be checked without applying it.

```sh
git apply benchmarks/http-server/primitive-comparison/root-width/rejected.patch
python3 scripts/validate-native-gc.py --output target/root-width-check
python3 benchmarks/http-server/primitive-comparison/root-width/validate.py \
  --output target/root-width-optimized
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/root-width-forced
python3 benchmarks/http-server/primitive-comparison/root-width/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline 2b789027c --output target/plain-root-width
```

Use the same Xcode SDK and compiled bundle as the preceding experiment. Only
root-probe.c differs between timed variants. Both use the same application object,
owned allocation scope/base cache, 1 MiB per owner, 16 Serve lanes and paced GC.
One/four isolated HTTP owners serve checked ten-byte responses through EOF with
32 connections, two seconds warmup and six measured, forward/reverse. CPU per
response includes server lifetime and warmup/window/drain completions, subtracting
client CPU. Builds/tests do not overlap timed trials. These short local trials do
not qualify sustained capacity, shared-heap HTTP or parity with other servers.

## Results and disposition

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 1,105.5, 1,093.0 | 905.1, 909.2 | 33.58, 47.36 |
| baseline | 4 | 3,930.3, 3,407.2 | 997.4, 1,097.4 | 16.08, 16.71 |
| candidate | 1 | 1,131.7, 989.3 | 887.4, 978.9 | 31.70, 59.04 |
| candidate | 4 | 3,639.3, 3,246.7 | 1,039.7, 1,076.8 | 12.81, 30.20 |

Mean one/four-owner throughput changes -3.5%/-6.2%; estimated CPU per response
changes +2.9%/+1.0%. Four-owner baseline drift is about 13%, and candidate samples
also vary substantially. This does not establish a precise intrinsic regression,
but provides no reliable benefit for retaining less obvious arithmetic. Smaller
static code is insufficient evidence of faster execution. Revert the candidate;
keep the more readable comparison and the new boundary regression coverage.

The restored implementation passes the same 90 optimized fixture executions across
native GC and probe-only builds; see restored-validation.json. Candidate sanitizer,
forced-GC HTTP and shared-worker results remain archived as experiment evidence,
not fresh validation of a new runtime change. Only tests/tooling/documentation
remain changed. No API snapshot or website build is needed.

The next substantive opportunity is avoiding repeated interpretation/validation
of compiler-owned metadata through an explicit registration and lifetime contract.
Mutable externally supplied descriptors must retain the checked path. Such a
change would need compiler/runtime integration and collection/cross-worker tests;
it is still proposed, not implemented by this experiment.
