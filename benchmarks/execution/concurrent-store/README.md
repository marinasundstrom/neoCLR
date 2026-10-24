# Generated scalar stores for live GC — 2026-10-24

The private `mutation_store` lowering pass emits the experimental noncollecting
`experiment_gc_store` hook from scalar Cranelift stores. It is exercised by focused
JIT/AOT consumers and an **internal opt-in in the ordinary value compiler**.
There is no CLI option. See the managed integration qualification below.
Synchronous GC remains the default. This is a correctness integration step, not a
performance result or admission of managed HTTP handlers.

## Contract and implementation

The pass requires a 64-bit entry context parameter and an I32 status return. It
validates all memory-writing instructions before modifying the function. Supported
scalar stores preserve widths and low bits; floats are bitcast, preserving signed
zero and NaN payloads. Offsets are folded into the destination. Each failed hook
call returns its status before subsequent stores execute. Proven `stack_store`
instructions stay direct. Generic stores through `stack_addr` and checked constant
`iadd_imm` chains also stay direct when their full extent is inside that slot.
Known out-of-slot stores are rejected before edits; loaded pointers, block
parameters and arbitrary arithmetic do not gain a stack exemption. Explicit-endian, vector, atomic and other unadmitted
writers are rejected; callees and native providers require separate admission.

This provenance handling matters before frame publication: the current value
compiler uses generic stores for argument copies before linking its GC frame.
Making those stores fallible would introduce an exit before frame initialization.
The pass must precede root-frame return cleanup instrumentation, so its added fault
return also unlinks frames. The isolated consumers have no compiler root frames;
the framed entry point now checks ordering and is qualified below. Ordinary pipeline
admission now has a restricted internal integration described below.
The hook cannot collect, including when an overflowing barrier buffer cancels the
marker. Expression temporaries need not be newly published at each store.
See [the native boundary](../concurrent-live/store-boundary.md).

Dynamic destination classification now indexes old allocations during an active
epoch; new/inactive destinations still walk allocation descriptors. See the
[paired lookup experiment](../concurrent-live/store-index-results.md). This is not
a qualified production fast path.
Calls, checks and extra branches also cost CPU/code size. Preserve proven storage
provenance and measure indexed byref lookup before admitting this broadly.

## Reproduction and scope

```sh
cargo test -p neoclr-codegen --lib mutation_store --no-run
# Substitute the test executable printed by cargo:
python3 benchmarks/execution/concurrent-store/validate.py \
  --runner target/debug/deps/neoclr_codegen-<hash> \
  --output target/concurrent-store/qualification
```

The harness emits ARM64 Mach-O and x64 Windows COFF objects and verifies the hook
import. It links the ARM64 object to the generated live collector, runs optimized
and UBSan/bounds consumers, then links a native fixture dylib and invokes it from
JIT-generated code. Windows qualification is object emission only.

The scalar test covers six types and six patterns, including signed zero and NaN
payloads. Mock-hook JIT tests additionally reject the first or second store and
verify status propagation and untouched bytes. The native consumer checks 100
cycles of deleted-edge retention, inserted-edge transitive retention, unrelated
object reclamation, invalid destination rejection and final empty-heap collection.
It gates the marker before mutation to make barrier necessity deterministic; this
is not broad scheduling stress or proof over every interleaving.

Ordinary compiler output, native provider writes, arrays, suspended tasks and HTTP
handlers are not admitted by this pass alone. Existing native gate/scalar tests
cover overflow and narrow writes; this slice does not replace those tests or close
the documented host race-detector gap. Runtime Contract configuration, Raven source
semantics, temporary CLI metadata and public APIs are unchanged.

## Design comparison and next gate

Reuse the [CLR/Go mutation-barrier research and writer inventory](../concurrent-mutation-audit/README.md).
The shared purpose is compiler/runtime cooperation for concurrent tracing. neoCLR's
current conservative payload scanner requires scalar-word safety, unlike a barrier
limited to semantically typed references. The resulting broader hook coverage is a
provisional cost of that representation, not a new managed semantic contract or a
claim of improvement over CLR. Native metadata/codegen must eventually carry enough
layout/provenance information to narrow this work safely.

Next admit a closed set of generated functions and native providers with an explicit
matching host capability, including construction/publication and return cleanup.
Only then run managed standalone workloads and plain/JSON HTTP with throughput,
process CPU, pauses and memory measured separately. This experiment does not justify
a default switch or predict application throughput.

## Recorded outcome

`validation.json.gz` records successful optimized/UBSan AOT runs and the real-host
JIT test (100 liveness/reclamation cycles in each run). `compiler-tests.json.gz`
records three focused lowering tests and two existing execution-metadata controls;
the host-dependent JIT test is skipped by the ordinary filter and explicitly run
by the harness. `native-gc.c.gz` and `stores.o.gz` retain generated inputs; the
validation report includes input hashes. No performance measurements were taken.

