# Native imports, exports and AOT sidecars

Author-directed design, 2026-10-12. This describes the intended interface; it is not
an implemented export format or a promise that existing AOT artifacts expose it.

The author requests an elaborate import/export interface, explicitly using neoCLR's
structural function signatures rather than delegates. The intended distribution is
an AOT code artifact with a sidecar describing public functions, types, members and
additional mapping annotations. A compatible runtime or native binding tool should
be able to consume that contract without knowing the source language.

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

## Sequence and gates

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
