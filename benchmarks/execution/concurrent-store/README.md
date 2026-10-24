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


## Grouped text-array publication

The internal compiler now uses `experiment_gc_store_text` for String/Char array
stores, passing the pointer value directly. The helper validates the traced-array
allocation and immutable shape. Ordinary arrays publish one pointer; reserved arrays
publish the pointer and initialization marker in a single gate transaction. It shares
noncollecting cancellation with record/scalar stores. The rooted strings allocator
is admitted: it initializes only fresh storage, with no callbacks after reservation.
Callable array stores remain rejected pending their own qualification. Default
compilation, public semantics and Runtime Contract/CLI metadata are unchanged.

`text-arrays.neoil` reserves a temporary String array, writes/reads it and publishes
into existing arrays. `text-arrays-validation.json.gz` records 600 forced/paced JIT
and optimized/UBSan AOT cycles covering ordinary null slots, reserved uninitialized
slots and both first/second bounds failures. Native helper checks reject an invalid
index and wrong allocation kind. Paced successful cases wait for marker completion,
then repeatedly write until the barrier log forces cancellation; the text and marker
remain correct and eventual collection empties the heap. Second-write failure retains
the first completed write. Managed Char-specific behavior is not separately exercised;
it shares the pointer-array storage protocol. Text views, decoding and other native
services are not admitted by this allocator/store qualification.

`text-arrays-compiler-tests.json.gz` records 14 compiler tests and four root controls,
including rejection of callable array stores and opt-in helper-import checks.
`text-arrays-regression.json.gz` records the prior reference-array consumers against
the rebuilt compiler/host. The object and generated collector are archived alongside
the reports. No throughput comparison was performed. Existing CLR/Go bulk-barrier
research applies; gate/copy costs remain provisional rather than a claimed improvement.

`http-import-inventory.json.gz` records the imports and identity of the previously
measured plaintext AOT object at `target/state-integration/plain.o`. It is a navigation
index, not evidence that the current live compiler accepts HTTP or that every imported
provider writes managed storage. Concrete next groups are static-storage publication;
byte/scalar/reserved allocation, byte views and text helpers; then task/callback roots,
async/socket services and full entry lifecycle. Internal replacement bodies also need
qualification: an import list alone does not cover their generated writes. Keep ordinary
standalone workloads in the qualification matrix as these groups are integrated.


## Guarded static-storage publication

Static owner storage can be allocated after its context table has entered a live
epoch. The production native provider writes the owner pointer directly into that
table, which would race the live marker. `native-providers.py` generates an explicit
experimental variant: table-slot publication calls the noncollecting scalar-store
barrier after allocation and before returning. A new table's initialization and
context-root assignment need no old-payload barrier; new allocations are excluded
from marker reads, and remark traces the current context root. No collection may
occur between allocating an owner and publishing it from the native temporary.

The internal compiler imports `experiment_gc_static_storage_rooted_v1`, with an
explicit ABI in the provider list. A distinct symbol prevents silent binding to the
unguarded rooted provider. Ordinary compiler output and production `text-arena.c`
remain unchanged. The generated provider source is not a general replacement for
unqualified native callers: its static publication requires an owned heap, and output
aliases are limited to the generated stack-result contract. Static initializer guards
remain rejected; this slice qualifies static fields without `.cctor` execution.

`statics.neoil` supplies a skeleton. Because static field syntax is not exposed by
the text assembler, `managed_input` in `mutation_store_tests.rs` adds two static
fields and the corresponding ordinary metadata instructions, following the existing
static-storage compiler tests. Actual generated code stores and reads references in
two static owners and returns them through output borrows.

`statics-validation.json.gz` records 600 forced/paced JIT and optimized/UBSan AOT
cycles: existing-table cold-owner publication, garbage-pressure recovery, all-live
exhaustion and creation of the table during an active epoch. Successful calls repeat
the warm path. Exact barrier counts distinguish old-table publication from the first
managed static field write. After native roots are cleared, only static roots retain
the graph; invocation completion then clears static ownership and collection empties
the heap. A failed second-owner allocation retains the first completed static assignment
and leaves output borrows unchanged. Linking against the original provider fails for
the distinct experimental symbol, as recorded in the same report.

`statics-compiler-tests.json.gz` records 15 compiler tests plus four root controls;
`statics-construction-regression.json.gz` retains passing construction consumers.
The emitted object and generated provider are archived; the collector is byte-identical
to `text-arrays-automatic-gc.c.gz`. This preserves the existing context-scoped static
semantics rather than introducing process-global roots. The CLR-style root/write-barrier
cooperation and conservative-layout tradeoffs remain as previously researched. Runtime
Contract configuration and CLI metadata are unchanged. Native byte/text helpers,
initializers, async/socket services and full HTTP entry remain unqualified; there is
no new throughput evidence or default switch.


## Static initializer guard integration

Live compilation now defers generated initializer guards alongside ordinary managed
bodies, lowers their stores through the noncollecting barrier and includes them in
whole-body call-closure validation. The body-count check includes these guards while
still rejecting unqualified replacement emission. Default compilation continues to
define guards on the original path. Shared-worker initialization remains unsupported.

