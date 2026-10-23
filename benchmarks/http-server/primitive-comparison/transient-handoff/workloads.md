# A coherent workload profile — 2026-10-24

The author asks to compare ordinary samples and HTTP configurations of varying
complexity, including JSON, to identify common metadata and GC costs. This first
cross-workload pass uses the new transient-handoff backend, the same retained
compiler/library bundle and instrumented native host. It keeps execution units and
GC policies explicit instead of ranking unlike programs by raw totals.

## Comparable HTTP work

Both handlers use the same async `HttpServer.Serve`, control channel, one isolated
owner, paced GC, native sanitizers and sequential loopback requests. Plain text
returns ten bytes. JSON allocates a three-property Person and uses the existing
reflection-based serializer/response helper, including UTF-8 “Café team”. All
responses are checked. Explicit Person reflection roots belong to the compiled
assembly; omitted metadata is not treated as equivalent configuration.

For each handler, run two complete server invocations with four and twelve requests
each. Subtract the eight-response totals from the 24-response totals and divide by
16. This removes fixed setup/teardown work; GC budget rounding and scheduling can
still affect the result. These counter runs do not measure capacity or CPU shares.

| Work per additional response | Plain text | JSON | JSON minus plain |
|---|---:|---:|---:|
| GC boundary publications | 3,676 | 5,535 | 1,859 |
| Transient-root publications | 830 | 1,488 | 658 |
| Managed allocations | 208 | 457 | 249 |
| Allocated payload bytes | 15,995.5 | 26,173.5 | 10,178 |
| Completed collections | 4.06 | 5.94 | 1.88 |

JSON increases boundary frequency about 51%, allocations 120% and allocated payload
64% for this fixture. Publication counts show where repeated validation can grow;
they do not identify metadata-resolution cache misses or prove a CPU attribution.
JSON also changes response length, headers and object creation, so the difference
is the complete handler/response increment, not serializer-only cost.

## Ordinary samples

Reuse the current repository's task-queue and collection application samples. Their
bodies are preserved; synchronous Main receives an async unit entry wrapper because
the shared host expects the generated queue-drain export. Each runs twice with the
same one-owner paced-GC/sanitizer configuration. Results below are averages per whole
sample invocation, including setup/teardown and wrapper cost, **not per HTTP request**.

| Sample invocation | GC boundaries | Transient publications | Allocations | Payload bytes | Collections |
|---|---:|---:|---:|---:|---:|
| Task queue completion/drain | 135 | 51 | 28.5 | 925 | 4.5 |
| Maps, lists, filtering and queries | 511 | 153 | 70.5 | 2,293 | 4.5 |

Output assertions check the samples' completion and identity behavior. Fractional
averages reflect process-wide setup shared by two invocations. Equal collection
counts here are dominated by invocation lifecycle; they do not show equal GC
pressure. These programs perform different amounts of application work and should
not be compared as throughput benchmarks.

The current JSON DOM/stream/error-composition sample reaches native compilation but
is rejected for an unsupported value-member contract (`$aot_linked_55`, impl 4096)
with this compiler/bundle configuration. It is an explicit coverage gap, not a zero
measurement or proof that every JSON path is unsupported. The reflection-based JSON
HTTP path above does compile and run. The independent 40-invocation async-consumer
qualification checks retention, faults and lifecycle but is not a timing cohort.

## What the combined evidence supports

1. The transient-handoff change removes work shared by ordinary and HTTP code;
   all collection/worker boundaries remain. Plaintext capacity evidence is separate
   in [the implementation report](README.md).
2. Root publication remains frequent in plain HTTP and rises further with JSON.
   The next compiler investigation should prove which calls cannot collect, suspend,
   initialize types or rendezvous with workers before considering omitted boundaries.
3. JSON adds both objects and publication/metadata work. Allocation counts alone
   cannot select a collector architecture. Add object survival/live-byte evidence
   and metadata resolution/cache counters before attributing the difference to GC
   or choosing generational policies.
4. The sample inventory needs configuration-qualified coverage. Preserve source,
   compiler, library, reflection roots, host and GC mode per run. Extend this table
   with ordinary compute/text/JSON and HTTP bulk/header configurations after their
   matching executable paths are qualified; historical measurements with different
   hosts or revisions are not silently pooled into this cohort.

## Evidence and repeatability

