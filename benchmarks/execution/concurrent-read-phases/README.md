# Distinguishing worker read phases from allocation-free operations

2026-10-25. Code inspection found that the compiled worker flag used
`candidate.is_some()` from the GC-point planner as its exclusive-lease predicate.
That planner covers allocating calls, construction, static storage and array/text
allocation. It does not cover ordinary field, array or indirect writes. Therefore
its shared phase was not safe to overlap with an unguarded live marker, even when
application workers wrote disjoint locations safely among themselves.

## Implemented contract

The native backend now uses a separate `worker_exclusive` predicate in both
emission and inspection reports. An allocating/GC point remains exclusive. Only
explicitly reviewed local operations and reads are shared; all other operations
are exclusive, including field/array/indirect stores, initialization, block/object
copies, provider calls and unreviewed operations. This fails conservatively for
new IL. Stack-local stores do not follow a borrowed pointer; writing through that
pointer is a separate exclusive operation. Some operations such as returns and
checked conversions remain conservatively exclusive pending further review.

This changes the experimental `--native-worker-safepoints` scheduling contract,
not the ordinary GC-point selection or managed language semantics. The new
`neoclr_gc_worker_read_phase_map_v1` import stamps `NEOCLR_GC_WORKER_READ_PHASE`
on published frames. Legacy worker map callbacks retain their older tag and remain
supported for synchronous execution. A future live-epoch scheduler must require
the stronger contract, including relevant caller frames; accepting an old callback
is not admission to concurrent marking. Rebuild new native images with a host
exporting the new callback. No default collector or HTTP-worker admission changes.

The previous root/GC service design and .NET comparison remain applicable. The
concrete benefit is an explicit scheduling boundary that does not confuse lack of
allocation with lack of mutation. The cost is more exclusive transitions and,
with the current forced-collection fixture, more collections. This is a native
backend choice; it changes neither Raven Runtime Contract nor temporary CLI
encoding. Both repositories' bridge notes record ownership and rebuild needs.

## Evidence

- Focused root-plan tests cover indirect, field, array, block and initialization
  mutations, reviewed reads, conservative fallback, and independent report/GC-point
  behavior. The initial fixture omitted its required `out` annotation and then used an
  indirect opcode not admitted by the value profile. It now uses the supported
  `stobj` path; direct classifier checks retain conservative coverage of indirect
  opcodes. These fixture failures are separate from executable consumer evidence.
- `native-validation.json.gz`: ordinary and UB/bounds-sanitized native tests each
  check 400 legacy/new map publications, both lease kinds, retained roots and
  reclamation. Tags remain distinct across repeated switches on the same frame.
- `owner-validation.json.gz` and `control-validation.json.gz`: rebuilt compiled AOT
  graph/fault/cancellation/shutdown tests pass with the new import in service-owned
  collection and direct/copied-root synchronous controls, including sanitized
  builds. Removing copied roots and disabling dispatch still fail deliberately in
  the service variant. The graph/fault workload now performs 59,456/768 forced
  collections instead of 24,544/448: exclusive acquisition and forced collection
  are still coupled. No throughput or CPU improvement is claimed.
- `legacy-validation.json.gz`: a retained image importing the old worker map links
  against the updated native runtime and passes graph/fault/shutdown controls.
  It is not relabelled as a read-phase image.

The initial debug test process waited at `_dyld_start` before reaching the harness;
this was not counted as a test failure. A subsequent release-mode build hit a linker/SDK mismatch; the final command
selects the known Xcode macOS 26.2 SDK explicitly. The release-mode run supplies
the focused compiler evidence. Existing TSan limitations remain open; sanitizer success does
not prove all interleavings.

## Next integration

Separate exclusive mutation permission from forced collection scheduling, then
coordinate a live epoch with read phases and finish before any exclusive mutation,
allocation, host heap operation or shutdown reclamation. All provider/classification
paths need to remain within that admission contract. Wider overlap with concurrent
writes still requires complete registered-barrier integration. JIT/AOT application,
HTTP throughput, CPU, allocations, memory and pause acceptance remain outstanding.

```sh
cargo test --release -p neoclr-codegen --lib gc_points::tests
python3 benchmarks/execution/concurrent-read-phases/validate-native.py \
  --output target/read-phase-native-repeat
python3 benchmarks/execution/concurrent-participant-roots/validate-pool.py \
  --collection-owner --output target/read-phase-owner-repeat
```

Use fresh output directories and rebuild the native compiler before compiling
consumer images. The report archives retain exact tested inputs/commands.
