# Modules

**Development foundation · Work in progress.** Modules are neoCLR's named containers
for types, functions and constants. They provide the qualified names used by imports.
Assemblies retain deployment and dependency identity.

## Names and ownership

A typical application can put one module in one assembly. Larger libraries can have
many: `System.Runtime` contains modules such as `System`, `System.Math` and
`System.Time`. An assembly name need not match any of its module names.

| Concept | Responsibility | Example |
| --- | --- | --- |
| Assembly | Artifact identity and dependencies | `System.Runtime` |
| Module | Declaration ownership and qualified names | `System.Math` |
| Member | Type, function or constant in that module | `System.Math.Pi` |

A type continues to own its methods and fields. An assembly-wide member view is an
aggregate of its modules. The API reference shows both **Module** and **Assembly**.

## Raven declarations and imports

This checked sample uses a module with a function and a reserved empty module:

```raven
module Example.Numbers {
    public func Answer() -> int {
        42
    }
}

module Example.Reserved {
}
```

The consumer uses the file-scoped form:

```raven
module Example.App

import Example.Numbers.*

func Main() -> int {
    Answer()
}
```

The assembly is named `DifferentPackage`. Both interpreted and native execution
return `42`. Dotted and nested module declarations use qualified lookup; a parent
path need not be an explicitly declared module. Imports retain existing ambiguity
rules. Identical module paths in two assemblies do not merge their declaration
identities.

## What the foundation provides

Native metadata has a versioned module table, preserving empty modules and their
assembly owner. Reader/writer APIs support module discovery and scoped declaration
authoring. Existing namespace syntax remains accepted during migration. The Raven language
server presents neoCLR scopes as modules in hover, completion descriptions and
document/workspace symbols. For .NET
output, Raven projects modules onto namespaces; explicit module identity and empty
module declarations do not survive that projection.

Older neoCLR metadata exposes an identified namespace projection. Development host
and guest `ModuleInfo` now describe logical modules. Interpreter traversal through
`RuntimeContext.Current.ExecutingAssembly.GetModules()` includes declared empty
modules. Native compilation supports explicitly retained type-to-module ownership
and `Module.Assembly` with assembly `Name`/`FullName`. Explicit module-catalog retention
also enables native `AssemblyInfo.GetModules()`, preserving declared empty modules;
a missing catalog faults instead of returning an incomplete list. Native RuntimeContext
and type enumeration remain open. See the [API scope](/docs/introspection.html#logical-modules-development-2026-10-10).
Module-private visibility, re-exports, guest module-function discovery and independently
loadable modules remain outside this foundation. A module does not create its own heap,
scheduler, native ABI or artifact file.

[Browse library modules](../../docs/namespaces.html) ·
[Metadata reader/writer APIs](../../docs/experimental-metadata.html) ·
[Checked sample source](https://github.com/marinasundstrom/neoclr/tree/main/docs/experiments/declaration-modules)
