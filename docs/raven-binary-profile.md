# Raven binary experiment: CLI container and explicit dependency closure

Recorded 2026-09-12, on the experiment branch. This is the bounded format decision
for slice 3 of the [Raven target experiment](raven-target-experiment.md), informed by
[emitted metadata](experiments/raven-target/results.json). The dependency audit is
implemented as experiment tooling. The runtime binary reader and translation described
below are planned, not implemented.

## Decision and comparison

Use a **standard CLI PE container with ordinary metadata tables and CIL bytes** for
this experiment. Require an explicit neoCLR target profile at the load boundary. Do
not allocate new opcodes or interpret standard instruction bytes with different stack
behavior. The existing JSON/neoIL path remains available and unchanged.

| Alternative | Benefit | Cost / reason for choice |
| --- | --- | --- |
| CLI PE subset, translated to runtime IR | Reuses Raven's existing writer, metadata references and inspection tools; useful for a later C# frontend | Requires a real metadata reader, explicit admission rules and semantic translation. Selected for the bounded experiment. |
| New versioned binary encoding plus PE declaration facade | Could represent all current IR choices directly | Requires a new writer/backend and two representations of library signatures before the first program works. Defer until a demonstrated extension needs it. |
| Change the runtime to execute CLI semantics directly | Could reduce adaptation for multiple compilers | Larger contract change, especially inhabited Void and storage/reference behavior. Reassess after the bounded path rather than making it a prerequisite. |

This is tooling/implementation reuse, not evidence that neoCLR improves the CLI binary
format or can execute arbitrary .NET assemblies. The metadata and instruction baseline
is [ECMA-335, sixth edition](https://ecma-international.org/publications-and-standards/standards/ecma-335/),
Partitions II and III (consulted 2026-09-12). Raven's pinned writer dependencies and
adaptation points are in the [backend map](raven-backend-integration-map.md).

## Explicit dependency set

Each invocation must provide the application, permitted reference artifacts, the core
identity and a profile/version. No machine framework directory, current-directory search,
network lookup or host runtime fallback participates in target linking. For the first
profile, match full assembly identity exactly; reject duplicates, missing dependencies,
multiple modules and type forwarding. This deliberately omits .NET's broader assembly
binding policies. It makes the preview reproducible, at the cost of requiring an explicit
complete input set and postponing version unification.

The new `ClosureAudit` in the emission probe supplies its own Cecil resolver. It resolves
AssemblyRef, TypeRef and MemberRef rows exclusively against the supplied images. It rejects
the previously observed incomplete target and host-fallback output. A self-contained
metadata fixture passes; missing types/methods, a changed parameter signature, a mismatched
assembly version and duplicate input identities fail. None of these fixtures are executed.

**This is not complete binary admission.** The audit delegates member lookup to Cecil;
it does not establish exact CLI signature compatibility for every generic/modifier case,
validate all attribute payloads, check type layouts or verify IL. Malformed/cyclic hostile
metadata needs bounded parsing and controlled diagnostics before production use. The
`Closed` fixture demonstrates a complete small metadata graph, not a closed Raven core
library. The original target still fails; nothing was silently rewritten to make it pass.

Compiler binding also needs isolation. Microsoft's
[MetadataLoadContext documentation](https://learn.microsoft.com/en-us/dotnet/standard/assembly/inspect-contents-using-metadataloadcontext)
explains that dependency lookup is controlled by the supplied resolver (consulted
2026-09-12). Raven's pinned `Compilation.Setup` explicitly adds host core and trusted
platform assemblies. A future Raven target option must select target references/core
without that fallback while leaving host compiler execution available. The output audit
catches leakage but does not fix binding, overload selection or diagnostics upstream.

## First executable profile: proposed limits

Start with one module, one static parameterless entry point and statically resolved
calls. Use CLI void, Int32 and String signatures for the initial programs; map these
explicitly to neoCLR's existing primitives. Reject generic signatures, instance dispatch,
managed-reference signatures, pointers, exception regions, native entry points and
unsupported instruction forms until their contracts are implemented. These limits apply
to this input profile, not to neoCLR's existing feature set.

Raven's generated Unit, attribute and Program constructor helpers exceed the tiny
executable subset. They must be addressed explicitly: either target emission omits
unneeded helpers or the reader validates supported metadata while admitting only the
reachable executable subset. Do not delete unresolved definitions opportunistically or
pretend that source Main is the entire assembly. This choice and a usable core declaration
artifact are prerequisites to claiming the first executable profile complete.

The first library projection binds the declared Console method to the real neoCLR System
Console implementation and its runtime call. Do not load the probe's empty WriteLine as
an implementation. Choose the actual core identity with that reference artifact; the
`NeoCLR.Probe.System` and `Closed` names are fixtures, not published platform identities.

## Call/return translation to validate next

The selected experiment direction is signature-driven translation at the input boundary:

| CLI input | Internal neoCLR operation | Validation obligation |
| --- | --- | --- |
| Static call returning no value | Call the mapped method, then discard its inhabited Void result | Arguments match the imported signature; the mapped method really returns Void. |
| Return from a no-result method | Construct Void, then return | CLI evaluation stack is empty before translation. |
| Int32-returning call / return | Preserve its one result | Exact compatible argument/result stack types. |
| String literal | Load the decoded string | Valid user-string token and configured size limit. |

This implements the boundary proposed in the [minimal contract](raven-minimal-target.md),
not a changed meaning for standard CIL. Retain input byte offset/token to internal
instruction mappings, including inserted operations, for diagnostics and later debugging.
A converter must verify the input stack before inserting anything; it must not repair an
invalid CLI return by dropping arbitrary values. Empty methods, nested no-result calls,
Int32 returns and rejected invalid stacks are required before enabling execution.

The production reader must also bound artifact size, table/heap sizes, method bodies,
recursion and dependency count, validate token/table kinds and branch targets, and report
unsupported features before execution. Numerical limits, Rust reader selection, profile
encoding/version negotiation and the library closure remain implementation decisions for
the next slice. A target profile supplied by the caller is required even for unmarked PE;
ordinary .NET executables must never be guessed to be neoCLR artifacts.

## Slice 4 implementation update

Raven now has an opt-in `MetadataImportOptions` API on its integration branch. This
selects the metadata core and excludes implicit host assemblies from the metadata
resolver. Default .NET imports remain unchanged, and import-policy changes block
incremental state reuse. The updated [probe](experiments/raven-target/README.md) tests
Console binding and omission through this mode. This addresses the tested compiler
isolation prerequisite; a complete neoCLR core reference artifact, helper handling,
full profile validation and runtime execution remain open.

## Core declaration follow-up

The [core reference artifact](raven-core-declarations.md) now closes the metadata
requirements of the static-call corpus. The earlier core-declaration gap is resolved
for those inputs only. Executable System binding, helper admission, binary parsing and
stack verification remain required. The follow-up also corrects the earlier synthetic
mscorlib entry in the application inventory.

## Runtime-first return convention (2026-09-12)

The author clarified that most compatibility work should be in neoCLR, with minimal
Raven changes of general utility. [No-result methods](no-result-methods.md) now implement
standard static no-result call/return behavior directly in the runtime. This supersedes
the above proposal to insert Void construction/disposal at every imported IL call/return.
Existing inhabited-Void System calls still need an explicit binding boundary. Binary
loading is not implemented by this return-convention slice.
