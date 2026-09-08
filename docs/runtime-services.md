# Runtime-service planning

The closed call-graph report now records logical runtime-service uses for each reachable
function. This identifies work a backend must provide through helpers or inline code;
it does not prescribe native symbols, a service ABI, mandatory ownership metadata,
or a garbage collector.

Explicit erasure reports `ValueStorage`: typed payload storage, value copies, exact
type testing and checked extraction. It does not imply native allocation or prescribe
GC/ownership. See [value storage](value-storage.md) for the interpreter subset.

```rust
let required = graph.required_services();
let missing = graph.missing_services(&[
    neoclr::RuntimeService::PointerMemory,
    neoclr::RuntimeService::ConsoleOutput,
]);
```

required_services returns a distinct list in RuntimeService enum order. missing_services
retains every use absent from the supplied set, including repeated uses. Each diagnostic
contains the report-local function index, service, and optional instruction index. None
identifies an import declaration; Some identifies a specialized IL instruction. The
function node retains the resolved signature and module/revision/definition identity.
Diagnostics follow function order, instruction order, then service order at the site.
Duplicate entries in the supplied service set have no additional effect.

## Current catalog

| Service | Direct uses |
| --- | --- |
| SlotReferences | ldloca/ldarga and the managed-reference operand path of ldflda/ldobj/stobj/initobj |
| InterfaceDispatch | interface.borrow and callvirt: explicit borrowed view formation, receiver access and implementation selection |
| TypeInspection | Type-only ldtoken and validated TypeName/TypeEquals/TypeArgumentCount/TypeArgument InternalCalls |
| ValueStorage | Explicit value.pack/value.is/value.unpack and current erased native return boundaries |
| NativeAllocation | heap.alloc and heap.free |
| FrameAllocation | localloc, including the frame-lifetime release contract |
| PointerMemory | allocation/free tracking, ptr.fromint, ptr.add, ldflda, typed/indirect memory loads and stores, object/block copy and initialization |
| ManagedHeap | heap.new and newarr (also require SlotReferences) |
| ManagedArrays | Array creation, length and element operations; access operations conservatively also require SlotReferences |
| ParseInt32 | Validated neoCLR.Runtime.ParseInt32 InternalCall |
| FormatInt32 | Validated neoCLR.Runtime.Int32ToString InternalCall |
| ConsoleOutput | Validated neoCLR.Runtime.WriteLine InternalCall |
| NativeInterop | P/Invoke declarations |
| StringOperations | Validated StringConcat, StringByteCount, and StringSliceUtf8 InternalCalls |
| ConsoleInput | Validated ConsoleReadByte InternalCall |
| FileInput | Validated bounded ReadAllText InternalCall |
| ErrorValues | Validated ErrorFromMessage and ErrorMessage InternalCalls |

heap.alloc/free report both NativeAllocation and PointerMemory; localloc reports both
FrameAllocation and PointerMemory. PointerMemory names the current pointer-access and
diagnostic contracts, not a mandate to retain the interpreter's side tables in all
backends. Frame storage can use a backend's own stack implementation. ManagedHeap
identifies managed allocation and tracing; SlotReferences supplies common checked
frame/heap address operations. See [heap references](heap-references.md).

Runtime helpers are classified through the same validated binding registry used by the
interpreter. A similar method name without InternalCall metadata grants no service
semantics. Import nodes are leaves: the catalog does not expand the internal dependencies
of a runtime helper or native library. Native library and entry-point metadata remain on
the function node, and analysis never loads or calls them.

The opcode classification is exhaustive, so new instruction variants require explicit
review. Instructions without a catalogued direct service remain ordinary backend work.
For example, newobj and field copying do not imply allocation, sizeof/alignof require
layout support rather than a service, and pointer casts/null values do not access memory.
String, record, union, and Fault representations still require backend implementations.

## Conservative scope

Uses are collected from each specialized body in the bounded closed call graph, including
syntactically unreachable instructions. Different closed generic instantiations have
separate sites; the aggregate requirement list deduplicates their services. Unreachable
functions outside the graph add no requirements. Analysis does not infer uses from a type
signature alone, optimize away operations, or prove pointer lifetime or initialization.

This is a service-set comparison, not a complete backend capability checker. No missing
services does not establish support for every opcode, type representation, target layout,
ABI, resource limit, cancellation protocol, or terminal Fault behavior. No backend is
selected or silently substituted. A future native backend must validate those contracts
before compilation, and may choose how to satisfy each logical service.

`cargo run --example reachability` reports HelloWorld's ConsoleOutput requirement and
its missing-service location when supplied with an empty service set. It executes no
HelloWorld guest code.

ldflda/ldobj/stobj/initobj conservatively report both PointerMemory and SlotReferences; service
analysis does not yet distinguish their operand kinds.
