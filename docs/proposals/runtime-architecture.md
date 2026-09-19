# NeoCLR Runtime Architecture and Language Interoperability

## Status

**Proposal / architectural direction**

This proposal establishes the architectural boundaries required for NeoCLR to become a true common-language runtime platform.

It does **not** attempt to fully specify the native ABI, language projection system, component model, or foreign-language bindings. Instead, it establishes the constraints necessary to support those capabilities later without requiring fundamental changes to the runtime.

---

## 1. Motivation

NeoCLR should not define participation in the platform by requiring a language to compile to NeoCLR IL or by requiring a language to adopt an object-oriented programming model.

A language may:

* compile to NeoCLR IL;
* ahead-of-time compile NeoCLR IL;
* compile directly to native machine code;
* use NeoCLR managed memory without using virtual execution;
* use only the NeoCLR type system and component facilities;
* consume NeoCLR components through generated projections without otherwise integrating with NeoCLR;
* model behavior through classes and interfaces;
* model behavior through structs and traits;
* use free functions, modules, namespaces or other non-object-oriented constructs.

NeoCLR should therefore be understood as a **runtime platform**, of which virtual execution and object-oriented dispatch are capabilities rather than defining characteristics.

The long-term goal is:

> **A language participates in NeoCLR through common semantic and runtime contracts, not through a mandatory instruction set or programming paradigm.**

NeoCLR IL remains an important portable execution format, but it is not the definition of a NeoCLR program.

---

# 2. The Common Type System

## 2.1 The CTS is foundational

NeoCLR defines a Common Type System describing concepts shared between languages and runtime services.

The CTS is independent of the NeoCLR instruction set and should avoid unnecessarily encoding the assumptions of any particular source language.

It may describe concepts including:

* values;
* references;
* records and structured values;
* classes;
* interfaces;
* unions;
* functions;
* methods;
* generic types and functions;
* signatures;
* strings;
* collections;
* tasks;
* `Option`;
* `Result`;
* visibility;
* component contracts.

Individual languages may have substantially richer or different type systems.

A language does not need to expose every language feature through the NeoCLR CTS.

---

# 3. Metadata is Language-Neutral

NeoCLR metadata describes the semantic structure of a NeoCLR program independently of its execution representation and source-language paradigm.

In particular:

> **Functions are first-class metadata definitions. They are not required to belong to a class or other type.**

For example:

```text
Namespace System.Math

Function
    Name: Sin
    Parameters:
        value: Float64
    Returns:
        Float64
```

does not need to be encoded as:

```text
Type: Math
    static Method: Sin(...)
```

merely because the original CLI metadata model requires methods to belong to types.

Likewise:

```text
Namespace MyApplication

Function:
    LoadUser(UserId)
        -> Task<Result<User, LoadError>>
```

is a valid NeoCLR program construct in its own right.

This allows languages with module-level or namespace-level functions to map naturally onto NeoCLR.

---

# 4. Metadata Does Not Imply Object Orientation

NeoCLR metadata should distinguish general semantic concepts from object-oriented specialization.

Conceptually:

```text
Definitions
│
├── Type
│    ├── Record
│    ├── Value
│    ├── Class
│    ├── Interface
│    ├── Union
│    └── ...
│
├── Function
│
├── Namespace / Module
│
└── ...
```

A method can then be understood as behavior associated with a type rather than the fundamental representation of all executable behavior.

Conceptually:

```text
Callable
│
├── Function
└── Method
```

This distinction should also appear throughout Introspection.

For example, `FunctionInfo` should not require a synthetic `TypeInfo` merely to identify its owner.

---

# 5. Mapping Language Concepts

NeoCLR should provide semantic building blocks that languages can map onto without requiring identical source-language constructs.

For example, Rust has concepts such as:

```text
struct
enum
trait
impl
fn
mod
```

NeoCLR should not require these concepts to pretend to be C# classes merely to participate in the runtime.

Some mappings may be straightforward:

