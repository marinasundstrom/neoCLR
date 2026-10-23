# Native stack profiling with verified method names — 2026-10-23

Add a repeatable macOS diagnostic alongside runtime counter snapshots: capture native
stacks while the checked plain HTTP client runs, reconcile every thread's self-sample
counts, retain immediate callers and map generated function IDs from a byte-reproduced
application object. No runtime, scheduling or compiler behavior changes.

## Findings

Both one- and four-owner profiles identify the same leading worker stack locations.
Percentages below are **worker wall-clock self-samples**, not measured CPU-time shares.
Blocked worker samples remain included. Coordinator waits are separate: all 2,713
four-owner coordinator samples end in a condition wait, not HTTP execution.

| Worker self location | One owner (3,704 samples) | Four owners (10,852 samples) |
| --- | ---: | ---: |
| GC allocation | 17.58% | 16.14% |
| Thread-local address lookup | 14.98% | 12.53% |
| GC root publication | 14.09% | 12.52% |
| Memory clearing (`__bzero`) | 9.45% | 12.21% |
| Indexed collection | 6.10% | 6.32% |
| String length scanning | 4.54% | 5.20% |

The immediate caller is important: **347/350** one-owner and **1,316/1,325** four-owner
memory-clearing samples come from `makecontext`. Only 3/8 respectively come directly
from GC allocation. Source inspection confirms `allocate_fiber` calls getcontext and
makecontext even when stack mappings are reused. The existing snapshot reports about
seven async bodies per response. This points to async context preparation as a concrete
runtime-wide investigation, rather than assuming clearing belongs to heap allocation.
It does not establish that all clearing can safely be removed or predict a speedup.

Four-owner strlen samples come through transient-root publication (228), ordinary
root publication (189) and generated-owned entry (147). Thread-local lookup spans
boundary checks (348), native stack checks (244), transient publication (213), root
publication (143), head access (126), entry (114), leave (114) and smaller callers.
These are exclusive leaf counts grouped by immediate physical caller, not additive
inclusive stack costs. Inlining/tail calls can hide logical callers.

Next bounded optimization candidate: investigate reusing a *completed* async execution
context as well as its stack mapping. Any candidate must preserve fresh callback/owner
state, fault isolation, stack bounds, parked-root lifetime and shutdown behavior, with
forced-GC and worker tests plus separate uninstrumented throughput/CPU measurements.
This is a proposal, not implemented context reuse. Allocation/root metadata costs also
remain substantial. Merely adding threads does not address these observed work paths.

## Capture and attribution

`capture.py` takes an existing successful HTTP snapshot build report, verifies its
binary hash and records the client/runner/build-report hashes. It waits for every
worker's READY record, runs eight seconds of wire-checked load, invokes
`/usr/bin/sample PID 5 1 -file sample.txt`, then drains/stops every owner and checks
clean process exit. The captures validate 11,148/31,003 responses at one/four owners.
Sampled request rates are not capacity evidence and are not compared to Kestrel.
The one-owner binary includes the prior opt-in GC timers; four-owner uses basic
snapshots. They corroborate locations, not a controlled scaling or overhead comparison.

`analyze.py` checks the build/capture/sample/object provenance, replays the recorded
compiler command and requires byte-identical object output before using method IDs.
It includes class-dispatch thunk identities, rejects unmapped generated leaf/caller
IDs and keeps native symbols. It parses the complete sample call tree rather than
macOS's summary, which omits counts below five. Self counts are parent counts minus
immediate child counts; totals must reconcile. Recursion does not multiply self cost.

For this fixture it requires one named main thread, the expected number of additional
threads and an observed application frame on each worker. Unknown layouts fail rather
than silently attributing helper threads to HTTP work. No elapsed-time multiplier is
applied to counts. The physical stacks may terminate at a fiber entry; they do not
reconstruct managed async causality across suspension. `sample` can perturb execution,
include off-CPU threads, miss short operations and sample fewer times than its requested
interval suggests. This is not a portable on-CPU profiler, live monitor or wait-duration
tracer. Keep exact CPU attribution and queue/I/O timelines as separate diagnostic work.

Six focused tests cover residual/self counts, small leaves, recursion, repeated symbols,
thread separation, malformed/overcounted trees and generated-name decoding. Both real
captures and byte-verified analyses pass. Raw samples, capture manifests, full summaries
and validation hashes are archived here. Normal runtime behavior is unchanged, so no
GC regression suite, API snapshot or website build is needed for this tooling slice.
The existing [.NET/Go diagnostic research](../../../../docs/design/allocation-metadata-runtime-comparison.md#standard-runtime-diagnostics-proposal--2026-10-23)
remains the design basis; this uses the OS profiler before implementing a new runtime sampler.

## Reproduce

Requires macOS sample access to the launched process and the matching compiler/bundle
from the previous experiments. Each output directory must be new. Build provenance
comes from the runtime-snapshot runner; a stale binary or changed object is rejected.

```sh
python3 benchmarks/http-server/primitive-comparison/stack-profile/test_analyze.py
python3 benchmarks/http-server/primitive-comparison/stack-profile/capture.py \
  --build-report target/runtime-snapshot-trial-1/profile.json \
  --client target/primitive-comparison-refresh/client \
  --output target/http-stack-profile-four
python3 benchmarks/http-server/primitive-comparison/stack-profile/analyze.py \
  --capture target/http-stack-profile-four/capture.json \
  --build-report target/runtime-snapshot-trial-1/profile.json \
  --compilation target/plain-owned-root/measurement.json \
  --object target/plain-owned-root/candidate.o \
  --output target/http-stack-analysis-four
```

Use the one-owner build report for a one-owner capture. Archived `.gz` files decompress
to the original text/JSON; capture manifests retain raw sample hashes. This is a macOS
fixture-specific tool; broader process/host identification remains future work.

Follow-up: executable-image matching now uses the verified build's output filename
instead of assuming `server`. The context-reuse experiment adds a seventh regression
case for named candidate binaries and retains a successful full attribution.
