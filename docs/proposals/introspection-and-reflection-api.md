# NeoCLR Introspection, Reflection, Runtime Context, and Emit

## 1. Overview

NeoCLR separates four concerns that are historically closely coupled in .NET:

* structural program information,
* metadata resolution,
* runtime loading and execution,
* dynamic code generation.

The proposed namespaces are:

```text
System.Introspection
System.Runtime
System.Runtime.Reflection
System.Runtime.Emit
```

The central model is:

> **Introspection describes program structure. A context defines the universe in which that structure is resolved. RuntimeContext defines the execution universe. Reflection binds introspection objects to that runtime. Emit constructs new program structure.**

This separation is intended to avoid making the introspection model synonymous with the set of assemblies currently loaded for execution.

---

# 2. One shared introspection model

`System.Introspection` defines the common structural representation of programs.

Tentative interfaces include:

```raven
namespace System.Introspection

interface AssemblyInfo
interface ModuleInfo
interface TypeInfo

interface MemberInfo
interface MethodInfo
interface ConstructorInfo
interface PropertyInfo
interface FieldInfo
interface EventInfo
interface ParameterInfo
interface AttributeInfo
```

NeoCLR does not use the `I` naming convention for interfaces, so `TypeInfo`, `MethodInfo`, and similar names are appropriate interface names.

The `Info` suffix is intentional.

A `MethodInfo` represents information about a method. It does not imply that the method is executable.

Likewise, `TypeInfo` represents a type within a structural model. It does not by itself imply that the type is loaded into any runtime.

---

# 3. No parallel `Type` and `TypeInfo` models

The current proposal does not define separate public pairs such as:

```text
Type        / TypeInfo
Assembly    / AssemblyInfo
Module      / ModuleInfo
```

Instead, the introspection model is itself the common representation.

For example:

```raven
interface MethodInfo {
    Name: String
    DeclaringType: TypeInfo

    Parameters: List<ParameterInfo>
    ReturnType: TypeInfo
}

interface ParameterInfo {
    Name: String
    Position: Int
    Type: TypeInfo
}

interface PropertyInfo {
    Name: String
    DeclaringType: TypeInfo
    Type: TypeInfo
}
```

Structural relationships therefore remain inside the same model:

```text
TypeInfo
 │
 ├── BaseType ───────────► TypeInfo
 ├── Interfaces ─────────► TypeInfo
 ├── GenericArguments ───► TypeInfo
 │
 └── Methods
       │
       └── MethodInfo
            ├── ReturnType ───► TypeInfo
            └── Parameters ───► TypeInfo
```

This avoids separate runtime, metadata, and generated type models that must constantly be translated into one another.

---

# 4. Introspection objects are context-bound

A fundamental rule is:

> **An introspection object never exists without a context that defines its identity and resolution universe.**

A `TypeInfo` is therefore not a globally meaningful type identity.

Its identity is meaningful within the context from which it originates.

For example:

```text
RuntimeContext:

    Contracts 2.0
        Contracts.Customer

MetadataContext:

    Contracts 1.0
        Contracts.Customer
```

Both entities can implement `TypeInfo`, while remaining distinct.

The shared interface does not erase provenance or resolution identity.

Therefore:

> **Same introspection model does not imply same context or same identity.**

---

# 5. `TypeInfo` need not be heavyweight

Using one representation does not mean that all structural information must be eagerly materialized.

A `TypeInfo` can behave as a stable descriptor or handle.

Information such as:

```raven
type.Name
type.Namespace
type.Kind
type.GenericArity
```

may already be immediately available.

Information such as:

```raven
type.Methods
type.Interfaces
type.Attributes
```

may require additional resolution, metadata access, or runtime cooperation.

The model therefore does not encode information cost through separate lightweight and heavyweight object types.

> **Materialization strategy is an implementation concern, not a reason to fragment the program model.**

---

# 6. `System.Runtime.RuntimeContext`

A `RuntimeContext` defines an execution universe.

```raven
namespace System.Runtime

class RuntimeContext
```

The current runtime may conceptually be exposed as:

```raven
let runtime = RuntimeContext.Current
```

A runtime context knows what has actually been loaded and realized for execution:

```raven
runtime.Assemblies
```

The entities exposed by the runtime use the same introspection interfaces:

```text
RuntimeContext
    │
    └── AssemblyInfo
          │
          └── ModuleInfo
                │
                └── TypeInfo
                      │
                      └── MethodInfo
```

These are runtime-backed implementations of the introspection model.

---

# 7. Runtime introspection is the initial implementation

The first version of NeoCLR does not need to support arbitrary external metadata files.

V1 can expose introspection information directly from `RuntimeContext`.

For example:

```raven
let runtime = RuntimeContext.Current

for assembly in runtime.Assemblies {
    for type in assembly.Types {
        print(type.Name)
    }
}
```

