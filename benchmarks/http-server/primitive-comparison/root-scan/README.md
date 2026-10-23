# Batched native root validation — 2026-10-23

Continuing the author's plain-request throughput/CPU priority, this slice batches
native GC root-descriptor validation into an accumulated failure check. Every
publication still checks every current storage/transient descriptor on every
ancestor, including ancestors belonging to a different heap. Fault validation,
mutable plan checks, root publication, collection policy and worker hooks remain
unchanged. There is no validation-result cache or assumption of immutable metadata.

The native scan reads descriptor metadata, never the described slot values. It
aborts before collection if any address, width or applicable flags are invalid.
Probe-only builds retain their validation followed by volatile slot observation.
Transient flags retain their previous ignored meaning. No public API or compiler
encoding changes. This is implementation tuning beneath the same root/lifetime
contract; it does not implement CLR's stack-map representation or change the
existing [comparison/design reasoning](../../serve/root-validation/README.md).

## Attribution

A fresh five-second stack sample of the preceding 256/1,024 paced one-worker server
shows 964 of 3,560 worker samples in root publication (27%), 799 in allocation
(22%), 358 in strlen (10%), and 352 in heap-base validation (10%). The main thread's
3,560 condition-wait samples are waiting for its worker; including them as HTTP
worker CPU would be misleading. These are sampled attribution estimates, not exact
CPU accounting. [sample-summary.json](sample-summary.json) records commands, input
hashes and response validation; the original full sample remains under target.

Clang's optimized ARM64 output uses vector loads/comparisons/reductions for the
accumulated scan, with scalar handling for short lists/remainders. The code remains
portable C without architecture intrinsics. Faster validation does not eliminate
roughly 3,678 publications per plain response or the substantial allocation and
heap validation costs; the reference-server gap remains large.

## Validation and measurement

The expanded root contract fixture warms valid descriptors, then mutates an
ancestor's address, width, flags or transient width inside a batch and at its tail.
Valid lengths zero through 33 include mixed 4/8-byte widths and all admitted flags.
Existing fault, mutable-plan, invalid-storage and cross-context cases remain.
The native GC validation suite passes with UBSan/bounds; a separate optimized
build runs all 28 root cases across native GC and probe-only modes. That matters
because sanitizer instrumentation may change vectorization.

The final comparison uses the same generated application and client as the
[primary baseline](../README.md), current 256/1,024 pacing in both variants, and
root-probe.c from commit e93327e03 as the baseline. Only that source differs.
One/four-owner variants run in forward/reverse order, each with two seconds warmup
and six measured seconds. No profiling or test compilation overlaps these trials.
Every response is checked and all processes shut down successfully. CPU per
response uses lifetime CPU divided by warmup/window/drain completions. Tail latency
is variable and these short closed-loop trials do not qualify a sustained rate.

## Final results

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 625.2, 626.3 | 1,601.0, 1,598.7 | 53.35, 65.12 |
| baseline | 4 | 2,182.0, 2,169.8 | 1,775.5, 1,786.8 | 21.01, 26.77 |
| candidate | 1 | 652.2, 649.5 | 1,535.6, 1,540.2 | 50.83, 60.54 |
| candidate | 4 | 2,284.7, 2,247.7 | 1,695.4, 1,705.1 | 21.84, 24.20 |

Mean throughput rises about 4% and estimated CPU per response falls about 4%
with both one and four owners. These small local improvements need broader
qualification before extrapolating to other CPUs or workloads.

## Reproduce

Use the retained matching build/client from the parent README. Source hashes are
checked; only the explicitly recorded collector/root-probe differences from the
original build are allowed. Select the macOS 26.2 SDK via DEVELOPER_DIR/SDKROOT on
this machine. Fresh output directories are required.

```sh
python3 scripts/validate-native-gc.py --output target/plain-root-gc-validation
python3 benchmarks/http-server/primitive-comparison/root-scan/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client --baseline e93327e03 \
  --output target/plain-root-scan-final
python3 benchmarks/http-server/primitive-comparison/root-scan/validate.py \
  --output target/plain-root-optimized-validation
```

For independent sampling, run the one-worker baseline, drive its READY port with
the compiled client for eight seconds, run `sample PID 5 1 -file sample.txt`, then
close through its READY control port and check DONE/exit status. Sampling results
are separate diagnostics, never substituted for uninstrumented capacity results.
The private implementation change needs no API snapshot or website build; existing
feature descriptions remain accurate. Windows performance remains unmeasured.
