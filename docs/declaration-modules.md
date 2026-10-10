# Declaration modules — development foundation

**Latest author clarification, 2026-10-10:** a module name is a namespace and may
span multiple assemblies. Referencing/importing that name combines matching declarations
from the referenced assemblies, using .NET-style namespace lookup and ambiguity rules.
Dotted names have ordinary namespace meaning; there is no separate module hierarchy
in native metadata. Each declaration retains its defining assembly identity.

**Simplicity clarification, 2026-10-10:** “module” means namespace in this model.
An assembly may contribute multiple namespaces, and namespace names and packaging
remain conventions with the same flexibility as .NET. Existing namespace projection
and resolution are the baseline; shared names do not require a new module resolver,
loading mechanism or reference graph. Validate that existing lookup combines the
referenced contributions and fix demonstrated gaps rather than redesigning resolution.
Introspection should describe this model with the smallest useful API; the shared-view
question below is not a commitment to introduce another module abstraction.

The implementation described below currently exposes assembly-scoped contributions
through ModuleInfo. Those descriptors are not yet a context-wide shared namespace
view. In particular, singular ModuleInfo.Assembly describes a contribution; the final
shared-view API and its relationship to these descriptors require follow-through.
Assembly-local declaration tables remain useful for recording each contribution,
including empty declarations. They do not claim exclusive ownership of a module name.

Author direction, 2026-10-09: modules replace namespaces as the named containers
for declarations. Assemblies remain packaging and binding identities and remain
visible in RavenDoc. One module per assembly is a common layout, not a constraint:
`System.Runtime` contains many modules. An assembly name need not match a module.

This implements the first ownership/format slice of the
[module assessment](design/module-system-assessment.md). The original proposal's
artifact extensions, private-access rules and independent loading are not adopted.

**Author clarification, 2026-10-10:** a module is a unit and namespace of members
within an assembly. A root namespace and dotted submodule names are a convention,
not a hierarchy in metadata. One module per assembly is a common layout; several
modules may be packaged together. Names need not match the assembly name:

| Assembly | Modules |
| --- | --- |
| `System.Runtime` | `System`, `System.Networking` |
| `Acme.CoffeeMaker` | `Acme.CoffeeMaker`, `Acme.CoffeeMaker.Factories` |

Host `ModuleInfo` now models these logical modules directly. The temporary separate
DeclarationModuleInfo facade has been removed; guest interpreter traversal and explicitly retained native type ownership now follow the same model.
Namespace resolution and ownership should help authors design distributable modules
for class libraries and APIs. No metadata hierarchy or new module-reference table is
introduced. Existing assembly-qualified binding still distinguishes same-name modules
in different assemblies; compiler ambiguity/access-policy changes are separate work.

## Format and identity

Native format-5 assembly manifests now include:

```json
"declaration_modules": {
  "version": 1,
  "names": ["Example.Data", "Example.Math", "Example.Math.Advanced"]
}
```

The current assembly-contribution descriptor identity is exact assembly identity
plus an ordinal, case-sensitive qualified name. The shared module namespace is its
name across the referenced assembly set. Names use the existing UTF-8 qualified-name contract: at most 1024 characters,
no control characters or empty/whitespace-only dotted segments; the empty name denotes
the global module. There are at most 4096 distinct modules. Writers order names
ordinally. A module can be empty. Dotted names express organization; a parent need
not be declared, and a child receives no additional access privileges.

A top-level type, free function or constant belongs to the module matching its
existing namespace field. The versioned table makes that owner explicit without
changing qualified names, callable references, nominal identities, IL or native ABI.
Readers reject unknown table versions, duplicate/invalid names and missing owners.
The Rust runtime validates these contracts before execution. Older readers reject
the unknown manifest field; rebuild tools together. Older input without the table
remains readable as a marked projection, not an explicit module declaration.

The manifest `modules` list and reader `ModuleDefinition` still describe physical
metadata images. Host `ModuleInfo` describes logical declaration modules. Physical
image names and token scopes remain reader/backend facts, exposed as
`MetadataScopeName` on host type/method descriptors when interpreting tokens.
Logical modules do not acquire separate files, loading or scheduling lifetimes.

## API foundation

`AssemblyBuilder.DefineModule(name)` returns a canonical
`DeclarationModuleDefinition`. Its `AddClass`, `AddFunction` and `AddConstant`
methods author members in that module. Existing namespace-argument APIs contribute
module owners when writing native metadata. `AssemblyDefinition.GetModules()`
enumerates reader definitions; each aggregate member exposes `DeclaringModule`.

Host `AssemblyInfo.GetModules()` and `MetadataLoadContext.GetModules()` return
canonical `ModuleInfo` views. `Resolve(DeclarationModuleDefinition)` returns the
same view. `Name`, `Assembly`, `IsProjection`, `GetMembers()`, `GetTypes()` and
`GetFunctions()` expose the module's declarations. Type, method and assembly-member
`Module` properties share this owner; nested types retain their outer type's module.
`GetMembers()` excludes nested/type-owned declarations; `GetTypes()` includes nested
types. Dotted names are compared exactly, without recursive membership or synthesized
parents. Identity is scoped to the owning assembly and context.

