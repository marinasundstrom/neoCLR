# System.Runtime: minimal managed foundation

Requested on **2026-09-17**, with project establishment directed on **2026-09-19**.
The first implementation slice establishes `runtime/raven/System.Runtime.rvnproj`
and its `System.Runtime` managed implementation identity. This is an assembly/project
boundary, not a namespace migration or a completed RuntimeContext API migration.

## Purpose and scope

Use the Raven-authored `System.Runtime` project/assembly containing the minimal
managed foundation an application needs. Start from the foundational types now
ported through the former `runtime/raven/System.rvnproj`; do not add a second
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
The current bootstrap deliberately uses explicit mapping: NeoCLR.CoreProbe supplies
checked compiler reference metadata, System.Runtime.dll supplies compiled Raven
implementation inputs, and runtime/System.neoil supplies executable library code.
The importer matches complete admitted contracts across those identities; no reference
stub body executes. The project rename changes the implementation identity, while
consumer reference identity and runtime module identity retain their explicit mapping.
Replacing the production introspection contract is the next coordinated slice.

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
need not be finished before demonstrating the API. No published assembly is renamed. All 73 existing source slices remain in the one
project, including the Console, collection and I/O conveniences used by current
working programs. Their inclusion is an explicit bootstrap choice, not a claim that
each is irreducible or that future packaging must stay monolithic. Native services
remain runtime-owned; the project contains the managed types and implementations.
