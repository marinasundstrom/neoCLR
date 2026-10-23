# Fault validation during root publication — 2026-10-23

This bounded experiment targets repeated work in root publication, the largest
worker self-sample category in the preceding [profile](../storage-zero/README.md).
In native GC builds, fault validation reads the current context and active fault
frame count directly. It does not construct the temporary root-descriptor array
used by probe-only observation. Collection still enumerates all fault roots.

Within one synchronous publication, consecutive frames with the same context share
one fault-shape check. No guest callback or collection occurs during that scan.
The last-context value is a local variable, discarded before the next publication;
there is no validation cache across calls. NULL contexts always fail, including
the first frame. Each distinct consecutive context is checked, and all ancestor
storage/transient descriptors are still validated. Probe-only observation is
unchanged. No header, allocation, ABI, compiler metadata or GC pacing change.

This follows the existing [CLR/Java/Go comparison](../../../../docs/design/allocation-metadata-runtime-comparison.md):
keep execution-time metadata work limited to what the operation requires. It is a
neoCLR implementation choice, not a claim that those runtimes perform this specific
check. Immutable image-owned root descriptors remain a separate proposal.

## Validation and reproduction

The root fixture now covers mutation after a successful publication through nested
frames sharing one context, NULL current/ancestor contexts, inactive fault fields,
and the largest valid active fault. Fault enumeration still returns 65 correctly
addressed roots and rejects insufficient capacity. Existing fixtures cover faults
in a different ancestor context and mutation of storage/transient descriptors.
All run in probe-only and native GC modes with sanitizer checks.

Native GC validation, forced-GC HTTP (eight responses over two invocations), and
compiled shared-worker optimized/UBSan stress pass before capacity trials. The
worker stress covers both healthy graph collection and isolated invocation faults;
UBSan does not establish race freedom. Retained JSON reports include source hashes
and commands.

```sh
python3 scripts/validate-native-gc.py --output target/fault-validation-check
python3 benchmarks/http-server/primitive-comparison/owned-heap/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/fault-validation-forced
python3 benchmarks/http-server/primitive-comparison/fault-validation/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --object target/plain-slot-metadata/candidate.o \
  --baseline 8da3619b2 --output target/plain-fault-validation
```

Use the same Xcode SDK and compiled bundle as the preceding experiment. Only
root-probe.c differs between timed variants. Both use the same application object,
owned allocation scope/base cache, one MiB per owner, 16 Serve lanes and paced GC.
One/four isolated owners serve checked ten-byte plain HTTP responses through EOF,
with 32 connections, two seconds warmup and six measured, forward/reverse. CPU per
response includes server lifetime and warmup/window/drain completions, subtracting
client CPU. Builds and tests do not overlap timed trials. This does not measure
shared-heap HTTP or establish parity with other servers.

## Results and limits

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 946.2, 1,043.5 | 1,023.8, 953.9 | 50.80, 48.61 |
| baseline | 4 | 3,197.3, 3,546.5 | 1,133.3, 1,074.7 | 29.37, 12.87 |
| candidate | 1 | 1,057.8, 1,057.5 | 952.4, 943.7 | 33.21, 47.50 |
| candidate | 4 | 3,705.2, 3,758.3 | 1,042.1, 1,039.1 | 12.79, 15.59 |

Mean throughput rises 6.3%/10.7% at one/four owners and estimated CPU per response
falls 4.1%/5.7%. Both candidate runs exceed both corresponding baseline runs, but
the baseline itself drifts about 10% between trials. The one-owner reverse pair
is especially close; these short local trials cannot establish the precise gain.
Tail latency remains variable. Retain the smaller validation operation and its
regression coverage without claiming a universal or sustained capacity increase.

All validation reports pass with matching input hashes. Shared workers complete
32 healthy and 32 fault invocations in each optimized/UBSan build, retaining graph
and fault roots through collection. This change does not reduce allocation counts
or make storage descriptors immutable. Repeated ancestor storage validation and
publication frequency remain larger follow-up questions; safely removing those
checks requires an explicit metadata ownership/lifetime contract.