The initial AOT fixture omitted PIC and failed linking imported-hook relocations
(`initial-link-failure.json.gz`). Enabling PIC, as production AOT does, fixes it.
The linker still notes the test object lacks a platform load command and assumes
macOS; this fixture is not a distributable native image.

An earlier newly built Rust test runner stalled before main at `_dyld_start`;
`loader-sample.txt.gz` retains the sample. A stable byte copy, ad-hoc signed, ran
successfully; the original waiting processes also later finished. This does not
establish a signing root cause or a collector failure. The recorded final runner
is the stable signed copy. No host security policy was disabled.

## Stack-provenance qualification

The follow-up `stack-validation.json.gz` and `stack-compiler-tests.json.gz` record
qualification of the stack exemption. A JIT consumer stores through a stack address
with constant displacement, reloads the value and copies it through a caller-provided
pointer. Exactly one hook invocation remains, including when that byref write is
rejected. Negative and over-end stack extents fail before any IR edits. This proves
call elimination for the tested compiler pattern, not application CPU savings.
Unknown provenance still pays dynamic classification. Framed ordering/cleanup qualification follows below; complete writer admission
and ordinary pipeline integration remain open.

## Framed lowering and cleanup qualification

`lower_framed` accepts the compiler instruction that completes frame linkage and
saved-state initialization. Before editing, dominance analysis requires that
instruction on every path to a fallible store or return. Proven stack stores may
precede it. A caller must supply the actual initialization-complete instruction;
this check does not authenticate arbitrary imports or replace host/image admission.
Unreachable paths may be conservatively rejected. The isolated unframed entry
remains for native fixture functions that do not publish compiler frames.

Return cleanup is factored from `gc_probe::Probes::finish` into a shared private
helper without changing its emitted operations. Tests use that production helper
after lowering. Mock JIT entry/leave hooks verify exactly one balanced entry/leave
on normal return and first/second-store rejection, for both legacy frames and
saved-state frames. Late initialization and a branch bypassing initialization are
rejected without IR changes.

The real native fixture now publishes an empty generated frame with the runtime's
state-entry hook, saves its handle at offset 104 and leaves through the production
cleanup sequence. It checks that the runtime root-chain head is restored on success
and rejected writes before collection resumes. AOT runs both unframed and framed
100-cycle consumers; real-host JIT runs the framed 100-cycle consumer. These frames
have no managed slots: wider root-value, nested/suspended-frame, native-writer and
full application coverage remain required. This is correctness evidence, not a
throughput result or enablement of concurrent GC for ordinary compiler output.

`framed-validation.json.gz` records passing optimized/UBSan AOT (200 cycles each)
and real-host JIT (100 framed cycles). `framed-compiler-tests.json.gz` records seven
lowering tests plus two metadata controls; the host-dependent test is run separately
by the harness. `framed-stores.o.gz` retains the new object, and generated collector
source is byte-identical to `native-gc.c.gz`. The initial real-runtime fixture passed
a null table for zero storage entries and aborted as required by frame entry;
`framed-initial-null-table.json.gz` retains that failure. Supplying a non-null empty
table corrects the fixture without weakening runtime validation.

## Call-closure gate

The private `mutation_admission::check_call_closure` checks completed IR against
all supplied generated bodies and an explicit native-provider ABI list. Each body
must match a non-import declaration. Direct calls must name either a supplied body
or an imported provider with an exactly matching signature, including calling
convention. Missing transitive bodies, duplicate bodies/contracts, unlisted imports,
forged call-site signatures and indirect/tail/exceptional calls are rejected.
It examines every supplied body, not only the entry or one level of callees.

The scalar and framed AOT fixtures and real native-host JIT fixture now invoke this
gate before defining code. Store lowering and cleanup still run first. The gate
is read-only and does not establish that lowering was applied, that a provider's
implementation is safe, or that the linker resolved an approved host. It is one
pipeline obligation, not complete image admission. A versioned host capability and
ordinary compiler integration remain required; no public switch enables live GC.

The provider list in the tests is constructed from explicit expected ABIs rather
than copied from emitted call signatures. New call-closure tests cover missing
callee bodies, missing/wrong providers, duplicated bodies, mismatched call sites,
indirect calls and tail calls. Runtime consumers exercise the existing scalar/frame
provider set after checking it; they do not admit application/native service APIs.

Backend-introduced libcalls also need validation after lowering. The AOT fixture
now checks its complete undefined-symbol set on both targets, not only presence of
the store hook. This fixture-level check is not yet a general image/link verifier.