```text
Rust                         NeoCLR

struct User             →    structured/value type
enum Result<T,E>        →    union
fn parse(...)           →    function
trait Display           →    interface/contract
impl Display for User   →    implementation
```

The exact mappings remain a matter for the Rust projection and the eventual NeoCLR component model.

The important architectural constraint is that NeoCLR metadata must provide sufficient primitives for such mappings without forcing everything through classes and virtual methods.

---

# 6. Consumption and Implementation Are Different Problems

Consuming a NeoCLR component from another language is generally easier than implementing a NeoCLR contract from that language.

For consumption:

```text
NeoCLR Metadata
       ↓
Projection Generator
       ↓
Rust / C++ / Go API
       ↓
generated runtime calls
```

The projection can hide substantial differences between the NeoCLR model and the consuming language.

For example, a NeoCLR interface could potentially be projected into an appropriate Rust trait-facing wrapper even if the underlying component uses managed objects.

Implementation is harder:

```text
Rust implementation
       ↓
???
       ↓
NeoCLR component contract
```

NeoCLR now needs to understand:

* how the implementation is identified;
* how calls enter native code;
* how lifetime is managed;
* how errors cross the boundary;
* how asynchronous operations behave;
* how generic instantiations are represented;
* how interfaces or contracts are implemented;
* how values are transported;
* how callbacks enter the foreign runtime;
* how metadata identifies the implementation.

These questions should therefore be treated as a later **foreign implementation model**, rather than something the initial projection system must completely solve.

---

# 7. Do Not Require Symmetric Projection

NeoCLR should not assume that every projection supports every capability in both directions.

A projection may initially support:

```text
NeoCLR → Rust
```

without supporting:

```text
Rust → NeoCLR
```

Likewise, a C++ projection may eventually support both directions more completely than a JavaScript projection.

This is acceptable.

Projection capabilities can be described independently:

```text
Language Projection

    Consume components       ✓
    Implement interfaces     ✓
    Export functions         ✓
    Export types             partial
    Export generic APIs      partial
    Managed references       ✓
    Native callbacks         ✓
```

The common runtime model should make richer integration possible without making it mandatory.

---

# 8. Metadata Describes Programs, Not IL

NeoCLR metadata describes the semantic structure of a NeoCLR program independently of its execution representation.

Conceptually:

```text
NeoCLR Module
│
├── Metadata
│    ├── Types
│    ├── Functions
│    ├── Methods
│    ├── Signatures
│    ├── Components
│    └── References
│
└── Implementations
     ├── NeoCLR IL
     ├── Native code
     ├── Runtime-provided implementation
     └── External implementation
```

A function or method existing in metadata must not imply that it has an IL body.

For example:

```text
Function
    Name: LoadUser

    Signature:
        UserId -> Task<Result<User, LoadError>>

    Implementation:
        Native
```

must be representable independently of whether the initial runtime commonly executes IL.

This distinction also forms the basis of the NeoCLR Introspection model.

A `MetadataContext` can understand a program without executing or loading its implementation.

---

# 9. The NeoCLR Host

A program using NeoCLR creates a **NeoCLR Host**.

Conceptually:

```text
Application Process
        │
        ▼
┌─────────────────────────────┐
│         NeoCLR Host         │
│                             │
│  Metadata / Types           │
│  Component Runtime          │
│  Module Loader              │
│  Memory Management          │
│  Execution                  │
│  Tasks / Suspension         │
│  Introspection              │
│  Platform Services          │
│  ...                        │
└─────────────────────────────┘
```

The host establishes the runtime context in which NeoCLR services operate.

The normal application model should be:

> **One application creates one NeoCLR Host.**

Creating a NeoCLR Host does not mean starting an IL virtual machine.

It means creating an instance of the NeoCLR runtime platform.

---

# 10. Runtime Services

NeoCLR functionality is divided into explicit runtime capabilities.

Examples include:

```text
Runtime
│
├── Metadata
├── Components
├── Memory
├── Execution
├── Tasks
├── Introspection
├── Diagnostics
└── ...
```

