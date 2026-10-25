# Native candidate-state borrow prototype

2026-10-25, following [whole-request cost attribution](whole-gap-results.md).
This is a native-only, opt-in prototype, not compiler integration or default admission.

## Contract and ownership

`NEOCLR_LIVE_STORE_HANDLE` exposes private experimental borrow/store hooks. The
borrow is the address of existing owner-thread candidate state, not a cached context,
heap boundary, epoch number, validated object or permission to skip checks. Every
store still reads current candidate activity/context and validates current heap
ownership, arena shape, destination, width and object bounds. Heap ownership remains
in its separate service. Existing cache revocation, marker synchronization, barriers,
cancellation and rejection paths remain in place.

The ordinary `experiment_gc_store` entry resolves current TLS and delegates to the
same body when the option is enabled. The borrowed entry substitutes an explicit
candidate pointer only within that body; helper services may still resolve TLS.
No global state merge or per-object allocation is introduced. Split candidate TLS
is explicitly incompatible; separate store TLS remains supported but gains less reuse.

This is a trusted generated-code/fixture contract: the pointer must stay on its
originating native thread, never escape to managed code or outlive the thread, and
must not enter a migrating continuation. It follows the existing root-state handle
contract in `runtime/native/gc/{root-probe.c,execution-state.h}`. A null handle is
rejected, but arbitrary/foreign handle validity is NOT dynamically established.
Passing a foreign candidate pointer with the current thread's valid context is
outside the contract; foreign-context rejection tests do not prove otherwise.
Only compiler-proven nonmigrating activation lifetimes may eventually use it.

Because no dynamic values are cached, native fixtures retain the pointer across
epoch transitions and owner release/reacquisition while all current checks still
execute. Generated admission should initially use an activation-bounded borrow,
matching existing root-state handle restrictions. The live host capability query
has not changed and does not advertise this feature. Ordinary compiled code still
uses the checked store entry. No Runtime Contract, Raven, JIT/AOT emission or public
API changes are included in this slice.

The design reuses existing [.NET/Go metadata research](allocation-index-results.md)
and neoCLR's qualified root-state contract: share current state addresses under an
explicit lifetime, not stale validity facts. It does not change .NET-shaped semantics.

## Native qualification

The full existing native suite passes with the checked wrapper selecting the handle
body, including mutation/deletion and stale-proof negative controls, optimized and
UB/bounds-sanitized fixtures. Dedicated retained-handle store and exact-base fixtures
pass in both build modes: range/width checks, outside-stack classification, active
old/new objects, cancellation, stale/reused allocations, owner reacquisition and two
independent owner threads with private counters and foreign-context rejection.
Each owner acquires its own pointer once. A deliberately removed heap-ownership
check fails with SIGABRT, showing that a candidate borrow is not a substitute for
heap ownership. These tests do not establish arbitrary cross-thread handle safety.

An initial fixture rename lost C's implicit return-at-end-of-main rule; the adapter
now explicitly returns zero. A non-profiled allocation-index build also exposed
calloc/free declarations appearing too late. The generator now includes stdlib.h
before index helpers. Both microbenchmark variants and the final retained-handle
sanitized fixtures compile with that correction. The earlier ASan host-initialization
gap remains open; no ASan evidence is claimed here.

## Amortized CPU experiment

Six alternating checked/borrowed pairs each perform one million outside-heap stores
and one million inactive-epoch heap stores: 24 million checked writes in total.
Final payload values, collection and heap release are verified. Both executables
use the same native source and flags except the handle define and corresponding
call contract; process startup and the one borrow acquisition are outside timing.
No marker is active in this microbenchmark. No additional heap/TLS allocation is
introduced; a caller must retain one opaque pointer and pay acquisition/register or
stack-lifetime costs, which this benchmark intentionally does not quantify.

Median paired owner CPU ratios (borrowed / checked):

| Path | Ratio | Observed reduction |
| --- | ---: | ---: |
| Outside heap | 0.8783 | 12.2% |
| Inactive-epoch heap | 0.9146 | 8.5% |

This is motivation for compiler qualification, not an HTTP speedup, allocation
reduction or guaranteed benefit for short activations. Compiler integration must
account for acquisition once per activation, fault cleanup, nested calls, park/resume,
worker/migration exclusion and host compatibility. Measure full plaintext/JSON and
standalone JIT/AOT with guarded and ordinary sync controls before promotion. Existing
compiled HTTP evidence does not yet exercise a retained store borrow.

## Reproduction and evidence

`store-handle-native.json.gz`, `store-handle-retained.json.gz` and
`store-handle-micro.json.gz` retain commands, outputs and hashes;
`store-handle-inputs.json` records the final sources. Run
`concurrent-live/validate.py --store-handle` with the allocation/exact/inline/sparse
flags from the manifest, `validate-store-handle.py --output <new-directory>` and
`measure-store-handle.py --output <new-directory>`. The retained SDK recipe is
required. Failed initial harness/build reports remain in
`target/concurrent-workloads/store-handle-{retained,micro}`.

Next implement a test-only compiler emission path only for existing admitted
same-thread activations, with explicit versioned host capability/binding checks.
Qualify it before considering any production contract. Concurrent default and
shared-worker acceptance remain open.
