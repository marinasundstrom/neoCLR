# Compiler-owned transient descriptors — 2026-10-24

The author explicitly requests validating compiler-owned transient descriptors once
within their lifetime while continuing to trace changing root values. This extends
the owned local-storage contract; it does not change safepoint selection or tracing.

## Contract and implementation

Native-GC lowering now imports `neoclr_gc_transient_owned_v1`. This hook invokes the
existing checked publication path, then marks the frame's transient descriptors
owned. Descendant boundaries skip revalidating those descriptor addresses/widths.
The collector still reads current slot values (and retains its existing tracing
checks). No heap object, root value or metadata validity is cached by pointer.

The generated table pointer, count, addresses and widths remain fixed from
publication until replacement, the next boundary in that frame, or leave/expiry.
`GcProbe::publish` initializes the table before the hook. Tables are local to the
function activation. The next pre-operation boundary clears the prior publication
before codegen can reuse the table; nested calls use separate activation tables.
The same lifetime applies to retained/parked native stacks. Values addressed by
those descriptors are mutable, including call result scratch and discriminator
lanes. No native code may rewrite owned descriptors while they are published.

Ordinary `neoclr_gc_transient_v1` remains checked and clears the ownership marker;
mutable native clients must use it. Each replacement is validated again. Current
frame boundaries clear the marker, and leave clears the complete frame. Worker
boundary setup already replaces the current frame's transient state; ancestor
flags remain untouched. No frame size change, new allocation, collection trigger,
fault-observation omission, or plan-string validation shortcut is introduced.
Diagnostic probe lowering retains its original hook and full checks.

New native objects require the matching runtime symbol. Old objects continue to
link and use the checked hook. This is private native codegen/runtime machinery;
Raven Runtime Contract options, public managed APIs and temporary CLI representation
are unchanged. Native metadata ownership can later replace JSON plans with typed
maps, but no image-registration/immutable-plan contract is implemented here.

This reuses the .NET/CLR comparison in
[allocation and metadata research](../../../../docs/design/allocation-metadata-runtime-comparison.md):
execution metadata has a separately enforced lifetime from mutable root values.
neoCLR's explicit private hook is its implementation choice, not a claim to copy
CLR stack maps. Its cost is an ownership obligation and matching-runtime linkage;
the checked native path remains available.

## Validation and measurements

The retained reports record commands, source hashes, object identity and results.
A focused native regression publishes a root slot, changes the value, parks its
frame, collects through a child, verifies the new object survives and the old one
is reclaimed, then clears the boundary and verifies reclamation. Malformed owned
publication and ordinary checked replacement abort. The codegen object test checks
the new import while retaining diagnostic-mode phase-map assertions.

Additional qualification reuses checked-root rejection cases, forced-GC HTTP,
recompiled shared-worker graph/fault consumers and async consumers. Counters and
capacity are separate runs; neither diagnostics nor builds overlap timing.

Plain-text per-additional-response diagnostics preserve 3,676 boundaries, 25,892
frame visits and 7,546 fault observations. `validate_storage` descriptor checks fall
from 44,462 to 8,474 (80.9%). This excludes publication's inline descriptor scan,
which remains. Plan validation length is unchanged at 429,629 logical bytes.
Counts identify removed work, not its CPU share or total tracing work.

Use the preceding `transient-handoff/measure.py` with baseline
`target/transient-handoff-build/candidate.o` and candidate
`target/owned-transient/candidate.o`. Both link the same new runtime; the only
application difference is the owned-transient import. One/four isolated owners,
32 connections, two-second warmup and ten-second measured windows run in
forward/reverse order. CPU per response includes startup/warmup/drain and subtracts
client CPU. Short desktop trials are not sustained capacity or Kestrel parity.

Qualification commands (with the recorded Xcode SDK environment):

```sh
cargo test --release -p neoclr-codegen --test native_gc --test root_probes
python3 benchmarks/http-server/primitive-comparison/owned-root-storage/validate.py --output NEW_DIRECTORY
python3 benchmarks/http-server/primitive-comparison/free-index/validate.py --output NEW_DIRECTORY
python3 benchmarks/http-server/primitive-comparison/context-reuse/qualify.py \
  --build target/primitive-comparison-build/build.json --output NEW_DIRECTORY
```

The initial debug test build stalled and was terminated; the release-mode test
run passes all 19 tests. A one-second macOS sampling attempt also stalled and was
stopped, so it supplies no diagnostic conclusion. System media-analysis and security
services were active before timing; they were not stopped. Qualification is local
macOS ARM64, not Windows execution evidence. Header wording was clarified after the
counter run; runtime executable source and generated objects did not change.
Paired Raven documentation: `e4a8336b6` on `codex/neoclr`; Raven main and compiler
binary remain unchanged.

### Paired capacity results

| Isolated owners | Baseline req/s | Candidate req/s | Change | Baseline CPU µs/response | Candidate CPU µs/response | Change |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2,504.75 | 2,606.25 | +4.1% | 396.07 | 381.33 | −3.7% |
| 4 | 6,917.0 | 7,604.5 | +9.9% | 513.44 | 473.34 | −7.8% |

These are means of two samples per configuration in eight forward/reverse trials.
Both candidate samples outperform both respective baseline samples on throughput
and CPU/response. Tails remain variable (one-owner p99 spans 13.7–28.6 ms for the
baseline and 18.8–21.3 ms for the candidate); no uniform latency gain is claimed.
System media-analysis/security activity remained during timing. The benefit supports
retaining the change, but the magnitude requires confirmation on a controlled host.
No new JSON capacity or peer-server parity claim is made. See measurement.json.gz.
