# Dual-target end-to-end implementation

Author-approved plan, 2026-10-03. The first gate is a broad application plus a
source-built library subset, not the entire runtime library. The author accepted an
explicit CLI primitive bootstrap and documented runtime seed for this gate. Native
application/rebuilt-library references must use direct metadata import.

## Ordered slices and acceptance

1. Paired driver-level Hello/helper and separate library/consumer execution baseline.
2. Route rvnc neoclr references through the direct native symbol importer, with exact
   dependency validation and failure before output publication.
3. Complete introspection constructor/classification/accessibility/parameter facts
   needed by these consumers; retain language rules in Raven.
4. External interface contracts across contracts, implementation and consumer assemblies;
   evolve metadata writer/reader/verifier/runtime together and preserve compatibility.
5. Establish one declared owner per bootstrap/library type, resolving source/seed iteration
   identity; build collection contracts, ArrayList, comparers/HashMap, Option/Result/query
   dependencies as independent libraries.
6. Run unchanged application-order-collections against those libraries on both targets;
   fix reduced regressions at their owning layer instead of rewriting consumers.
7. Focused regression stabilization and a refreshed full-library gap inventory.

Success requires ordinary compiler commands, consumer compilation without library source,
expected stdout/exit/identity/mutation behavior and explicit artifact/revision records.
Reference-only bridge emission does not count as runtime success.

## Boundaries and defaults

Keep the existing .NET Reflection/Emit backend; a Cecil migration is later work.
Metadata readers -> introspection -> Raven symbols; symbols/lowered program -> compiler
emission contracts -> target builders -> definitions -> metadata -> PE. Library ILGenerator
and compiler emission contracts are independent. Importer objects never become emitter
operands. Introspection owns metadata resolution/substitution; Raven owns binding,
inference, overload resolution, conversions and accessibility decisions.

Retain .NET-compatible metadata/instruction behavior where supported. Capabilities expose
extensions and unsupported categories explicitly. Keep structural Function on feature
branches. Isolate independently useful compiler fixes for later main-line validation.
Each verified slice gets changelog/documentation and a commit; no performance claim without
measurement. API changes get C# contract tests and host reference documentation. Broader
runtime-library completion remains subsequent work.

Initial paired-driver baseline: 3/4 cases pass; native library consumption fails with
NEOMETA001 undeclared instance field. Both .NET cases pass. See
[baseline evidence](../experiments/extended-cli-metadata/dual-driver-baseline-2026-10-03.json).


## Direct native compiler command (2026-10-03)

`rvnc neoclr --core-reference NeoCLR.CoreProbe.dll --reference Library.dll -o App.dll App.rvn`
now imports API-produced native assemblies using NeoClrMetadataReference and artifact-only
emission bindings. It selects CompilationOptions.NeoCLR and its explicit primitive/runtime
contracts. Native references never fall back to CLI projection. The core reference is
required when --reference is supplied; this is an experimental command compatibility change.
Without native references/core selection, the existing host primitive bootstrap remains
available. --system-symbols/--system-method is labelled legacy and remains an explicit
partial callable projection, not a native library import fallback.

Paired driver acceptance now uses `--dual-driver <rvnc.dll> <neoclr> <NeoCLR.CoreProbe.dll>
<fresh-output>`. Both .NET and NeoCLR Hello/helper and separate generic library consumers
execute (42); the consumer has no library source. Duplicate identities, missing transitive
dependencies, ordinary CLI references, malformed input and unsupported source reject without
publishing output. Host .NET runtime execution uses an explicitly hashed runtimeconfig.
The wider source-library/collections gate remains open; runtime encoding is unchanged.

Declaration-fact slice complete for the current by-value signature profile: metadata
accessibility/flags and constructors are facade-owned and consumed by Raven. Existing
accessor associations remain canonical. Byref/out still fail reader materialization.
Next: external interface declarations across separately compiled native assemblies.


External-interface gate baseline (2026-10-03): the paired harness now also supports
--dual-driver-external / --dual-driver-external-inventory with the same driver/runtime/core/
output arguments. Contracts, Box<T> implementation and consumer compile in separate
invocations; source is removed before downstream compilation. The .NET split executes 42.
Native Hello passes, but native implementation emission rejects in SourceTypePlan before
encoding because the interface identity is external. This is a failing future acceptance
case, not a claim that the external-interface slice is complete.

Required changes span SourceTypePlan/SourceInterfacePlan capability admission, symbol-only
external relationship authoring in the NeoCLR adapter, metadata relationship validation/
round-trip, and runtime dependency/dispatch verification. Current writer/reader relationship
attachment requires owned definitions. Do not simply remove admission checks or duplicate
contracts into the implementation assembly. No compiler/runtime behavior changes in this
baseline test slice.
