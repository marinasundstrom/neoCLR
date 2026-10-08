# neoCLR Proposal: First-Class Module System

Architecture Proposal · Version 0.2 · October 2026

## 1. Overview

This proposal introduces a first-class module system for neoCLR as an evolution of the existing .NET Common Language Infrastructure (CLI) metadata model.

The goal is to extend the original CLR concept of modules beyond physical compilation units, making them the primary organizational containers for types, functions, fields, and other declarations.

The design preserves established .NET conventions, including naming, accessibility, assembly identity, and metadata-based interoperability.

Rather than introducing an entirely new programming model, neoCLR builds upon existing CLI concepts and removes historical limitations that require executable members to belong to types.

Core principle:

> An assembly is a unit of deployment and identity. A module is a unit of organization and declaration ownership. A type defines data and behavior. Functions may exist independently of types.

The module system is a runtime and metadata feature, not merely a Raven language feature.

## 2. Background: Modules in the existing CLR

The Common Language Infrastructure, standardized through ECMA-335, already defines modules as fundamental metadata units.

A traditional CLR assembly may conceptually contain multiple modules.

```
Assembly: MyLibrary
 ├── Manifest Module
 ├── Core.netmodule
 └── Networking.netmodule
```

Each physical module contains its own metadata and potentially executable IL.

The CLR also provides namespaces, but namespaces are not first-class metadata entities. Instead, type definitions carry namespace names.

For example:

```
TypeDef
  TypeName: Socket
  TypeNamespace: System.Net.Sockets
```

Namespaces organize type names but do not own declarations as separate metadata entities.

The existing CLR also supports global methods and fields, represented through the special `<Module>` type.

Consequently, the CLR already contains several mechanisms relevant to module-oriented programming, but they have not been developed into a unified logical module system.

neoCLR proposes to extend these mechanisms rather than replace the CLI architecture wholesale.

## 3. Architectural model

The proposed structure is:

Assembly: MyApplication

Module: MyApplication

Function: Main()

Field: Configuration

Module: MyApplication.Services

Function: CreateOrder()

Type: OrderService

Type: OrderResult

Module: MyApplication.Models

Type: Order

Type: Customer

An assembly contains one or more modules. A module contains declarations, including types and functions.

Each module definition belongs to exactly one assembly.

Unlike traditional namespaces, modules have explicit metadata identity and ownership semantics.

### 3.1 Responsibilities

| Concept  | Responsibility                                  |
| -------- | ----------------------------------------------- |
| Assembly | Identity, deployment, dependencies, linking     |
| Module   | Declaration ownership, organization, visibility |
| Type     | Data representation, nominal identity, behavior |
| Function | Executable behavior independent of a type       |
| Method   | Executable behavior declared on a type          |

The module system does not replace types, classes, interfaces, or other nominal abstractions.

It provides an additional level of organization above them.

## 4. Module declarations

A module is declared using the `module` keyword.

The syntax is deliberately similar to .NET namespace declarations.

```
module System.Networking

public fn Connect(address: String) -> Result<Connection, Error> {
    // ...
}

public class Connection {
    public fn Close() {
        // ...
    }
}
```

Here:

- `System.Networking` is a module.
- `Connect` is a module-level function.
- `Connection` is a type owned by the module.
- `Close` is an instance method owned by `Connection`.

Usage:

```
let connection = System.Networking.Connect(address)?
connection.Close()
```

No synthetic static class is required to represent `Connect`.

### 4.1 Naming conventions

neoCLR should preserve the established .NET naming conventions.

| Declaration     | Convention         | Example                     |
| --------------- | ------------------ | --------------------------- |
| Assembly        | PascalCase         | `System.Networking`         |
| Module          | PascalCase, dotted | `System.Networking.Sockets` |
| Type            | PascalCase         | `TcpClient`                 |
| Function        | PascalCase         | `Connect`                   |
| Method          | PascalCase         | `Close`                     |
| Property        | PascalCase         | `IsConnected`               |
| Public constant | PascalCase         | `DefaultTimeout`            |
| Parameter       | camelCase          | `connectionString`          |
| Local variable  | camelCase          | `client`                    |

