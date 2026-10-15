# Native structural type qualification

Development branch, 2026-10-15. This is integration evidence, not a release claim.

## Tuple AOT cost comparison

Runtime implementation: `4fe78429b`. Tool hashes and machine details are in the
raw result files. On Apple M1, the two-integer structural tuple and nominal record
both emit the **same 800-byte object**; sixteen-integer versions both emit the
**same 1,352-byte object**. Full object hashes match in each pair, not merely sizes.
Neither has undefined symbols, so these closed workloads cannot call a heap
allocator. Tuple construction uses inline slots with the same ABI as nominal records.

| Elements | Structural median | Nominal median | Structural / nominal |
| --- | ---: | ---: | ---: |
| 2 | 3.9608 ns/call | 3.9624 ns/call | 0.9996 |
| 16 | 13.124 ns/call | 13.122 ns/call | 1.0002 |

These differences are noise, not evidence of a speedup. Identical object code is
stronger evidence of equivalent cost for these workloads. The increased cost with
size is shared aggregate copy/call work and an optimization candidate for both
representations. Do not introduce a tuple-only optimization without measuring the
shared codegen path first.

Each workload constructs a value, passes it through Echo, copies it into a local
and reads the first and last elements. Inputs vary and the native host verifies
the accumulated result. Nine interleaved samples follow warmup, with five million
calls per pair sample and one million per sixteen-element sample. Timing uses the
monotonic clock around the loop, excluding compilation, linking and process launch.
The backend uses its normal Cranelift settings; the C timing host uses `-O2`.
Emission time is separately retained and is not a compilation benchmark.

Raw evidence: [pair](tuple-pair-results.json), [sixteen](tuple-16-results.json).
Exact generated inputs are retained next to these files. Reproduce with a matching
macOS ARM64 SDK (this host required Xcode's MacOSX26.2 SDK):

```sh
python3 docs/experiments/native-structural-types/benchmark-tuples.py \
  --codegen target/debug/neoclr-codegen --output target/tuple-cost-pair
python3 docs/experiments/native-structural-types/benchmark-tuples.py \
  --codegen target/debug/neoclr-codegen --elements 16 --iterations 1000000 \
  --output target/tuple-cost-16
```

The script preserves previous output directories. A flat nominal record currently
admits at most sixteen fields; larger comparisons need equally nested layouts.
There is no packed-layout, cache-pressure, managed-reference/GC, boxing, union,
interpreter or JIT qualification in this result. These remain explicit follow-ups.

## Direct JIT tuple execution

The shared JIT backend also executes both workloads successfully, with one million
calls per sample, nine interleaved measured samples after warmup and checked
results. The optimized Rust harness measured approximately 3.94 ns/call for the pair
and 13.9 ns/call for sixteen integers in both representations. These are comparable
within noise. Timing includes status checks/checksum but excludes compilation and
finalization. These measurements do not directly compare JIT and AOT speed because
the timing hosts differ. [Raw JIT samples and hashes](tuple-jit-results.json).

```sh
cargo test --release -p neoclr-codegen --lib structural_tuple_jit_costs \
  -- --ignored --nocapture
```

This explicit ignored performance test owns JIT code until synchronous invocation
finishes, then frees it. It validates direct backend construction, copying and
calls; the full managed host, reference-bearing tuples and collector stress remain
unqualified. Native intersections progress in the following slice; native unions remain a backend obligation.

## Native intersection reference views

The next backend slice preserves intersections as one checked reference slot.
Selection/specialization retain the complete bounds. Static assignment requires
all expected bounds; dynamic casts accept only concrete classes satisfying every
bound through explicit nominal conformance. Interface dispatch projects a guaranteed
nominal view. No member-name matching or carrier allocation is introduced. This
extends CLR-style explicit interface reference views with a conjunction signature;
it does not introduce a trait implementation model.

`intersection-views.neoil` and the focused console test compare AOT and interpreter
results/diagnostics across eight modes, including nulls, identity, interface calls,
managed output slots, missing interfaces and unrelated casts. The test repeats with
the second implementation removed, and with an extendable class/interface bound.
A closed-to-extension class bound is rejected before emitting an object file.
Tracing recipes retain one strong reference; collector-stress and dedicated JIT
intersection execution still need qualification. The failed-cast path uses native
InvalidCast status 11 and its standard message; legacy nominal cast behavior stays
unchanged. Native structural union storage remains unsupported.

## Native reference-alternative union storage

The next slice represents unions of nominal reference alternatives with one payload
reference slot. This is a structural value representation: no synthetic nominal
union and no reference identity for the union container are introduced. The pointed
to object's tag provides the concrete identity for any-alternative membership.
Selection and specialization traverse native union signatures; assignment from a
union requires every possible alternative to satisfy the destination contract.
Extraction remains checked and explicit when a guarantee is absent.

The console test derives union signatures from `intersection-views.neoil`, inserting
an explicit View extraction before dispatch and an Echo function returning the union.
A further run exercises implicit admission of a conforming reference into a union
parameter. It compares eight modes for each of
four implementation sets: both bounds, just the first, just the second and neither.
The cases cover identity-preserving copies, calls, output slots, nulls, accepted and
rejected casts and missing membership. A mixed Int32/interface signature is rejected
before object emission with a tagged-storage diagnostic. A separate projection test
ensures whole-union boxing cannot be rewritten into an Object payload cast.

This extends CLR-style reference casts with a structural union value contract. The
benefit for reference-only unions is a single slot with no carrier allocation; the
cost is runtime membership checks when static guarantees are insufficient. Numeric
or nominal value alternatives require tagged inline storage and conversions, still
unimplemented. Existing scalar-box projection happens before typed aggregate
lowering, so simply enabling mixed unions would preserve avoidable heap allocation
or lose union identity. That pipeline must be addressed before the mixed path is
admitted. Native reflection, array elements, JIT/collector stress and full Task
consumer qualification remain open. No general performance claim follows from the
single-slot layout test.
