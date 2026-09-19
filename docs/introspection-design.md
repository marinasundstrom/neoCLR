# Introspection, RuntimeContext, Reflection and Emit

Current author proposal, refined **2026-09-17**. This supersedes the public
Type/TypeInfo split and class-based descriptor direction in the
[earlier review](reflection-model-review.md). It is a target design, not a claim
that the current runtime already exposes these interfaces or RuntimeContext.

## Author-directed implementation step — 2026-09-19

After completing the Raven source port, establish the System.Runtime project
(currently System), then implement the basic System.Runtime.RuntimeContext with
runtime-backed model implementations. Retire the public System.Type/Type.Info split
in favor of System.Introspection.TypeInfo. Object.GetTypeInfo() is the canonical
instance acquisition method; typeof(T) remains declared-type acquisition through
the selected RuntimeContext contract. This supersedes the tentative Object.GetType
and value.Type spellings discussed earlier.

These are implementation directions, not completed API claims. Keep concrete
Runtime*Info providers internal and structural signatures within the Info model.
Preserve the current BindingFlags query behavior. The smallest working context and
provider model precedes optional invocation, metadata-file loading and Emit. The
project may make documented breaking changes while preserving useful .NET ergonomics.

### Implementation checks for the selected acquisition API

Research refreshed 2026-09-19 against the .NET 10 API documentation:
[Object.GetType](https://learn.microsoft.com/en-us/dotnet/api/system.object.gettype?view=net-10.0)
returns the actual instance type, including through a base-typed reference;
[TypeInfo](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.typeinfo?view=net-10.0)
is a class in .NET's reflection model. neoCLR deliberately chooses an interface
and direct Object.GetTypeInfo acquisition instead of retaining the public Type/Info
pair. This changes source and metadata compatibility and requires rebuilding callers.
It does not claim that .NET lacks runtime type acquisition or descriptor APIs.

A local comparison executed on .NET 10.0.0 checks derived-instance acquisition
through a base reference, conversion to TypeInfo, distinct generic arguments and
array element types, and null-instance failure. All checks pass. This is a behavioral
baseline, not evidence for cross-context binding or a performance comparison.

The production migration must preserve actual allocation type through base/interface
views; substituting the receiver's declared type would violate the selected API.
Typeof remains declared-type resolution. Member and parameter descriptions must
return TypeInfo throughout. Concrete runtime providers remain internal, queries
retain BindingFlags filtering, and descriptor identity uses resolved metadata
identity rather than display names. A single current execution universe is the
initial scope; multiple contexts, interning and cross-context binding need separate
contracts before those capabilities are exposed.

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

## Sealed model decision — 2026-09-19

The author selected a sealed introspection hierarchy for the current development
stage, using Raven's support for sealed interfaces. The Info interfaces are public
but closed to external implementations. Runtime*Info providers remain internal;
concrete providers are sealed leaves, and the shared runtime member base belongs
to a closed hierarchy. No additional universal Info base is introduced merely to
express this policy. Future AssemblyInfo and ModuleInfo contracts follow the same
closed-model direction.

This supersedes the assistant's in-progress open-interface sample and its proposed
external implementation check. Future metadata/Emit providers require an intentional
extension to the permitted family. The benefit is a controlled set of representations
while runtime identity and lifetime contracts are developing; the cost is reduced
third-party extensibility and a compatibility change when the permitted family grows.
Structural use of Info interfaces remains independent of dynamic invocation.

Raven's closed interfaces use permitted-type custom metadata rather than the CLI
Sealed bit. The reference assembly must preserve that metadata and include internal
permitted-type definitions for the compiler to resolve, without exposing them as
public API. Compare Raven's
[sealed hierarchy specification](https://github.com/marinasundstrom/raven/blob/main/docs/lang/spec/inheritance-and-partial-types.md#sealed-hierarchies-and-permits):
ordinary CLR interfaces do not enforce this language-level closure for C# callers.
The neoCLR target importer must independently reject external implementations of
these known model contracts and validate the admitted provider family. This does
not add general closed-hierarchy enforcement to raw neoIL metadata or execution.

## Collection return contracts under review — 2026-09-19

The author asked whether APIs should return arrays at all, or a suitable interface
that conveys the collection. No library-wide prohibition or replacement has been
selected. The assistant recommends choosing the interface by the capabilities the
result promises, with Sequence<T> as the leading candidate for materialized
introspection results. Existing neoCLR contracts provide:

- Iterable<T>: iteration.
- Collection<T>: iteration and Count, without mutation operations.
- Sequence<T>: Collection<T> plus indexed reads.
- MutableSequence<T>: indexed writes in addition to Sequence<T>.

Sequence<T> fits parameter lists and other indexed metadata snapshots while leaving
storage private. Collection<T> is an alternative when count matters but indexed
access should not be promised; Iterable<T> allows streaming without promising count
or random access. An interface alone promises neither immutability nor snapshot
semantics: specify ordering, repeat enumeration, lifetime, deferred failure and
whether later context changes appear. A read-only view over a mutable array also
does not prevent another alias from mutating that array.

.NET ships both array-returning Assembly.GetTypes and enumerable
[Assembly.DefinedTypes](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.assembly.definedtypes?view=net-10.0).
Its [collection design guidance](https://learn.microsoft.com/en-us/dotnet/standard/design-guidelines/guidelines-for-collections)
prefers collection abstractions generally while retaining low-level array uses;
that guidance is explicitly an older, 2008-edition excerpt, not a current universal
rule. For neoCLR, interface results buy implementation flexibility and clearer
capability promises at the cost of dispatch/wrapper overhead and possible loss of
array-specific operations. Measure allocation and repeated-query costs. Arrays may
remain appropriate for explicit buffers or caller-owned storage; a universal ban
is not justified by the introspection use case alone.

The in-progress provider migration retains array signatures as a transitional
contract. Converting those results should be a deliberate subsequent API slice,
with tests for enumeration, indexing/count where promised, empty results, aliasing,
GC lifetime and inability to mutate provider-owned data through the public contract.

## Runtime-backed v1

The initial provider is the runtime. `System.Runtime.RuntimeContext` represents the
execution universe. The selected first discovery surface is the executing assembly,
returned as AssemblyInfo, from which callers can query modules and types. The
provisional spelling is `RuntimeContext.Current.ExecutingAssembly`; the author
selected the responsibility, not this exact member spelling. An all-loaded-assemblies
inventory remains a possible extension, not a prerequisite for the preview.
Instance acquisition is `Object.GetTypeInfo()`; declared-type acquisition is
`typeof(T)` through the selected context resolver.

### Context-owned discovery — 2026-09-19

The author clarified that RuntimeContext should own many of the static operations
that .NET places on Assembly, making the context the actual entry point for runtime
discovery. AssemblyInfo describes one assembly and exposes its modules and types;
ModuleInfo describes one module and exposes its types. These interfaces do not own
ambient runtime discovery or loading. The first implementation covers discovery of
the executing assembly and traversal of its model. Additional resolution and loading
operations need explicit contracts before being added to RuntimeContext.

The .NET comparison is
[Assembly.GetExecutingAssembly](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.assembly.getexecutingassembly?view=net-10.0),
which returns the assembly containing the executing code. Executing, entry and calling
assemblies are distinct concepts. The neoCLR implementation must identify the user
code requesting discovery, not accidentally report the System.Runtime helper's own
assembly or substitute the program entry assembly. Validate this across a call into
another assembly; frame selection is a runtime concern, not a name-based lookup.

The benefit of moving discovery to a context is explicit ownership of runtime
resolution while leaving Info contracts usable by other metadata providers. The cost
is a deliberate source/API divergence from .NET and the need to specify context
selection and lifetime. This direction does not require implementing all Assembly
static APIs, multiple contexts, or loading for the initial preview. Concrete
Runtime*Info providers remain internal. These paragraphs describe the selected
contract, not a completed production API.

### Possible context capabilities — 2026-09-19

The author suggested that other services might later belong to RuntimeContext,
including an optional garbage collector instead of a standalone GC class. This is
an exploratory direction, not a selected API or a change to the current collector.

.NET exposes collector control and observations through the static
[System.GC API](https://learn.microsoft.com/en-us/dotnet/api/system.gc?view=net-10.0).
A context-associated collector capability could make availability and the affected
runtime explicit. Keeping a standalone static facade is an alternative with familiar
.NET ergonomics but less explicit context selection. The context capability costs
additional availability handling and requires precise heap and lifetime semantics.

Before implementation, distinguish an absent public collection-control capability
from an execution mode without a collector. Neither implies the other. Define which
heap collection and statistics concern, whether contexts share that heap, how
cross-context references remain rooted, and what reclaims allocations when collection
is unavailable. Context association alone must not promise one private collector per
context or that a collector can be disabled or replaced safely. Validate those
contracts before exposing control operations. No member name, optionality encoding,
collector interface or non-GC execution mode is selected here; the discovery preview
continues independently of this future design work.

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

### Hidden implementations and unified type acquisition — 2026-09-17

The author clarifies that RuntimeTypeInfo and other Runtime*Info implementations
must not be exposed as public API. Consumers use the Info interfaces, not concrete
constructors or backend casts. The runtime nevertheless knows the implementation
needed to resolve a type handle or equivalent type-of instruction result.

Type acquisition follows a runtime-owned path: a handle/token resolves to a hidden
RuntimeTypeInfo implementation returned as TypeInfo. RuntimeContext must obtain the
same object or a value-equivalent description for that same runtime type. Type-of
resolution and context discovery must not create unrelated identity models.

This does not require every access to return the identical allocation. Before
production migration, specify descriptor equivalence and lifetime in terms of the
actual runtime type identity and context, including constructed types and modifiers;
matching display names is insufficient. Keep the resolver/factory out of the public
structural contracts. The selected compiler path is
`RuntimeContext.Current.GetTypeInfoFromHandle(handle) -> TypeInfo`: acquire Current,
load the type token, then invoke the resolver. Interning and full context lifetime
remain to be specified; the handle is not a member of TypeInfo.

Validate equivalent acquisition through type-of and RuntimeContext, hidden concrete
implementations in public reference metadata, generic/array distinctions, and
non-equivalence of same-named but distinct types. Cross-context executable binding
remains Reflection's responsibility, not an operation on TypeInfo.

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
[implemented API](raven-reflection-api.md). Dynamic Reflection, Emit and the complete
interface migration are not implemented. RuntimeContext acquisition is an isolated POC.

The [isolated Raven probe](experiments/raven-target/introspection-v1/README.md)
exercises minimal TypeInfo/MemberInfo interfaces backed by existing runtime objects.
It keeps experimental and existing descriptors in separate assembly identities;
production migration is still required. BindingFlags is retained for acquisition.
The probe now also queries fields through TypeInfo and exposes FieldInfo.Type and
MemberInfo.DeclaringType through that same interface. Its collection representation
is experimental; filtering and invalid-flag behavior delegate to the runtime.

The POC now includes a minimal RuntimeContext and a compiler-configured
`typeof(Date)` sample. Raven binds the expression as TypeInfo and routes execution
through Current.GetTypeInfoFromHandle; the private adapter alone uses the legacy
System.Type factory. The source sample runs without an injected entry stub.
This establishes the acquisition boundary, not assembly loading, production
descriptor replacement, object.Type, or complete context discovery.

The library importer now also admits checked nongeneric interface declarations,
with Clock as its first integrated Raven contract. This removes the declaration
authoring blocker; multi-descriptor identity migration and runtime acquisition
remain separate work. See [library progress](raven-system-library.md).

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
