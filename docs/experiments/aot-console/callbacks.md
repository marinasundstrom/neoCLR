# Native stored callbacks — work in progress

The HTTP app stores callbacks in tasks and cancellation objects. The reference-arena
AOT profile now admits the existing native Function metadata for static functions and
nonvirtual heap-class methods. This extends execution of already-supported metadata
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
shape are admitted. Recursive call graphs, including conservative possible callback
edges, remain rejected pending a native stack budget; this can reject a program whose
actual dynamic path is acyclic. Borrowed/output signatures, callback operations inside
output-parameter methods, virtual/interface-bound receivers, Function equality,
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
