# Explicit execution-state handle prototype — 2026-10-24

**Isolated native prototype; not integrated into generated code or production.**
This follows the measured frame/TLS bottleneck and the rejected helper-outlining
experiment. It establishes a candidate lifetime contract and tests potential cost
savings before changing imports, stack budgets or supported native artifacts.

## Contract and implementation

`prepare.py` derives `prototype.c` from production `efeea6a66`. The stack-root and
transient implementations receive a state pointer instead of reading TLS directly.
Existing adapters obtain the current TLS pointer; experimental adapters receive it
from their caller. A separate leave adapter uses the same head/depth/fault checks
and clears the frame. Capture returns the current runtime library's TLS address.
The published frame stays 104 bytes; the prototype adds no field or managed object.
Production `execution-state.h` still forbids retaining its pointer. This experiment
proposes a narrowly scoped exception; it does not silently change that contract.

A generated activation may retain a handle only on its owner native thread, while
that thread/runtime instance and activation remain alive. Cooperative park/resume
on that same thread preserves the handle but changes the active root head. A native
reentry may use another context on the same thread; it also changes the head, so
cached handles must read current state, never cache frame chains, roots, collection
epochs or metadata validation results. Root values remain mutable and traced afresh.

The handle must not escape into managed objects, host callbacks, work queues,
thread-pool submissions or migrating continuations. Cross-thread use is **outside
the trusted contract**, not dynamically rejected by this prototype. Checking the
current TLS identity on every use would reintroduce the lookup. A non-null forged
or stale handle is likewise not a supported native input. Ordinary native APIs
retain their TLS-based checked path. This is not a public embedding capability or
memory-protection mechanism. Worker coordination/shared-heap integration is untested.

## Validation and measurements

Four native configurations (optimized/sanitized × forced/paced GC) pass normal
main-thread plus two fresh-thread exercises, and seven rejection cases each:
null handle, bad map, parked/inactive frame, wrong current head, invalid transient
phase, invalid fault shape and repeated leave. There are 4 builds, 4 successful
fixture processes and 28 expected aborts. Tests cover same-thread park/resume,
foreign-context reentry, independent thread counters, changing root values,
transient retention/expiry and heap release. The prototype does not replace the
existing shared-worker/fiber integration qualification.

A single optimized prototype binary compares its TLS-checked adapters against
cached-handle adapters, including a separate capture call on **every activation**.
Each activation enters a one-descriptor frame, publishes/transient-updates 1, 4 or
16 times, then leaves. Each row performs 20 million activations on an empty heap
with ordinary paced collections. Four runs alternate checked/cached/cached/checked;
no other builds or profiling run during measurement. Timings are process CPU.

| Publications per activation | Checked mean seconds | Cached mean seconds | CPU reduction |
|---|---:|---:|---:|
| 1 | 0.350350 | 0.319453 | 8.8% |
| 4 | 0.896992 | 0.837932 | 6.6% |
| 16 | 3.207449 | 2.950279 | 8.0% |

This is a hook-loop comparison within the prototype, **not a production baseline,
HTTP throughput, JIT placement or application speedup**. The first checked run is
slower, especially for one publication. Two runs do not establish confidence
intervals. The shorter initial runs are retained but are too noisy for conclusions.
The checked adapter also uses the refactored implementation, so compiler layout
can differ from production. Real generated code introduces state storage/load and
register costs that this native caller does not reproduce.

ARM64 assembly confirms no TLS reference in experimental stack/transient/leave
adapters. The common stack implementation still contains a TLS path for legacy
string-marker lookup; typed maps skip it. Slow collection and diagnostics retain
their own state accesses. No claim of removing all TLS access is made.

## Generated integration selected for the next slice

Use explicit opaque arguments rather than changing the collector-visible frame:

1. A new trusted entry import returns the state already resolved for admission.
   Keep current entry/storage/map validation. Store the pointer in a separate
   eight-byte compiler-owned stack slot for this activation.
2. Pass that value to new typed stack/transient/leave imports. Keep 104-byte frame
   traversal unchanged and include the extra slot in generated stack-budget checks.
   All early/fault returns must load an initialized handle and use matching leave.
3. Retain existing imports for ordinary native/probe code. Initially exclude shared
   worker-safepoint lowering; coordinator participation must be qualified separately.
   Never infer permission to migrate an activation from managed Task.Run semantics.
4. Version private imports and qualify matching JIT/AOT hosts, package/native-kit
   helpers and compiler tests. Raven source semantics and Runtime Contract selection
   need no change; document the backend integration in both compiler records when
   implemented. Installed older hosts must not silently satisfy the new imports.
5. Rebuild ordinary/async/worker consumers, force collection across changing roots,
   suspension, host reentry, faults and expired stack owners, then measure matched
   plain and JSON HTTP behavior. Retain only with meaningful CPU/capacity evidence.

Alternatives: storing the handle in a larger runtime frame avoids an extra argument
but couples every native frame layout and fixture to it; storing it on the context
fails when contexts have differing thread lifetimes or sharing; globally pinning a
register affects calling conventions and native interoperability. Explicit per-frame
state uses extra stack/register traffic but scopes ownership clearly. The existing
[CLR/Java/Go owner-local state comparison](../../../docs/design/allocation-metadata-runtime-comparison.md)
provides context; this is not an implementation of CLR Thread or a new .NET API.

## Reproduce

Run `prepare.py --output target/state-handle/prototype.c`, then `validate.py` or
`measure.py` with `--source` pointing there and a fresh `--output` directory.
The fixed production revision supplies the root implementation; use matching
native headers/collector from that revision. Compiler/SDK commands and hashes
are recorded in reports. `prototype.c.gz` preserves the generated source;
`prototype.s.gz` preserves the optimized assembly. `tests.json.gz`,
`cost-short.json.gz`, `cost-long.json.gz` retain raw evidence; `initial-cost.c.gz`
records the shorter loop. `summary.json` contains the two raw timings per mode/count
and `100 * (1 - mean(cached) / mean(checked))`. No compiler or production source is
changed by this slice.