Likewise, the Raven language may expose a runtime type as a `TypeInfo`:

```raven
let type: TypeInfo = value.Type
```

The important architectural point is that the interfaces do not require the implementation to be runtime-backed.

That allows additional implementations to be introduced later.

---

# 8. Future `MetadataContext`

A future `MetadataContext` will define an independent metadata universe.

Conceptually:

```raven
let metadata = MetadataContext.Create()

let assembly =
    metadata.OpenAssembly(path)?
```

Opening an assembly in a `MetadataContext` means:

> Make this assembly's metadata available for inspection and metadata resolution.

It does not mean:

> Load this assembly for execution.

This distinction is a hard architectural requirement.

```text
MetadataContext.OpenAssembly(...)
        ≠
RuntimeContext.Load(...)
```

Metadata inspection must never implicitly modify an execution context.

---

# 9. Metadata and runtime contexts expose the same model

Eventually, the architecture should allow:

```text
                  System.Introspection

            AssemblyInfo / TypeInfo / ...

                    ▲             ▲
                    │             │
             RuntimeContext  MetadataContext
```

The contexts have different responsibilities but expose the same structural model.

`RuntimeContext` might contain:

```text
System
Application
Contracts 2.0
```

while a separate `MetadataContext` contains:

```text
Plugin.dll
Contracts 1.0
ReferenceFramework.dll
```

Code that only depends on structural information can operate on either:

```raven
fn Analyze(type: TypeInfo) {
    for method in type.Methods {
        print($"{method.Name}: {method.ReturnType.Name}")
    }
}
```

This is one of the defining goals of the design:

> **One model, multiple contexts.**

---

# 10. Metadata resolution and runtime resolution are different operations

The architecture distinguishes two kinds of resolution.

## Metadata resolution

A metadata context answers questions such as:

> Which type does this metadata reference mean?

For example:

```text
Contracts.Request
        │
        ▼
TypeInfo within this metadata universe
```

This concerns assembly references, scopes, signatures, generic arguments, and other structural relationships.

## Runtime resolution

Reflection answers:

> Which executable entity in this particular RuntimeContext corresponds to this TypeInfo or MethodInfo?

For example:

```text
TypeInfo
    │
    │ bind
    ▼
RuntimeContext
    │
    ▼
realized executable type
```

A type may therefore be fully resolved from a metadata perspective while having no corresponding runtime entity.

---

# 11. `System.Runtime.Reflection`

Reflection is explicitly part of the runtime layer:

```raven
import System.Runtime.Reflection
```

Reflection understands both:

```text
System.Introspection
System.Runtime.RuntimeContext
```

Its main responsibility is:

> **Bind structural program descriptions to realized entities in a specific runtime context.**

Potential capabilities include:

```text
runtime binding
dynamic invocation
object activation
dynamic member access
runtime generic construction
```

For example, conceptually:

```raven
let bound =
    runtime.Reflection.TryBind(method)
```

Only a successfully runtime-bound method can be invoked.

---

# 12. Binding must be explicit

Having a `TypeInfo` or `MethodInfo` does not mean that it exists in a given runtime.

Given an external descriptor:

```raven
let type: TypeInfo = ...
```

Reflection may ask:

```raven
runtime.Reflection.TryBind(type)
```

If that type already exists within the runtime context, binding succeeds.

If it does not, binding fails.

It must not silently load the corresponding assembly merely because an introspection descriptor exists.

This suggests distinct operations such as:

```text
TryBind
    associate with an already realized runtime entity

Load
    explicitly introduce code into RuntimeContext
```

A convenience operation combining them could potentially exist later, but the primitive operations should retain the distinction.

---

# 13. Explicit runtime loading

`RuntimeContext` owns the execution universe and therefore owns explicit loading.

Conceptually:

```raven
let runtime = RuntimeContext.Current

let assembly =
    runtime.Load(source)?
```

The exact loading API remains future work.

The important semantic distinction is:

```text
MetadataContext.Open(...)
    → metadata enters an inspection universe

RuntimeContext.Load(...)
    → code enters an execution universe
```

These must never be confused.

---

# 14. Runtime assembly resolution

A `RuntimeContext` must also define how executable dependencies are located.

For example, when loading:

```text
Foo, Version=1.0
```

the runtime needs a policy for determining where `Foo` comes from.

Potential conventions might include:

```text
application directory
runtime/framework directories
dependency manifests
configured probing paths
custom resolvers
```

This behavior belongs to the runtime context rather than Introspection or Emit.

Conceptually, a resolver may look something like:

```raven
interface AssemblyResolver {
    Resolve(
        identity: AssemblyIdentity,
        context: RuntimeContext
    ) -> Result<AssemblySource, ResolveError>
}
```

The exact API is still open.

