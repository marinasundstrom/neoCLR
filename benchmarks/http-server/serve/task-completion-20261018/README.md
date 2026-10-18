# Avoid empty continuation-list replacement — 2026-10-18

The author requests the next Serve/GC bottleneck investigation, a matched Go
comparison, honest sustained-load reporting, and particular attention to unnecessary
HTTP/Task allocations. They also question Task reference-type overhead and whether
the Serve callback should receive HttpContext.

## Profile and selected change

A fresh five-second wall sample of the previous qualified Serve guest linked with
current native GC again finds allocation (802 top-of-stack samples), indexed
collection (674), root publication (609), strlen (259), and zeroing (252) prominent.
These are sampled stacks, not exact CPU percentages. The diagnostic request run
is excluded from capacity results. [Sample](evidence/sample.txt.gz),
[command](evidence/profile-command.json), [outcomes](evidence/profile.json).

Inspection finds that Promise completion and cancellation allocate a new empty
callback list even when no continuation is registered. The implementation now
returns successfully after settlement when the existing list is empty. Later
registrations still post to the dispatcher; callbacks already registered retain
the existing detach-and-post path. Task identity, result storage, rooting and GC
policy are unchanged. This shared managed-library change applies to interpreter,
JIT and AOT, rather than specializing HTTP behavior.

Task is not the only allocation involved: Promise also owns result storage and a
continuation list; Task.Run adds queued work. This slice measures one avoidable
allocation path, not total Task-attributed allocations or the benefit of a value-type
replacement. No claim that the wrapper is the dominant cost follows from this profile.

## Task and HTTP API comparison

.NET also defaults to reference-type Task. [ValueTask guidance](https://learn.microsoft.com/en-us/dotnet/api/system.threading.tasks.valuetask-1?view=net-10.0)
explains its synchronous-result allocation benefit and the extra fields/copying and
consumption restrictions. A struct wrapper does not automatically eliminate promise,
continuation or suspended-state allocations. Keep Task semantics and first remove
avoidable allocations; a separate awaitable needs measured justification and its
own compiler/runtime contract.

Serve's supervised concurrency/lifetime is useful, while its request-to-response
callback is a convenience contract, not a complete web hosting foundation.
[ASP.NET Core RequestDelegate](https://learn.microsoft.com/dotnet/api/microsoft.aspnetcore.http.requestdelegate)
accepts HttpContext and returns Task. A future context-based handler would better
support streaming request/response access and middleware. It must define who
completes/disposes the response, errors after headers, cancellation and context
lifetime. Prefer retaining the existing response-returning callback as an adapter
rather than silently changing its ownership contract. This is a proposal, not a
new API or an assertion that a context signature alone improves throughput.
Primary sources reviewed 2026-10-18.

## Correctness and diagnostic allocation counts

[Contracts.rvn](Contracts.rvn) checks empty-list completion/cancellation, duplicate
settlement, retained results, deferred late callbacks and pre-registered callbacks,
repeated ten times. It passes interpreter/JIT/AOT with the rebuilt library.
[Three-mode evidence](evidence/contracts.json.gz). The first 100-iteration interpreter
attempt exceeded the normal instruction budget; ten repetitions establish the
behavior without changing runtime limits. No GC or native memory contract changed.

Separate instrumented runs each validate 200 measured requests plus 20 warm-up
requests and the stop exchange. Counts include startup/shutdown and are not
per-request allocation measurements or capacity timings:

| Counter | Baseline | Candidate |
| --- | ---: | ---: |
| Allocations | 85,838 | 85,172 |
| Allocated payload bytes | 4,164,908 | 4,145,372 |
| Collections | 20,911 | 19,638 |
| GC nanoseconds | 205,449,000 | 208,992,000 |
| Peak live payload bytes | 46,260 | 46,260 |

There are 666 fewer allocations (0.78%) and 19,536 fewer payload bytes (0.47%).
GC elapsed time increases in this single diagnostic pair; fewer collections alone
do not establish lower CPU or latency. Timing and scheduling are perturbed by
instrumentation. All allocations are reclaimed by shutdown and GC failures are zero.
[Baseline counts](evidence/baseline-gc-counts.json),
[candidate counts](evidence/candidate-gc-counts.json),
[baseline requests](evidence/baseline-diagnostic.json),
[candidate requests](evidence/candidate-diagnostic.json).

## Reproduction and comparison boundaries

`build.py` compiles the same Serve source with an explicit compiler/backend and
library, then reuses the qualified native link recipe. Both arms use the current
primitive/enum-capable compiler/backend and current native GC. The baseline library
is the earlier `library-primitive-4` cohort: its recorded Raven source/project hashes
match current sources except Tasks.rvn. The candidate rebuild uses the same compiler.
This is a new guest cohort, not a direct speedup comparison with earlier fixed-guest
HTTP reports. The source, library, object and executable hashes are retained in
compressed build manifests and library evidence.

Example (use matching absolute tool paths):

```sh
python3 build.py --library LIBRARY --compiler COMPILER --backend CODEGEN \
  --link-build QUALIFIED_NATIVE_BUILD --output NEW_BUILD
python3 verify.py --library LIBRARY --compiler COMPILER --runtime RUNTIME \
  --backend CODEGEN --host NATIVE_HOST --output NEW_CONTRACT_RESULTS
```

Sustained-load results will be recorded separately after the uninstrumented runs
finish. No higher operating rate or HTTP speedup is claimed by this allocation slice.