`paced-plain8/24.json.gz`, `paced-json8/24.json.gz`, `corpus.json.gz` and
`work-summary.json.gz` preserve raw inputs and normalized results. `summarize.py`
checks common native source and diagnostic hashes, uses response-count differences,
and keeps invocation units and exclusions separate. The corpus report is deliberately
marked incomplete because the JSON DOM/stream sample failed admission; its two
completed cases remain valid data. `work-inputs.json` records runner/source identities.

The first attempt used stale copies of samples inside the retained bundle; current
repository sources fix the old Option/Result naming mismatch. A second attempt
exposed the synchronous-entry host mismatch and led to the documented async wrappers.
Both failed reports are retained. The initial JSON HTTP build omitted reflection
roots and produced no response; corrected rooting succeeds. A retry also hit the
compiler's output-file-exists guard before switching to a new filename. These are
preparation failures, not discarded performance trials.

Use `count.py --paced` with `--requests 4` and `--requests 12` for each generated
object; add `--json` for the JSON handler. `corpus.py --build BUILD --diagnostic
PACED_JSON_REPORT --output NEW_DIRECTORY` compiles and runs the existing samples.
Use `summarize.py --help` for the five input reports. Generated-source counters are
intentionally diagnostic-only and are not added to production hot paths.

`build-json.py --build BUILD --output NEW_DIRECTORY` reproduces the JSON handler
with the matching reflection roots before native compilation. Its retained report
is `json-reproduction.json.gz`.

The rebuilt JSON object differs bytewise; eight independently checked responses
validate reproduction (`json-reproduction-run.json.gz`). Original counter pairs
retain their recorded object identities.

## Follow-up: repeated validation work

A second diagnostic pass keeps the same objects and host configuration, adding
counters in temporary copies of root-probe.c. Four matched runs check 64 HTTP
responses in eight server invocations. The earlier boundary counts reproduce
exactly. An initial pass produces the same normalized counts; the final rerun
renames a misleading descriptor-counter label before archiving the reports.
Production runtime code and generated objects are unchanged.

| Per additional response | Plain text | JSON |
|---|---:|---:|
| Frames visited by boundary validation (including current frame) | 25,892 | 43,753 |
| Storage descriptors checked by validate_storage | 44,462 | 122,197 |
| Fault-context observations | 7,546 | 11,355 |
| Sum of plan lengths supplied to strlen validation | 429,629 | 659,629 |
| Mean frames visited per boundary | 7.04 | 7.90 |

The descriptor counter includes owned-frame entry validation and boundary scans of
ordinary storage and transient descriptors. It excludes the separate inline scan
inside transient publication. Fault observations include frame leave, not just
boundaries. String lengths describe logical validation input bytes; they are neither
unique metadata size nor measured memory traffic, and exclude marker searches.
These counters do not measure type metadata resolution, cache misses or CPU shares.

JSON raises frame visits 69% and descriptor checks 175%, versus 51% more boundaries.
This identifies repeated ancestor validation as a concrete follow-up candidate:
more complex handlers amplify work even when no collection is due. The existing
owned-storage contract already removes repeated ordinary local-table validation,
but transient descriptors remain checked at every descendant boundary.

The next bounded experiment should consider a separate compiler-owned transient
contract: prove that descriptor addresses/widths/count remain fixed between
publication and the next boundary or leave, while values remain mutable and traced.
Keep the checked path for mutable native callers, faults, nested contexts and
parked/shared-worker lifetimes. Compare CPU and throughput before retaining it.
Do not skip safepoints or cache validity by pointer based on these counts. Image-owned
typed maps remain another candidate for repeated JSON-plan checks; the previous
marker-prefix experiment did not justify a production change.

This follows the existing CLR comparison in the allocation/metadata research:
trusted execution metadata with an explicit lifetime can be validated separately
from changing root values. It does not imply CLR-compatible stack maps or claim
that generational GC would remove publication cost. Survival/live-byte data and
CPU attribution remain open.

`profile-count.py` generates diagnostic sources with checked insertion sites;
its options match count.py. `profile-summary.py` accepts plain small/large then
JSON small/large reports, verifies matched objects/runtime/diagnostic identities,
and subtracts startup work. Four focused aggregation tests pass. Raw reports are
`profile-{plain,json}{4,12}.json.gz`; `profile-summary.json` records derived results
and input hashes. For example:

```sh
python3 benchmarks/http-server/primitive-comparison/transient-handoff/profile-count.py \
  --build target/primitive-comparison-build/build.json \
  --object target/transient-handoff-build/candidate.o \
  --output target/new-root-profile --paced --requests 4
```

Repeat with twelve requests and the rooted JSON object plus `--json`. These
sanitized diagnostic trials cannot support a new throughput-improvement claim.
