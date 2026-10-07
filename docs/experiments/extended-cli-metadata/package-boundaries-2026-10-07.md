# Candidate distribution packages — 2026-10-07

**Status: development direction, not an implemented package format or release layout.**
The author suggested separate platform packages and potentially further packages.
Assembly boundaries and distribution packages answer different questions: an assembly
owns metadata identities; a package delivers a compatible set of artifacts. Moving an
assembly between packages must not change its metadata identity or silently select a
second owner for its types.

## Author-directed foundation rule (2026-10-07)

System.Runtime is the well-defined managed foundation. Higher-level packages depend
on its fundamental contracts; Runtime must not depend on Data, Networking or Web.
The author explicitly permits different boundaries from .NET when they benefit neoCLR.
This sets dependency direction, not a claim that the current Runtime candidate is
already minimal or that its exact membership is final.

Place an operation in Runtime because it owns a fundamental mechanism, not merely
because an optional package needs to call it. In this slice managed array reflection
belongs with Runtime's execution/reflection support, while serialization policy stays
in Data. Avoid moving JSON models into Runtime or exporting the whole internal service
facade. Evaluate further abstractions by dependency direction, reuse, API cost and
runtime constraints; no divergence is automatically an improvement.

## Candidate layout

| Package role (names provisional) | Contents | Dependencies and gate |
| --- | --- | --- |
| Managed foundation | System.Runtime assembly and API documentation | Explicit bootstrap contract; the independent Runtime/orders gate passes |
| Data | System.Data assembly and documentation | Runtime; separate JSON/array execution passes (development) |
| Networking | System.Networking assembly and documentation | Runtime and matching native service capability; separate cancellation/socket consumer passes (development) |
| Web | System.Web assembly and documentation | Runtime, Networking and Data; separate compilation exposes the internal shared-deadline boundary, execution remains open |
| Platform runtime, per OS/architecture | Native runtime executable and required native assets | Matching metadata/runtime-service contract; macOS arm64 is the current POC qualification target |
| Developer tools | Raven compiler integration, language server, disassembler and build support | Matching metadata contract plus current .NET host requirements; can initially be one bundle |
| Bootstrap/development inputs | Primitive core and retained service seed, ownership catalog and provenance | Explicitly development-only until their bootstrap role is eliminated |

A convenience SDK bundle can compose these packages. Do not require users to discover
all dependencies manually, and do not equate a package with a single DLL. Public IDs,
archive format, registry, installation mechanism and independent version policy remain
open. Avoid splitting compiler/LSP/disassembler into separate downloads without a
concrete distribution or update benefit.

The first platform package can contain the existing native executable. This proposal
does not imply pluggable native service modules or runtime dynamic loading. Native
networking/storage implementations may remain in that executable while their managed
APIs ship separately. Further native packages need evidence of an independent dependency,
ABI and deployment benefit. Likewise, splitting Runtime's remaining IO/reflection/tasks
surfaces needs dependency evidence; their presence does not make them a minimal core.

## Real project graph (author direction, 2026-10-07)

System.Networking, Data, Web and other selected libraries must become actual projects
with explicit project references to System.Runtime and required Platform integration
projects. Source lists in audit scripts are validation scaffolding, not the final build
model. Ship the outputs of the project graph together with matching runtime assets,
documentation and provenance. Runtime stays below optional APIs in dependency order.

After separate Web execution, inspect native Raven SDK project-reference support and
introduce the graph incrementally. Platform integration must expose an intentional
cross-assembly contract; moving internal service declarations to a new project alone
will not work. Exact integration project names and service membership remain open.
Validate clean builds, dependency ordering, source-free consumption, editor catalogs
and execution from the collected artifacts. Existing legacy bridge build targets do
not establish this native project gate.

## Unified documentation (author direction, 2026-10-07)

Keep one class-library API reference across the assembly/package split. Namespace/type
navigation and search are unified; actual declaring assembly is provenance on the page,
not a separate API section. A PE container or package name does not replace assembly
identity. This does not change binary ownership or dependency direction. The current
aggregate documentation bridge remains explicit until RavenDoc can consume the split
native inputs into one model. See [the migration gate](../../../api-docs/README.md#one-class-library-reference-across-assemblies).

## Compatibility and catalogs

Keep compile-time assembly references, native runtime service requirements and package
membership as distinct catalog facts. Record exact assembly identities, artifact hashes,
source/compiler/runtime revisions and bootstrap owners. A platform selection must not
silently change System.Object, System.Void, primitive or collection ownership.

The initial implementation should reuse the existing explicit dependency catalog and
qualification artifacts rather than add a package resolver now. Qualification must
compile a source-free consumer from the extracted layout and run it with that layout's
runtime. Missing dependencies, wrong platform artifacts, incompatible service contracts,
duplicate owners and stale artifacts must reject explicitly. Test compiler and LSP
reference resolution against the same catalog. Package selection must never reintroduce
CLI projections for native application/library references.

## Comparison and alternatives

.NET uses runtime identifiers to select platform-specific assets; its SDK dependency
model distinguishes native and managed runtime assets. See Microsoft's
[RID catalog](https://learn.microsoft.com/en-us/dotnet/core/rid-catalog) and the SDK's
[runtime configuration design](https://github.com/dotnet/sdk/blob/main/documentation/specs/runtime-configuration-file.md).
These are useful distinctions to reuse, not a requirement to implement NuGet restore
or its compatibility graph for neoCLR.

Keeping one qualified bundle is the lowest-complexity alternative and remains useful
for the POC. Separate managed/platform packages could avoid distributing unnecessary
platform binaries and allow optional libraries, at the cost of compatibility catalogs,
installation logic and more extracted-package tests. No size or speed improvement is
claimed before measurement. One package per namespace would multiply deployment work
without establishing useful assembly or native ABI boundaries; the current dependency
inventory is a better basis for additional splits.

## Next work, in dependency order

1. Close imported native Object authoring consistently: select an output-owned root
   reference from Raven symbols and exact host dependency identity; apply that identity
   to definition/builder override validation, authored/imported method references and
   native reader reconstruction. Test wrong owners, conflicting local/external roots,
   exact Equals signatures and runtime dispatch. Do not just relax AttachMethod.
2. Expose the bounded array reflection operations required by Data through a supported
   API; retain internal native service adapters and explicit errors. Execute JSON mapping
   and socket/task consumers against separately emitted optional libraries.
3. Compile Web and execute its HTTP/JSON consumers with the same dependency catalog.
4. Map those proven assemblies and the existing native runtime into an extracted package
   layout, then propagate the catalog through CLI/project/LSP and release tooling.

This refines delivery work without replacing the roadmap's immediate end-to-end
priority. See the [optional-library frontier](optional-library-frontier-2026-10-07.md)
and [assembly inventory](library-boundaries-2026-10-07.md).