The key principle is:

> **Runtime loading policy belongs to RuntimeContext.**

---

# 15. `System.Runtime.Reflection` does not leak into Introspection

`MethodInfo`, `PropertyInfo`, and similar interfaces remain descriptive.

They do not intrinsically define:

```raven
Invoke(...)
GetValue(...)
SetValue(...)
CreateInstance(...)
```

Reflection may provide these through extension methods or properties:

```raven
extension MethodReflection for MethodInfo {
    Invoke(...): ...
}
```

allowing ergonomic usage:

```raven
import System.Runtime.Reflection

method.Invoke(...)
```

while preserving the actual dependency direction:

```text
System.Introspection
        ▲
        │
System.Runtime.Reflection
```

This allows runtime capabilities to compose with the introspection model instead of being embedded into its inheritance hierarchy.

---

# 16. `System.Runtime.Emit`

Dynamic construction and code generation live in:

```raven
System.Runtime.Emit
```

Emit may provide capabilities such as:

```text
type construction
method construction
member construction
IL generation
assembly generation
metadata generation
```

Reflection and Emit are sibling facilities:

```text
                  System.Introspection
                          ▲
                 ┌────────┴────────┐
                 │                 │
System.Runtime.Reflection   System.Runtime.Emit

bind / execute             construct / generate
```

Emit does not require Reflection.

---

# 17. Emit consumes structural identity, not runtime identity

Emit needs to know how to refer to:

```text
a type
a method
a field
an assembly
```

but it should not require those entities to exist in a runtime.

For example, generated code may reference:

```text
ExternalContract
RuntimeService
AnotherGeneratedType
```

through the common introspection model:

```text
GeneratedProxy
 │
 ├── implements ─────► TypeInfo
 ├── field type ─────► TypeInfo
 └── method
       ├── input ────► TypeInfo
       └── output ───► TypeInfo
```

The referenced descriptors may originate from different contexts or providers, subject to whatever compatibility rules are required when constructing the output metadata.

The essential requirement is:

> **Emit depends on structural metadata identity rather than executable runtime identity.**

---

# 18. Metadata tokens and references

For Emit, the important operation is translating structural identities into valid references in the generated metadata.

A `TypeInfo` or `MethodInfo` must provide enough contextual identity for Emit to generate the appropriate:

```text
assembly reference
type reference
member reference
metadata token
signature
```

Emit should not need to know where the runtime will eventually locate the dependency.

That belongs to runtime loading policy.

Therefore:

```text
Emit
    structural descriptors
           ↓
    metadata references/tokens
           ↓
    generated image


RuntimeContext
    generated image
           ↓
    dependency resolution
           ↓
    runtime load
```

This separation allows code to be generated entirely offline.

---

# 19. Emit may expose introspection views

Emit will probably require mutable builders:

```raven
TypeBuilder
MethodBuilder
AssemblyBuilder
```

but these should not create a separate structural universe.

Builders should be able to expose the same introspection contracts:

```raven
builder.Info       // TypeInfo
methodBuilder.Info // MethodInfo
```

This enables the same analyzer to inspect:

* runtime-backed types,
* metadata-backed types,
* types currently being generated.

The source and mutability differ; the structural model does not.

---

# 20. Emitting code does not load it

This is another hard guarantee:

> **Emission does not imply runtime loading.**

A compiler or code generator should be able to operate entirely without a `RuntimeContext`:

```text
MetadataContext
       ↓
System.Introspection
       ↓
System.Runtime.Emit
       ↓
assembly / module / image
```

Only when the program wants to execute the generated result does it cross the boundary:

```text
generated assembly
       ↓
RuntimeContext.Load(...)
       ↓
execution universe
       ↓
Reflection may bind/invoke
```

---

# 21. Compiler scenario

A compiler does not need Reflection.

Its architecture can eventually be:

```text
MetadataContext
      │
      ▼
System.Introspection
      │
      ▼
Compiler
      │
      ▼
System.Runtime.Emit
      │
      ▼
output image
```

No executable runtime universe is required.

This is an important long-term reason for keeping Introspection independent of Reflection.

---

# 22. Runtime scenario

A normal running application instead starts from:

```text
RuntimeContext.Current
        │
        ▼
runtime-backed
System.Introspection
```

and can optionally add:

```text
System.Runtime.Reflection
```

when dynamic runtime operations are required.

The program therefore does not need to adopt a different structural model merely because its source happens to be the runtime.

---

# 23. V1 scope

Full `MetadataContext` support is deferred because it requires substantial infrastructure:

* reading NeoCLR/.NET metadata files,
* metadata table parsing,
* signature decoding,
* reference resolution,
* assembly identity handling,
* metadata-backed implementations of all introspection interfaces.

V1 should instead focus on:

```text
System.Introspection
    shared interfaces

System.Runtime.RuntimeContext
    runtime-backed implementations

System.Runtime.Reflection
    runtime capabilities

System.Runtime.Emit
    generation capabilities
```

The design requirement for V1 is:

> **Adding MetadataContext later must not require redesigning the introspection interfaces.**

This provides a concrete test of whether the abstractions are correct.

---

# 24. Introspection interfaces and composition

The introspection model should not automatically reproduce .NET's original Reflection class hierarchy.

For example, instead of assuming:

```text
MemberInfo
   └── MethodBase
         ├── MethodInfo
         └── ConstructorInfo
```

NeoCLR can consider interfaces, unions, traits, and composition.

For example:

```raven
union MemberInfo =
    MethodInfo
    | ConstructorInfo
    | PropertyInfo
    | FieldInfo
    | EventInfo
```

The exact representation remains open.

The rule is:

> **Model program structure first. Choose inheritance or composition based on the semantics of that structure rather than historical API constraints.**

---

# 25. Query ergonomics

Despite the stronger architecture, ordinary introspection should remain straightforward.

The target experience is:

```raven
assembly.Types

type.Methods
type.Properties
type.Fields

method.Parameters
method.ReturnType
```

not constant manual context resolution.

Contexts should be explicit when establishing a universe, but ordinary navigation should remain ergonomic.

Long term the goal is:

> **Metadata isolation comparable to Cecil, with navigation ergonomics comparable to .NET Reflection.**

---

# 26. Future typed introspection

The shared dynamic model does not prevent a typed facade later.

Potential future APIs include:

```raven
TypeInfo<Foo>
MethodInfo<Foo>
MethodInfo<Foo, String, Int>
```

and:

```raven
TypeInfo<Foo>
    .GetMethod<String, Int>("Do")
    -> Option<MethodInfo<Foo, String, Int>>
```

This remains exploratory and is not required for the initial introspection model.

---

# Overall architecture

```text
                         SYSTEM.INTROSPECTION

                ┌─────────────────────────────┐
                │                             │
                │ AssemblyInfo                │
                │ ModuleInfo                  │
                │ TypeInfo                    │
                │ MethodInfo                  │
                │ PropertyInfo                │
                │ FieldInfo                   │
                │ ...                         │
                │                             │
                │ Shared structural model     │
                │ Context-bound identity      │
                └──────────────▲──────────────┘
                               │
                ┌──────────────┴──────────────┐
                │                             │
                │                             │
           SYSTEM.RUNTIME              future MetadataContext
                │
          RuntimeContext
                │
                │ execution universe
                │
        ┌───────┴────────┐
        │                │
        ▼                ▼
System.Runtime.Reflection    System.Runtime.Emit

bind / invoke / activate     construct / generate
runtime capabilities         metadata / code output
```

With full metadata support:

```text
                       SHARED INTROSPECTION MODEL

                  TypeInfo / MethodInfo / ...

                         ▲             ▲
                         │             │
                         │             │
                 RuntimeContext   MetadataContext
                         │             │
                execution world   metadata world
                         │             │
                         └──────┬──────┘
                                │
                         System.Runtime.Emit
                                │
                         generated output
                                │
                                ▼
                     RuntimeContext.Load(...)
                                │
                                ▼
                       executable program
```

# Core principles

The proposal can be reduced to the following rules:

1. **There is one shared introspection model.** Runtime, external metadata, compiler inputs, and generated structures do not require separate public type/member models.

2. **Introspection objects are context-bound.** Identity and resolution only make sense relative to the context that produced them.

3. **`RuntimeContext` defines an execution universe.** It knows what is loaded and what the runtime can act upon.

4. **A future `MetadataContext` defines an independent metadata universe.** Opening metadata never implicitly loads code for execution.

5. **Metadata resolution and runtime binding are separate.** Introspection resolves program structure; Reflection binds that structure to a specific runtime.

6. **Runtime loading is explicit.** `TryBind` should not silently turn metadata inspection into execution loading.

7. **Reflection is a runtime capability.** It depends on both the introspection model and a `RuntimeContext`.

8. **Emit consumes structural identity, not runtime identity.** It can generate valid metadata references and tokens without loading the referenced code.

9. **Emission and execution are separate stages.** Generated code only becomes executable when explicitly loaded into a `RuntimeContext`.

10. **Capabilities compose with the model.** Reflection and Emit do not need to contaminate the introspection interfaces or force capability-specific inheritance hierarchies.

The shortest description is:

> **NeoCLR has one context-bound model of program structure. `RuntimeContext` represents the execution universe; `MetadataContext` represents an independent metadata universe. Reflection binds structural descriptions to a runtime, while Emit consumes those descriptions to generate new code. Metadata inspection, code generation, runtime loading, and execution remain explicit, separate operations.**