This ensures that neoCLR libraries retain the familiar appearance of .NET APIs.

## 5. Modules as assembly subdivisions

A fundamental design decision is that modules are subdivisions of assemblies, rather than independent deployment units.

For example:

```
Assembly: System.Runtime
  Module: System
  Module: System.Collections
  Module: System.Threading

Assembly: System.Networking
  Module: System.Networking
  Module: System.Networking.Sockets
```

Each module is associated with a single assembly.

An assembly may contain multiple modules, and their names are qualified using familiar dotted notation.

### 5.1 Module identity

A module's identity consists of:

```
Assembly Identity + Module Name
```

For example:

```
[System.Networking]System.Networking.Sockets
```

This distinguishes modules with identical names originating from different assemblies.

Modules are not required to have globally unique names.

However, references to declarations must resolve unambiguously.

### 5.2 Module hierarchy

Dotted module names form a logical hierarchy.

For example:

```
System
 ├── Collections
 │    └── Generic
 ├── IO
 └── Networking
      └── Sockets
```

A child module does not automatically inherit access to private declarations in its parent.

The hierarchy primarily establishes qualified naming and organization.

Each module remains a distinct metadata entity.

### 5.3 Partial module declarations

Multiple source files may contribute to the same module within an assembly.

```
module System.IO

public fn OpenRead(path: String) -> InputStream {
    // ...
}
```

And in another file:

```
module System.IO

public fn OpenWrite(path: String) -> OutputStream {
    // ...
}
```

The compiler merges these declarations into one module definition.

No explicit `partial` modifier is necessary.

## 6. Module-level declarations

Modules may directly contain:

- Types
- Functions
- Fields
- Constants
- Properties
- Events

These declarations have explicit module ownership in metadata.

### 6.1 Functions

```
module System.IO

public fn OpenRead(path: String) -> Result<InputStream, IOError> {
    // ...
}
```

A function is represented as an executable declaration owned by a module rather than a type.

Functions support familiar method-like features, including parameters, return types, generics, attributes, and visibility.

### 6.2 Fields and constants

```
module System.Diagnostics

public const DefaultTraceLevel = TraceLevel.Information

private var currentLevel: TraceLevel = DefaultTraceLevel
```

Module-level fields are static in nature.

They are associated with runtime storage rather than an instance of a module.

### 6.3 Properties

```
module System.Diagnostics

public TraceLevel: TraceLevel {
    get => currentLevel
    set => currentLevel = value
}
```

Properties follow the same conceptual model as .NET properties, with accessors represented through executable members.

### 6.4 Types

Types retain their familiar structure.

```
module System.Networking

public class Connection {
    public IsConnected: Bool { get; }

    public fn Close() {
        // ...
    }
}
```

The module owns the type definition, while the type owns its instance and static members.

## 7. Metadata evolution

The module system should extend the existing CLI metadata model wherever possible.

The goal is to preserve familiar metadata concepts rather than invent an unrelated representation.

### 7.1 Existing CLI metadata

In ECMA-335, the metadata includes tables such as:

```
Module
TypeDef
MethodDef
Field
Property
Event
Assembly
AssemblyRef
ModuleRef
```

Methods and fields are ordinarily associated with type definitions.

Global methods and fields are represented using the special `<Module>` type.

### 7.2 Proposed neoCLR metadata

neoCLR should support explicit module definitions and ownership relationships.

A conceptual representation:

```
AssemblyDef
 ├── ModuleDef
 │    ├── FunctionDef
 │    ├── FieldDef
 │    ├── PropertyDef
 │    ├── EventDef
 │    └── TypeDef
 │         ├── MethodDef
 │         ├── FieldDef
 │         ├── PropertyDef
 │         └── EventDef
```

