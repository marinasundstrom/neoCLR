# Binary storage and transient execution maps

Development evidence, 2026-10-24. This continues the execution-state workstream in
[the bottleneck investigation](../../../docs/design/runtime-bottleneck-investigation.md)
and completes the count-header representation started by
[typed boundary maps](../../http-server/primitive-comparison/root-map/README.md).
The prior worker profile still placed 965 self samples in owned entry and 323 in
strlen; those are wall samples, not attributable CPU percentages.

## Contract and tradeoffs

Native-GC lowering emits eight-byte little-endian version/kind and slot-count
headers (`NGS1` storage, `NGT1` transient). Entry/publication checks the exact length,
magic and count rather than strlen. Existing address/width checks, ownership proofs,
root tracing and safepoints remain. No allocator or GC policy changes. The 104-byte
frame retains its layout; two reserved bits distinguish binary plan pointers.
Legacy string hooks and full diagnostic probes remain checked. Native-GC lowering
also avoids constructing rich field-layout JSON that execution subsequently discards.
Rich inspection of the plaintext program is byte-identical (22,182,209 bytes).

Headers are validated on **every entry/publication**, not admitted once per image.
The existing immutable descriptor lifetime remains the basis for reusing slot-table
validation. Mutable root values are never cached. This bounded format change avoids
an image registry and its unload/registration complexity; that admission cost and
per-invocation tables remain future work. New native objects require matching new
private helpers; old objects retain their helpers. ARM64/x64 are little-endian;
another endian target needs explicit porting. No managed API or Raven CLI/Runtime
Contract changes. Interpreter metadata and its separate Rust collector are unaffected.

The .NET comparison is CoreCLR's versioned GCInfo execution metadata, as recorded in
[the design research](../../../docs/design/allocation-metadata-runtime-comparison.md).
These headers are much simpler than PC/register maps: the collector still consumes
explicit address/width/root arrays. Benefits are less string work and smaller objects;
costs are private symbol/format coupling and discriminator maintenance, not a CLR ABI.

Plaintext object bytes fall from 8,450,192 to 8,339,456 (1.31%); JSON from 11,527,784
to 11,378,296 (1.30%). The 1,133/1,554 plaintext and 1,567/2,063 JSON storage/transient
schema strings disappear. These are object-file sizes, not measured resident memory
or managed allocation reductions. See `objects.json` for identities and counts.

## Paired plaintext capacity

Baseline is `c470640fd`; candidate changes native lowering and the matching root
helpers. Same managed inputs, host ownership, pacing and checked ten-byte response;
32 client connections, close-after-response, Go client GOMAXPROCS=2, two seconds
warmup plus ten seconds measured, two trials/configuration in opposing orders.
No builds or diagnostic profiles overlap these runs.

| Configuration | Requests/s before → after | Change | CPU µs/response before → after | Reduction |
| --- | ---: | ---: | ---: | ---: |
| AOT, one owner | 3,411.6 → 3,618.3 | +6.1% | 286.7 → 270.5 | 5.6% |
| AOT, four owners | 9,914.1 → 10,312.2 | +4.0% | 366.7 → 351.1 | 4.3% |
| Owned JIT, one owner | 2,071.2 → 2,141.7 | +3.4% | 479.2 → 465.3 | 2.9% |

AOT CPU covers server lifetime with client CPU subtracted. JIT CPU uses process CPU
between readiness and shutdown, excluding startup/teardown. Do not pool these CPU
scopes. Four-owner AOT uses isolated heaps, not the shared-heap stress topology.
Each candidate range is above its matched baseline; two short trials do not establish
confidence intervals or sustained peer parity. AOT one-owner p99 is noisy/mixed
(9.80/20.03 ms baseline, 11.31/9.34 candidate); four-owner and JIT tails improve in this
cohort. Earlier cohorts remain separate: machine/load variation means these rates
must not be compared directly with historical Kestrel runs or summed with earlier gains.
No fresh diagnostic profile or JSON capacity claim is made for this slice.

`summary.json` is reproduced by the existing two-tested paired summary tool.
`capacity.json.gz` and `jit-capacity.json.gz` retain all trials, commands and identities.

## Validation

- Two execution-metadata unit tests, six root-probe integration tests and six Windows
  object-emission tests pass. Windows runtime execution is not claimed.
- 230 native executions: 28 normal and 202 expected aborts. Optimized/sanitized,
  forced/paced and worker-hook variants cover kind/version/count/length/null checks,
  changed headers on reuse, invalid widths, unaligned headers, changing live roots,
  reclaiming replaced roots, parked roots, format-bit transitions and legacy checks.
- Newly generated shared-worker graph/fault stress passes optimized and sanitized
  execution: 32 graph and 32 fault invocations, 24,544/448 coordinated collections.
  The initial sanitized run fails `peak > 1` after the graph correctness checks:
  the scheduling-sensitive gate observed only one active participant. Preserve this
  failure in `worker-initial.json.gz`; three unchanged-binary replays each pass with
  four graph/three fault participants (`worker-rechecks.json.gz`). No assertion was
  weakened. The concurrency gate remains nondeterministic, so this is not a claim
  that every stress invocation passed.
- Five newly generated async consumers pass 20 optimized/sanitized and reuse/default
  configurations, 40 invocations total, including expected terminal-fault outcomes.
- Forced-GC plaintext and reflection-JSON HTTP each pass two responses in both AOT
  and JIT (four server invocations). Production-paced capacity is measured separately.

## Reproduction and limits

Runners reuse the qualified bundle/managed assemblies and recipes named in the raw
reports. They are local evidence harnesses, not clean-checkout installers. Save the
baseline codegen executable from `c470640fd` as `target/slot-maps/baseline-codegen`
before rebuilding. Recompile the retained plaintext and JSON recipes to
`target/slot-maps/plain.o` and `json.o`; `build.json.gz` records commands and hashes.
Use fresh output directories. Explicit Xcode 26.2 paths avoid an unrelated SDK mismatch.
Do not overlap compilation, diagnostics or other loads with capacity trials.

```sh
python3 benchmarks/execution/slot-maps/validate.py --output target/slot-maps-new/tests
python3 benchmarks/execution/slot-maps/consumers.py --output target/slot-maps-new/consumers
python3 benchmarks/execution/slot-maps/counts.py --sample plain --forced --requests 2 --output target/slot-maps-new/forced-plain
python3 benchmarks/execution/slot-maps/counts.py --sample json --forced --requests 2 --output target/slot-maps-new/forced-json
python3 benchmarks/execution/slot-maps/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/root-map/candidate.o --candidate-object target/slot-maps/plain.o \
  --client target/primitive-comparison-refresh/client --output target/slot-maps-new/capacity
python3 benchmarks/execution/slot-maps/jit.py --output target/slot-maps-new/jit-capacity
python3 benchmarks/execution/owned-root-chain/summarize.py \
  target/slot-maps-new/capacity/measurement.json target/slot-maps-new/jit-capacity/results.json target/slot-maps-new/summary.json
```

No unrelated suite or website build. No public API snapshot change is needed for
these private hooks. Raven compiler documentation is synchronized on `codex/neoclr`;
documentation commit `18a9c728f`; main and compiler binaries remain unchanged. Earlier object-trace prototypes remain
separate. Collection, TLS/state access, precise object tracing, metadata resolution
and the JIT layout gap remain open architectural work.