A guard uses its caller's published frame. Across the initializer call it retains only
the static owner already rooted by the context table; its result scratch is Void.
It therefore does not add an independent root frame. Managed initializer bodies still
publish their own normal frames. Guard state writes use the scalar barrier, preserving
the existing running/completed/cached-status representation, reentry and failure behavior.
This follows the established [initialization contract](../../../docs/construction-and-initialization.md)
and CLR comparison; it adds no new exception semantics or lifetime divergence.

`initializers.neoil` is augmented with static fields/instructions in `managed_input`,
as for the previous static fixture. First's initializer increments an attempt counter
and reenters through a static method; Second increments its counter and optionally
raises arithmetic overflow. Repeated calls verify one attempt for both success and
cached failure. `initializers-validation.json.gz` records 600 forced/paced JIT and
optimized/UBSan AOT cycles, also covering table creation, allocation pressure, static-only
retention, root-chain restoration and invocation cleanup. Generated native sources
are unchanged from the archived static-provider/text-array collector variants.

`initializers-compiler-tests.json.gz` records 16 compiler tests and four root controls;
`initializers-statics-regression.json.gz` records previous static consumers.
`initializers-default-regression.json.gz` records the existing six-case initializer
integration test (field/generic/strict/relaxed/cycle/reference) across interpreter,
ordinary JIT and AOT, with sanitized AOT consumers. Its first test-runner launch stalled
before main at `_dyld_start`; a sample and explicit termination result are retained in
`initializers-default-sample.txt.gz` and `initializers-default-launch-abort.json.gz`.
A stable byte copy, locally ad-hoc signed, completed. This does not identify the launch
stall's root cause or establish a GC failure; no host security policy was disabled.

The emitted live object is retained as `initializers.o.gz`. No throughput measurement
was taken. Callable/native replacement bodies, remaining text/byte services, async/socket
providers and full host entry still require qualification before HTTP comparisons.


## Native text replacement bodies

The internal live compiler admits the existing rooted StringConcat and StringByteCount
bindings. Native replacement bodies now use the same store-lowering, shared frame-cleanup
and deferred call-closure path as ordinary bodies. Other native replacements remain
rejected. The fixture uses production UTF-8 binding validation before value lowering, then
gives the validated replacement placeholders private names to avoid collision with
bundled System during backend verification. Full selection/entry integration is
not covered by this direct backend fixture.

Live text wrappers pass compiler-owned stack scratch to their providers, then publish
successful results through checked stores. A provider cannot write directly through an
arbitrary managed result alias. Concat retains both inputs over allocation, initializes
only fresh immutable TEXT storage, and returns without another collection; byte count
reads immutable text and writes only its stack result. The noncollecting publication
barrier preserves that result until the caller can publish it. The synchronous/default
path and production C providers are unchanged.

