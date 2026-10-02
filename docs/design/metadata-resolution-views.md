# Metadata resolution contexts and resolved views

Design checkpoint: 2026-10-02. The fixed context and nominal facade below are now
implemented in C#; constructed/member views and wider model alignment remain planned.
This supports the native metadata milestone and the existing builders → definitions →
metadata → PE architecture. It does not add another serialized representation.

## Author direction and existing foundation

The author proposes a pure metadata projection above the reader/writer infrastructure
so information can be accessed uniformly for local and external declarations, with
explicit dependency resolution. This is not a request to recreate Reflection.

Existing AssemblyReference.Resolve and TypeReference.Resolve use IAssemblyResolver
and recheck exact assembly identities. Loaded definitions own their snapshots.
ReferencedGenericType retains a definition reference plus arguments. Native interface
relationships now retain constructed arguments. Physical MemberReference resolution
remains bounded. Raven now uses the shared metadata context for nominal dependency resolution and
maps its views into compiler-owned symbols. Signature/member adaptation remains in Raven.

The missing convenience is a coherent session and resolved, constructed member view.
A second type resolver, compiler symbol system or CLI projection would duplicate the
existing infrastructure rather than fill that gap.

## Proposed responsibilities

1. **Reader and definitions:** preserve declaration identity, ownership, original
   signatures, references and unsupported data. Reading an assembly must not silently
   load its dependency closure. Definitions describe what the declaring assembly says.
2. **Resolution context:** own one explicit catalog/source policy and stable snapshot
   identity for a bounded operation. Reuse IAssemblyResolver. Exact dependency lookup,
   conflict detection, caching and diagnostic provenance belong here. No implicit
   filesystem search, package restore, version unification or runtime assembly loading.
3. **Resolved metadata views:** expose a resolved definition together with the type and
   method arguments under which it is being inspected. A field view exposes both its
   original declaration and substituted field type. Method/property/interface views do
   the same. Preserve declaring assembly identity and parameter-owner scope even when
   two assemblies contain identically named types.
4. **Consumers:** Raven imports the view into its own symbols. Future Introspection can
   consume the same metadata contracts. Writers/builders may explicitly import from a
   resolved view into their own output graph; a view is never an output-owned operand.

Definitions and views are different: Box<T>.Value stays T in the loaded definition;
a view of Box<Int32>.Value reports Int32 and still points to the original declaration.
Resolution should work the same way when Box is local or comes through an AssemblyRef.
No API names are committed by these examples.

## Prototype scope and possible future consumers

Author clarification: NeoCLR will eventually need this kind of metadata-only library.
Subsequent clarification: the present work is a prototype for the .NET-hosted Raven
compiler, not a requirement to use this exact implementation or API in NeoCLR.
It may inform a future compiler bootstrapped to compile for NeoCLR while running on
NeoCLR. That reuse or port remains exploratory, not an implementation commitment. A metadata-only introspection layer must read dependencies without
loading them into the execution runtime, and be usable by System.Runtime.Reflection.Emit.
That API's namespace does not imply that metadata inspection requires runtime Reflection.

The reusable contracts are assembly/type/member identity, declaration navigation,
signature construction/substitution, dependency resolution and explicit output import.
The current C# implementation proves those contracts on .NET; a possible future Raven/NeoCLR
implementation can reuse lessons from them without depending on
System.Reflection.Assembly/Type/MemberInfo, AssemblyLoadContext or execution handles.
Host-specific byte access and dependency search remain adapters. A direct dependency
from NeoCLR's future library onto the host C# binary is not assumed.

The prototype and potential future consumers have independent roles:

- Raven maps metadata views into compiler symbols and emits from symbols.
- Metadata-only Introspection exposes declaration information without invocation or
  runtime object inspection.
- System.Runtime.Reflection.Emit uses builders and IL generators, explicitly importing
  metadata contracts into output-owned definitions before encoding an assembly.

No constructor execution, static initialization or code loading is needed for these
operations. If generated code is later loaded for execution, that is a separate explicit
runtime boundary. The metadata layer must not silently acquire execution capability.
It may inspect assembly bytes produced by Emit through the same reader/context path.

The existing runtime Introspection API and naming do not automatically become this
metadata-only layer. Inventory their contracts before migration or reuse; distinguish
metadata queries from runtime-backed operations. Public namespace/type names and the
porting choice and schedule remain open. Prototype names and APIs need not be preserved
in a later implementation. The separation of metadata from execution remains an author
requirement; speculative portability must not delay a useful .NET-hosted prototype.

## Identity, lifetime and failure contracts

Start with an immutable catalog of already-read snapshots. Reject multiple snapshot
objects for the same full identity unless the same object is registered again; do not
silently pick an artifact or introduce implicit hashing on each lookup. Keep provenance
available so a later source-backed context can distinguish conflicting artifacts.
Canonicalize resolved definitions and constructed views within a context, not globally.
Cache only against that fixed catalog; failed lookups in one context must not poison a
new context. The context retains snapshots for the lifetime of its views.

Register declaration identities before traversing signatures. Legal assembly dependency
cycles and diamond graphs must share definitions; invalid interface/inheritance cycles
remain errors. The initial preloaded catalog needs no asynchronous loading state machine.
If lazy artifact loading is added later, specify loading/ready/failed states and concurrent
publication first. Do not expose half-built views.

Distinguish missing dependency, identity mismatch, conflicting snapshot, absent/ambiguous
member and unsupported metadata profile. Include the requested identity and reference
path. Reuse existing resolver exceptions initially where appropriate; do not change all
public error types incidentally. Unsupported signatures must stay explicit, not become
Object, empty member lists or invented semantic facts.

