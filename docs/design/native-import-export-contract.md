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
2. Add Raven/native metadata authoring for that binding and shared JIT/AOT lowering.
3. Produce a small AOT library with fixed-width scalar exports and a versioned sidecar;
   generate/validate a C consumer against those exports. Include artifact mismatch and
   unsupported-ABI rejection, and avoid claiming arbitrary public members are exportable.
4. Add stable record layouts, bounded buffers and opaque owned handles with explicit
   error and release protocols. Qualify targets individually.
5. Add callbacks and managed-state exports with runtime entry, rooting, unregistration
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
