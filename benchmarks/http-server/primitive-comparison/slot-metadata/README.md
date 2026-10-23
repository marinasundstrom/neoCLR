# Compact storage and transient metadata — 2026-10-23

Continue the [CLR-guided execution/diagnostic separation](../allocation-metadata/README.md)
from point payloads to function-storage and transient payloads. Native GC now emits
`neoclr-gc-storage-v1` and `neoclr-gc-transient-v1` JSON with `slotCount`. These are
small descriptive payloads; the existing explicit slot tables remain authoritative.
Diagnostic probe mode retains the complete storage entries and transient layouts.

The backend still emits identical slot addresses, 4/8-byte widths, discriminator/
borrow flags, transient phases, stack-lane counts and ordered spill maps. Root
validation reads current descriptors at every publication; no validation caching,
new pointer-lifetime assumptions or private runtime ABI change is introduced.
Only native-GC frame consumers expecting full layout JSON need to switch to full
inspection/diagnostic mode. This remains a temporary JSON bridge, not CLR GCInfo
or the proposed versioned binary image-owned maps.

The CLR comparison is architectural: runtime consumers should use purpose-specific
execution data, while richer metadata serves inspection. See the primary sources
and allocation-context discussion in the previous experiment. This does not copy
CLR's type layout, allocator or stack unwinder. The remaining metadata string
checks and explicit publication work remain optimization opportunities.

A [paired-root admission experiment](../root-pair/README.md) preceded this change.
It combined two heap walks during async resume but did not improve measured CPU/
throughput; it was reverted rather than adding an unsupported private API.

## Validation and measurement

Focused codegen contracts cover empty, ordinary and wide slot counts, schema
identity and lossless diagnostic projection, alongside the prior spill-map test.
Existing native probe tests exercise real spills, constructor roots, faults,
reentry and borrowed-slot handling. Forced-GC HTTP and compiled shared-worker
stress runs qualify the emitted metadata against the runtime collector.

The comparison uses the **preceding compact-point object** as its baseline, not
the older rich-point object. Both variants link against the same current runtime,
1 MiB heap and 16 Serve lanes per isolated owner, with 256/1,024 pacing. The same
source assembly and EOF-aware client drive 32 connections for two seconds warmup
and six measured seconds. One/four-owner cases run forward and reverse; no tests
or builds overlap timed trials. CPU/response includes startup, warmup, drain and
shutdown. All responses and process exit status are checked.

These are short local closed-loop comparisons. They are not sustained-rate limits,
RSS measurements, cross-platform qualification or evidence of parity with other
servers. Compare paired variants, not absolute throughput from different desktop
sessions. The public HTTP API remains unchanged; no website build is required.

## Reproduction

Select the same macOS SDK and matching source bundle as the parent baseline.
Use fresh output directories, and finish builds/tests before measurement.

```sh
CARGO_INCREMENTAL=0 cargo build --release -p neoclr-codegen
CARGO_INCREMENTAL=0 cargo test --release -p neoclr-codegen --lib execution_metadata_tests
CARGO_INCREMENTAL=0 cargo test --release -p neoclr-codegen --test root_probes
python3 benchmarks/http-server/primitive-comparison/slot-metadata/measure.py \
  --build target/primitive-comparison-build/build.json \
  --client target/primitive-comparison-eof/client \
  --baseline-object target/plain-compact-metadata/candidate.o \
  --output target/plain-slot-metadata
python3 benchmarks/http-server/primitive-comparison/allocation-metadata/forced.py \
  --build target/primitive-comparison-build/build.json \
  --object target/plain-slot-metadata/candidate.o --output target/plain-slot-forced
```

Use native-worker-pool/generated.py --parallel with the matching bundle/libraries
and invocation-context-native-validation.json recipe for shared-GC qualification.
The Raven compiler itself and bundle 1128b98cf are unchanged. The backend owns this
lowering; both bridge documents record the schema migration and future replacement.

## Results

| Variant | Owners | Requests/s (two trials) | CPU µs/response | p99 ms |
| --- | --- | --- | --- | --- |
| baseline | 1 | 720.0, 736.8 | 1,387.2, 1,365.6 | 47.40, 56.64 |
| baseline | 4 | 2,454.3, 2,432.0 | 1,570.5, 1,574.7 | 19.73, 24.48 |
| candidate | 1 | 760.8, 759.5 | 1,314.1, 1,312.8 | 47.64, 56.22 |
| candidate | 4 | 2,616.5, 2,634.0 | 1,474.3, 1,466.9 | 22.77, 25.91 |

Mean throughput improves about 4.4% with one owner and 7.5% with four; estimated
CPU per response falls about 4.6% and 6.5%, respectively. Four-owner p99 is higher
in these trials, so this is a throughput/CPU improvement with a tail-latency tradeoff,
not an across-the-board latency win. The application object shrinks from 11,628,216
to 9,569,264 bytes (17.7%); runtime heap/RSS savings have not been established.

Validation passes: two focused metadata contracts, five native root-probe tests,
four forced-GC sanitized HTTP requests with clean shutdown, and optimized/UBSan
shared-worker tests. Each worker build completes 32 healthy and 32 fault calls,
24,544 coordinated graph collections and 448 fault collections, with up to four
concurrent graph participants. JSON evidence records exact commands and inputs.

Integration: neoCLR main development; paired Raven codex/neoclr documentation
commit 1f05922e2. Raven main and compiler behavior remain unchanged.
