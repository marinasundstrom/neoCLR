# Native imports, exports and AOT sidecars

Author-directed design, 2026-10-12. This describes the intended interface; it is not
an implemented export format or a promise that existing AOT artifacts expose it.

The author requests an elaborate import/export interface, explicitly using neoCLR's
structural function signatures rather than delegates. The intended distribution is
an AOT code artifact with a sidecar describing public functions, types, members and
additional mapping annotations. A compatible runtime or native binding tool should
be able to consume that contract without knowing the source language.

## Runtime and module dependency boundaries — 2026-10-22

Author direction: intrinsically runtime-owned mechanisms may live in the Rust core
and be reached through InternalCall. Native implementations owned by a library
module should instead use NativeImport and separate native support libraries.
This follows the .NET-style distinction between runtime services and platform
invocation; provider ownership, not implementation language, determines the boundary.

The dependency direction is:

- Applications depend on public Raven library APIs.
- Library modules depend on runtime contracts and, where needed, their own native
  support library through an explicit C ABI.
- The runtime core depends on its foundational host/OS services; it must not acquire
  a dependency on an optional module's implementation or native support library.
- Native support libraries must not include private GC/object-layout headers or
  call private runtime symbols. Any necessary runtime interaction passes through
  an explicitly supported runtime contract or an adapter owned by the runtime.

Introspection and Reflection are runtime services: their authoritative metadata,
member resolution and type identity belong behind InternalCall. HTTP client/server
modules may own native protocol or transport support behind NativeImport, with
runtime-owned scheduling/rooting adapters where required. Both routes require
performance qualification; separate one-time linking/resolution from per-call
conversion, allocation and synchronization costs.

GC allocation/rooting, managed-reference access, runtime type identity, exception
machinery and scheduler/thread attachment are runtime responsibilities. An InternalCall
may implement those responsibilities in Rust; it is not a shortcut for arbitrary
module functionality. Pure byte/numeric transformations and external library APIs
are candidates for NativeImport when their ABI and ownership can be stated independently.
For example, the native byte formatter can be separated from the adapter that
allocates and roots its resulting managed String. Moving the latter wholesale
would leak runtime ownership across the boundary.

AOT must preserve these same contracts. Internal calls lower to matched runtime
implementations; native imports lower to native ABI adapters and declared library
symbols. Packaging must include the appropriate support artifacts and retain ABI,
architecture and dependency identity. Static versus dynamic linkage is a deployment
choice, not permission for a module to depend on private runtime internals. Static
NativeImport binding is a future implementation option, not currently qualified
functionality: the present scalar macOS ARM64 resolver loads dynamic libraries.

Before migrating a service, qualify its managed-visible behavior, error/ownership
rules and import/export signatures in interpreter, JIT and AOT; test the extracted
package without relying on checkout libraries. Measure call/marshalling overhead,
startup, allocation and binary-size consequences where relevant. NativeImport is
not presumed faster than InternalCall. Current ordinary managed-backend import,
callback and pinning gaps remain release work; this direction does not claim they
are resolved or require moving every existing adapter before the next preview.

## Code and metadata have separate jobs

The executable/shared library contains callable entry points. The sidecar describes
how to locate and use them; it is not executable code. Public managed type/member
metadata alone does not establish a C-compatible ABI. The export contract must
identify direct native representations and generated adaptation stubs explicitly.
Existing CLI bridge annotations remain temporary transport, not the final format.

A versioned contract should cover:

- Artifact identity and a hash/build identifier tying metadata to the exact code;
  target architecture, pointer width, endianness and supported ABI version.
- Stable export identities and symbols (not process-specific addresses), structural
  parameter/result signatures, calling conventions and mapping annotations.
- Public type/member identities plus explicit native size, alignment, packing,
  field offsets, enum storage and nested layout where direct layout is supported.
- Parameter direction, encoding, length/capacity relationships, nullability,
  ownership, release functions and callback retention/unregistration requirements.
- Runtime initialization, thread attachment, error/fault boundaries and destruction
  requirements for exports that enter managed execution.

Managed object layout must not leak as a portable native layout. Types without a
published native representation need an opaque handle or an explicit generated
conversion. A native consumer may call direct exports by ordinary ABI rules; exports
that operate on managed state must arrange runtime entry and GC coordination through
an adapter. Rich metadata does not make an arbitrary native pointer safe to invoke.

