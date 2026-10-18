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
callback is a convenience contract, not a complete web hosting foundation. The
author clarifies the primary motivation: Accept already returns HttpContext, and
Serve should initialize that context for the consumer so the callback can configure
its response without constructing and returning HttpResponse.
[ASP.NET Core RequestDelegate](https://learn.microsoft.com/dotnet/api/microsoft.aspnetcore.http.requestdelegate)
accepts HttpContext and returns Task. A future context-based handler would better
provide that consistent, ready-to-use request/response experience, with streaming
and middleware as further benefits. The proposed default is server completion and
disposal after the handler task settles. It must define who
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

## Sustained-load comparison

Two reversed-order rounds run baseline/candidate/Go then Go/candidate/baseline,
reversing rates in round two. Each point has a separate warm-up and a 60-second
measurement window. The same ten-byte connection-close Serve workload, 16 server
lanes and 64 generator slots is used. neoCLR has experimental paced GC; Go uses
GOMAXPROCS=1 and GOGC=100. No builds or profilers run during these windows.

Qualification requires all offered arrivals to succeed, offered p99 <=250 ms and
generator dispatch-lag p99 <=5 ms. Missed arrivals remain failures; drain completions
are excluded from timed throughput. These are same-host observations, not production
capacity or a Go maximum-throughput measurement.

| Arm | Offered/sec | Timed completions/sec | Misses | CPU ms/completion | Offered p99 ms | Generator lag p99 ms | Target |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| baseline-1 | 300 | 300.00 | 0 | 2.429 | 57.43 | 1.12 | Pass |
| baseline-1 | 400 | 399.47 | 31 | 2.273 | 116.04 | 1.45 | Fail |
| candidate-1 | 300 | 299.80 | 11 | 2.409 | 187.03 | 14.80 | Fail |
| candidate-1 | 400 | 399.78 | 12 | 2.252 | 101.57 | 1.22 | Fail |
| go-1 | 300 | 300.00 | 0 | 0.208 | 23.31 | 10.64 | Fail |
| go-1 | 400 | 400.00 | 0 | 0.217 | 9.39 | 4.64 | Pass |
| go-2 | 400 | 400.00 | 0 | 0.212 | 8.31 | 3.67 | Pass |
| go-2 | 300 | 300.00 | 0 | 0.239 | 7.91 | 3.37 | Pass |
| candidate-2 | 400 | 399.18 | 48 | 2.274 | 158.49 | 2.76 | Fail |
| candidate-2 | 300 | 293.58 | 385 | 2.443 | 281.85 | 388.39 | Fail |
| baseline-2 | 400 | 399.27 | 43 | 2.262 | 106.23 | 1.58 | Fail |
| baseline-2 | 300 | 299.65 | 21 | 2.413 | 154.96 | 1.90 | Fail |

The first pair uses about 0.8–0.9% less CPU per completion, but the second pair
reverses the difference (candidate about 0.5% higher at 400/sec and 1.3% at 300/sec).
There is **no repeatable CPU speedup and no higher reliable sustained request rate**.
All four candidate points miss arrivals; the second 300/sec candidate also suffers
388 ms generator-lag p99, so that deterioration cannot be attributed solely to the
server. Baseline passes only its first 300/sec point. Go validates every offered
request and passes three of four points; its first 300/sec generator-lag failure
remains visible. At 400/sec, candidate CPU/completion is about 10.4–10.7 times Go's.

All admitted requests validate, server request failures are zero and shutdown
accounting agrees. Sampled candidate RSS is 13,040–13,056 KiB versus baseline
12,928–12,944 KiB, so lower allocation count is not a lower process-RSS claim.
The isolated allocation reduction is worth retaining, but it does not resolve the
larger allocation/collection/root-publication costs. This matrix establishes no
repeatably qualified neoCLR point at the tested rates under current host conditions.

[Run order](evidence/comparison.json); compact reports link their lossless full data:
[baseline 1](evidence/baseline-1.json), [candidate 1](evidence/candidate-1.json),
[Go 1](evidence/go-1.json), [Go 2](evidence/go-2.json),
[candidate 2](evidence/candidate-2.json), [baseline 2](evidence/baseline-2.json).
Reproduce with `../root-validation/compare.py --baseline BASELINE --candidate CANDIDATE
--go GO --rates 300 400 --duration 60 --output NEW_DIRECTORY`.



## Next allocation targets

A separate candidate-only diagnostic charges each successful managed allocation to
its current published guest frame. It validates the same 200-request workload plus
warm-up/stop. This is caller attribution, not allocated-type identity or exact CPU
cost: ReserveArray, for example, serves many callers. Instrumentation is excluded
from every capacity run.

The report exposes thousands of ArrayList.GetIterator allocations, plus TaskQueue
Drain and Promise construction/completion bookkeeping. These are stronger concrete
follow-ups than changing Task's reference semantics blindly: consider indexed
iteration over private callback lists and reducing empty/backing-storage objects,
while preserving deferred delivery, reentrancy rules, cancellation and GC roots.
HTTP response configuration and the already-initialized context response are further
inspection targets, not measured improvements in this slice.

[Allocation caller report](evidence/allocation-summary.json),
[instrumented build](evidence/allocation-build.json),
[request outcomes](evidence/allocation-requests.json).
Reproduce on macOS with `allocation-sites.py --build CANDIDATE --output NEW_DIRECTORY`.

Website validation: regenerated the unchanged public reference from the matching
bridge, refreshed source fingerprints and built/checked all 4,449 pages. No deployment.