`admission-compiler-tests.json.gz` records nine passing focused compiler tests;
`admission-validation.json.gz` records complete AOT import checks, optimized/UBSan
native AOT consumers (200 cycles each) and framed real-host JIT (100 cycles).
The generated object remains byte-identical to `framed-stores.o.gz`: this gate
adds validation without changing the fixture's emitted code. No timing comparison
was made during this qualification.

## Versioned experimental host handshake

The private `experiment_gc_live_capabilities_v1(uint32_t version)` query returns a
64-bit feature mask, or zero for unsupported versions. Version 1 requires bits for
noncollecting scalar stores, old/new candidate logging and owned nonmoving epochs.
Unknown additional bits are allowed. The query is side-effect-free: it allocates
nothing, starts no worker and does not publish roots or collect.

The JIT fixture checks the loaded library before binding providers or defining
code. The native AOT driver checks before invoking generated writers; a missing
query is a link failure. Providers still need explicit ABI/call coverage and trusted
implementation qualification. This declaration is a compatibility handshake, not
proof that an arbitrary native library implements the protocol or a security boundary.
It does not grant shared-heap, scheduler, array-provider or HTTP admission.

`host-validation.json.gz` records rejection of missing, legacy and partial-query
libraries, followed by successful native JIT and optimized/UBSan AOT consumers.
`host-compiler-tests.json.gz` records ten focused tests; two host-dependent tests
are explicitly executed by the harness. `host-native-gc.c.gz` retains the generated
collector including the query. Unknown-version rejection is checked by the native
consumer. No application throughput measurements or default switch occur here.

## Managed value-compiler integration

The internal `Options.experimental_live_gc_stores` flag now lowers ordinary managed
bodies before shared root-frame cleanup, then checks the complete supplied call
closure before defining those bodies. Default compilation leaves the flag off.
Native replacements, shared-worker safepoints, host bootstrap/services, process
arguments and initialization guards are rejected. The approved provider list contains
only scalar stores, state-frame entry/leave/publication and stack checks. Unknown
service imports fail closure validation. This is not general image admission.

`managed.neoil` is assembled and compiled through the ordinary value pipeline.
Its `Write` method calls `Put` twice with reference-valued output borrows. The native
consumer invokes the private compiled writer (through a test export for AOT), with
native-allocated storage and a published host root frame. It covers changing reference
roots, nested managed frames, invalid destinations, fault propagation and root-chain
restoration. It does not exercise managed constructors, the public entry wrapper,
allocation providers, arrays, suspended tasks or HTTP services.

Published collection now completes an active owned epoch before resuming managed
execution. The intercept precedes collection profiling to avoid counting nested finish
work twice; finish's phase guard avoids recursion. Capture establishes the pacing
baseline, so a paced publication does not immediately finish because of capture's own
collection. Explicit callers can query active state and must not finish an epoch twice.
Arena release still requires completion. Host feature bit 8 advertises this additional
boundary protocol; a host advertising only the earlier three bits is rejected by the
managed driver. The query is trusted compatibility information, not proof of safety.

`managed-validation.json.gz` records 100 cycles per forced/paced JIT run and per
forced/paced optimized/UBSan AOT run (600 managed cycles). Counter assertions verify
exactly two guarded-old heap writes during a paced epoch and two inactive/new writes
once forced collection has completed it. Each cycle checks reclamation and eventual
empty-heap collection. Existing scalar/framed consumers and incompatible-host controls
also pass. `managed-compiler-tests.json.gz` records 11 focused tests and four existing
root-map/root-plan controls; three host-dependent tests run explicitly in the harness.
`managed-native-validation.json.gz` retains the broader native suite. Generated
collector/object inputs are archived with the report's hashes.

Initial fixtures exposed an abort at the first managed collection boundary and an
unexpected immediate paced completion. The two archived `managed-initial-*-failure`
reports retain these failures; lifecycle integration and the capture baseline fix them.
The earlier fixture also needed explicit output-borrow contracts and reference-valued
arguments so the compiler could publish their roots. Tests do not prove all thread
interleavings; previously documented race-detector host limitations remain.

This follows CLR's compiler/runtime barrier cooperation, while retaining neoCLR's
provisional conservative scalar-store coverage and owned nonmoving heap. See the
existing mutation-audit research above. Raven semantics, Runtime Contract configuration
and temporary CLI representation are unchanged. Native layout/provenance metadata must
eventually replace conservative coverage where justified. neoCLR owns the private
lowering and host protocol; native-writer coverage, automatic epoch scheduling and host
entry integration are next prerequisites for HTTP measurement. No throughput result
or default switch is claimed by this correctness qualification.


## Managed object construction admission

