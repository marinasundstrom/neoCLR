# Serve CPU and allocation caller diagnostics — 2026-10-18

Follow the header-reservation slice with measurement before selecting another
optimization. Runtime behavior, GC policy and public APIs are unchanged.

## Instrumentation

capture.py relinks the existing Serve object against a diagnostic copy of the native
collector. Each successful allocation records payload byte size, allocation kind
and up to four published frame function/instruction pairs. No guest pointers are
retained or followed. This is logical caller attribution, not an allocated-type
census or native-stack capture. Payload sizes include representation overhead and
must not be interpreted as array element counts. Truncated chains are explicit.

The capture uses a fixed 32,768-entry host table and no allocation in the hook.
Overflow counts and bytes remain explicit, reconcile with totals and cause the
runner to reject the profile. The thread-affine instrumentation matches this host;
it is not a production or multithreaded tracing API. Startup/warm-up/shutdown are
included. Whole-process counts use 200 measured requests, 20 warm-up requests and
one stop exchange, matching prior diagnostics. Timing under instrumentation is
never capacity evidence.

Paced-boundary counters distinguish context/collection-epoch changes, the
16-allocation threshold and the 64-boundary threshold. Idle calls include these
boundary-triggered calls and explicit idle collection. Capture does not change
short-circuit evaluation of the pacing condition or reset counters differently.
These counters do not classify all explicit/conditional collector entry points.

This extends existing allocation profiling rather than changing the platform's
[.NET/GC comparison](../../../../docs/experiments/aot-console/native-gc.md#comparison-and-provisional-choices).
No .NET diagnostic API compatibility is implied.

## Reproduction and tests

Use the same SDK environment as the existing native Serve build. Run:

```sh
python3 benchmarks/http-server/serve/caller-profile-20261018/test_callers.py
python3 benchmarks/http-server/serve/caller-profile-20261018/capture.py \
  --build target/http-header-reserve-20261018/candidate \
  --output target/http-next-profile-20261018/callers
python3 benchmarks/http-server/serve/diagnose.py \
  --executable target/http-header-reserve-20261018/candidate/app-paced \
  --sample --rate 400 --duration 20 --output target/http-next-profile-20261018/cpu
```

Two focused native tests check key separation by kind/size/callsite, repeated and
unframed allocations, bounded depth, instruction capture, pacing counters and
loss accounting under deliberate table overflow. The original guest/object and
native source hashes are retained in the diagnostic build manifest.

## CPU sample

The current header-reservation Serve executable is sampled for five seconds during
400/sec offered load (20-second diagnostic window). The main-thread sample count is
3,557. Leaf counts: allocation 739 (20.8%), root publication 729 (20.5%), indexed
collection 702 (19.7%), strlen 292 (8.2%), and conditional collection 147 (4.1%).
These are sample proportions, not exact CPU durations or additive inclusive costs.
String scanning appears beneath root publication/entry in the call tree; the
existing validation contract must remain intact in any optimization.
[Raw sample](evidence/cpu-sample.txt.gz), [requests/provenance](evidence/cpu.json).
This sampled run is not a new sustainable-capacity qualification.

## Array caller attribution

The 61,601 allocations and 3,327,856 payload bytes exactly reconcile with the prior
header-reservation diagnostic. No table overflow. ReserveArray accounts for 15,583
allocations and 1,613,427 bytes. Grouping the recorded callers:

| Caller | Allocations | Payload bytes |
| --- | ---: | ---: |
| Promise constructor → result-slot list constructor | 2,909 | 1,115,421 |
| Promise.Register → callback-list Add | 2,687 | 139,724 |
| Promise.Register → callback-list constructor | 2,687 | 42,992 |
| UTF-8 decode | 1,326 | 38,008 |
| HttpRequest.Text → list Add | 1,105 | 35,360 |
| HttpContext constructor → Array.Empty | 442 | 7,072 |
| HttpContext.Finish → Array.Empty | 221 | 3,536 |

These rows count arrays only, excluding their containing list/Promise objects.
Generic specializations are grouped by readable owner; the full report retains
function and instruction IDs. [Grouped array callers](evidence/reserve-callers.json),
[full caller report](evidence/callers-summary.json.gz).

Source inspection confirms Promise.Register creates an empty callback list and
immediately adds its first callback. A capacity-one reservation can avoid the
empty allocation and the first growth; later registrations must still grow.
Promise result storage is a separate larger investigation: it uses ArrayList<T>(1)
even for a single result, including large inline union payloads. Replacing that
storage requires value/reference/union semantics and GC retention qualification.

## Why collection counts can vary

The encoding-cache and header-reservation diagnostic objects record exactly 11,125
paced boundary triggers each. The allocation threshold changes from 1,802 to 2,019;
the boundary threshold changes from 9,322 to 9,105; context/epoch changes remain one.
So this capture does not show more boundary-triggered collections after reservation.
Total idle calls differ (19,491 versus 19,756), including explicit host idle calls.
These timing-sensitive counts do not reproduce the earlier collection-total delta
exactly and do not prove a cause for it. They rule out treating allocation count
alone as a predictor of collection count. Broader collection-origin attribution
remains open. [Baseline counters/callers](evidence/baseline-callers-summary.json.gz).

The selected next implementation is first-callback reservation. Result-slot
representation and root/allocator redesign remain separate. No speedup or new
Serve/Go result is claimed by this diagnostic slice.
