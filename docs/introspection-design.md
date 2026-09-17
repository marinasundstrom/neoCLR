# Introspection, RuntimeContext, Reflection and Emit

Current author proposal, refined **2026-09-17**. This supersedes the public
Type/TypeInfo split and class-based descriptor direction in the
[earlier review](reflection-model-review.md). It is a target design, not a claim
that the current runtime already exposes these interfaces or RuntimeContext.

## Responsibilities

Introspection describes structure. RuntimeContext defines the execution universe.
Reflection operates dynamically against that universe. Emit constructs new program
structure. All use one descriptive model, with different backing implementations.

| Area | Responsibility | Not implied |
| --- | --- | --- |
| System.Introspection | Shared structural contracts | Invocation, activation or executable association |
| System.Runtime.RuntimeContext | The execution universe and its realized entities | A second public descriptor model |
| System.Runtime.Reflection | Bind descriptions to that context; invoke, activate and access members | Operations on every possible descriptor |
| System.Runtime.Emit | Construct descriptions, IL and assembly/module images | Loading or executing generated code |

## One interface-based model

The `*Info` names are interfaces, deliberately without an `I` prefix. Candidate
contracts include AssemblyInfo, ModuleInfo, TypeInfo, MemberInfo, MethodInfo,
ConstructorInfo, PropertyInfo, FieldInfo, EventInfo, ParameterInfo and AttributeInfo.
This is a vocabulary, not a requirement to add unused interfaces to v1.

There are no parallel public Type/TypeInfo, Assembly/AssemblyInfo or Module/ModuleInfo
pairs in the target model. Structural type references use TypeInfo throughout:
base types, interfaces, generic arguments, declaring types, return types and
parameter/property/field types. Runtime-backed implementations may retain opaque
handles internally without exposing another public type representation.

Illustrative shapes (exact Raven syntax and collection contracts remain to be fixed):

```text
MethodInfo
  Name: String
  DeclaringType: TypeInfo
  Parameters: List<ParameterInfo>
  ReturnType: TypeInfo

ParameterInfo
  Name: String
  Position: Int
  Type: TypeInfo

PropertyInfo
  Name: String
  DeclaringType: TypeInfo
  Type: TypeInfo
```

An interface model does not require copying .NET's MemberInfo inheritance tree.
Shared contracts or a future member union remain alternatives; do not introduce
MethodBase or builder-specific public descriptor hierarchies without a need.

TypeInfo need not eagerly materialize all metadata. Name, Namespace, Kind and
GenericArity may be cheap while Methods, Interfaces and Attributes require resolution.
The model describes semantics, not materialization cost. Allocation, caching and
retention guarantees must be specified and measured rather than inferred from names.

## Runtime-backed v1

The initial provider is the runtime. `System.Runtime.RuntimeContext` represents the
execution universe; conceptually `RuntimeContext.Current.Assemblies` exposes
AssemblyInfo, ModuleInfo, TypeInfo and member contracts from runtime metadata.
Ordinary type acquisition should return TypeInfo directly (`value.Type` or an
equivalent Raven operation); exact language lowering remains open.

Runtime-backed introspection is still descriptive. MethodInfo does not gain Invoke,
and PropertyInfo does not gain GetValue or SetValue because of its backing source.
The interfaces must not require that all future implementations be runtime-backed.

Two questions remain distinct:

- Structural resolution: what structure does this description represent?
- Reflection binding: what executable entity in this RuntimeContext corresponds
  to it, and is the requested operation permitted?

Reflection owns the second question. It can attach capabilities through composition
or Raven extensions. A conceptual `runtime.Reflection.TryBind(method)` illustrates
the boundary; neither that signature nor implicit selection of Current is settled.
A MethodInfo alone is not sufficient authority to invoke executable code.

## Emit and cross-origin composition

Reflection and Emit are sibling capabilities over the same introspection contracts.
Builders may implement them or expose an Info view, so analysis can accept existing
runtime definitions and unfinished generated definitions without backend downcasts.

Code generation does not imply execution. Emit may produce IL, metadata, modules or
an assembly image without loading it. Loading into a RuntimeContext is explicit;
the eventual Load API, validation, lifetime and authority rules remain future work.