This follows CLR's separation of compiler roots, native-helper cooperation and managed
write barriers, using the [previously researched](../concurrent-mutation-audit/README.md#alternatives-and-research)
nonmoving conservative live protocol.
The deliberate UTF-8 text contract is unchanged; this does not introduce CLR UTF-16
semantics. Shared heaps, other provider writes, async/socket services and full HTTP entry
still need qualification. These checks establish integration behavior, not throughput.

`native-text-validation.json.gz` records 1,200 forced/paced JIT and optimized/UBSan AOT
cycles: 600 nested allocation/pressure/exhaustion/null-fault cycles and 600 direct
native-wrapper heap-result-alias cycles. Exact store counters confirm both pointer
and scalar results use the barrier; host roots, immutable contents and final reclamation
are checked. `native-text-initializer-regression.json.gz` records 600 existing initializer
cycles. The emitted object is archived; generated GC is byte-identical to
`text-arrays-automatic-gc.c.gz` and the production text provider is unchanged.

`native-text-codegen-tests.json.gz` records the full codegen library run: 156 passed,
12 ignored and one pre-existing reflection diagnostic assertion failure. The same
failure reproduces with the previous initializer runner in `native-text-existing-failure.json.gz`:
the test expects Optional<TypeInfo>, but the diagnostic says Option<TypeInfo>.
`native-text-fixture-failures.json.gz` retains initial local/argument, binding-name,
missing binding preparation, duplicate System identity and reserved-module fixture
failures. These are corrected fixture setup failures, not collector race evidence.
No HTTP performance measurement or default switch is claimed.


## Read-only HTTP text comparison providers

The live provider set now includes canonical comparison, ordinal case-insensitive
comparison and ordinal prefix matching, all present in the recorded plaintext HTTP
import inventory. The private bindings keep their existing UTF-8 policies; neither
Raven semantics nor the existing .NET comparison/divergence documentation changes.
These helpers read immutable text, write a scalar result, retain no pointer and do not
collect or reenter managed code. The C canonical/case wrappers delegate to the same
pinned Rust normalization/folding kernels used by ordinary execution. Their output
is the compiler scratch slot; publication to a managed result alias uses the existing
checked path. Availability of other symbols in the linked native library does not
admit them: compiler replacement and exact call-closure checks remain mandatory.

The expanded native-text fixture checks composed/decomposed e-acute equality,
Kelvin-sign/ASCII-k folding, positive/negative prefix matches, null faults without
result publication, root-chain restoration and final reclamation. Exact counters
require six successful heap publications per alias cycle, including Boolean results;
failed comparisons add none. The Rust helper library is rebuilt and hashed alongside
the C adapters, shared normalization sources and emitted object.

`text-comparisons-validation.json.gz` records 1,200 forced/paced JIT and optimized/UBSan
AOT cycles (600 allocation/pressure/fault and 600 expanded heap-result-alias cycles).
`text-comparisons-compiler.json.gz` records 17 focused compiler tests passing, with
nine host-dependent tests ignored by that run; the text host test runs explicitly in
the validation harness. The emitted object is archived; generated collector source is
unchanged from `text-arrays-automatic-gc.c.gz`. Initial incorrect binding spelling and
missing native-header include-path failures are retained separately in
`text-comparisons-setup-failures.json.gz`. No new throughput measurement was taken.
Mutable byte views, remaining allocation/text services, async/socket providers and
full HTTP entry still require qualification before a default comparison.


## Byte-buffer native boundary

The live scanner's `shadow_scan` reads no BYTES/TEXT payload. Byte views validate the
managed allocation base and require `NEOCLR_GC_BYTES` before mutable access; array
headers and allocation kinds stay fixed. Byte contents and reserved initialization
bytes therefore do not need a marker mutation barrier. This conclusion concerns
collector concurrency on an owned heap, not concurrent application writers or shared
heaps. View descriptors contain owner references and remain subject to normal root
publication and guarded managed writes. Outputs from the C helpers must remain
compiler/host scratch; admitting arbitrary native result aliases would be unsafe.

This follows the pointer-free-array distinction in the existing
[CLR/Go GC comparison](../concurrent-mutation-audit/README.md#alternatives-and-research).
neoCLR currently proves the exemption through a checked coarse allocation kind,
rather than CLR's precise reference layout. It avoids unnecessary synchronization
for HTTP byte payloads, but does not exempt conservatively scanned scalar fields in
OBJECT/RECORDS allocations or relax buffer lifetime requirements.

`byte-views-validation.json.gz` records optimized and undefined/bounds-sanitized
synchronous/live runs: 800 epochs, 800,000 writes, both ordinary/reserved storage,
overlapping copies, bounds/readonly rejection, forged OBJECT and interior-owner
rejection, empty-window owner retention and final reclamation. The existing exhaustive
native memory-view test also passes against both collectors in both configurations.
Writes run with the marker delayed, released and completed; the marker reads zero
payload bytes in these byte-only rooted graphs. Managed allocation counters stay
unchanged during the operations. Pointer-shaped payload bytes do not retain an
otherwise dead object, matching the synchronous collector's cleanup behavior.

The intentional negative scanner traces byte payloads; the test rejects its nonzero
payload-read count. That generated control is retained in `byte-views-negative-gc.c.gz`.
The positive generated collector is byte-identical to `text-arrays-automatic-gc.c.gz`.
These native correctness checks are not throughput measurements or a race-detector
qualification. No compiler provider admission, production behavior or default changes
in this slice.

The next generated-code integration must cover view factory/slice results, getter
scalar results and setters through ordinary JIT/AOT wrappers. Existing emission builds
24-byte C descriptors on the stack and copies owner/bounds back into three managed
lanes; that publication must pass live lowering and closure checks. Then qualify byte
allocation/reservation, UTF-8 conversion and socket/async lifetimes before full HTTP
comparisons. Native helper success alone does not establish those managed paths.


## Generated byte-view wrapper integration

The internal live compiler admits byte-view creation/windowing, slicing, length,
get/set and UTF-8 projection through exact native provider ABIs. C descriptors/results
remain stack-local; owner/bounds and scalar results use ordinary checked generated
stores. The setter's checked BYTES payload write remains unbarriered under the native
boundary proved above. Capability validation remains the existing memory-view binding
contract; this does not admit arbitrary native result pointers or buffer aliases.

UTF-8 projection was a noncollecting frameless leaf on the default path. Live-mode
publication now gives it a typed-argument frame before checked stores and shared
cleanup. Default compilation remains unchanged. The direct backend fixture validates
nine production bindings across borrowed slices, mutable/read-only persistent memory,
and UTF-8 readonly slices. Its private compiled names avoid rebinding bundled System;
it is not a full selection/host-entry test.

This retains the existing Span/Memory-style owner and capability contracts and the
UTF-8 read-only text view divergence. The live collector uses the previously documented
nonmoving, conservative root/write-barrier protocol; a borrowed view still does not
authorize suspension or shared-heap access. Byte allocation/reservation, capability
conversion wrappers, other native services and full HTTP entry remain separate work.

`managed-byte-views-validation.json.gz` records 600 forced/paced JIT and optimized/UBSan
AOT cycles with real generated wrappers. Whole/window factories, slicing, get/set,
length and UTF-8 results target an old heap object. The fixture tests reserved and
ordinary buffers, bounds faults, rejected writes through text views, root restoration,
and owners retained solely through heap-published descriptors. Exact counters confirm
zero heap barriers for setters and one for getter result publication.
`managed-byte-views-compiler.json.gz` records 18 mutation/admission tests, nine view
contract tests and four root-map/planning tests passing. Ten host-dependent tests are
ignored in the generic run; the new host test runs explicitly in the harness.

The object is archived; the generated collector and production native providers are
unchanged. `managed-byte-views-initial-failures.json.gz` retains a corrected stale fixture
definition index and the missing-frame rejection that exposed UTF-8 projection's live
integration gap. No throughput measurement or default change is claimed.


## Managed primitive buffer allocation

Live call-closure admission now includes the existing rooted byte allocation,
byte reservation and scalar-array allocation providers. Generated calls already
publish their collecting-boundary roots and pass stack result slots. Providers
initialize fresh storage without another collection before return; managed output
publication uses checked stores. Scalar arrays use the existing BYTES allocation
kind so pointer-shaped integers are not reference edges. The default compiler and
production provider implementations are unchanged.

`primitive-arrays.neoil` executes ordinary/reserved Byte, Int32 and Int64 array
construction, stores and reads. The native consumer supplies old heap output aliases
and tests normal operation, garbage-pressure recovery, all-live exhaustion, reserved
byte/scalar uninitialized reads, and bounds failures. Earlier published arrays survive
faults, result storage remains unchanged on failure, and the host root chain is
restored. Only the published heap owners retain exported buffers across collection;
clearing the final owner reclaims everything. Local scalar arrays stay rooted across
subsequent allocating calls, and the final sum checks stored and zero-initialized data.

`primitive-arrays-validation.json.gz` records 720 forced/paced JIT and optimized/UBSan
AOT cycles; `primitive-arrays-compiler.json.gz` records 19 focused compiler tests passing
(11 host-dependent tests ignored there; this fixture's JIT host test runs explicitly).
The emitted object is archived. The generated collector is byte-identical to
`text-arrays-automatic-gc.c.gz`. This extends the existing CLR-style distinction between
reference-containing and primitive arrays, with neoCLR's already documented reserved
initialization checks. No new semantic divergence, HTTP throughput result or default
switch is claimed. Capability conversions, remaining text/native services and async/
full-entry integration remain open.


## Scoped UTF-8 slicing (2026-10-24)

The internal live compiler now binds StringSliceUtf8 to the exact private
`experiment_gc_string_slice_utf8_rooted_v1` adapter. This synchronous leaf enters
the owned native fallback, calls the existing rooted implementation, and leaves on
success or failure. It cancels active marking without reclaiming unpublished values;
collection during allocation is synchronous. This does not admit arbitrary native
calls, async pumps, retained asynchronous writes or shared heaps.

Generated UTF-8 helper scratch storage now uses the actual result width instead of
an eight-byte assumption. Slicing returns a two-lane erased Value (tag plus payload);
the wrapper reads both lanes and uses checked managed stores only after success.
The result tag remains the existing private ABI, not a new public contract. Default
compilation still binds the original helper. A live image needs the scoped adapter
symbol, which earlier experimental hosts do not provide.

`validate-scoped-slice.py --runner <codegen-test-runner> --output <fresh-directory>`
executes actual generated wrappers in forced/paced JIT and optimized/undefined-bounds
AOT: 720 cycles pass, with valid multibyte/empty results, invalid range/boundary values,
null faults, all-live exhaustion, unchanged outputs on failure, heap aliases, owner-only
result retention, root-chain restoration and resumed pacing. Scope counters verify
one balanced entry and cancellation per call. The production binding validator checks
the StringSliceUtf8 signature before the backend fixture renames it privately.
Nineteen focused mutation/compiler tests pass, with twelve host-dependent tests skipped
in that general run; the scoped JIT tests execute explicitly in the harness.

`scoped-slice-validation.json.gz`, `scoped-slice-compiler-tests.json.gz`, the generated
native source and AOT object preserve evidence. The initial adapter build omitted the
explicit cast from the root frame's opaque context pointer; its failed compilation
is preserved in `scoped-slice-initial-cast-failure.json.gz`. Correcting the typed access
resolved it. No HTTP/performance measurement or default change is claimed. The next
integration work remains remaining native services, capability/callable wrappers,
async callbacks and full entry. See the [runtime scope tradeoff and CLR comparison](../concurrent-live/README.md#native-call-synchronous-fallback-prototype-2026-10-24).


## Generated ordinary callback dispatcher (2026-10-24)

The live compiler now defers ordinary host-callback definition until after its
store lowering and the complete managed-body/callback call-closure check. Previously
this dispatcher was emitted after the live closure had already been checked. The
exact noncollecting `neoclr_gc_callback_read_v1` ABI joins the provider list; this does
not admit arbitrary host calls. The strong host handle retains the callable descriptor
and instance receiver while the generated callee publishes its own frame. The reader
rejects entry when a matching managed frame is already active. Entry-pump dispatch,
queue draining, bootstrap and native async replacement admission remain separate.

`callbacks.neoil` supplies inhabited-Void and no-result instance targets. The first
allocates a byte array, stores it in its receiver and writes a scalar; the second
raises division-by-zero. The real collector consumer invokes the exported dispatcher
with a host-root handle, including allocation pressure, invalid identities, stale
handles, malformed descriptors and already-active-frame rejection. It checks receiver
and child retention, unchanged rejected receivers, fault status, root-chain restoration
and complete reclamation after releasing the handle.

`validate-callbacks.py --runner <codegen-test-runner> --output <fresh-directory>`
passes 720 forced/paced JIT and optimized/undefined-bounds AOT cycles. Twenty focused
compiler checks pass (13 host-dependent checks skipped in that general run; the
callback JIT test runs explicitly in the harness). Reports and AOT object are archived
as `callbacks-*`. The first rejection fixture incorrectly passed null storage to the
root-frame API, which requires a non-null descriptor even for zero slots; the native
assertion aborted before dispatch. Its failure report is retained. Correcting the
fixture resolves it. The local LLDB launcher also crashed during diagnosis; a temporary
abort backtrace located the fixture issue. This is not race-detector qualification.

Default compilation and managed API semantics are unchanged. This narrows the gap
between the native carrier evidence and generated HTTP code, but does not qualify
full entry/pumps, asynchronous retained writes, shared heaps or server throughput.
Native async/socket result aliases still need guarded publication before admission.
The existing CLR rooted-transition comparison applies; this adds no public contract.


## Generated async operation wrappers and suspension (2026-10-24)

Live compilation now admits six exact async operation providers: Create, CompleteUnit,
AwaitUnit, Release, IsCompleted and WaitCompletion. Their generated wrappers use
result-type-sized stack scratch and guarded publication after success, retaining the
existing frame and fault cleanup. No-result AwaitUnit allocates scratch for the native
ABI but never reads or publishes it. Ordinary compilation retains its existing path.
CreateOwned, CurrentOwner, Start/StartManaged and entry pumps remain unadmitted.

`async-wrappers.neoil` validates reserved signatures through the production binding
validator before privately renaming the backend fixture. Its generated callback calls
AwaitUnit, suspends on the real native carrier, resumes, allocates a buffer and stores
it into its receiver. C starts/pumps that callback; those generated start/pump wrappers
are explicitly outside this test. The native scheduler is an idle stub, not a socket
server. C owns the host-root handle and uses the guarded store hook for its own old
heap mutations. There is no native fallback scope around suspension or pumping.

`validate-async-wrappers.py --runner <codegen-test-runner> --output <fresh-directory>`
passes 600 cycles across forced/paced JIT and optimized/undefined-bounds AOT. Collection
runs while generated frames are parked; half the cycles force collection during the
resumed callback's allocation. Checks cover heap-result aliases, barrier counts,
unchanged fault outputs, waiting outside a fiber, no-result destination preservation,
completion/release, invalid released IDs, restored root chains, empty final heap and
zero reserved stack bytes after shutdown. Twenty-one focused compiler tests pass,
with fourteen host-dependent tests skipped in the general run; this JIT fixture is
invoked explicitly by the harness. `async-wrappers-validation.json.gz`, the AOT object
and compiler report retain evidence. No new performance claim or default change.

This integrates generated suspension with the previously tested carrier/root protocol.
The CLR comparison remains explicit published/parked roots and rooted transitions;
neoCLR's private cooperative carrier is not CLR thread scheduling. Remaining work is
owned/start wrappers, native socket result publication, entry/queue/bootstrap integration
and the full HTTP/standalone correctness and performance matrix.


## Generated strong async startup (2026-10-24)

RuntimeAsyncStart now joins live native-provider admission, using the already guarded
result wrapper and checked generated callback dispatcher. The startup frame is parked
while the callback runs on its native carrier; after the callback suspends, startup
resumes and publishes the operation handle through the caller's heap alias. This
adds no whole-pump fallback or new public scheduling contract. CreateOwned, CurrentOwner,
StartManaged and generated pump entry remain unadmitted.

The async-wrappers fixture now binds seven exact reserved signatures. Its 600 forced/
paced JIT and optimized/undefined-bounds AOT cycles alternate generated strong startup
(300 suspended callbacks) with native owned startup (300). Every cycle additionally
executes generated startup with a faulting callback and a null callback, requiring
unchanged heap output and restored roots. Success paths verify operation release and
clean shutdown; failure paths deliberately clear fault state, check shutdown reports
outstanding failed operations, then verify state reset and zero reserved stack bytes.
Twenty-one focused compiler tests pass; the fourteen host-dependent checks skipped
in the general run include the async JIT test executed explicitly by the harness.

`async-start-validation.json.gz`, the AOT object and compiler report preserve this
extension; earlier six-wrapper evidence remains archived separately. Three initial
fixture assertions failed and are retained: callback boundaries can complete an epoch
before the result store (so count both safe inactive and guarded-old stores); wrapper
status is 3 while the original division fault detail remains 1; and shutdown reports
outstanding failed operations if the caller has cleared its fault. Correcting these
expectations preserved existing runtime semantics rather than changing them to fit
the experiment. No HTTP throughput/default claim; owned startup, socket publication,
pump/full-entry integration and full workload performance remain open.


## Generated owned async startup and owner access (2026-10-24)

CreateOwned, CurrentOwner and StartManaged now join live-GC admission through their
exact native provider ABIs. The fixture validates all ten reserved async signatures
with the production binder, using the existing explicit System.Object base shape and
a derived Counter. These wrappers reuse typed stack results and protected publication.
Generated startup now drives both strong and owned callbacks; C only pumps the native
carrier with an idle scheduler. The callback obtains/stores CurrentOwner before it
suspends, so its receiver may contain a self-reference.

The expanded `validate-async-wrappers.py` passes 600 forced/paced JIT and optimized/
undefined-bounds AOT cycles: 450 callbacks resume, while 150 owned callbacks are
abandoned and reclaimed. Dropping the last strong host handle must reclaim the
self-referential owner and conditionally parked generated frames; later completion
must not revive that callback. Retained cases cover pressure allocation, callback
faults, safe heap aliases, null-owner creation/start rejection, CurrentOwner outside
and inside callbacks, release, restored root chains and zero reserved stacks after
shutdown. Twenty-one focused compiler tests pass; fourteen host-dependent checks
are skipped in the general run, with this JIT fixture explicitly run by the harness.

`async-owned-validation.json.gz`, the AOT object and compiler report preserve results.
The initial C fixture omitted the existing empty-base storage lane, so generated code
read the wrong operation field and faulted. Correcting its allocation/offsets to the
actual inherited layout resolved the failure; the report is retained as
`async-owned-initial-layout-failure.json.gz`. No runtime layout change was made.

All ten async service wrappers are now covered in this controlled owned-heap profile.
This is not full selection/HTTP admission: generated pump/entry/queue code, socket
result wrappers, remaining native services and shared heaps still need qualification.
No new throughput/default claim. Existing CLR ownership/root-transition comparisons
and CLI/native metadata replacement plans remain unchanged.


## Generated socket primitives and array transfers (2026-10-24)

Thirteen exact socket provider ABIs now join live admission: Listen, LocalPort, Close,
Accept, ConnectResult, Cancel, DeadlineAfter, DeadlineExpired, Receive, Send,
TransferResult, ReceiveUntil and SendUntil. These are the array-buffer transfer forms;
memory-view variants have distinct native symbols and remain excluded by call-closure
checking. Providers reuse the typed native-result scratch/guarded publication path.
Production binding validates all thirteen reserved signatures before private fixture
names are assigned. Ordinary compilation and native socket behavior are unchanged.

`socket-wrappers-consumer.py` derives the existing real loopback socket-transfer test,
redirecting those calls through generated wrappers. An old managed result carrier is
strongly rooted; every successful scalar or two-lane Value result must pass exactly
one or two protected heap stores. Failed calls must not alter that carrier or the
original caller output. The adapter deliberately clears diagnostic fault state after
recording returned status, preserving the native fixture's error-recovery policy.
The test starts an epoch before its first socket call and after each collection.
Raw receive-buffer writes remain safe because the marker excludes BYTES payloads.
Added close/double-close and stale-port checks exercise every admitted wrapper.

`validate-socket-wrappers.py --runner <codegen-test-runner> --output <fresh-directory>`
passes 192 real loopback cases across forced/paced JIT and optimized/undefined-bounds
AOT: accept/delivery roots, partial receive, reserved-byte initialization, send snapshots
under later mutation, deadline/cancellation errors, EOF, fault teardown and empty final
heap. This reuses the synchronous/native comparison fixture already qualified in
concurrent-live/socket-transfer-*; the generated runs here use the live candidate.
Twenty-two focused compiler tests pass (15 host-dependent tests skipped in the general
run; the socket JIT tests execute explicitly in the harness). Reports, generated C and
AOT object are archived as `socket-wrappers-*`.

This is generated network I/O qualification, not an HTTP benchmark. Memory-view
transfers, remaining native services, generated pump/queue/bootstrap entry and shared
heaps remain open. No default switch, throughput gain or race-detector claim. The
existing CLR rooted native-call comparison and private CLI Value representation still
apply; no new public API or deliberate semantic divergence is introduced.


## Generated socket memory transfers (2026-10-24)

The four persistent memory transfer providers (Receive, Send and their deadline
variants) now join exact live-GC admission. Receive requires Memory<Byte>; Send
requires ReadOnlyMemory<Byte>. The existing production binder enforces these
capabilities. Their flattened owner/offset/length ABI uses the same guarded result
publication as the array variants. Ordinary compilation and native semantics are
unchanged; no public API is added.

`socket-memory.neoil` is an assembler scaffold: the test installs retained memory
metadata and replaces array/offset/count parameters with a single memory parameter
before production binding and compilation. It clears obsolete parameter names. The
initial fixture failed metadata validation because it retained those names; preserve
that failure in `socket-memory-initial-fixture-failure.json.gz`.

`validate-socket-memory.py --runner <codegen-test-runner> --output <fresh-directory>`
passes 192 loopback cases across forced/paced JIT and optimized/undefined-bounds AOT.
It reuses the native transfer fixture and generated-wrapper adapter with memory
providers: sliced receive buffers, empty views, initialization, send snapshots after
later mutation, deadlines/cancellation, EOF, protected heap results and cleanup.
A forged out-of-range memory descriptor returns runtime status 3 rather than the
legacy array path's Value error; this is existing native validation, not a semantic
change. Failed calls preserve the heap output carrier. Twenty-three mutation tests
and the exact memory-capability binding test pass; sixteen host-dependent tests are
skipped by the general filter, with both JIT configurations explicitly executed here.
Reports, emitted object and generated consumer are retained as `socket-memory-*`.

The existing CLR rooted native-call and persistent memory comparisons apply. Temporary
CLI memory metadata and flattened native representation remain unchanged; neoCLR owns
backend qualification and eventual native metadata replacement. Generated pump/queue/
bootstrap entry, remaining providers and shared heaps remain unqualified. These are
correctness tests, not throughput measurements or evidence for switching the default.


## Generated public entry (2026-10-24)

Internal live-mode public entry now calls `experiment_gc_prepare_entry_v1` instead
of resetting arena/static-storage cursors directly. The entire wrapper joins store
lowering and call-closure validation, with exact prepare/completion provider ABIs.
The Windows final machine-frame limit remains enforced after deferred definition;
execution evidence here is macOS ARM64. Ordinary compilation is unchanged. Live
objects require the matching automatic candidate host with the new prepare import;
the explicit marker-only host is no longer sufficient for generated entry objects.
The base validation harness now uses automatic.py with required profiling enabled.

`validate-entry.py --runner <codegen-test-runner> --output <fresh-directory>` passes
600 cycles across forced/paced JIT and optimized/undefined-bounds AOT. Each uses
`neoclr_entry_v4`, not a private managed function: retained-root rejection preserves
output and heap; successful integer Main allocates/reads an array; repeat entry
reuses the same extent; division failure preserves output and unwinds roots; the
next call clears the fault and succeeds. Invocation completion leaves no active
epoch. Final collection/release succeeds. Twenty-four focused compiler checks pass;
17 host-dependent tests are skipped by the general filter, with public-entry JIT
checks explicitly executed by this harness. The existing mutation/managed/host
compatibility validator also passes against the automatic candidate.

Reports and emitted object are archived as `entry-*`. The initial fixture hit an
over-broad test assertion that treated an integer array store as a record store;
the corrected assertion and initial failure are retained. This was not a collector
failure. Exact prepare-import selection is checked for live/default compilation.

This qualifies the integer-returning synchronous public entry with managed allocation
and faults. Unit/async entry pumps, bootstrap, arguments, queues and remaining native
services still need qualification; it is not full HTTP server admission. The existing
CLR root-transition comparison and temporary CLI/native representation remain
unchanged. neoCLR owns this backend transition and its eventual native metadata
replacement. No throughput gain or default switch is claimed.


## Generated default-queue dispatch (2026-10-24)

`neoclr_drain_default_queue_v1` now joins live-mode store lowering and call-closure
validation instead of being defined outside that set. Its exact native reader
contract is admitted: `neoclr_task_queue_host_read_v1` neither allocates nor collects,
and the real task scope's strong root retains the queue until managed Drain publishes
its frame. Ordinary emission and native queue semantics remain unchanged.

`validate-queue.py --runner <codegen-test-runner> --output <fresh-directory>` passes
600 forced/paced JIT and optimized/undefined-bounds AOT cycles against the automatic
candidate and real task-queue.c provider. A controlled managed Queue/Drain shape
increments a scalar, allocates/replaces a byte child and can fault after publication.
Tests cover missing scopes, empty queues, scope-only ownership through collection,
existing faults, active-frame rejection, fault unwinding, recovery and final release.
The fixture selects the drain index directly; it does not claim to exercise the full
System TaskQueue implementation or its binding/selection pipeline.

Twenty-five focused compiler checks pass; 18 host-dependent tests are skipped by
the general filter, with both JIT configurations run explicitly by this harness.
Reports and object are archived as `queue-*`. Two initial assembler fixture errors
(field separator and unsupported branch spelling) are retained; neither reached
runtime execution. The corrected fixture uses existing accepted syntax.

The existing CLR rooted transition comparison applies: the noncollecting read-to-
managed-call gap relies on persistent host ownership, not a transient unrooted copy.
Temporary CLI/native queue representation is unchanged; neoCLR owns backend validation
and eventual native metadata replacement. Async/legacy entry pumps, bootstrap,
remaining providers and full HTTP performance qualification stay open. No default
or throughput change is claimed.


## Generated async pump (2026-10-24)

Live admission now accepts entry-task drain replacement when async startup/callback
machinery and a generated queue drain are both selected. The pump's exact native
ABI is admitted; the emitted body already uses ordinary frame/store lowering and
joins the call closure. Live mode omits unused legacy entry-pump declarations and
callback emission. Default emission remains unchanged; legacy pumping, host-I/O
entry and bootstrap restrictions remain. Negative compiler checks reject missing
async dispatch or queue drain.

`validate-async-pump.py --runner <codegen-test-runner> --output <fresh-directory>`
passes 600 forced/paced JIT and optimized/undefined-bounds AOT cycles through the
actual generated pump. The real native async carrier and real task scope park and
restore the pump's published frame, resume 450 callbacks and discard 150 abandoned
owners. Tests retain strong/owned startup, self-reference, current-owner observation,
late completion, pressure allocation, faults and zero retained async stack storage.
A pre-existing fault rejects pump entry without changing output or losing roots.
Twenty-seven focused compiler checks pass; 19 host-dependent tests are skipped by
the general run, with both pump JIT configurations explicitly executed here.

This reuses the async ownership fixture, replacing native pump calls with generated
calls and the no-op native drain with generated default-queue dispatch. The task
scope is empty; populated queue behavior is qualified separately by `queue-*`.
Scheduler acquire remains an idle stub, so socket delivery through the pump and
full HTTP integration are **not** established. The fixture selects pump/drain
indices directly; full System binding/selection remains untested by this fixture.

Archive reports and object as `async-pump-*`. The initial assembler fixture wrongly
marked an unbound private Pump as InternalCall; it failed before compilation. The
corrected fixture uses a managed placeholder selected for backend replacement.
Existing CLR suspension/root-transition comparisons, Runtime Contract and temporary
CLI/native async representation remain unchanged; neoCLR owns backend qualification
and eventual native metadata replacement. No HTTP throughput or default-switch claim.


## Real scheduler and loopback accept delivery (2026-10-24)

`validate-network-pump.py --runner <codegen-test-runner> --output <fresh-directory>`
replaces the async-pump fixture's scheduler stubs with actual socket-listener.c and
real socket scopes. Six forced/paced JIT and optimized/undefined-bounds AOT runs pass
600 carrier cycles, including 450 loopback accepts. The generated callback consumes
the accept result through the generated SocketConnectResult wrapper and clears its
operation field before continuing. Existing pressure, abandonment, fault/recovery,
root restoration and zero reserved-stack checks remain. Twenty-eight compiler
checks pass; 20 host-dependent tests are skipped by the general filter, with both
network JIT configurations executed explicitly here.

The first fixture consumed the result in C only after pump return. The real scheduler
correctly treated the unconsumed operation as pending, and the validator timed out
after 90 seconds. Move consumption into the callback, matching the completion path;
no scheduler/runtime behavior change was required. Preserve that timeout report and
its observed termination separately. The validator now records timeout commands
explicitly. Final artifacts and generated consumer are retained as `network-pump-*`.

`network-pump-consumer.py` derives the existing async fixture, expanding its controlled
owner layout for the accept operation. The real socket operation retains the callback
through collection and scheduler delivery; the generated callback can allocate after
consumption while the carrier retains dispatch ownership. Accepted sockets are closed
by scope teardown. The SocketConnectResult binding is validated before private naming;
pump/drain selection remains controlled backend configuration.

This qualifies network accept delivery through generated callback/pump code with live
GC and real idle collection. It does not yet exchange HTTP request/response payloads,
exercise the full System library selection, or measure server throughput. Host-I/O
entry/bootstrap, remaining providers and full HTTP qualification remain open. Existing
CLR ownership/suspension comparison and temporary CLI/native representation apply;
ordinary code generation and native behavior are unchanged. No default-switch claim.


## Whole-server dependency audit (2026-10-24)

`audit-http-dependencies.py --output <fresh-directory>` compares the retained,
previously measured plain/JSON AOT objects under target/state-integration/final-objects
against current live-provider names and known compiler substitutions. It records
object/source hashes, nm commands and selected native binding identities where
available. This is an audit of that benchmark cohort, not a fresh full-server live
compilation, execution proof, or statement about today's rebuilt System libraries.
The plaintext selection report is available (1,131 selected functions); the JSON
selection report is unavailable at the expected path and is explicitly recorded
as null. Its actual object imports are still inspected.

| Import classification | Plain | JSON |
| --- | ---: | ---: |
| Exact name in current live provider set | 44 | 44 |
| Replaced by an admitted live provider | 3 | 3 |
| Unused legacy pump import omitted in live mode | 4 | 4 |
| Still unadmitted | 13 | 14 |
| Total retained-object imports | 64 | 65 |

Name membership is only navigation evidence: whole-body validation, precise ABI,
writer safety and lifetime tests remain separate obligations. The substitutions are
static storage, scoped string slicing and entry reset. The omitted imports belong
to legacy entry pumping; these objects also import the async pump.

Remaining groups and next integration work:

- TaskQueue Register/Default/Current and Timer Start/Cancel: qualify generated
  publication through the existing strong-root/native ownership contracts. Host
  queue reading, queue dispatch and generated async pumping are already qualified.
- Int32/Int64 formatting, UTF-8 encode/decode, String.Intern and Console.WriteLine:
  qualify native result publication and allocation/retirement. Reuse scoped native
  fallback where its audited contract applies; measure cancellation costs later.
- Reserved record arrays: qualify allocation/initialization with generated element
  stores; the existing ordinary record-array evidence does not establish reservation.
- TLS: the plaintext selection retains CreateSession/Release/Abort/Pump. Do not
  remove these merely because the measured endpoint is plaintext; investigate the
  selected graph and either qualify the required path or prove safe specialization.
- JSON additionally needs ordinal string comparison; its other missing imports
  match plaintext's set.

The plaintext selection's DrainEntryTasks binding at compiled index 38 explicitly
requires host I/O. Current live admission rejects that flag. Even after provider
coverage, host-I/O entry must be integrated and exercised with real delivery. The
same audit does not qualify capability conversions, callable arrays, all selected
managed bodies, process arguments, bootstrap or shared heaps. Whole-program live
compilation must expose those remaining failures rather than assuming import
membership is sufficient.

Proceed by closing the grouped provider/host-entry dependencies, then compile and
run the full plaintext selection before expanding isolated fixtures further. Keep
JSON alongside it to detect additional dependencies. Only after full correctness
qualification should matched synchronous/live HTTP and standalone runs measure
throughput, CPU, pause distributions and memory. No new performance result or
change to the default criteria follows from this inventory.

Evidence: `http-dependencies.json.gz`. The existing CLR rooted-provider and native
transition research applies; this audit introduces no API or semantic divergence.
