# Typed native boundary maps — 2026-10-24

## Contract

Native GC and worker-boundary lowering now emit an eight-byte execution header:
little-endian `NGC1` magic/version followed by a 32-bit lane count. The explicit
published lane array still carries the roots; numeric holes are still zeroed and
all existing spill selection is unchanged. The collector does not interpret JSON
plans or use this header to discover pointers. Runtime admission checks header
length, version and count at each publication, without strlen/marker searches.
Storage and transient plans retain their checked JSON paths in this bounded slice.

New private imports are `neoclr_gc_stack_map_v1` and
`neoclr_gc_worker_stack_map_v1`. They share snapshot publication, ancestor validation,
collection and worker logic with the existing JSON entry. Legacy native callers
and diagnostic-probe lowering continue using the checked string API. A native frame
with `NEOCLR_GC_TYPED_PLAN` has binary `plan` bytes, not a C string; private frame
readers must discriminate that flag. Returning to the JSON hook clears it. Worker
state masks exclude the format bit while retaining exclusive/nonexclusive behavior.
Frame size stays 104 bytes; managed object headers and APIs are unchanged.

New objects require the matching runtime symbols. Older objects continue to use
legacy JSON validation. There is no global cache, pointer-identity validation cache,
new allocation or assumption that mutable externally supplied headers never change.
Unaligned headers are read with memcpy and validated again. Supported native targets
are little-endian ARM64/x64; another endian target needs an explicit format port.

Execution images no longer retain a human-readable boundary spill list. Full layout,
spill order and pre-operation plans remain in `--inspect`, and diagnostic probes
remain lossless. A current/retained-compiler inspection comparison is byte-identical
(22,182,209 bytes); the retained binary predates transient-ownership work, while
its inspection contract remains the same. This is not metadata stripping from the
consumer-facing inspection contract.

## Comparison with .NET

CoreCLR's [GCInfo definitions](https://github.com/dotnet/runtime/blob/main/src/coreclr/inc/gcinfo.h)
make the execution format and version explicit and discuss diagnostics compatibility.
The version is tracked by method ranges rather than repeated for each method.
Its [threading design](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/threading.md)
also distinguishes safe-point metadata from the act of polling for collection.
neoCLR follows the separation of execution metadata from textual diagnostics;
this is not CLR GCInfo or a register/PC stack map. Its compiler already spills
explicit lanes, so this header only checks the publication shape.

Per-boundary magic/count costs eight bytes and a fixed validation read. It is a
simple provisional format with no image registry or unload bookkeeping; future
image-level version admission could amortize it. The immediate benefit is smaller
execution data and no repeated boundary-string scanning. The compatibility cost is
new private imports and discriminated frame metadata. No safepoint is removed.

## Validation

Thirty focused release-mode Rust tests pass: 14 native-GC, six root-probe, six
Windows object-emission/admission, two metadata encoding and two borrowed/erased
root consumers. Windows results are object generation on macOS, not Windows runtime
execution. Twelve optimized/sanitized map cases exercise version/count/length/null/
current-head rejection, mutation after successful use, unaligned bytes, changing
root values, old-value reclamation and JSON fallback. Existing checked-native
rejection cases remain valid.

Recompiled shared-worker graph/fault stress passes optimized/sanitized checks;
40 async-consumer invocations pass across reuse/default contexts and sanitized/
optimized builds. Forced-GC plain HTTP checks eight responses. Paired small/large
paced plain and reflection-JSON diagnostics check 64 responses. Native codegen
symbol checks verify the new imports and absence of `requiredSpillLanes` text from
the native object; diagnostic emission retains its original hook/phase behavior.

## Work counts

Subtract two four-request invocations from two twelve-request invocations and divide
by the 16 additional responses. Startup/teardown is removed, but scheduling and GC
budget rounding can affect collections. These are sanitized diagnostics, not timings.

| Per additional response | Plain | JSON |
|---|---:|---:|
| Boundaries | 3,676 | 5,535 |
| Allocations | 208 | 457 |
| Payload bytes | 15,995.5 | 26,173.5 |
| Frame visits | 25,892 | 43,753 |
| Storage-descriptor checks | 8,474 | 13,710 |
| Remaining plan strlen input bytes | 222,669 | 346,634 |

Boundary/slot publication and allocation counts match the preceding workload shapes.
Plan strlen input lengths fall from 429,629/659,629 to 222,669/346,634 (48.2%/47.5%).
This excludes the new fixed header reads and marker-search work, and is not physical
memory traffic or CPU attribution. The earlier JSON object predates owned transient
descriptors; its descriptor-check reduction is not attributable to this map change.
No new JSON capacity claim is made. Collection increments here are 3.875/6.0, not
identical to earlier cohorts. The common policy and forced-GC checks remain intact.

The plain object shrinks from 8,650,888 to 8,450,192 bytes (2.3%). Storage/transient
JSON remains a separate possible follow-up.

## Paired measurement and reproduction

Baseline uses root-probe.c from `bd1f6649d` and the retained owned-transient object;
candidate uses the current runtime and newly generated object. This compares the
complete change, avoiding a baseline slowed by the new common implementation.
The GC implementation, heap/context policy, host, 32 connections and ten-byte checked
response are shared. One/four isolated owners run forward/reverse with two-second
warmup and ten-second measured windows. CPU/response includes startup/warmup/drain,
subtracting client CPU. No builds, tests or diagnostics overlap the timed load.

```sh
cargo test --release -p neoclr-codegen --test root_probes --test native_gc --test windows_stack
cargo test --release -p neoclr-codegen --lib execution_metadata_tests
cargo test --release -p neoclr-codegen --test console native_gc_preserves_
python3 benchmarks/http-server/primitive-comparison/root-map/validate.py --output NEW_DIRECTORY
python3 benchmarks/http-server/primitive-comparison/root-map/measure.py \
  --build target/primitive-comparison-build/build.json \
  --object target/owned-transient/candidate.o --candidate-object target/root-map/candidate.o \
  --client target/primitive-comparison-refresh/client --output NEW_DIRECTORY
```

Use the recorded Xcode SDK and matching library inputs. Raw compressed reports and
work-summary.json preserve commands, object/source hashes and results. The rich
compiler stdout itself is represented by its hash; the inspect comparison records
both compiler identities. No managed API snapshot or unrelated website build is needed.

## Result: retained

| Isolated owners | Baseline req/s | Candidate req/s | Change | Baseline CPU µs/response | Candidate CPU µs/response | Change |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2,830.90 | 2,943.80 | +4.0% | 349.01 | 334.31 | -4.2% |
| 4 | 8,327.20 | 8,655.80 | +3.9% | 436.51 | 419.28 | -3.9% |

Both candidate samples beat both respective baseline samples on throughput and
CPU/response. Means cover two samples per configuration in eight paired trials.
One-owner latency tails remain mixed (baseline p99 12.09/18.48 ms versus candidate
14.48/19.64 ms); four-owner candidate p99 is lower in these trials. These are short
local measurements, not sustained capacity, cumulative speedup or Kestrel parity.
The change is retained for CPU/throughput and smaller execution metadata.

Paired Raven bridge documentation: `40e459aac` on `codex/neoclr`. Raven main and
the Raven compiler binary are unchanged. Historical experimental hosts with explicit
native export allowlists need matching new symbols before rebuilding consumers; no
new Windows dynamic-host validation is claimed.
