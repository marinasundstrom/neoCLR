# Simplify Promise result storage — 2026-10-19

The adopted change replaces Promise<T>'s one-element ArrayList<T> with one owned
array slot and an explicit completion flag. It removes the list allocation while
preserving the array's initialization marker and existing GC representation.
Task identity, repeatable reads/awaits, cancellation and queued callbacks remain
unchanged. No public API, runtime default or compiler contract changes.

An earlier inline-result candidate is **not adopted**: it saved more allocations
but regressed bulk p99 in both matched rounds. Its patch and evidence remain below.
This optimization is a bounded improvement to the current implementation, not the
long-term solution to Promise wrappers in HTTP. The author subsequently directs a
[runtime completion bridge](../../../../docs/design/runtime-task-completion-bridge.md).

## Inline candidate correctness and toolchain findings

[Contracts.rvn](Contracts.rvn) retains all earlier completion/cancellation/queue
contracts and adds generic payload coverage: reference objects, structs containing
references, larger union payloads, null, zero, unit, empty cases, repeated reads and
awaits, value-copy isolation, pending collection and collection across suspension.
Interpreter, JIT and AOT pass against the rebuilt five-library bundle.

The initial explicit `result = default` attempt exposed a native generic-default
validation error (`method type parameter outside declaring context`). Ordinary
zero-initialized field storage compiles and passes the tests; explicit generic
initialization remains a separate compiler/runtime investigation. An await inside
a test argument also needed a separate lexical binding for native emission.
Neither finding changes public semantics or requires a Raven branch modification.

The initial interpreter invocation accidentally selected an older local binary
without current Socket Memory bindings; the matching retained bundle runtime passes.
AOT compilation passed but the default command-line SDK could not link its system
stub library. Relinking the exact object with the matching Xcode 26.2 SDK passes.
Failed attempts and the successful link recovery are retained in the evidence.

## Inline candidate allocation diagnostic

The same 200 measured plain requests, 20 warmups and shutdown exchange run against
baseline/candidate builds. Counts include startup/shutdown and reconcile with zero
lost records. Only Tasks.rvn differs among library source inputs. Compiler, Serve
source, native runtime and heap settings match. Instrumented times are not capacity
measurements.

| Counter | Baseline | Candidate | Change |
| --- | ---: | ---: | ---: |
| Allocations | 54,715 | 48,897 | -10.63% |
| Allocated payload bytes | 3,113,755 | 2,947,942 | -5.33% |

Exactly 5,818 allocations disappear: two for each of 2,909 Promises. Result payloads
still need space, now inline in the Promise, so removing backing arrays does not
remove their entire payload-byte total. Native allocation headers are not included
in the payload-byte counter. This is allocation-volume evidence, not a measured
live-heap reduction or proof of higher sustained HTTP capacity.

## Reproduction

Build the library with scripts/build-native-class-library.py and the same Raven
compiler revision dcf7ac41f. Use the matching source-built runtime bundle for
execution. The retained manifests identify exact hashes and link settings.

```sh
python3 benchmarks/http-server/serve/task-completion-20261018/verify.py \
  --library LIBRARY --compiler COMPILER --runtime RUNTIME --backend CODEGEN \
  --host HOST --source benchmarks/http-server/serve/promise-result-20261019/Contracts.rvn \
  --expected-line 'Promise result contracts passed' --output CONTRACT_OUTPUT
python3 benchmarks/http-server/serve/task-completion-20261018/build.py \
  --library LIBRARY --compiler COMPILER --backend CODEGEN \
  --link-build target/http-memory-gc-cohort --output CANDIDATE
python3 benchmarks/http-server/serve/caller-profile-20261018/capture.py \
  --build CANDIDATE --output ALLOCATION_OUTPUT
python3 benchmarks/http-server/serve/promise-result-20261019/compare.py \
  --baseline-build target/http-memory-gc-cohort --candidate-build CANDIDATE \
  --rates 400 --duration 30 --output PLAIN_OUTPUT
# Repeat with --workload bulk --rates 100 and a separate output directory.
```

Use the matching Xcode 26.2 clang/SDK for native linking. Profiling and builds must
finish before timed load. The inline candidate is not production behavior. Decompress inline-candidate.patch.gz and apply it in an
isolated checkout at 99480b002 to reproduce it; retain the original baseline build.

## Inline candidate HTTP result: reject for now

Eight 30-second uninstrumented windows, in reversed arm order, compare paced
synchronous GC with identical 1 MiB heap, 16 lanes and 64 client slots. All admitted
responses validate and all shutdown counts match. These timings are separate from
allocation tracing. Loopback desktop activity remains uncontrolled.

