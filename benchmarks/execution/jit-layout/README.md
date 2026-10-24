# JIT placement and symbolized profiling — 2026-10-24

A temporary compiler experiment selects Cranelift 0.121.2's existing
`ArenaMemoryProvider` for managed JIT, reserving 64 MiB. All compiler flags,
long-range calls, native helpers, managed heap policy and synchronous code ownership
remain unchanged. Production source and compiler binary were restored. The fixed
reservation is an experimental bound, not a new supported runtime setting.

## Capacity

Four unprofiled invocations run system/arena/arena/system. Each uses one exclusively
owned 1 MiB paced managed heap, one-second warmup, five-second load, 32 connections,
client GOMAXPROCS=2, and checked ten-byte plaintext responses over new connections.

| JIT memory provider | Mean requests/s | Range | CPU µs/response |
| --- | ---: | ---: | ---: |
| System | 1,829.1 | 1,825.0–1,833.2 | 544.5 |
| Arena | 2,183.2 | 2,179.0–2,187.4 | 457.1 |

Mean throughput increases 19.4%, while server CPU per response falls 16.1%.
CPU uses `ps time` after READY through warmup and measurement, divided by all
completed/drained responses, at approximately 10 ms resolution. Compilation and
teardown are excluded. No sampling or disassembly overlaps these trials.
This short macOS ARM64 cohort is promising, not a confidence interval or peer-server
comparison. It does not measure the optimized worker host or any managed-heap change.
The previous AOT cohort is not a simultaneous control for this experiment.

The provider changes placement of both code and JIT static data. These results
support testing that combined placement policy; they do not isolate instruction
cache misses, TLB misses or data-cache locality as the cause.

## Separate profiles

Two additional invocations enable Cranelift's existing Unix `PERF_BUILDID_DIR`
function-map output and take three-second Apple `sample` captures at 1 ms after
warmup. Profiling capacity is intentionally excluded from the table above.
Both live runs and their checked responses pass.

| Function map | System | Arena |
| --- | ---: | ---: |
| Functions | 1,138 | 1,138 |
| Code bytes | 6,971,328 | 6,971,328 |
| First-to-last function span | 664,253,636 | 7,135,300 |
| 16 KiB pages intersecting code | 495 | 426 |

The span is an address interval, **not resident or committed memory**. The maps
cover code, not all static-data allocations. Equal sizes do not prove byte identity:
relocation values necessarily depend on placement. Long-range lowering is retained.

`symbolize.py` joins only addresses inside exact half-open function ranges from the
same invocation; it never guesses across gaps. It resolves 1,896/2,410 unknown frame
occurrences in system/arena captures, leaving two/zero unresolved. These counts are
text occurrences, not CPU samples. `functions.json.gz` retains the matching linked
inspection names and indices; encoded bridge names remain explicit. The inspection
uses the same closed-world application/load set as the byte-verified AOT baseline.

Self-sample aggregation subtracts only immediate child counts. It identifies
`HttpRequestDecoder.PushCore` (80/63 self samples), header-buffer indexing (34/44),
and other parser/collection methods among managed frames. Native root walking,
TLS access, `collect_indexed` and `validated_bases` remain substantial. Samples are
wall-clock observations, not call counts or normalized CPU/request. Raw counts
between profiles cannot establish an individual helper's speedup.

## Qualification still needed

The existing `CodeOwner` frees JIT memory after synchronous execution and on partial
compilation failure. The arena provider handles permissions/cache coherence through
Cranelift. However, a fixed 64 MiB bound can reject larger images; reserve exhaustion,
partial-build cleanup, repeated invocations, faults, representative JSON/async
consumers and platform-specific reservation behavior need focused qualification
before adopting a production policy. Cranelift notes that Windows reservations
also commit memory. Neither Windows nor arbitrary FFI is qualified here.

The experiment uses an existing provider rather than introducing a new allocator.
Its source is in the pinned dependency's `src/memory/{system,arena}.rs`; function-map
support is in `src/backend.rs::record_function_for_perf`. The system provider obtains
another page-rounded allocation when the current allocation cannot fit a request.
The arena reserves one range and extends compatible unfinalized segments within it.
This is executable/static-data placement, independent of moving managed objects.

## Reproduction and retained evidence

Apply `experiment.patch`, build a separate release compiler and save it as
`target/jit-layout/experiment-codegen`; restore production source/binary. Run:

```sh
python3 benchmarks/execution/jit-layout/measure.py --output target/jit-layout/capacity
python3 benchmarks/execution/jit-layout/measure.py --profile --output target/jit-layout/profile
python3 benchmarks/execution/jit-layout/symbolize.py target/jit-layout/profile target/root-map/codegen.json
python3 -m unittest discover -s benchmarks/execution/jit-layout -p 'test_*.py'
```

The driver refuses existing directories. It depends on retained artifacts from
`call-layout`, including the earlier owned helper dylib. Reports retain commands,
input hashes, client results and guest exit codes. `capacity.json`, `profile.json`,
`summary.json`, `symbols.json`, compressed maps/captures/symbolized captures and the
linked function-name table preserve evidence. Decompress maps/captures together to
rerun symbolization, using `functions.json.gz` as the inspection argument.
Ten tests cover ranges/gaps, malformed maps, self-count arithmetic, capacity
arithmetic and rejection of incomplete/failed/profiled cohorts.