This replaces the temporary DeclarationModuleInfo/physical ModuleInfo host split.
Consumers must migrate and rebuild; no compatibility aliases remain. See the
[API reference](../api-docs/experimental-metadata.md#context-owned-declaration-views-development-2026-10-10).
Loaded modules reject mutation. `IsProjection` identifies inferred views of older
native or CLI inputs; CLI output still loses explicit empty declarations. Guest traversal through RuntimeContext.Current.ExecutingAssembly now uses logical
module names too. ModuleInfo.MetadataToken is removed. Interpreter queries include
exact-name type enumeration; native support currently covers explicitly retained
type-to-module descriptors and their owning AssemblyInfo.Name/FullName. This does
not retain all assembly types. Explicit native module-catalog retention now enables
AssemblyInfo.GetModules, including empty declarations. See [the guest consumer](experiments/guest-modules/README.md).

## Comparison and tradeoffs

The [recorded primary-source comparison](design/module-system-assessment.md#comparison-and-alternatives)
contrasts CLI namespace strings/physical Module rows with Rust's semantic modules.
This foundation makes declaration containers discoverable and preserves empty
containers across native metadata. It costs a new manifest contract and coordinated
reader updates. It does not claim faster execution or stronger access isolation.
Compiler-only syntax would be cheaper but could not preserve empty native containers;
repurposing physical module rows would conflate organization with loading identity.

## Soundness review — 2026-10-10

This assessment predates the author’s shared-namespace clarification above. Its
assembly-qualified identity discussion now applies to contributions, not exclusive
logical namespace ownership. The author selects .NET-style namespace combination;
exact public introspection changes remain open.

The author asks whether the diverging module semantics are sound. The assistant's
assessment is yes for the implemented scope: an assembly packages declarations,
and an assembly-owned logical module gives their namespace an inspectable identity.
Types, free functions and constants share a natural container. A flat table avoids
making dotted naming conventions into access rights or runtime lifetimes. This is
a deliberate semantic difference from .NET, not a claim of Module API compatibility.

Microsoft's [System.Reflection.Module contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.module?view=net-10.0)
describes a physical metadata module: it may contain multiple namespaces, and a
namespace may span modules. The [C# namespace specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/namespaces)
describes open-ended namespace declaration spaces. Sources checked 2026-10-10.
neoCLR instead makes exact assembly identity plus module name the ownership key,
with descriptors belonging to a metadata/runtime context. Multiple source files
can contribute to that same owner; a module is not a source-file identity.

The central unresolved issue is **lookup across dependencies**. Assembly A's
`Example.Data` and assembly B's `Example.Data` remain distinct owners. That does
not by itself prohibit overlapping names or decide whether an import combines
public lookup candidates. The model encourages intentional distribution; it does
not yet enforce globally exclusive namespace ownership. As a follow-up recommendation,
preserve distinct owners, diagnose ambiguous references deterministically, and
provide an explicit qualification/alias route when needed. This is an assistant
recommendation, not an adopted new compiler policy.

Other boundaries should remain explicit:

- Dotted prefixes imply neither recursive membership nor parent-private access.
- Assembly binding remains the packaging/versioning boundary; independently loading,
  unloading or scheduling modules would require separate contracts.
- Module ownership does not create module-private visibility or a security boundary.
  Existing accessibility rules remain in effect.
- CLI adapters must retain physical token provenance separately. Empty logical
  modules and any future module-only metadata need an explicit projection policy;
  they must not silently become physical CLI module semantics.
- AOT enumeration needs a documented retention/completeness contract. A module
  descriptor alone does not retain every declaration or executable implementation.

The benefit is coherent declaration ownership and discovery, particularly for
module-level functions such as tests. The costs are compiler/adapter work, different
reflection expectations, and explicit decisions about lookup and metadata retention.
There is no demonstrated execution-performance benefit. The assistant recommends
keeping this bounded design and validating cross-assembly lookup before adding more
module semantics. Existing host/interpreter and retained native ownership evidence
supports feasibility; it does not establish completion of those open contracts.

## Native module-catalog retention — development, 2026-10-10

The bounded AOT backend now supports `AssemblyInfo.GetModules()` using an explicit
catalog root. Existing type roots retain their owners, but do not imply a complete
assembly module list. A missing catalog faults with
`native assembly module catalog was not retained`; it never returns a filtered list
of modules that happen to own retained types.

The private `--reflection-roots` schema 4 adds `moduleCatalogs`, an array of exact full
assembly identities. Its `types` entries use schema 3's six fields (`definition`,
`construct`, `properties`, `getters`, `setters`, `customAttributes`). Existing schemas
1–3 retain their behavior. For example, a catalog-only policy can use:

```json
{
  "schemaVersion": 4,
  "types": [],
  "moduleCatalogs": ["Example, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null"]
}
```

The identity must match the loaded source catalog exactly; this example is illustrative.
At most 64 unique assembly identities and 64 type roots are admitted. At least one
catalog or type root is required. Unknown identities, duplicates, malformed policies
and older metadata with only an inferred namespace projection are rejected. An explicit
empty declaration table is valid and returns an empty module sequence.

A catalog roots every declared module name, ordered ordinally by the shared VM query,
including empty modules. Each descriptor supports Name and Assembly.Name/FullName.
It does not root declaration types, functions, constructors or attributes. Assembly
references and type enumeration within assemblies or modules in native code remain
separate work. Native RuntimeContext.ExecutingAssembly now identifies the lexical
caller independently of module-catalog retention. The catalog is an immutable build-time snapshot, not a
runtime loader or a new assembly discovery API.

Comparison: [.NET Assembly.GetModules](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.assembly.getmodules?view=net-10.0)
returns the assembly's physical modules (source checked 2026-10-10). neoCLR keeps the
assembly-wide enumeration shape while applying the author's logical-module semantics.
Reconstructing names from retained types would lose empty modules; retaining every
assembly declaration would couple descriptive discovery to unrelated code and metadata.
Explicit name-catalog retention preserves completeness with that narrower scope. Its
cost is build configuration plus generated lookup branches, strings and runtime
wrapper allocations; no performance benefit is claimed. A future native metadata
backend can replace these factories without changing logical ownership. This private
experimental policy is not a permanent user-facing retention API.

The [consumer](experiments/guest-modules/README.md) exercises flat ordering, empty
modules, owning assembly inspection and independent missing-type/missing-catalog
faults. Unit checks cover scoped identities, empty catalogs, policy rejection and
legacy projection rejection. Cross-platform evidence remains scoped to recorded runs.

## Native executing assembly — development, 2026-10-10

`RuntimeContext.Current.ExecutingAssembly` now follows the calling code's assembly
in the bounded AOT profile. An application sees its own assembly; a helper executing
inside a dependency sees the dependency, including when reached through an ordinary
function callback. The runtime's exact query facades are transparent, as in the
interpreter; arbitrary user wrappers are not. A direct query retains the assembly's
identity/name without requiring `--reflection-roots`. It does not retain modules,
types, methods, referenced assemblies or load anything dynamically.

The baseline is [.NET Assembly.GetExecutingAssembly](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.assembly.getexecutingassembly?view=net-10.0),
which reports the assembly containing the executing code (checked 2026-10-10).
neoCLR exposes that operation through RuntimeContext rather than the .NET static
Assembly API. This is not GetEntryAssembly or GetCallingAssembly: a library helper
must not report the application's assembly merely because the app invoked it.

The backend validates the scoped getter/service forwarding chain after source
verification, then carries caller identity through a synthetic String argument on
the private getter projection. Call versus callvirt is preserved; branch targets and
sequence points are relocated. The synthetic parameter has no source metadata token,
and private nullable annotations are cleared; original signatures/annotations remain
in the descriptive source catalog. An inlined caller retains its original identity
because the literal is attached before specialization. Exact module and assembly
scopes prevent similarly named foreign methods from becoming transparent wrappers.

This avoids introducing stack walking solely for this query. The cost is generated
identity branches and descriptor allocation; no performance improvement is claimed.
Changed facade bodies are rejected, not discarded. Binding the getter itself as a
function is not supported; direct queries inside ordinary bound functions are supported.
Dynamic loading, general reflective getter invocation and new runtime-context APIs
are not established by this slice. A future native intrinsic can replace the private
argument without changing the guest contract or native assembly format.

See the [executable consumer](experiments/guest-modules/README.md) and its recorded
.NET comparison. Unit checks cover distinct caller assemblies, branch relocation,
preserved virtual call kind, foreign same-name scopes and unsupported facade/binding
shapes. Native type enumeration and in-process test-function discovery remain open.

## Next boundaries

Module-private visibility, re-exports, module-specific ambiguity diagnostics, guest
runtime discovery/executing-module APIs, and metadata forwarding remain follow-ups.
Existing public/internal rules and qualified-name ambiguity behavior remain in force.
No .NET namespace semantics change unless the source uses the new Raven spelling;
ordinary CLI emission remains a namespace projection.

Validation: metadata checks `--declaration-modules` cover scoped authoring, binary
PE/NEOX round trips, empty modules, nested-type exclusion, same-name modules in two
assemblies, older-input projection and invalid manifests. Rust tests
`declaration_modules` and `function_namespaces` cover admission and unchanged scalar
execution. Raven syntax, import and native-consumer evidence is recorded alongside
its compiler contract and the [checked consumer](experiments/declaration-modules/README.md).
Raven ced9e1a686b3946de517b06e6817b324a5847819 supplies the module syntax, native
ownership emission/import and target-aware language-server presentation. The compiler
retains namespace-shaped syntax/symbol scopes in this foundation; guest RuntimeContext
migration and independent module loading are not implied.

The expanded host checks also cover canonical identity across traversal paths,
separate contexts, foreign-snapshot rejection, global and child modules, read-only
collections and shared type/function views in both PE/NEOX native containers.
