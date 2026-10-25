# Begin, capture and marker CPU

2026-10-25. Follow-up to [owner-pause attribution](stable-byref-pause-results.md).
Diagnostic-only --profile-begin measures owner begin CPU/wall and its nested snapshot
capture/dispatch. It reports existing marker CPU transferred after job completion.
There are no per-object clocks, new marker synchronization or active-job reads.
Normal generated collectors are unchanged.

Three trials each of plaintext/JSON (1,024 requests plus 16 warmups, four clients)
validate **6,240 responses**. Tasks/collections/arrays each run three times at 4,096
iterations, giving nine checked standalone executions. All use the immutable-byref
candidate and matching native host. The rebuilt fixed-work plaintext control is
byte-identical to its retained baseline. These are diagnostic, not throughput runs.

| Workload | Median begin CPU ms | Nested capture CPU ms | Marker CPU ms | Jobs/trial |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 11.708 | 6.708 | 17.848 | 484–488 |
| JSON | 24.038 | 13.121 | 31.073 | 950–954 |
| Tasks | 1.180 | 0.749 | 0.087 | 129 |
| Collections | 5.196 | 3.210 | 0.815 | 465 |
| Arrays | 1.665 | 1.045 | 0.124 | 169 |

Capture includes allocation, header/offset copying, root-bit and conditional-edge
capture, worker initialization when needed and dispatch. It is nested within begin;
never add both columns. Begin's remainder includes preparation, root seeding and
instrumentation. Marker CPU covers scan/gate/drain work, excludes idle waiting and
includes cancelled jobs after join. These are not whole-program CPU attribution or
pause distributions.

Median cumulative copied header bytes are 15.37 MB plaintext / 32.79 MB JSON per run;
marker payload reads are 9.50 MB / 19.74 MB. These are traffic, not native allocated
bytes or retained memory. Standalone payload reads are approximately 0.04/0.29/0.05 MB.
The candidate copies headers, not the entire heap payload.

The parser checks unique rows, successful begin/capture counts equal to begun epochs,
valid joined jobs matching completed plus cancelled epochs, no active/invalid jobs
and nonnegative timing remainders. Fake-clock UB/bounds tests cover nesting, argument/
result preservation and failures. Negative controls reject missing/duplicate rows,
impossible nested CPU and missing jobs. Existing owner-profiler tests pass. An initial
test macro shadowed its fake-clock variables; renaming those test variables corrected
compilation before any workload measurement. No runtime fix is implied.

Evidence: stable-byref-evidence/{plain-begin,json-begin,ordinary-begin}-*,
plain-fixed-build-*, begin-summary.json.gz and begin-tests.json. Use the existing
measurement drivers with --profile-owner-pauses --profile-begin and fresh output
directories; summarize-begin-phases.py validates the reports.

The author then requested an [industry-GC comparison](industry-gc-comparison.md).
Its proposed direction uses these measurements without changing collector defaults,
public APIs, compiler contracts or ownership boundaries.