| Workload | Round | Baseline CPU ms/completion | Inline CPU ms/completion | Baseline p99 ms | Inline p99 ms | Baseline / inline unsent misses |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Plain, 400/sec | 1 | 2.097 | 1.910 | 175.46 | 166.16 | 77 / 12 |
| Plain, 400/sec | 2 | 2.114 | 1.832 | 183.97 | 7.11 | 138 / 0 |
| Bulk, 100/sec | 1 | 7.320 | 7.713 | 46.64 | 286.89 | 0 / 0 |
| Bulk, 100/sec | 2 | 7.287 | 9.363 | 9.35 | 289.18 | 0 / 0 |

The bulk regression repeats despite reversed order. Fewer allocations alone are
insufficient justification to adopt this layout. Its experimental concurrent-GC
recovery/suspension/peer test passes with actual reclamation, but comparative
concurrent throughput was not measured for this candidate.

A follow-up bulk stack sample puts collect_indexed at 42.2% of main-thread wall
samples versus 26.8% in the baseline investigation. This localizes increased GC
work but is not exact phase timing or proof of a single cause. Source inspection
shows a material distinction: ordinary objects scan all words, whereas record
arrays skip elements with an unset initialization marker. Inline pending generic
result payloads therefore lose that skip. Precise field tracing or conditional
payload tracing would need separate runtime design and qualification.

The next candidate keeps one array slot and removes only the ArrayList wrapper,
preserving the existing initialization-marker representation. It is a smaller
allocation reduction intended to avoid that tracing tradeoff; measurement must
establish its behavior rather than assuming it fixes the regression.

## Adopted array-slot candidate

The same extended contracts pass interpreter, JIT and AOT. Experimental concurrent
GC also passes malformed-request recovery, suspended request ownership, 16 peers,
actual application/reclamation and complete shutdown cleanup. The fresh matched
source comparison confirms only Tasks.rvn changes; compiler, native runtime and
Serve source match. Concurrent throughput for this storage change is unmeasured.

The diagnostic records **51,806 allocations** and **3,043,939 payload bytes** versus
54,715 / 3,113,755 baseline: **5.32% fewer allocations and 2.24% fewer bytes**.
Exactly one allocation disappears per Promise (2,909). No attribution records drop.

Eight further 30-second synchronous-GC load windows use the same operating points
and reverse arm order. Every measured arrival succeeds: 48,000 plain and 12,000
bulk requests across both arms/rounds. Body/framing and shutdown counts validate,
and every provisional admission/latency/generator-lag gate passes.

| Workload | Round | Baseline CPU ms/completion | Array-slot CPU ms/completion | Baseline p99 ms | Array-slot p99 ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| Plain, 400/sec | 1 | 2.015 | 1.853 | 8.70 | 6.58 |
| Plain, 400/sec | 2 | 2.013 | 1.857 | 20.00 | 7.60 |
| Bulk, 100/sec | 1 | 7.313 | 7.050 | 8.76 | 9.31 |
| Bulk, 100/sec | 2 | 7.383 | 7.153 | 12.92 | 57.29 |

CPU/completion improves about 7.8–8.1% for plain and 3.1–3.6% for bulk. Offered
throughput is unchanged (400/100 per second); the admitted-rate trials alone do not test higher capacity.
Bulk p99 is higher, particularly in round two, so this is not a general latency
improvement. The repeated ~287–289 ms inline regression is not reproduced by the
array-slot candidate. Retain all earlier inline failures, without comparing
noncontemporaneous arms to manufacture a larger gain.

[Array-slot evidence](evidence/array), [inline experiment evidence](evidence/inline)
and [hash manifest](evidence/manifest.json) retain raw request reports losslessly,
build/library manifests, contract results and allocation profiles. Baseline
allocation evidence is in the preceding investigation. No profiler/build overlaps
these timing windows. Local desktop activity and short runs limit generalization.
Public documentation fingerprints are refreshed; website publication is separate.

## Throughput follow-up at overload

After the author questioned whether Promise allocations actually limit throughput,
four further 30-second windows offer 600 plain requests/sec. Baseline completes
483.73/497.33 per second; the array-slot candidate completes 515.73/512.17. Mean
completed throughput is 490.53 versus 513.95 (**4.8% higher**). Unsent slot-limit
misses sum to 6444 versus 5038; all admitted responses and shutdown counts validate.
p99 is 142.06–143.07 ms baseline versus 132.29–135.65 ms candidate. Neither arm
admits all arrivals, so these are overloaded delivery rates, not sustainable 600/sec
capacity. This provides bounded evidence that the allocation change contributes to
throughput, not that Promise is the main remaining bottleneck. No bulk overload or
concurrent throughput claim is made for this change. The author next focuses work
on allocations during initial request read/parsing.
