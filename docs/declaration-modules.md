# Declaration modules — development foundation

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
DeclarationModuleInfo facade has been removed; guest migration remains open.
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

Module identity is exact assembly identity plus an ordinal, case-sensitive qualified
name. Names use the existing UTF-8 qualified-name contract: at most 1024 characters,
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
native or CLI inputs; CLI output still loses explicit empty declarations. Guest
RuntimeContext and its physical ModuleInfo API have not yet migrated.

## Comparison and tradeoffs

The [recorded primary-source comparison](design/module-system-assessment.md#comparison-and-alternatives)
contrasts CLI namespace strings/physical Module rows with Rust's semantic modules.
This foundation makes declaration containers discoverable and preserves empty
containers across native metadata. It costs a new manifest contract and coordinated
reader updates. It does not claim faster execution or stronger access isolation.
Compiler-only syntax would be cheaper but could not preserve empty native containers;
repurposing physical module rows would conflate organization with loading identity.

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
