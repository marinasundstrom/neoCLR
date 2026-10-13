# HTTP collector investigation — 2026-10-13

Author-directed follow-up to the Preview 14 baselines: profile plain text and object
JSON, reduce their shared cost, retain GC correctness and rerun matched measurements.

`profile.py` relinks the exact previously measured AOT guest object with the existing
opt-in GC counters, a diagnostic exit reporter and macOS stack sampling. Every one
of the 32 HTTP responses is validated by the benchmark controller. Whole-process
counters include startup, warmup and teardown; sampled/instrumented durations are
not benchmark scores. Sampling includes loader activity and is not a CPU percentage.

| Case | Collection attempts | Allocations | GC time | Peak live payload | Peak arena extent |
| --- | ---: | ---: | ---: | ---: | ---: |
| Plain text | 201,546 | 17,908 | 340 ms | 8,119 bytes | 13,584 bytes |
| Object JSON | 296,391 | 40,214 | 611 ms | 12,542 bytes | 19,640 bytes |

Both processes reclaim every recorded allocation and report zero failed collections.
Native stack samples also identify collector work as the largest named application
leaf: 104 plain-text and 235 JSON samples in `neoclr_gc_collect_conditional_v1`,
with root traversal also visible. Raw stack-file hashes and exact link inputs are
retained in [plain](profile-plain-20261013.json) and [JSON](profile-json-20261013.json)
reports. Local stack captures remain in the recorded target directories.

This supports optimizing shared collection work first. It does not establish the
same fraction in interpreter mode or isolate serialization costs. The current native
collector performs stress collection at every root boundary. Reducing frequency
requires preserving allocation-failure behavior and complete roots; it is not a safe
substitute for this investigation. Explicit collection, poisoning, boundary frequency
and dependent-owner semantics should remain intact for the initial optimization.

Reproduce on macOS ARM64 with a passing HTTP build record:

```sh
python3 benchmarks/http-server/profile.py \
  --build target/http-json-preview14-20261013/native/build.json \
  --workload json --output target/http-json-profile-new
```

The profiler is provisional development tooling. It does not change installed SDKs,
public APIs or published benchmark history.

## Matched sweep/coalescing result

The collector now sweeps and merges adjacent free spans in one forward pass. Its
rooting, collection boundaries, failure behavior, poisoning and allocation limits
are unchanged. Baseline and candidate hosts were freshly linked from the same
sources with only `native-gc.c` substituted; each AOT pair uses the identical object,
and JIT pairs use the same assembly and released compiler. Instrumentation and
sampling were disabled for the comparison. No concurrent builds were launched.

Five alternating rounds plus one warmup process for each variant/case/mode validate
**1,536 responses across 48 processes**, with 960 requests retained for timing.

| Workload/mode | Baseline requests/s | Candidate requests/s | Observed increase |
| --- | ---: | ---: | ---: |
| Plain text / JIT | 61.27 | 65.68 | 7.2% |
| Plain text / AOT | 63.83 | 68.84 | 7.9% |
| Object JSON / JIT | 33.58 | 36.12 | 7.5% |
| Object JSON / AOT | 35.04 | 37.80 | 7.9% |

These are medians within this comparison, not ratios against earlier published runs.
All four candidate ranges sit above their corresponding baseline ranges. This is
useful local evidence, not cross-host statistical qualification. GC remains a major
cost; this does not close the overall runtime performance gap.

[Raw rounds and inputs](gc-sweep-comparison-20261013.json) retain all request timings,
startup and ranges. [GC validation](gc-sweep-validation-20261013.json) includes the
new mixed-hole test and retained suspension checks. Post-change diagnostics for
[plain text](profile-plain-sweep-20261013.json) and
[JSON](profile-json-sweep-20261013.json) reclaim all 17,908/40,214 allocations,
with identical allocated payload and peak live/arena bytes. Collection attempts
are 201,545/296,393 versus 201,546/296,391 before: scheduling/poll boundaries vary,
but collection policy is unchanged. Instrumented elapsed GC times are noisy (the
plain-text profile increased); performance claims use the uninstrumented comparison.
[Baseline and candidate native stack captures](profiles/) are retained separately.

Reproduce the matched comparison using a passing bundle preparation record:

```sh
python3 benchmarks/http-server/compare-gc.py \
  --plain benchmarks/http-server/results-20261013.json \
  --json benchmarks/http-server/json-results-20261013.json \
  --host target/release-callback-bundle-20261013/preparation.json \
  --baseline c1d16de6 --output target/http-gc-comparison-new
```

The interpreter collector is unchanged and was not rerun. Windows was not requalified.
The installed Preview 14 SDK and public baseline pages remain unchanged; this result
belongs to development history. Next investigate root-boundary traversal and safe
allocation-pressure scheduling, preserving explicit collection and stress coverage.

## Scheduler diagnostics (2026-10-13)

The [separate scheduling probe](../http-scheduling/README.md) retains native poll,
pause, CPU and GC counters for plain/JSON responses and a concurrent slow-peer test.
It measures compiler-state-machine AOT guests, not the runtime-suspension carrier;
results do not establish a JIT/interpreter or throughput comparison.
