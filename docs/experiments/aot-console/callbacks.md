# Native stored callbacks — work in progress

The HTTP app stores callbacks in tasks and cancellation objects. The reference-arena
AOT profile now admits the existing native Function metadata for static functions and
nonvirtual heap-class methods and closed class-interface dispatch. This extends execution of already-supported metadata
on main; it adds no Raven syntax, Runtime Contract setting, CLI bridge encoding or
public library API, and does not merge a separate structural-language experiment.

## Private implementation and limits

A binding allocates a 24-byte managed descriptor: private header, selected target
identity, and strong receiver pointer (zero for static functions). Arguments, locals,
fields, return values and closed generic owners can carry the typed pointer. Copies
alias the descriptor and receiver. Typed root plans distinguish it from numeric lanes;
conservative object tracing keeps its receiver alive. Binding and invocation have
pre-operation root boundaries; reference-bearing results use the ordinary pending
result handoff. Allocation never collects inside the service. No callback escapes the
invocation-owned heap to a native scheduler or host handle in this slice.

Invocation emits a bounded dispatch chain over selected bindings of the exact Function
shape, followed by an ordinary typed native call. Target IDs are private to the image,
not addresses, stable symbols or an external ABI. At most 32 possible targets per invoked
shape are admitted. The default profile rejects recursive call graphs, including conservative callback
edges; this can reject an acyclic dynamic path. The explicit
[native stack budget](native-stack.md) now admits guarded recursion. Borrowed/output signatures, callback operations inside
output-parameter methods, virtual class methods, borrowed/value interface receivers, Function equality,
Object views and introspection remain outside this profile. Ordinary no-result calls
remain distinct from inhabited Void results.

A null receiver at binding reports the interpreter's existing RuntimeError; invocation
of a null callback reports NullReference. Callee faults propagate without publishing a
result and retain the same managed frames. These are current contracts, not a proposal
to revise null-binding diagnostics. Reassessing that behavior would require changing
both execution modes and their evidence together.

## .NET comparison and alternatives

.NET's [Delegate API](https://learn.microsoft.com/en-us/dotnet/api/system.delegate?view=net-10.0)
represents a static method or an instance together with its method (primary .NET 10
API documentation checked 2026-10-08). That ownership model is the ergonomic baseline;
neoCLR already has structural Function metadata and interpreter support. This slice
implements a bounded native representation, not the full .NET delegate API or an
alternative language contract. Multicast invocation and reflection are not added.

A raw native function pointer cannot retain a receiver by itself. A pointer plus
receiver and generated thunks could provide constant-time dispatch, but would introduce
another calling convention and more code-generation/lifetime machinery now. The
selected target-ID chain reuses verified signatures, ordinary calls, fault propagation
and GC roots. Costs are an allocation per binding (including static bindings), a linear
bounded dispatch, image growth at call sites and conservative recursion rejection.
This is a provisional correctness foundation, not a performance improvement claim.
Target-table/thunk dispatch and static-binding allocation policies can be measured once
real task consumers establish their costs. Interpreter callbacks already retain receivers;
no interpreter optimization or behavior change is implied by this work.

## Validation and next boundary

`callbacks.neoil` tests static and bound callbacks, a receiver reachable only through a
callback stored in another object, callback returns, 1,000 discarded bindings in 2 KiB,
returned references, no-result/Void distinction, null faults and divide-by-zero trace
parity. Sanitized host checks cover heap canaries, frame unlinking and quiescent collection.
Negative tests reject recursion, borrowed signatures and excessive dispatch candidates.

The [Raven callback samples](../../../benchmarks/native-web/README.md) compile through
PE/#Neo, run in both modes, and compare exact success/fault output. The standalone image
links only libSystem; the GC and fault adapters are included. See the reproduction script
and [recorded evidence](../../../benchmarks/native-web/callback-validation.json).
The initial callback slice advanced the full HTTP app to Function-array rejection.
The container follow-up below advances it again. Task dispatch, asynchronous socket
completion and host-held GC roots remain necessary before native HTTP benchmarking.

## Callback arrays and ArrayList (2026-10-08 follow-up)

The reference profile now carries exact Function shapes in managed arrays, generic
method specialization and closed ArrayList owners. Ordinary newarr initializes nulls;
array.reserve uses checked initialization markers. Indexed reads/stores, array length,
and callback/array null tests are supported. Function element borrows, nominal array
views, array/function equality and general value-record arrays remain unadmitted.
A callback of a different structural signature cannot be stored by treating both
values as machine pointers: original metadata verification and typed stack analysis
still enforce the exact element type.

The backend reuses the existing private String-array allocator and collector's initialized
pointer-slot layout. The historical C symbol `neoclr_allocate_strings_v1` and storage kind
name remain unchanged; they contain no String-specific dereference. Typed diagnostic
roots separately label `callable-array`. Each initialized slot traces its descriptor,
then its receiver. Overwriting a slot drops that edge at the next collection. Unwritten
reserved slots remain unreadable and are not traced; explicitly stored null is readable.
The layout costs eight bytes per pointer plus one byte per reserved initialization marker,
with the existing 65,536-element and caller heap-byte bounds. It is private image storage,
not a guest-visible ABI or a claim of .NET array covariance.