This is a logical metadata model, not necessarily the exact binary table layout.

### 7.3 Reuse of existing metadata tables

I recommend retaining `MethodDef` as the fundamental representation of executable declarations.

A module-level function and a type method share many characteristics:

- Signatures
- Parameters
- Return types
- Generic parameters
- Attributes
- Method bodies
- Visibility
- Implementation flags

Therefore, neoCLR does not necessarily need a separate `FunctionDef` table.

Instead, it could introduce explicit declaration ownership.

Conceptually:

```
MethodDef
  Name
  Signature
  Flags
  Body

MethodOwner
  OwnerKind: Module | Type
  OwnerIndex
```

The exact encoding requires further design because the existing CLI metadata uses implicit ownership through table ranges.

A compatible implementation might introduce an extension table rather than changing existing table semantics directly.

The distinction between function and method would then be determined by ownership.

```
Module-owned MethodDef → Function
Type-owned MethodDef   → Method
```

This preserves much of the existing CLI execution model.

### 7.4 Eliminating the synthetic `<Module>` type

For new neoCLR metadata, module-level members should not require the special `<Module>` type.

Instead:

```
ModuleDef: System.Networking
  MethodDef: Connect
```

Existing CLI metadata using `<Module>` could be translated into the new representation by the loader or interoperability layer.

The module system therefore generalizes an existing CLI capability rather than introducing an entirely foreign concept.

## 8. Member resolution

The runtime must distinguish between module-owned and type-owned members.

For example:

```
System.Networking.Connect(address)
```

Resolves to:

```
Assembly: System.Networking
Module: System.Networking
Function: Connect
Signature: (String) -> Result<Connection, Error>
```

Whereas:

```
connection.Close()
```

Resolves to:

```
Type: System.Networking.Connection
Method: Close
Signature: () -> Unit
```

### 8.1 neoIL instructions

I would initially reuse the existing call model.

For example:

```
call System.Networking.Connect
```

The instruction resolves to a module-owned executable declaration.

No special invocation mechanism is necessary simply because the declaration belongs to a module.

Existing concepts such as direct calls, virtual dispatch, function references, and native function pointers remain applicable.

A function reference may refer to either a module-level function or a method bound to an instance.

## 9. Accessibility

The module system should extend existing .NET accessibility rules.

| Modifier    | Meaning                                  |
| ----------- | ---------------------------------------- |
| `public`    | Accessible outside the defining assembly |
| `internal`  | Accessible within the defining assembly  |
| `private`   | Accessible within the defining module    |
| `protected` | Applicable to type members, not modules  |

Example:

```
module MyApplication.Security

private fn ComputeHash(data: Byte[]) -> Byte[] {
    // ...
}

internal fn ValidateToken(token: String) -> Bool {
    // ...
}

public fn Authenticate(credentials: Credentials) -> Result<User, Error> {
    // ...
}
```

The runtime and metadata verifier should enforce accessibility rules consistently.

A module's private members are not accessible merely because another module has a related qualified name.

## 10. Module imports

Module imports are primarily a language-level feature.

For Raven:

```
use System.IO

let stream = OpenRead("data.txt")?
```

Fully qualified access remains possible:

```
let stream = System.IO.OpenRead("data.txt")?
```

Imports affect name resolution but do not inherently cause runtime module loading.

The compiler resolves imported declarations to metadata references.

Assembly dependencies remain the responsibility of the assembly system.

## 11. Introspection and reflection

Modules should be represented explicitly in neoCLR's introspection APIs.

For example:

```
module System.Introspection

public class ModuleInfo {
    public Name: String { get; }
    public FullName: String { get; }

    public Types: ReadOnlyList<TypeInfo> { get; }
    public Functions: ReadOnlyList<FunctionInfo> { get; }
    public Fields: ReadOnlyList<FieldInfo> { get; }
}
```

An assembly exposes its modules:

```
let assembly = Introspection.GetAssembly("MyApplication")

for module in assembly.Modules {
    Console.WriteLine(module.FullName)

    for function in module.Functions {
        Console.WriteLine(function.Name)
    }
}
```

This enables tools to inspect module-level declarations without treating them as static members of artificial types.

Introspection describes metadata. Reflection adds runtime operations such as dynamic invocation.

These capabilities should remain conceptually separate.

## 12. Module initialization and lifetime

Modules are not objects.

They cannot be instantiated and do not have instance identity.

Module-level state is associated with a runtime context.

I recommend the following initial rules:

1. Constants require no runtime initialization.
2. Module-level mutable fields are initialized once per relevant runtime context.
3. Initialization is coordinated by the runtime.
4. Modules do not have destructors or instance disposal.
5. Explicit module initialization hooks are deferred until their semantics are established.

This avoids introducing an implicit singleton-object model.

The exact initialization ordering and handling of cyclic dependencies should be specified separately.

## 13. Native compilation

The module model should remain consistent across neoIL and native compilation.

A native-compiled assembly should retain its logical module definitions through metadata, either embedded in the binary or provided through an associated metadata sidecar.

For example:

```
System.Networking.native
System.Networking.metadata
```

The metadata identifies:

```
Assembly: System.Networking

Module: System.Networking
  Function: Connect
  Type: Connection

Module: System.Networking.Sockets
  Type: Socket
```

Native function implementations may be resolved through runtime-managed symbol mappings.

This preserves module identity and introspection regardless of compilation strategy.

## 14. Interoperability with existing .NET

neoCLR should support projecting existing .NET namespaces into the module system.

For example:

```
namespace System.IO
{
    public static class File
    {
        public static Stream OpenRead(string path);
    }
}
```

Could be represented as:

```
Module: System.IO
  Type: File
    Method: OpenRead
```

The method remains owned by `File`.

The interoperability layer must not silently convert type-owned methods into module-level functions, because doing so would change their metadata identity and potentially their semantics.

Existing global methods represented through `<Module>` may be projected as module-level functions.

This compatibility layer allows neoCLR to expose existing .NET APIs without requiring changes to their original metadata.

## 15. Compatibility and versioning

The proposal should distinguish two goals:

Semantic continuity: Preserve existing CLI concepts, metadata identities, calling conventions, and type-system behavior wherever practical.

Binary compatibility: Determine whether existing ECMA-335 metadata can be read directly, translated, or extended through versioned metadata formats.

Introducing multiple first-class logical modules per assembly and changing member ownership cannot be assumed to work with existing CLR metadata readers.

I recommend a versioned neoCLR metadata format with an explicit compatibility path for ECMA-335 assemblies.

This makes the evolution deliberate rather than relying on undefined behavior in existing tools.

## 16. Open design questions

Several details should be resolved before defining the binary metadata format.

| Question                                                 | Proposed direction                      |
| -------------------------------------------------------- | --------------------------------------- |
| Can an assembly contain multiple modules?                | Yes                                     |
| Can a module belong to multiple assemblies?              | No                                      |
| Can modules have identical names across assemblies?      | Yes, identities remain distinct         |
| Can source files contribute to one module?               | Yes                                     |
| Can modules own executable members?                      | Yes                                     |
| Should `MethodDef` represent functions?                  | Preferably yes                          |
| Should modules have runtime instances?                   | No                                      |
| Should modules have mutable state?                       | Yes, with explicit initialization rules |
| Should modules replace namespaces?                       | Yes, for native neoCLR metadata         |
| Should assembly dependency resolution change?            | No, not initially                       |
| Should nested modules imply privileged access?           | No                                      |
| Should existing .NET namespaces be projected as modules? | Yes                                     |

## 17. Implementation roadmap

Phase 1 — Metadata model

Introduce explicit module definitions, ownership relationships, and versioned metadata support. Define how existing `TypeDef` and `MethodDef` entries map to modules.