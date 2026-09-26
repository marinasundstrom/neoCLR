# HTTP/JSON array-budget accounting cost — 2026-09-26

Follow-up to the [managed-pair timeout](repeatability-20260926.md). This reduces a
measured interpreter cost without changing socket/HTTP deadlines, application code,
compiler artifacts or the array quota contract.

## Finding

A temporary native-service trace separates DNS, connection and transfer completion
from time spent in managed code. In the retained [baseline trace](array-budget-trace.json),
POST request receipt precedes response-send admission by 4,687 ms. The client's
receive result arrives 4,694 ms after receive admission, close to the existing
five-second native transfer cap. GET processing also takes around two seconds.
DNS and connect result delivery are much shorter in this run. It succeeds; this
trace does not establish the precise operation that failed in the earlier run.

A two-second macOS sampling profile taken during POST processing contains 1,147
main-thread samples. Of these, 1,034 are under the interpreter's array-budget check
location, including 749 under ManagedHeap::array_usage. The sampled stacks include
repeated allocation/reallocation in arrays::measure. This identifies a concrete
source of processing cost, not a network-throughput or .NET performance comparison.

The interpreter deliberately checks aggregate logical array payload at instruction
boundaries. The old walker allocates a work vector even for a leaf and expands
every array/object child into that vector. It repeats this across frame values and
live heap slots. HTTP byte arrays make that overhead frequent.

## Change and preserved contract

Walk borrowed reverse sibling iterators, retaining only ancestors that still have
unvisited siblings. Leaf values, flat arrays and single-child chains require no
traversal allocation. Branching nested structures use a growable iterator stack;
the walk remains iterative rather than risking recursive host-stack exhaustion.

Reverse order matches the former last-in-first-out traversal. Each array still
contributes its element count. Each value inside array storage still contributes
its Value-sized cost, including object/erasure wrappers and logical UTF-8 bytes
per text occurrence. Shared string storage is not deduplicated for quota purposes.
Saturating counters, quota faults, collection-before-rejection, instruction-boundary
checks and reference traversal rules remain unchanged. There is no usage cache or
new invalidation/GC ownership mechanism.

Reuse the [managed-array model and budget contract](../../managed-arrays.md) and
[Raven array-shape comparison](../../raven-array-shapes.md). The Raven-facing array
identity, element access and assignment contract is unchanged. neoCLR's host logical
payload budget is implementation policy, not an assertion that CLR performs this
same per-instruction walk. The benefit is less host allocation in the current
interpreter; the cost is a slightly more involved walker. Linear scans of the live
payload remain, so this does not settle a future incremental accounting design.

## Reproduction and validation

Build the baseline runner from `181cb214` and a candidate runner containing the
arrays::measure change, using the same release build settings. The comparison
driver compiles the mapped client/server once, copies System.neoil once, hashes
those inputs and runs both runtimes against exactly those files. It alternates
runner order across repetitions, starts fresh processes, validates the JSON result
and zero final live objects, and retains failures instead of retrying them away.
Any failed observation makes the final command fail.

```sh
python3 docs/experiments/http-json/compare-runners.py \
  --toolchain-root /absolute/path/to/matching/development/bundle \
  --baseline /absolute/path/to/baseline/measure_async \
  --candidate /absolute/path/to/candidate/measure_async \
  --repeat 3 --output /absolute/path/to/comparison.json
```

The [result snapshot](array-budget-cost-results.json) records input/runner hashes
and all observations. Timings are whole-program phases; server execution includes
waiting for client preparation. This is a small local comparison, not a statistical
benchmark, latency SLA or qualification under arbitrary competing load.

| Iteration | Baseline client execution | Candidate client execution |
| --- | --- | --- |
| 1 | 18,221 ms, passed | 5,556 ms, passed |
| 2 | 15,100 ms, TimedOut | 7,082 ms, passed |
| 3 | 15,973 ms, passed | 13,487 ms, TimedOut |

Both variants pass two of three runs; **repeatability remains unresolved**. The
successful candidate observations are substantially faster, but do not establish
an improved pass rate. All successful runs retain identical managed counts:
761 client / 626 server allocations, 13 / 11 collections and zero final live objects.
Failed runs do not provide final heap counts. The comparison command exits nonzero
because the failures are retained. Keep this as an internal cost reduction, not a
completed HTTP timeout repair. Next capture per-operation traces for a candidate
failure and distinguish the five-second native transfer cap from the outer exchange
budget before changing policy or scheduling.

Three focused accounting tests cover exact nested element/byte boundaries, shared
UTF-8 text, wrapper costs, cumulative roots, restoration of outer context and a
4,096-level iterative erasure chain. The 41 managed/reference/reserved-array
integration tests pass, including aliasing, GC roots, quotas and collection under
payload pressure. HTTP acceptance uses the unchanged mapped GET/POST application.
No full suite, compiler rebuild or website build was run for this internal
runtime change. Website behavior/limitations were reviewed and remain accurate.

## Diagnostic trace reproduction

[io-trace.patch](io-trace.patch) is temporary diagnostic instrumentation for the
baseline's vm.rs, not an integrated runtime feature. On an isolated baseline
checkout, apply it, build the measured example and run the existing pair verifier
with NEOCLR_IO_TRACE_DIR pointing to an existing scratch directory. The runner
writes one PID-named log per invocation with native service names, timestamps and
numeric results; it does not log HTTP bodies. Remove the patch after capturing
the trace. Production comparison runners above contain no trace instrumentation.
The raw sampling profile was kept locally; the measured sample counts and trace
needed to interpret the finding are preserved here.

The separate unresolved-call compiler diagnostic defect remains open. Neither this
runtime improvement nor successful HTTP exchanges establish full SDK acceptance.

A [subsequent slot-summary experiment](slot-budget-summary.md) builds on this
walker. The no-cache description above records this first optimization only.
