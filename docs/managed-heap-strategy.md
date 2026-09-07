# Managed heap strategy

Status: proposed next implementation strategy after checked managed reference
returns. The [slot contract](reference-slots.md) is implemented; this document does
not change newobj, initobj, constructors or the legacy Ref arena yet.

## One reference type, distinct storage lifetimes

Use T&/ByRef for both frame-backed access and managed heap references. Neither the
programmer nor a library wrapper manually retains, releases or invalidates them.
A reference identifies a root and an optional field path. Its public type describes
the referent; the root records the storage lifetime.

| Root | Lifetime and return rule |
| --- | --- |
| Ordinary frame storage | The frame owns the value. Returning an address into the current frame faults; a caller-backed address can return while the caller remains active |
| Managed heap storage | Independent of the allocating frame. Each live retaining reference keeps the allocation alive; returning it transfers a valid reference to the caller |

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
| Address a destination; initobj T | Default-initialize existing storage, with no constructor invocation | Extend native-only initobj to supported managed T& destinations and typed defaults |
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

The existing heap.new is a candidate for explicit placement. Evolving its result
from Ref<T> to T& would reuse the managed address/load/store path and remove the
separate arena-access model. A sequence that constructs an ordinary value then
places it on the heap is adequate only when construction has not exposed that
temporary's identity. Construction directly into the final destination is needed
before supporting heap-self references or publication during construction. Resolve
that contract before choosing a fused opcode or prefix; no encoding is assigned here.

CLR boxing is useful comparison material, but does not automatically select NeoCLR's
representation: CLR unbox converts an object reference to a managed address into its
boxed value. NeoCLR need not introduce another public reference category merely to
reproduce that sequence. See Microsoft's [unbox reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.unbox).

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

## Automatic retention and deterministic cleanup

Start with an acyclic managed heap subset. Reference copies preserve the root;
reference replacement or discard releases the old claim automatically. Once no
live roots/references retain an allocation, reclaim its storage. A heap registry
must use non-retaining bookkeeping so Execution does not accidentally own every
allocation until teardown as the current arena does.

Keep live-allocation limits separate from identity generation. Repeated allocate,
use and release must stay within a live-object limit; stale identities must never
refer to reused objects. Host implementation temporaries must not extend observable
guest lifetimes when destructor execution is introduced.

Next, reference-valued fields must participate in ordinary value-copy and release
rules. Copying an inline record copies its value fields and retains its reference
fields; Clonable remains an explicit operation. Retain incoming references before
releasing replaced ones so self-assignment and overlapping aliases remain safe.

Reference counting alone does not reclaim cycles. Do not silently enable arbitrary
cyclic stores and claim deterministic cleanup. Decide between checked cycle
restrictions for an initial subset, weak edges, and supplemental cycle handling
before enabling general reference-valued fields. A scoped borrow checker or manual
ownership discipline is not a prerequisite for the ordinary language experience.

Guest destruction is a separate next layer over these storage events. Dispatch it
through the runtime at defined execution boundaries, not from arbitrary Rust Drop
callbacks while slot storage is borrowed. Specify reentrancy, order, Fault cleanup
and cancellation first. Dispose and Close remain explicit resource protocols;
reference liveness does not mean a resource is still open.

## Bounded implementation order

1. Define typed default initialization and constructor destinations. Extend initobj
   to the supported managed T& subset, with independent checks when verification is
   skipped. Keep unsupported default states explicit.
2. Add managed heap root provenance, automatic retention and live-object accounting.
   Select explicit heap placement producing T&, using an existing operation where
   possible, while preserving ordinary value construction. Initially keep
   reference-valued fields gated.
3. Exercise heap-root and interior-field returns across frames, shared mutation,
   final-reference release, failed construction and repeated allocation under a
   live-object limit. Ensure current-frame address returns still fault.
4. Retire the legacy Ref/heap.new/load/store path in a coordinated preview migration.
   Review whether any existing heap operation remains useful as an internal lowering;
   retain no public ownership wrapper merely for compatibility.
5. Add reference-valued fields with a cycle policy, then destruction and host-rooted
   handles. Define native pinning and concurrency before permitting those boundaries.

Changing an existing allocation instruction's result type requires a format/version
transition, reassembly and
coordinated updates to the runtime library, examples, verifier, services and hosting.
Old artifacts must not be silently interpreted with new stack effects. Preserve
CLR instruction meanings where possible and identify intentional differences.
