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