## Trust and ownership across the boundary

Author clarification, 2026-10-12: each side can have its own memory management and
security model. A foreign call trusts the other implementation to honor the agreed
ABI and data contract. neoCLR does not require a foreign receiver to participate in
its GC, use its allocator, or adopt its security model. Metadata describes the
agreement; it cannot prove foreign code follows it. In-process FFI is not a sandbox.

The author further clarifies that unsafe describes leaving neoCLR's guarantees,
not an implementation being unmanaged. A foreign implementation may itself use
another managed runtime, collector or security model; that does not establish
neoCLR's guarantees across the call. Conversely, a native implementation can belong
to neoCLR's trusted runtime and uphold a safe managed service contract. InternalCall
and NativeImport therefore continue to identify different providers rather than
classify machine code as inherently safe or unsafe.

The design criterion is which ABI, memory, lifetime and runtime-entry obligations
neoCLR can establish, and which require explicit external trust. A wrapper may
present a safe interface when its implementation upholds those obligations for all
permitted inputs; a signature annotation alone cannot establish that. This is a
contract principle, not a newly implemented source-level unsafe modifier, wrapper
verification mechanism or security sandbox. The existing trusted execution entry
points remain the interpreter's enforcement boundary.

Each side remains responsible for storage it owns. Passing a value or address does
not by itself transfer ownership: the contract distinguishes a borrowed view, an
independent copy and an ownership transfer, including which allocator or release
operation applies. The receiver decides how to use received data within that
agreement. neoCLR can retain managed references or adapter state while the receiver
uses a native pointer; foreign retention is not automatically visible to the GC.
Likewise, wrapping a returned native callable does not make its external state
managed or establish how that state is released.

Runtime checks protect neoCLR's own invariants and reject locally detectable
mismatches. They do not validate the foreign implementation's memory or security
model, or promise recovery from arbitrary native misuse. A callback entering neoCLR
must still satisfy its runtime-entry and lifetime requirements. The current
call-scoped adapter is a bounded implementation, not a requirement that every
foreign interface share that lifetime policy. Longer-lived mappings need an explicit
way to keep the managed adapter alive and eventually release it; the native side
can use its own ownership mechanism without adopting a neoCLR registration API.

## One signature model, several bindings

The structural Function shape describes managed input/output types. A native ABI
mapping adds native representation and calling rules without changing that managed
shape. Binding instance data distinguishes a managed method, a named native import,
or a raw native address; separate export records describe callable native entry points.
No delegate type is introduced. Runtime-owned InternalCall providers remain distinct.

The initial address-binding experiment retains the existing per-slot ABI vocabulary
and resolves no synthetic managed target. It is deliberately not an export annotation
or a final sidecar schema. ABI descriptors should be reusable by future exported
entry stubs and generated native bindings after validation of the reverse direction.

## Earlier sequence and gates (2026-10-12)

The 2026-10-22 sequence below supersedes these priorities; completed evidence remains valid.

1. Bind a returned native address to a structural Function and invoke it using the
   existing marshaller. Reject null/data pointers and unsupported signatures before
   entry; retain loaded code owners. Qualified scope is the interpreter first.