This extends the [managed array](../../managed-arrays.md) and
[checked reservation](../../reserved-array-capacity.md) designs and their .NET comparison.
Like copying elements of .NET reference arrays, copying these entries shares callbacks
and receivers; it does not clone receiver state. neoCLR additionally distinguishes
unwritten reserved slots from null. Reusing the current pointer-slot collector avoids
another array ABI and tracing kernel; the cost is historical internal naming and continued
conservative retention. No new language/compiler/library contract or performance claim
is introduced. The interpreter already implements this behavior.

`callback-arrays.neoil` holds a callback receiver solely through an array and repeatedly
replaces/clears another slot 1,000 times within 2 KiB. Native/interpreter checks compare
unwritten-slot, negative-index and default-null invocation faults. Negative tests reject
element borrows and incompatible Function shapes. `CallbackList.rvn` exercises the real
library's growth to 20 entries, Copy, mutation through shared callbacks, replacement,
1,000 discarded receivers and the surviving copied entries. The sanitized and standalone
native consumer matches the interpreter and links only libSystem.
[Container evidence](../../../benchmarks/native-web/callback-array-validation.json) records
reproduction and the next server boundary: arrays of `Result<Void, HttpError>` used by
its task state. [Reserved value-array storage](record-arrays.md) now handles admitted records; bounded
full-server selection and HTTP enum storage are next. The HTTP server itself has not yet run natively.

## Private host dispatch — 2026-10-08

GC-enabled images containing bound `fn<Void>` targets now export the experimental
`neoclr_invoke_void_callback_v1(handle, context)` entry. The host supplies an existing
strong root handle, not a raw code pointer. The helper validates a live Function
allocation and the generated dispatcher admits only selected targets of the exact
zero-argument, inhabited-Void shape. It calls the ordinary compiled body with its
retained receiver. The dispatcher neither resets the arena nor releases the handle.

This is a quiescent host adapter: no guest frame for that context may be active and
no prior guest fault may be pending. Invalid/stale/wrong-context/null/non-Function or
wrong-shape handles return RuntimeError (3) without creating a guest fault. Guest
execution failures preserve their existing code/message/frames; the host must stop
dispatch after a fault and clean up before resetting the invocation. A successful
callback may be invoked again; exactly-once I/O completion is the scheduler's separate
responsibility. Contexts belong to one image and thread; selected IDs are not stable
method IDs, persistent metadata or a cross-image ABI. No argument-bearing callback,
no-result callback, frame migration or runtime suspension is added.

This uses the existing Function/.NET delegate comparison above and the
[cross-runtime reassessment](../../runtime-scheduling-design.md#cross-runtime-reassessment--author-direction-2026-10-08).
Its bounded target chain favors implementation reuse over dispatch optimization.
Pending operations should own opaque runnable work and roots, allowing this callback
adapter to be replaced by runtime activation resumption later. No throughput claim
or extra microbenchmark is justified before that workload exists.

The sanitized `native_host_callbacks_preserve_roots_state_and_faults_without_entry_reset`
test compiles `host-callbacks.neoil`, dispatches static and bound callbacks after entry
returns, checks receiver mutation across GC, and compares divide-by-zero diagnostics
with the interpreter. It rejects wrong signatures, released/foreign/null/ordinary-object
handles, pending faults and active guest frames; checks output atomicity and canaries;
and releases/reclaims the entire heap. All five native GC kernel tests also pass.
The fixture discovers its descriptor by inspecting the private heap solely to isolate
codegen; actual services must receive and register callbacks while submitting work.
Host submission and task-queue/socket completion remain the next integration steps.

The existing Raven Callbacks consumer also passes interpreter, sanitized native and
standalone native execution after this change; the standalone binary links only
libSystem. [Recorded inputs and results](../../../benchmarks/native-web/host-callback-validation.json).
This consumer exercises existing guest invocation; host invocation is covered by the
focused fixture above. Full Server admission still stops at SocketConnectResult.

## HTTP ordinal-comparison prerequisite — 2026-10-08

The existing UTF-8 binding opt-in now includes exact StringCompareOrdinal(String,
String) -> Int32. The native helper compares bytes and normalizes the result to -1/0/1,
without allocation. For valid neoCLR text this is Unicode scalar ordering, preserving
[the existing String contract](../../raven-string-api.md); it deliberately differs from
.NET UTF-16 ordinal ordering for some supplementary/BMP pairs. No case folding or
normalization is introduced. Null input faults leave the result unpublished.

The expanded sanitized ordinal fixture compares 88 text/null cases against the
interpreter, including empty/prefix/embedded-NUL strings, composed/decomposed text and
U+10000 versus U+E000. It also rejects ordinary same-named methods with matching result
types. Full Server admission now reaches the specialized RegisterTaskQueue service.
This closes a missing binding, not a compiler lookup or guest text-semantics change.

## Closed class-interface callbacks (2026-10-08)

A Function bound through a nominal interface uses the existing closed-world
interface thunk and keeps the original receiver object alive. The selected
implementations must all take ordinary heap-class receivers; string/array
projections and borrowed value receivers remain rejected. The descriptor's target
is a private thunk identity, not a promised reflection/equality identity or ABI.
Neither target inspection nor hot replacement is added.

Focused sanitized native/interpreter checks cover two implementations, retained
mutable state across 1,000 garbage callback allocations, null binding/invocation,
callee fault frames, untouched output on failure and complete root/heap cleanup.
Three real Raven async samples also match both modes; see the
[sample assessment](../aot-sample-assessment/README.md). Entry task draining remains
a separate service gap. .NET's bound-interface delegates likewise retain a receiver;
this POC uses a bounded thunk table rather than exposing CLR delegate internals.
