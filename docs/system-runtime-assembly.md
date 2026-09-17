# System.Runtime: minimal managed foundation

Planned direction, requested by the author on **2026-09-17**. This is an assembly
and project boundary, not a completed rename or a namespace migration.

## Purpose and scope

Plan a Raven-authored `System.Runtime` project/assembly containing the minimal
managed foundation an application needs. Start from the foundational types now
being implemented through `runtime/raven/System.rvnproj`; do not add a second
competing implementation of them.

Candidate contents are the existing fundamental value/reference types, strings,
arrays, delegates, compiler-required contracts, basic Result/Option conventions,
and the minimum shared Introspection contracts and runtime context/resolution
support needed by the POC. Derive the precise set from working application and
compiler requirements, not from every type currently bundled under System.
Console, collections, I/O and other bundled conveniences need an explicit placement
audit; their present inclusion does not make every API a minimal-runtime requirement.

Assembly ownership and namespaces are separate. System.String remains in System;
TypeInfo remains in System.Introspection; RuntimeContext remains in System.Runtime.
Runtime*Info implementation classes are internal implementation details. Optional
Reflection and Emit capabilities must not become prerequisites for ordinary type
acquisition or descriptive queries. Additional assembly splits can follow need;
there is no requirement to scaffold all future projects now.

## .NET comparison and deliberate choice

.NET distinguishes compile-time reference assemblies from runtime implementations.
Its System.Runtime library exposes foundational APIs with implementation sources
including System.Private.CoreLib. That layering is a useful baseline, not a reason
to reproduce every facade or core-library assembly in this POC. Sources consulted
2026-09-17: [reference assemblies](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies)
and [System.Runtime library overview](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Runtime/README.md).

Prefer one clear managed foundation project for neoCLR initially, while keeping its
compiler reference contract separate from executable implementation artifacts. The
benefit is a smaller ownership/deployment model; the cost is less packaging flexibility
and a future compatibility decision if the library is split. No .NET binary
compatibility, footprint reduction or performance advantage is implied by the name.
Whether reference and implementation artifacts use one identity or explicit mapping
must be settled before migration, not inferred from filenames.

## Migration slices

1. Inventory the actual dependencies of the running POC and compiler contract;
   record which current System sources belong to the minimum assembly.
2. Establish the System.Runtime Raven project and its intentional assembly identity,
   continuing to generate executable neoIL as needed. Avoid duplicate type definitions.
3. Move reference generation and runtime-contract selection together. Today's
   NeoCLR.CoreProbe and NeoCLR.System authoring identities, generated manifests,
   bootstrap includes, importer identity checks and installed profiles must migrate
   coherently. A file rename is insufficient.
4. Compile and execute a managed sample against the new reference surface and
   implementation, including typeof -> RuntimeContext -> TypeInfo. Check that
   concrete Runtime*Info implementations are not public API and that no accidental
   host .NET assembly or optional Reflection/Emit dependency is introduced.
5. Document rebuild/migration requirements and update installation/tooling samples.

The immediate milestone remains **correct API boundaries and a running POC**.
Assembly packaging, complete runtime context discovery and every library backend
need not be finished before demonstrating the API. This plan does not rename any
published assembly or claim the new project exists yet.