The exact interfaces remain to be designed.

The important architectural property is that these facilities have **defined boundaries**.

The execution engine consumes memory-management facilities. It does not define memory management.

Metadata likewise does not belong to the execution engine merely because execution uses it.

A service boundary is an architectural boundary and does not require expensive runtime indirection for every operation.

---

# 11. Virtual Execution

Virtual execution is a NeoCLR service.

A Raven application may use:

```text
Raven
  ↓
NeoCLR Metadata + IL
  ↓
NeoCLR Host
  ↓
Execution Service
  ↓
JIT
```

Another language may use:

```text
Language
  ↓
NeoCLR Metadata + Native Code
  ↓
NeoCLR Host
  ↓
Native Execution
```

Both can remain NeoCLR programs.

Therefore:

> **NeoCLR IL is an execution format, not the interoperability format of NeoCLR.**

---

# 12. Memory Management

Memory management is likewise a NeoCLR runtime service.

The managed execution environment may rely heavily upon it, but managed memory must not be architecturally inseparable from IL execution.

This permits combinations such as:

```text
Virtual execution + NeoCLR GC

Native execution + NeoCLR GC

Native execution + native memory

Native component + references to NeoCLR-managed objects
```

The exact interoperability rules for ownership, rooting, handles, pinning and borrowing are deferred.

Public runtime contracts must not assume that every participant uses the NeoCLR garbage collector for its own objects.

---

# 13. Managed and Native Are Orthogonal Properties

NeoCLR should avoid treating **managed versus native** as the primary division of the platform.

A program has several independent properties:

```text
Execution
    virtual
    native

Memory
    NeoCLR managed
    native
    external

Metadata
    NeoCLR
    foreign / opaque

Component participation
    NeoCLR
    foreign

Runtime capabilities
    selected as required
```

A native compiler may generate machine code while still using NeoCLR memory management.

A Rust library may use Rust ownership while exposing NeoCLR component metadata.

Raven may use NeoCLR IL, GC, runtime suspension and the complete NeoCLR environment.

These implementations should still be capable of interoperating.

---

# 14. Component Boundary

NeoCLR should eventually provide a component boundary through which implementations communicate independently of their execution mechanism.

The component model should support more than object-oriented interfaces.

A component might export:

```text
Functions
Types
Contracts / interfaces
Constants
Services
```

For example:

```text
component Users
{
    func Load(id: UserId)
        -> Task<Result<User, LoadError>>
}
```

does not inherently require the existence of a `UserService` class.

A language that naturally models this as a module should be able to do so.

A language that prefers an object or service abstraction may project it accordingly.

---

# 15. Native Runtime Boundary

NeoCLR should eventually expose its host and component facilities through a stable native boundary.

It does **not** need to use COM.

The underlying interface can potentially be substantially smaller and designed specifically around NeoCLR.

The architectural requirement is:

> **NeoCLR runtime facilities must be capable of being represented through a stable native boundary without exposing implementation objects.**

Public boundaries should therefore favor stable identities such as:

```text
RuntimeHandle
ModuleHandle
TypeHandle
FunctionHandle
MethodHandle
ObjectHandle
MetadataToken
```

rather than implementation-specific pointers such as:

```text
JitMethod*
GcObject*
ExecutionFrame*
LoadedType*
```

---

# 16. Language Projections

A language does not need native NeoCLR compiler support to consume NeoCLR components.

Instead:

```text
NeoCLR Metadata
       │
       ▼
Projection Generator
       │
       ▼
Language API + Runtime Bindings
```

A compiler may provide deeper NeoCLR integration, but this is an optimization and tooling choice rather than a prerequisite.

Projection should optimize for the target language rather than mechanically reproducing NeoCLR syntax.

The runtime ABI and projection model therefore serve different purposes:

> **The ABI optimizes for stability and universality. Projections optimize for language semantics and ergonomics.**

---

# 17. Producing NeoCLR Metadata