Common interfaces alone do not solve mixed-origin identity. Before implementing
Emit binding, specify provenance, assembly/module identity and target-context
resolution/import rules. Do not equate types by display name or assume an arbitrary
implementation is executable. Validate mixed-origin signatures, generic arguments
and constraints, unfinished builders, same-named types from different assemblies,
equivalent definitions in distinct contexts and incompatible revisions. Require
clear binding diagnostics rather than representation-dependent late failures.
Automatic binding when unambiguous versus explicit import remains undecided.

## Deferred work

Full MetadataContext and metadata-file loading are outside v1. They require parsing,
signature decoding and cross-assembly resolution, plus metadata-backed implementations
of the same interfaces. Reserve the design space without adding an empty subsystem.

Adding MetadataContext later should not require redesigning the introspection model.
Opening metadata must never implicitly load it into a RuntimeContext. Future compiler,
analyzer, linker and Emit consumers should be able to inspect/generate without an
execution universe; one is needed only to run the result.

Typed introspection such as TypeInfo<T> or typed MethodInfo variants is also deferred.
Build the dynamic structural model first.

## Query ergonomics

**Author follow-up — 2026-09-17:** Keep BindingFlags for now. The interface migration
must retain existing flag-based queries and their filtering behavior. Do not remove
flags or require collection filtering as part of this work.

The refined proposal's longer-term option is collection properties (`type.Methods`, `type.Properties`, `type.Fields`,
`method.Parameters`) and ordinary filtering such as
`type.Methods.Where(m => m.Name == "Parse")`. This is not the immediate migration
contract. Convenience Find helpers can follow concrete needs.
Declared/inherited defaults, order, visibility, materialization and unavailable
metadata errors still need explicit contracts; changing syntax must not silently
change these semantics.

## Comparison and tradeoffs

Reuse the .NET Type/TypeInfo and MemberInfo baseline research in the
[review](reflection-model-review.md#baseline-and-problem) and
[hierarchy comparison](reflection-hierarchy.md). Keeping the existing .NET-inspired
classes is the smaller migration. The selected interface direction instead lets
runtime, generated and future metadata-backed descriptions share structural contracts
without inheriting executable operations. Costs include interface dispatch, adapter
and lifetime design, explicit binding policy and breaking existing public signatures.
No performance advantage or solved interoperability is claimed.

Before finalizing the cross-context contract, compare pinned .NET Reflection,
MetadataLoadContext and Emit probes and relevant alternative implementations under
the [research workflow](design-research.md). That evidence is still required; the
proposal does not assert that all such .NET combinations fail.

## Implementation and migration plan

Today the runtime still uses System.Type, class-based System.Introspection descriptors,
Type.Info and GetMethods/GetFields/GetProperties with BindingFlags. Type, TypeInfo
queries and ParameterInfo readers have Raven-authored implementations. See the
[implemented API](raven-reflection-api.md). RuntimeContext, dynamic Reflection, Emit
and the complete interface migration are not implemented.

The [isolated Raven probe](experiments/raven-target/introspection-v1/README.md)
exercises minimal TypeInfo/MemberInfo interfaces backed by existing runtime objects.
It keeps experimental and existing descriptors in separate assembly identities;
production migration is still required. BindingFlags is retained for acquisition.

1. Prove the smallest interface contract and a runtime-backed implementation in
   Raven; validate source, metadata and actual execution without claiming a complete
   provider model. Keep transitional System.Type use internal to that experiment.
2. Migrate the runtime/reference/consumer type identity together so public signatures
   close over TypeInfo. Define typeof/value.Type lowering, equality, lifetime and
   handle access before removing System.Type and Type.Info.
3. Add the minimum member contracts and runtime-backed RuntimeContext discovery
   needed by working demos. Preserve BindingFlags and test filtering semantics;
   reconsider collection query properties separately.
4. Add Reflection and Emit as separately validated capabilities. Offline metadata
   loading and typed introspection remain deferred.

Compile declarations for the POC only when they describe an intentional contract;
label unimplemented operations. Preserve existing working samples during the
transition, and update them with each integrated breaking slice. Do not merely
rename the old classes as interfaces or publish placeholder execution APIs.
