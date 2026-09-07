# Managed heap strategy

Status: direct [heap-backed T&](heap-references.md), checked frame references, typed
initobj and tracing GC are implemented. Constructor destinations and source-language
heap construction remain future work. Memory management reaches this milestone before
the next [object-model slices](object-hierarchy.md).

## One reference type, distinct storage lifetimes

Use T&/ByRef for both frame-backed access and managed heap references. Neither the
programmer nor a library wrapper manually retains, releases or invalidates them.
A reference identifies a root and an optional field path. Its public type describes
the referent; the root records the storage lifetime.

| Root | Lifetime and return rule |
| --- | --- |
| Ordinary frame storage | The frame owns the value. Returning an address into the current frame faults; a caller-backed address can return while the caller remains active |
| Managed heap storage | Independent of the allocating frame. Reachability keeps the allocation alive; returning a reference adds it to the caller's roots |

An interior field reference retains the whole heap root, even when it is the only
remaining reference to that root. Replacing the root's value preserves identity and
field paths. A local containing T& owns a reference binding; taking/loading that
reference is distinct from taking the address of the local binding itself.

The runtime needs explicit root provenance. A host Rc allocation is not proof of a
guest heap allocation. Preserve current-frame return checks and allow a heap-root
return only when managed allocation established that lifetime. Raw Ptr<T> addresses
remain separate and acquire no retention merely by being copied.

## Familiar MSIL patterns before selecting allocation syntax

MSIL distinguishes initialization, constructor invocation and construction of a new
result. initobj initializes an addressed destination without invoking a constructor;
newobj invokes a constructor and can also produce a value-type result on the
evaluation stack. See Microsoft's [initobj reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.initobj)
and [newobj reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.newobj).
The [call instruction](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.call)
uses a supplied instance receiver and can target a constructor.

Recommended consistency rule: preserve these roles, then expose allocation placement
explicitly at the use site. NeoCLR has no permanent value/reference flag on a type,
so it cannot copy CLR allocation selection through a class/valuetype distinction.
An instruction's stack effect must remain knowable from its operands or an explicit
modifier; the receiving local's type must not silently choose allocation behavior.

| Pattern | Role | NeoCLR next step |
| --- | --- | --- |
| Address a destination; initobj T | Default-initialize existing storage, with no constructor invocation | Managed typed defaults are implemented for scalars and recursively supported records |
| Address a destination; arguments; call .ctor | Construct into supplied storage | Add destination-oriented construction; current ordinary calls copy their receiver |
| Arguments; newobj .ctor | Construct a fresh result | Preserve the existing ordinary-value path while deciding explicit heap placement |
| Explicit managed heap placement | Establish an independently managed root and return T& | Evolve an existing heap operation or define a narrow construction modifier |
| ldloca/ldarga; ldflda; ldobj/stobj | Form references, project fields, read and replace values | Keep the same operations for frame and future heap targets |

Source T(args) constructs an ordinary value; new T(args) explicitly requests managed
heap construction. Source keywords need not map one-to-one to IL mnemonics. Physical
register/stack placement remains a backend choice, but cannot authorize a reference
to escape its owning frame.

A heap-only newobj plus value construction through initobj/call is still an option,
but it deliberately removes CLR newobj's value-construction role. A newval instruction
would also duplicate a role already available through newobj. Neither is selected.
Prefer retaining recognizable patterns unless a concrete lowering gap justifies the
change; do not make a broad breaking rename merely to match instruction names to
source keywords.

heap.new now performs explicit placement and returns T&, sharing the managed
address/load/store path. The former Ref and heap.load/store split is removed. A sequence that constructs an ordinary value then
places it on the heap is adequate only when construction has not exposed that
temporary's identity. Construction directly into the final destination is needed
before supporting heap-self references or publication during construction. An explicit
copy of an existing scoped value to a fresh heap root remains useful independently:
it creates a new identity, preserves old aliases to the original and applies ordinary
value-copy semantics. See [copying to the heap](lifecycle.md#explicitly-copy-an-existing-value-to-the-heap). Resolve
that contract before choosing a fused opcode or prefix; no encoding is assigned here.

Managed heap placement is direct. No boxing/unboxing bridge is needed to obtain T&,
and a future Object base must not impose one. Allocation mode remains independent
of type inheritance.

initobj is not a substitute for invoking a constructor. A constructor need not first
fabricate a default value if it fully initializes its destination. Specify which
types admit default initialization. Native zero bytes cannot stand in for arbitrary
String, reference-containing or invariant-bearing guest values. Managed references
currently have no null state; default/reference semantics need an explicit decision.
Do not remove today's aggregate-value construction until the replacement can express
records whose fields have no default value.

## Construction and publication

Use one construction protocol for frame and heap destinations, with different
storage owners. A heap allocation reserves a root and its live-allocation budget
before running the constructor. The constructor accesses its destination through
a checked initialization capability. Publish an ordinary readable T& only after
the required initialization succeeds.

Construction into supplied storage must preserve the same root throughout; do not
construct a temporary whose address can escape and then copy it into a different
heap location. Uninitialized references cannot be returned, erased or stored into
ordinary reference fields. Define field initialization tracking and receiver access
before permitting partial initialization through ldflda.

Failed construction releases reserved storage and its budget. Once cleanup metadata
exists, release initialized contained state exactly once; do not run a full-object
destructor on an incompletely constructed instance. Constructor failure remains a
terminal Fault under the current runtime model. Recoverable factories can return
ordinary Result values.

## Automatic memory management and resource cleanup

Tracing GC is the normal managed heap policy. Copies of references preserve identity;
roots and their transitive reference graphs keep allocations alive. Unreachable
objects, including cycles, become eligible for collection. Losing the final reference
does not promise immediate destruction. No manual ownership discipline is required.

The implemented collector separates live-object limits from monotonically increasing
identities and scans active frames, inline values and heap edges. It also collects
at execution completion, retaining the result's reachable graph. See
[the collector contract](garbage-collection.md) for boundaries and limitations.

Copying an inline record copies its value fields and preserves embedded reference
identity; Clonable remains an explicit operation. The T& heap representation
participates in tracing through fields, erased payloads and interface views.
General cycles are supported by GC; an acyclic ownership restriction is unnecessary.

Dispose and Close provide timely resource cleanup independently of GC reachability.
Frame-owned value destruction, if introduced, has separate deterministic boundaries.
Guest destructors/finalizers need explicit ordering, reentrancy and failure contracts;
do not dispatch them from arbitrary Rust Drop callbacks or infer them from Disposable.

## Bounded implementation order

1. Maintain the direct T& allocation milestone: heap/field/interface roots, cycles,
   scope escape checks, heap-only stored references, and bounded collection diagnostics.
2. Begin object-model slices after this memory milestone. Define optional bases,
   base views and dispatch without making inheritance select an allocation mode.
   Adapt layout and tracing so a base view preserves the complete derived root.
3. Define constructor destinations and partial initialization, rooted throughout
   construction; preserve ordinary value construction and explicit heap placement.
4. Specify block-scope enforcement, persistent host roots, native pinning, weak
   guest references and concurrency. Value destruction/finalization remain separate.

Changing an existing allocation instruction's result type requires a format/version
transition, reassembly and
coordinated updates to the runtime library, examples, verifier, services and hosting.
Old artifacts must not be silently interpreted with new stack effects. Preserve
CLR instruction meanings where possible and identify intentional differences.