A language capable of producing NeoCLR components needs some mechanism for describing its exported surface in NeoCLR metadata.

For example:

```text
Rust / C++ source
        │
        ▼
NeoCLR integration tooling
        │
        ├────────► native executable code
        │
        └────────► NeoCLR metadata
```

This does not necessarily require modifying the language compiler.

Possible future mechanisms include:

* compiler integration;
* build-time metadata generation;
* source annotations;
* external interface descriptions;
* generated adapter code;
* post-compilation metadata generation.

This proposal deliberately does not choose between them.

The important property is that NeoCLR metadata remains independently producible from NeoCLR IL.

---

# 18. Interoperable Type Surface

NeoCLR should not restrict its complete CTS to the lowest common denominator of every possible language.

It may eventually distinguish:

```text
NeoCLR Common Type System
             │
             ▼
Interoperable Component Surface
             │
             ▼
Language Projection
```

Some language concepts may not be exportable.

Some NeoCLR concepts may require transformation.

Some projections may support more NeoCLR features than others.

That is preferable to weakening the entire NeoCLR programming model.

---

# 19. Relationship to Windows Runtime

Windows Runtime demonstrated that language interoperability can be based upon common metadata, runtime contracts and language projections rather than requiring every language to target the same virtual instruction set.

NeoCLR adopts that architectural lesson without adopting COM or Windows-specific assumptions.

In particular, NeoCLR should not inherit the assumption that cross-language APIs must fundamentally be object-oriented.

The intended model is:

> **Common semantics + metadata + runtime contracts + language projections**

rather than:

> **Every language must look like C# at the runtime boundary.**

---

# 20. Initial Implementation Strategy

The initial implementation can primarily support:

```text
Raven
  ↓
NeoCLR Metadata + IL
  ↓
NeoCLR Host
  ↓
Execution + Memory + Runtime Services
```

However, it should preserve several architectural invariants from the beginning:

* metadata does not assume IL;
* metadata supports free functions;
* functions do not require synthetic containing classes;
* runtime APIs do not expose JIT internals;
* memory management remains conceptually separate from execution;
* runtime services belong to an explicit host;
* runtime identities can be represented through stable handles;
* native implementations remain possible;
* the CTS does not assume object-oriented source languages;
* consuming and implementing foreign components are treated as separate interoperability problems.

---

# 21. Design Rule for Future NeoCLR APIs

Every major NeoCLR subsystem should be reviewed using two questions:

> **Have we accidentally made IL, the JIT, the GC or another particular implementation part of this service's public model?**

and:

> **Have we accidentally required a source language to adopt an object-oriented abstraction where the underlying concept does not require one?**

These questions apply throughout the platform.

---

# 22. Summary

NeoCLR is intended to be a **common runtime platform rather than merely a common virtual machine or common object system**.

Its conceptual foundation becomes:

```text
                 Common Type System
                        │
                     Metadata
                        │
               Component Contracts
                        │
                 Runtime Contracts
                        │
                    NeoCLR Host
                        │
       ┌────────────────┼────────────────┐
       │                │                │
    Memory          Execution         Tasks
       │                │                │
       └────────────────┼────────────────┘
                        │
                  Platform / OS
```

Languages sit around this common model:

```text
 Raven       Rust       C++        Go       ...
   │           │         │          │
   │      structs/     classes/   structs/
   │       traits       funcs      funcs
   │           │         │          │
   └───────────┴─────────┴──────────┘
                     │
                NeoCLR semantics
```

No language is required to compile to NeoCLR IL.

No language is required to use NeoCLR-managed memory for its own objects.

And importantly:

> **No language is required to become object-oriented in order to participate in NeoCLR.**

The commonality exists at the level of semantics, metadata and runtime contracts—not at the level of source-language paradigm.

The initial goal is therefore not universal interoperability. It is to establish a sufficiently neutral architecture that universal interoperability can be built incrementally rather than retrofitted later.

> **Design the semantic boundaries now; design individual language projections when they become necessary.**