Views enumerate declared members first. Inherited member lookup, visibility filtering,
overload resolution, language conversions and runtime dispatch are separate concerns.
Raven's binder continues to own language semantics. No Invoke, instance construction,
execution handles or Reflection-compatible Type/MemberInfo hierarchy is proposed.

## Compatibility, costs and comparison

Reuse the ECMA-335/Cecil/MetadataReader research recorded in
[extended metadata design](extended-cli-metadata.md). Relative to low-level .NET metadata
handles, this layer adds dependency navigation and constructed signatures. Relative to
Reflection, it stays in the metadata domain with explicit inputs and no runtime type
loading. Cecil-like definitions and references remain the underlying model.

Benefits are one reusable resolution/substitution implementation and fewer importer
adapters. Costs are view/cache allocations, retained snapshots, explicit context lifetime
and another API whose invariants need testing. No performance improvement is claimed.
Resolve lazily on demand and cache within the context; do not eagerly expand the graph.
The same model should support .NET and native metadata where readers already support
those categories, without pretending their current coverage is equal.

This layer does not make native external interface declarations legal by itself. That
writer/runtime capability remains separate, as do type forwarders, multimodule resolution,
variance, constraints, full CLI member decoding and broader native declaration profiles.

## First implementation slices and acceptance

1. Add the explicit snapshot catalog/context over IAssemblyResolver. C# tests cover
   exact identities, same-name/different-version assemblies, duplicate/conflicting inputs,
   missing dependencies, diamond identity, legal assembly cycles and context isolation.
2. Add nominal/constructed type views and declared field/method signatures, preserving
   open type/method parameter owners and simultaneous substitution. Test local versus
   external equivalence, nested constructed/vector signatures and wrong arity/scope.
3. Adapt one Raven importer path to consume views. Compare symbol identity, diagnostics
   and emitted runtime behavior against current native generic field/interface consumers.
   The emitter must have no reference to the context, resolver or loaded definitions.
4. Extend views to properties, interface relationships and broader reference categories
   only with focused tests. Continue external-interface emission as a distinct slice.

First success means a separately compiled native dependency is inspected through the
context, imported into Raven symbols, emitted through symbol contracts and executed in
neoCLR. Keep .NET metadata tests as a compatibility control. The current seven-consumer
runtime results are baseline evidence, not evidence that these proposed views exist.


## First implemented checkpoint

MetadataLoadContext and AssemblyInfo/ModuleInfo/TypeInfo/NominalTypeInfo now live in
NeoCLR.Metadata.Experimental.Introspection next to the Model builder/definition APIs.
Raven removes its private NativeAssemblyResolver and shares one context per immutable
compilation through a weak-key lifetime adapter. Nominal resolution returns scoped
metadata views; Raven maps assembly identity and module-local token to symbols. Its
emitter does not receive this context. Repeated resolution caches views per context.

109 C# groups pass, including context identity/conflict/cycle tests. Existing native
consumer execution provides integration coverage; no runtime/format changes are needed.
The runtime interfaces informed names and navigation, but remain unchanged: this slice
has no execution handles, no invocation and no guest implementation. Next move scoped
generic and member projection into the facade, using current symbol-loader behavior as
regression evidence. Language binding and diagnostic policy remain Raven responsibilities.


Constructed/field facade checkpoint (2026-10-02): the C# Introspection model now has
canonical primitive, vector, owner-parameter and constructed-type views plus declared
FieldInfo views. Definitions remain open; constructed owners substitute field signatures
simultaneously, preserving caller parameter scope and declaration identity. Recursive
nominal fields resolve without eagerly expanding members. Foreign/Void/bare-generic
arguments, wrong arity and unsupported method-parameter scopes reject explicitly.

Raven now consumes facade field types and closed signature projections, caching symbol
mapping by canonical view identity. This preserves array identity across fields, methods
and constructors; the existing integration assertion caught and verified that boundary.
Open method-signature adaptation remains in Raven until method/parameter views exist.
No emitter dependency on the context, Runtime Contract change, bootstrap change or
runtime/metadata encoding change is introduced. The runtime model informs names and
semantics but its guest implementation is unchanged. No performance claim is made.

Validation: 109/109 C# groups and all seven Raven runtime consumers pass. See
[field-view evidence](../experiments/extended-cli-metadata/introspection-field-views-2026-10-02.json).


Method/parameter facade checkpoint (2026-10-02): MethodInfo, ParameterInfo and
MethodGenericParameterTypeInfo now project namespace functions and declared methods,
including methods viewed on constructed owners. Owner and method argument scopes are
separate and substitution is simultaneous. Method/parameter identities remain canonical
within the context; no invocation or runtime loading is introduced.

Raven now builds native return/parameter symbols from these views and maps scoped
parameter identities back to the declaring compiler symbols. Its recursive signature
walkers and type/method generic-signature caches have been removed; the view-to-symbol
cache preserves signature identity. Language binding and special constructor return
semantics stay in Raven. This changes no Runtime Contract, primitive core/System
bootstrap, emission contract or metadata/runtime encoding. Properties and interface
relationship views remain next; generic method construction is not yet a facade API.

109 C# groups pass, including mixed owner/method scopes, generic function vectors,
constructed-owner returns, canonical method identity and invalid/foreign scopes.
All seven Raven native consumers compile and execute (42).

See [method-view validation](../experiments/extended-cli-metadata/introspection-method-views-2026-10-02.json)
for the tested Raven revision and native artifact/runtime hashes.