2. Author follow-through on 2026-10-12 brings both callable directions forward:
   pass a managed Function to native code and wrap a returned native pointer as a
   Function. Main now includes the interpreter implementation of call-scoped, same-thread scalar
   callbacks and typed callable results. See [the bounded contract](../native-interop.md#callable-parameters-and-results-development).
   Raven/native metadata authoring and shared JIT/AOT lowering remain next gates.
3. Produce a small AOT library with fixed-width scalar exports and a versioned sidecar;
   generate/validate a C consumer against those exports. Include artifact mismatch and
   unsupported-ABI rejection, and avoid claiming arbitrary public members are exportable.
4. Add stable record layouts, bounded buffers and opaque owned handles with explicit
   error and release protocols. Qualify targets individually.
5. Extend call-scoped callbacks to retained registrations and managed-state exports
   with runtime entry, rooting, unregistration
   and in-flight call teardown. Never allow a managed fault to unwind through an
   unspecified foreign ABI.

The initial sidecar schema, export annotation spelling and compatibility policy remain
open design work. The user direction establishes the goal, not acceptance of every
field or sequence above. Packaging metadata and the implementation must be validated
together before any format is declared stable.

## Comparison

.NET exposes native-pointer/delegate adapters and unmanaged entry points; neoCLR uses
structural Function shapes rather than delegate declarations. Reusing shape metadata
reduces parallel type machinery, but requires explicit ABI/ownership data and tooling
for consumers that do not understand managed types. .NET's unmanaged entry-point
restrictions illustrate why a public managed signature is not automatically an export.

Primary references reviewed 2026-10-12:
[GetDelegateForFunctionPointer](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.marshal.getdelegateforfunctionpointer),
[GetFunctionPointerForDelegate](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.marshal.getfunctionpointerfordelegate),
[UnmanagedCallersOnly](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.unmanagedcallersonlyattribute).
These are comparative external contracts, not neoCLR implementation evidence.


## Main integration decision — 2026-10-12

The author explicitly requests merging the bounded implementation now and leaving
expansion as future work. This supersedes compiler-readiness gating for this FFI
slice only. Interpreter address bindings, callable parameters and callable results
are included; the remaining sequence above describes potential follow-through,
not conditions for this merge. No source attribute spelling, retained ownership
API, native export format or AOT sidecar is finalized by that decision.


## Language-neutral interoperability direction — 2026-10-22

Author direction: AOT compilation is the primary deployment target for Raven and
managed languages on neoCLR. JIT and interpretation remain valid execution modes,
including development and appropriate smaller-device deployments. This is a target
and sequencing decision, not a claim that all current features work in AOT.

The author describes the eventual goal as “mix-and-match universes”: languages with
different representations and ownership systems can use runtime services and each
other's code through metadata and the object/component model. The immediate work is
NativeImport, not a new component framework or universal reference-counted wrapper.

### Sequence

| Stage | Author-directed capability | Status and boundary |
| --- | --- | --- |
| 1 | NativeImport into managed languages, with pointers, stack/heap allocation and managed-memory pinning | Interpreter imports and tracked native allocation exist; memory integration, pinning and JIT/AOT imports remain incomplete |
| 2 | Referenceable metadata bound to native implementations by the runtime | Future contract; managed languages reference it first, other consumers can follow |
| 3 | Native runtime API extending hosting with type/member queries and invocation | Future bindings for native languages and other language runtimes |
| 4 | Dynamic dispatch for dynamically typed languages | Future resolution path over shared type/member and invocation semantics |

Do not require stages 2–4 for the first NativeImport release. Do not infer acceptance
of a binary schema, export annotation, Ref<T> API or universal object layout from
this direction. Component metadata must distinguish type identity, public contract,
implementation binding and native representation. Existing managed layout is not a
portable foreign ABI.

### First-stage acceptance gates

The following are assistant-proposed implementation gates within the author's scope.
They are not descriptions of shipped functionality.

- Use the target C ABI with explicit calling convention and supported scalar,
  pointer and aggregate layouts. A struct constraint alone does not establish
  reference-free storage or ABI compatibility. Reject unsupported shapes before entry.
- Qualify imported pointer input/output and no-result calls in an AOT application;
  carry the same semantics to JIT and interpreter, with explicit capability rejection
  until each path supports them. Interpreter success alone does not close this gate.
- Qualify stack storage lifetime, bounds and alignment, and heap allocation/free with
  overflow, failure and allocator-pairing behavior. Native memory ownership is separate
  from pointer identity; borrowed or foreign-owned addresses must not be freed by the
  wrong allocator.
- Establish native-compatible managed byte storage before exposing a true pin. Keep
  the owner alive, stabilize its address for the pin lifetime, preserve subview offsets
  and validate access during GC. Empty views need a defined no-dereference policy.
  Interpreter arrays currently contain tagged Values, so a stationary array is not
  a C byte buffer. No implementation may label a copied buffer as pinned storage.
- Use scoped pins for call-duration borrowing. A retained pointer requires an explicit
  longer-lived lease that lasts until foreign access has actually ended, including
  cancellation teardown. A lexical scope alone cannot guarantee this for asynchronous
  native operations. Native resource disposal must not race an active borrow.
- Keep Slice/Memory as managed projections; use pointer plus count at the C boundary.
  Copying is an explicit fallback with bounds and initialization checks, not a pinning
  substitute. ReadOnlySlice cannot prevent a foreign function from writing illegally.
- Preserve existing callback limitations until retained callbacks, foreign-thread entry,
  in-flight teardown and fault barriers are qualified. No implicit exception unwinding
  through an unspecified foreign ABI, and no automatic Promise allocation requirement.

Use built-in runtime tests plus a small C fixture for the buffer contract. Include
write-back, subviews, UTF-8 byte access, GC while pinned, multiple pins, release,
invalid layouts and lifetime rejection where the relevant capability is implemented.
Report managed allocations separately from native allocations and boundary overhead;
zero managed transfer objects does not imply a zero-cost call. Compare the same
consumer across supported execution modes rather than claiming parity from metadata.

### Trust and memory management

The VM's sandbox remains a requirement. Arbitrary trusted in-process native imports
are outside its enforceable memory-safety boundary; metadata and AOT compilation do
not themselves provide confinement. A sandboxed consumer needs controlled native
capabilities, and untrusted native implementations need an enforceable isolation
boundary. This direction does not grant all guests arbitrary library loading.

The native runtime API should expose stable handles and documented runtime entry,
rooting, release and thread rules. A handle retaining a GC object is distinct from a
pin stabilizing its storage and from ownership of a native allocation. Foreign
implementations may use GC, explicit ownership or reference counting internally.
Ref<T> is deferred; any future retain/release protocol must specify copying, teardown,
thread safety and cycles rather than imply that reference counting solves all lifetimes.

### Comparison and tradeoffs

.NET remains the API/semantic baseline. Modern LibraryImport, NativeMemory,
SafeHandle and Memory pinning already address substantial parts of this problem.
neoCLR's intended benefit is a consistent UTF-8, Slice/Memory and metadata contract
across frontends and execution engines, not novelty from renaming these mechanisms.
Costs include compiler/runtime layout coordination, GC pin support, per-target ABI
validation and adapters for representations that cannot be shared directly.

Rust contributes explicit borrowing/ownership obligations and callback teardown;
these can inform a foreign boundary without imposing Rust's whole ownership model.
Go's cgo rules distinguish call-scoped pointers from retained pinned storage and
restrict references reachable through foreign-visible memory. Node-API illustrates
a stable C boundary with opaque values and explicit reference lifetimes, without
requiring consumers to know an engine's object layout. These are comparative models,
not declarations of Rust, Go or JavaScript ABI compatibility.

Primary references reviewed for this discussion:

- [.NET native interoperability guidance](https://learn.microsoft.com/en-us/dotnet/standard/native-interop/best-practices)
- [.NET Memory ownership and leases](https://learn.microsoft.com/en-us/dotnet/standard/memory-and-spans/memory-t-usage-guidelines)
- [.NET MemoryManager.Pin](https://learn.microsoft.com/en-us/dotnet/api/system.buffers.memorymanager-1.pin?view=net-10.0)
- [Rust FFI ownership and callbacks](https://doc.rust-lang.org/stable/nomicon/ffi.html)
- [Go cgo pointer rules](https://pkg.go.dev/cmd/cgo)
- [Node-API ABI and object lifetime](https://nodejs.org/download/release/v26.9.0/docs/api/n-api.html)

The byte-copy experiment is separate from this architecture. Its initial RAV2001
failure was subsequently reduced to invalid test literals: `0b` is a binary prefix,
not byte zero. Explicit `(byte)0` fixes that fixture error; direct slice collection
binding was not the cause. A subsequent default-pointer failure reproduces on ordinary
.NET and is repaired in shared Raven default typing/emission (main 9193bdff1, native
fa8ecb50c). Native 21c279632 also closes empty-slice literal lowering. The
[copy API evidence](../experiments/native-buffers/README.md) now qualifies the bounded
interpreter fallback; real pinning and AOT imports remain separate gates.


## Declaration-level projections

Author clarification, 2026-10-22: map language types at the NativeImport declaration
so callers need not manually unwrap raw pointers. `Ptr<T>` should project to one
native pointer when its pointee contract is supported. A structural Function should
project to a compatible native callable through an adapter where required, rather
than requiring callers to manufacture an integer address.

Keep source types and ABI descriptors separate. Recognize the exact pointer wrapper
contract; do not infer ABI compatibility from arbitrary one-field structs or from
`T: struct`. Unsupported layouts and calling conventions must fail before invocation.
A typed pointer improves expressiveness but does not validate foreign memory.

For foreign function pointers, the callable must retain the library owner. For
managed callbacks, the adapter must retain captures, arrange runtime/thread entry,
and contain managed faults at the native boundary. A call-scoped callback may not
escape; a retained callback requires explicit registration and release. Neither
mapping implies automatic ownership transfer or permanent pinning.

Compared with .NET's explicit interop declarations and marshalling adapters (see the existing research references), the intended benefit is consistent use of neoCLR's existing
Ptr and structural Function types. Costs include adapter generation, lifetime
metadata, validation and backend qualification. This is a provisional mapping design,
not a claim that typed signatures alone make native interop safe. The current
[scalar AOT proof](../experiments/aot-native-imports/README.md) handles Int32 only;
full declaration projections, retained callbacks and pinning remain validation gates.


## Void values and native no-result calls

Author clarification, 2026-10-22: neoCLR Void is deliberately an inhabited normal
type. Raven's separate Unit representation on .NET addresses a CLR restriction;
it is not a reason to introduce Unit into neoIL. The assistant's earlier suggestion
to rename the inhabited type is withdrawn.

A Void value result and a no-result call still have different operand-stack effects.
A C void signature has no return register. When language semantics need a Void value
from such an operation, the compiler must explicitly adapt the no-result call to
that value contract; otherwise it must not manufacture an unused result. Structural
callable identity and verification must preserve the difference. The implementation
currently records Void plus a no_result flag for no-result calls. Standardization
of the representation/spelling remains open; no type rename, new schema or automatic
adapter is claimed by the bounded native-call increment.


### Canonical Void contract — author clarification

The author subsequently specifies that `void` is an alias for `System.Void`:
qualification and casing must not create different return conventions. A directly
Void-returning declaration has no result, following CLR call/return behavior, while
neoCLR additionally permits Void as a real type in value/storage and generic type
positions. There must be no special `noresult` IL keyword. Compiler-generated value
materialization is driven by the language's expression requirements.

Raven source inspection confirms that its expression emitter synthesizes a unit
value only for a preserved no-result invocation. It separately recognizes a generic
return whose original declaration returns a type parameter: that call already
supplies the value, so it must not synthesize a second one. Its runtime-unit projection
maps storage to the configured neoCLR System.Void while retaining CLI void returns.
This is source-inspection evidence, not fresh end-to-end qualification of the new IL
spelling. The earlier case/qualification split is rejected. The migration must cover
assembler syntax, signature rendering, producers and fixtures together, and validate
direct returns, used/discarded expressions and generic value returns. The binary
return convention must preserve generic call behavior; eliminating a public keyword
must not accidentally change a generic value-return ABI. Implementation is pending.

### Syntax migration evidence — 2026-10-22

The first implementation increment removes the obsolete `noresult` spelling from
active text producers, bootstrap fixtures and generated export inventories. The
assembler accepts `-> void` and `fn<void>` and rejects the old spelling. Internal
return-mode metadata and original-generic value ABIs are unchanged. This does not
yet implement the alias-parity decision above: legacy named-Void return bodies and
callers must be migrated together before that decision can be marked complete.

Validation: 14 no-result, 18 structural-function, 10 generic-function, 28 interpreter
native-import, seven AOT native-import, 24 scalar AOT and one frontend round-trip
test pass. All 24 website checks pass. The temporary Raven CLI bridge builds with
11 existing nullable warnings. The metadata SDK runs 185/191 tests successfully;
the same six failures reproduce with the previous display spelling (field-count
limits, authored function-reference expectations, grapheme naming and generic
signature expectations). These remain stabilization gaps, not successful release
qualification. Two SDK build blockers were fixed separately; the focused generic
definition-token check passes. Raven compiler semantics were not changed.

### IL name clarification and InternalCall prerequisite — 2026-10-22

Author direction: accept the keyword `void` or qualified `System.Void`; reject
bare `Void`. Both accepted spellings must have identical direct-return semantics.
This syntax rule remains pending alongside migrating legacy bodies/callers.

InternalCall admission now checks the declared return convention against the
signature, without a separate allowlist of service names permitted to return no
result. Exact service identity, parameter types and result types remain mandatory;
no-result metadata with a non-Void result is rejected before specialized binding.
This aligns the interpreter service boundary with ordinary CLR void-return behavior
and avoids maintaining a second incomplete list of Void-returning services. It
changes no native C ABI, authorization boundary or public library signatures.
Generic value-return services retain their original return convention.

The new interpreter tests execute real TaskQueue registration and worker completion,
check caller-stack preservation, retained callback data and queue identity, and
preserve the original UserFault. Signature tests reject incorrect parameter/result
contracts and protect generic value-return conventions. The existing native C import
ABI is unchanged. Website/native-interop capability claims remain unchanged; no new
public API or native backend support is introduced by this prerequisite.

Validation: 11 native-binding unit tests (including focused reruns), 15 no-result
integration tests, all 11 TaskQueue tests, five native-allocation service tests and
one no-result worker-notification test pass: 43 distinct tests. The initial minimal
queue fixture lacked the real TaskQueue method contract; it was replaced with a
consumer of the actual library. No website build was needed for this internal fix.

## Export sequencing — 2026-10-22

The author directs completing foreign function NativeImport across interpreter,
JIT and AOT before prioritizing name mangling for native AOT libraries. The intended
consumers are other native libraries and neoCLR itself importing compiled AOT
binaries through NativeImport. Metadata sidecars mapped to those binaries come later.
This preserves the earlier code/metadata separation while making the order explicit.

The assistant proposes treating symbol identity and the callable ABI together:
name stability alone cannot expose the current private status/result convention
as an ordinary foreign signature. No mangling format, default export visibility,
compatibility version or sidecar schema is selected by this exchange. The later
review should compare .NET-style explicitly named C exports with encoded identities
for overloads and generic specializations, including collision handling, reproducible
builds, runtime entry/ownership and library versioning. This is planned design work;
existing experimental AOT symbols do not become public exports through this note.

## Free-function C export increment — 2026-10-22

The author now makes the immediate scope explicit: native functions and pointer
memory must work across interpreter, JIT and AOT, including importing an AOT-built
binary through the same mechanisms. Class/object mapping is future work. The author
also requests NativeExportAttribute as the reverse source contract to NativeImport.
The backend increment below precedes that attribute; it is not the complete source API.

The assistant selects platform C calling conventions for direct exports, following
[.NET Native AOT C entry points](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop).
[Rust likewise separates symbol names from ABI](https://doc.rust-lang.org/reference/abi.html).
neoCLR's private status/result ABI stays internal. Each exported adapter calls it,
returns the declared C result on success and traps on an escaping fault. There is
no invented zero result, foreign unwinding or hidden thread-local error channel.
Callers needing recoverable failures must design an explicit result/status contract
in their exported function. Fault termination is a material limitation of this first
direct-export boundary; no recoverable public fault ABI is introduced here.

The macOS ARM64 development driver accepts:
`--native-library input output.o --export Function [--export Other ...]`.
It emits a relocatable object for the native linker, requires explicit selections
and never exports private helpers or the experimental neoclr_entry_v2 entry.
Libraries containing NativeImport declarations additionally require --native-imports
and link the same imports.c resolver. Existing explicit metadata load contexts apply.
No managed object layout, heap initialization, thread attachment or callback
registration is implied by this scalar library mode.

### Symbol identity v1

Logical names have this ASCII form:

`neoclr_c_v1_m<assembly-utf8-hex>_n<namespace-utf8-hex>_f<function-utf8-hex>_a<parameter-tags>_r<return-tag>`

Parameter tags are separated by underscores; an empty parameter list has no tags.
Tags are i32/u32/i64/u64/is/us/f32/f64, with v for a no-result return and pv/pu8/pi32
for the admitted pointers. This is a versioned development contract, not an encoded
managed object ABI. Native linkers may apply their normal object-format decoration;
NativeImport.EntryPoint uses the logical name without the Mach-O underscore.

Assembly simple name and preserved source function identity are used for native
metadata. Executable content hashes, definition row numbers, revision hashes,
body contents and file paths do not participate. Legacy assembler modules use their
declared module and function names. UTF-8 is encoded exactly; no Unicode normalization
or hashing introduces collisions. Full signatures distinguish scalar types and return
contracts. Overloaded source selection remains unsupported by the scalar backend;
the naming scheme does not claim otherwise.

Benefits over opaque content-derived names are deterministic interfaces and no hash
collision policy. Costs are long, less readable names and a convention that must be
versioned when changed. Explicit entry-point overrides belong to the forthcoming
attribute, analogous to .NET's named exports. Duplicate exports and unsupported
signatures reject before object output. The JSON stdout report is build output only,
not the future runtime metadata sidecar.

The Raven Exports.rvn fixture is compiled to a shared library. A native C consumer
reads/writes pointer storage and a subprocess checks escaping-fault termination.
The shared Raven transport consumer imports the generated names for allocation,
read/release and numeric returns through interpreter, scalar JIT and AOT. The object
symbol table is checked for exactly the requested exports. This establishes the
bounded free-function round trip, not ordinary SDK integration or full marshalling
and callback parity.


### Source exports and generated headers — 2026-10-22

NativeExportAttribute now selects source free functions in the opt-in macOS ARM64
scalar native-library backend. Ordinary custom-attribute metadata is sufficient:
the backend resolves the attribute owner through the loaded type's source identity,
not an interpretation of content-hashed executable names. Constructors choose
automatic versioned mangling or an exact name, with Cdecl/System conventions.
The underlying native mapping is unchanged; no CLR managed layout crosses the
boundary. Unlike .NET NativeAOT's explicit EntryPoint export selection, the default
name here is versioned source/signature mangling. That supports automatic names
at the cost of a neoCLR-specific symbol contract; explicit names remain available.

Optional --header emits C/C++ declarations for admitted scalars and pointers.
C/C++ identifier validation happens before output; existing files are preserved.
The source fixture uses both automatic and explicit export names. Native C and C++
consumers compile against the generated header, and NativeImport re-imports the
same library in interpreter, scalar JIT and AOT. Escaping faults still terminate
the process. This does not add callback parameters or managed-layout marshalling.

The author's clarified boundary is C conventions first, with neoCLR runtime
conventions inside AOT output. Opaque pointers can support C wrappers around C++
objects without describing their layout; direct C++ ABI/vtable mapping is deferred.
Callback signatures should eventually map to C function pointers and header
typedefs, but retained callbacks/captured state require an explicit lifetime model.

## Native struct layout and pointer metadata — 2026-10-22

Development support on macOS ARM64 now separates native storage layout from the
managed value layout and the aggregate calling ABI. The scalar backend admits
pointers to checked closed value structs. Its generated C11/C++17 headers include
source-identity-derived struct names and size, alignment and field-offset assertions.
Sequential packing, minimum size, nested values, self pointers and explicit overlapping
fields are covered. Explicit fields use named packed views inside a union; access uses
`field_<encoded-name>.value`. Auto layout, managed fields, invalid offsets/packing,
unresolved annotations and recursive by-value storage reject before output.

The metadata layout decoder recognizes canonical StructLayoutAttribute and
FieldOffsetAttribute identities and field MarshalAs Bool8/Bool32. Boolean storage
defaults to four-byte BOOL, following .NET native marshalling. This is a native storage
description, not automatic conversion of a managed Boolean field or permission to
pass a struct by value. Aggregate register/stack classification remains unimplemented.
The comparison is grounded in [default marshalling](https://learn.microsoft.com/en-us/dotnet/framework/interop/default-marshalling-behavior)
and [blittable types](https://learn.microsoft.com/en-gb/dotnet/framework/interop/blittable-and-non-blittable-types);
[AAPCS64](https://github.com/ARM-software/abi-aa/blob/main/aapcs64/aapcs64.rst) distinguishes
aggregate transport from memory layout. Qualification remains Apple ARM64 only.

Raven's native emitter and metadata SDK now preserve nominal value-pointer targets.
The compiler opts into a value-pointer capability; ordinary .NET/default portable
profiles are unchanged. This machinery is absent from Raven main and remains on
codex/neoclr. Existing pointer metadata is reused, with ordinary nominal identity;
no new Runtime Contract option or lossy CLI bridge encoding is introduced.

Initial checkpoint limits (resolved by the public-attribute increment below): Raven
source StructLayout/FieldOffset APIs and field MarshalAs admission were pending. Native-library layout decoding now resolves physical attribute owners
through the supplied System.Runtime source identity. It normalizes only the three
recognized layout annotations in a private emission inventory; original constructor
references and assembly metadata remain in the verified source program. An unrelated
assembly or mismatched full assembly identity does not authorize a physical owner. Raven Point pointer exports now compile and execute from C11/C++17 callers.
The native-library backend verifies the original source, then selects exported
functions and their direct-call dependencies. Unused synthesized constructors remain
in source metadata but do not enter scalar code generation. Unsupported selected
bodies and malformed unused bodies still reject before output. The header retains
nominal layouts and source identity. Ordinary scalar executable/JIT admission is
unchanged; no installed-SDK or by-value struct qualification is claimed.

## Public layout attributes and generated Raven imports — 2026-10-22

Raven now exposes LayoutKind, StructLayoutAttribute and FieldOffsetAttribute in
System.Runtime.InteropServices. MarshalAsAttribute accepts instance fields of value
types; the native emitter preserves their annotations instead of treating them as
misplaced import declarations. Sequential packing, explicit offsets/overlap, minimum
size and Boolean Bool8/Bool32 field representations reach the native layout decoder.
The attribute API records values; unsupported layout/marshalling is rejected before
native output. Managed object storage is not automatically converted or pinned.
This is the native-storage subset of the .NET attribute model, not aggregate-by-value
ABI support, class marshalling or parameter/result MarshalAs parity across engines.

A general parser bug was fixed on Raven main as 2073ee766 and carried deliberately
to codex/neoclr: following field attributes no longer parse as array suffixes on the
previous type. Valid multiline array ranks remain supported. Forty-nine focused
parser tests pass on main and fifty on the native target line. No new Runtime Contract setting
or bridge encoding is required; attributes retain ordinary source identities and
field-targeted native metadata. The native emitter capability change is target-only.

The native-library command optionally emits `--raven-imports Imports.rvn` together
with `--import-library <loader-name-or-path>`. Numbered NativeExport0/1/... declarations
bind exact native symbols. Struct declarations use explicit checked native offsets
and physical field widths; Bool8 storage becomes byte, not an implicit managed
Boolean conversion. The file is binding source, not a native runtime sidecar. The
object, C header and Raven file are created without replacing existing files; a
later output failure removes only files created by the current invocation.

Validation: all 23 native import/export integration tests and 19 focused native unit
tests pass. Source fixtures cover packed Boolean storage, explicit overlapping
fields/minimum size, Auto/invalid Pack/missing FieldOffset/incompatible MarshalAs
rejection, C11/C++17 callers and generated-import interpreter calls back into the
AOT library. All supported pack values produce C/C++ headers without unsupported
packing pragmas. Built-in Rust integration tests are used because the scalar backend
cannot execute the managed Raven test runner. Generated struct imports are not yet
qualified through the ordinary managed JIT/AOT executable pipeline.

## Compiled import call-site caches — 2026-10-22

The macOS ARM64 scalar JIT/AOT import thunk now owns an aligned, writable
eight-byte address slot. A warm invocation atomically loads that slot and calls
the native target with its mapped signature, bypassing resolver lookup and its
mutex. Successful cold resolution publishes the address atomically. Concurrent
cold callers can resolve redundantly; the shared resolver serializes library
ownership and returns the same retained address. Failed lookups publish nothing
and remain retryable, preserving the caller's output on failure.

The resolver owns successful library handles for process lifetime, so cached
addresses cannot outlive their libraries. The generated image owns the slot; JIT
image disposal still requires all invocations to have completed. No resolver ABI,
native signature mapping or library-unloading contract changed. This implements
the author's intended compiled-call fast path without implying that marshalling,
copying or pinning is free. It does not expand the ordinary managed-backend import
profile or the interpreter's implementation.

A regression uses eight concurrent cold callers, a failed initial lookup and five
million warm calls, checking exact results and zero further resolver invocations.
A local instrumented fixture compares the same native function and real resolver
before/after; its timings include wrapper instrumentation and do not establish
HTTP throughput or cross-platform performance.

Validation: all 26 native import/export integration tests pass, including Raven
source fixtures and generated bindings. API source/reference checks and all 5,681
website pages pass.