The internal provider list now admits `neoclr_allocate_object_rooted_v1` with its
exact `(arena, type, bytes, output) -> status` ABI. This is the first allocation
provider admitted for ordinary managed bodies. Review of `text-arena.c` establishes
that it reserves fresh object storage, writes identity/initial zeros and returns
without callbacks or collection after reservation. Rooted reservation uses the
published compiler frame; constructor lowering passes a compiler stack result slot.
These constraints matter: arbitrary output aliases into old heap storage are not
covered by this native initialization exemption. The marker does not inspect new
allocations, and remark traces all of them. Field writes in the managed constructor
and publication into old storage still pass through generated scalar-store lowering.

`construction.neoil` constructs two Nodes holding an existing Node reference and
publishes them through output borrows into a previously allocated object. It uses
the actual ordinary compiler and native object provider, with the automatically
scheduled live host. Forced/paced JIT and optimized/UBSan AOT each run 100 cycles:
34 ordinary constructions, 33 filled-heap garbage-recovery cases and 33 all-live
heap exhaustion cases. Checks cover transitive liveness, distinct constructed
objects, active old-object barrier counts, new-payload writes, pressure triggering,
unchanged output fields on allocation failure, root-chain restoration and final
empty-heap reclamation. The failure occurs before the first object is allocated;
partial multi-object-operation rollback is neither required nor claimed.

`construction-validation.json.gz` records these 600 cycles; `construction.o.gz`
retains the emitted AOT object. The collector source is identical to the automatic
candidate retained in concurrent-live; input hashes are in the report.
`construction-compiler-tests.json.gz` records 12 passing compiler tests plus four
root-plan/map controls. `construction-regression.json.gz` records passing previous
scalar/reference-store consumers and host rejection controls with the rebuilt runner.

This follows the existing CLR-style rooted allocation/constructor lifetime model;
no language, Runtime Contract or CLI representation changes. Conservative tracing
and owner-thread allocation remain provisional implementation choices. Array/native
service writers, full host entry and HTTP workloads remain unqualified. This result
admits object construction to the internal experimental path; it does not enable
production concurrent GC or demonstrate a throughput gain.


## Grouped reference-array publication

The internal compiler now emits `experiment_gc_store_record` for traced record-layout
array element stores. A compiler stack snapshot holds the whole element; zeroing
its padding prevents narrow lanes from copying uninitialized bytes. The noncollecting
helper validates allocation kind, immutable shape, width and index, then publishes
payload and initialization marker in one mutation-gate transaction. New/inactive
allocations copy directly. If the bounded barrier log cannot hold the transaction,
it cancels/joins the epoch before copying, without collecting unpublished temporaries.
The scalar adapter shares this cancellation path. Snapshot stack space, zeroing and
helper calls are additional costs, not a demonstrated performance improvement.

The rooted nominal-reference-array allocator is now in the explicit provider list.
It reserves fresh record-layout storage and initializes its null slots/markers before
returning, without a callback after reservation. Typed array identity stores remain
on the new-allocation path. Text/character/callable array stores are rejected on the
internal live path pending their grouped publication integration. Ordinary compilation
and managed array semantics are unchanged. The new private helper must exist in the
matching host; this is not a public switch or general native-provider admission.

`arrays.neoil` allocates a temporary reference array, writes/reads an element, then
publishes it into two existing arrays. Forced/paced JIT and optimized/UBSan AOT run
100 cycles each: initialized null slots, uninitialized slots, first-array bounds
failure and second-array bounds failure. Checks cover marker publication, transitive
liveness, exact old-array transaction counts, temporary-array writes, fault/root-chain
cleanup and final reclamation. A second-array failure preserves the already completed
first write, as normal sequential semantics require. Native grouped-helper tests cover
1/2/32/65-lane elements, rejected extents/widths and cancellation for a transaction
larger than the log. Wider managed record/structural arrays remain to be qualified.

The records helper follows the existing CLR/Go compiler/runtime bulk-barrier research
in the mutation audit. neoCLR's conservative layout and initialization markers require
broader grouped snapshots than a precise reference-only barrier; this is a provisional
cost. Future native layout/provenance metadata should narrow coverage when validated.
Runtime Contract configuration, Raven semantics and temporary CLI representation are
unchanged. Full host entry, additional native-service writers and HTTP remain open.

Evidence: `arrays-validation.json.gz` (600 managed cycles), `arrays-compiler-tests.json.gz`
(13 compiler tests and four root controls), `arrays-native-validation.json.gz` (including
400 grouped native cycles, broader native fixtures and deletion-barrier negative control),
`arrays-construction-regression.json.gz` and `arrays-scheduling-regression.json.gz`.
Generated objects and explicit/automatic collector sources are archived alongside them.
The initial assembly fixture incorrectly used value-array syntax `Node[]`; it now uses
`arrayref<Node>`. An initial overflow assertion used exactly 64 lanes, which fit the log;
65 lanes correctly exercise cancellation. Both initial failures are retained. No
throughput measurements or default switch are claimed.
