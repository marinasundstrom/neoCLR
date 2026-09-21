# NeoCLR Runtime Architecture and Platform Strategy

## Status

**Proposal / architectural direction**

## Summary

NeoCLR is a language-neutral managed runtime and application platform.

It provides:

- a managed execution environment;
- a common type and metadata system;
- a platform-independent set of foundational APIs;
- a composable library and SDK model;
- a common interoperability boundary for languages;
- mechanisms through which host environments can implement platform services without exposing host-specific details through the common API.

NeoCLR should not be designed around the assumptions of a particular operating system, language, packaging model, or deployment environment.

The platform should instead establish a stable common foundation upon which languages, libraries, applications, optional capabilities, and potentially platform-specific application models can coexist.

Raven is the first language targeting NeoCLR and is used to exercise its design, but Raven does not define the boundaries of the runtime.

---

# 1. Goals

NeoCLR should provide a modern managed application platform that can evolve independently of the historical constraints of .NET and other existing managed runtimes.

The architecture should support:

- multiple managed languages;
- managed memory and runtime safety;
- expressive value and reference semantics;
- generic programming;
- runtime-supported asynchronous execution;
- portable application APIs;
- explicit error models such as `Result` and `Option`;
- introspection without requiring runtime loading;
- optional runtime reflection and dynamic generation;
- native interoperability;
- modular libraries and capabilities;
- constrained execution environments;
- application-controlled or mockable platform services;
- future platform-specific APIs where common abstractions are insufficient.

These capabilities should not be coupled unnecessarily.

A language should not be required to compile to NeoCLR IL or adopt an object-oriented programming model merely to participate in NeoCLR.

A language may:

- compile to NeoCLR IL;
- ahead-of-time compile NeoCLR IL;
- compile directly to native machine code;
- use NeoCLR managed memory without virtual execution;
- use only the NeoCLR type system and component facilities;
- consume NeoCLR components through generated projections;
- model behavior through classes and interfaces;
- model behavior through structs and traits;
- use free functions, modules, namespaces, or other constructs.

NeoCLR should therefore be understood as a **runtime platform**, of which virtual execution and object-oriented dispatch are capabilities rather than defining characteristics.

> **A language participates in NeoCLR through common semantic and runtime contracts, not through a mandatory instruction set or programming paradigm.**

---

# 2. Architectural Layers

NeoCLR is a layered system.

Conceptually:

```text id="pbpn6x"
┌──────────────────────────────────────────────┐
│                 Application                  │
├──────────────────────────────────────────────┤
│       Optional platform-specific APIs        │
│           Future decision / external         │
├──────────────────────────────────────────────┤
│       Optional NeoCLR capabilities           │
│      System.Media / Graphics / etc.          │
├──────────────────────────────────────────────┤
│         Common NeoCLR Platform APIs          │
│                  System.*                    │
├──────────────────────────────────────────────┤
│        NeoCLR Foundational Libraries         │
│              System.Runtime                  │
├──────────────────────────────────────────────┤
│               NeoCLR Runtime                 │
│                                              │
│ type system · execution · GC · metadata      │
│ tasks · suspension · interop · neoIL         │
├──────────────────────────────────────────────┤
│             Host Integration                 │
├──────────────────────────────────────────────┤
│              Host Environment                │
└──────────────────────────────────────────────┘
```

These are architectural layers, not necessarily physical deployment units.

An implementation may combine several layers into one distribution, while another environment may provide them separately.

The architecture therefore distinguishes:

1. runtime mechanisms;
2. foundational libraries;
3. common application APIs;
4. optional NeoCLR capabilities;
5. host integration;
6. possible platform-specific application models.

These layers should be allowed to evolve independently where their contracts permit it.

---

# 3. NeoCLR Runtime

The runtime provides the mechanisms required to execute and interoperate with NeoCLR programs.

Its responsibilities include areas such as:

```text id="f29x4w"
NeoCLR Runtime
├── Common Type System
├── Metadata
├── Module Loading
├── neoIL Execution
├── Native Execution Integration
├── Managed Memory
├── Garbage Collection
├── Method / Function Dispatch
├── Generics
├── Managed References
├── Tasks / Runtime Suspension
├── Fault Infrastructure
├── Native Interoperability
├── Introspection Support
└── Runtime Services
```

The runtime should remain focused on mechanisms that genuinely require runtime participation.

Higher-level application functionality should not become intrinsic merely because it is commonly used.

JSON, HTTP, dependency injection, logging, localization, configuration, and storage APIs are platform functionality rather than fundamental execution primitives.

---

# 4. The NeoCLR Host

A program using NeoCLR operates within a **NeoCLR Host**.

Conceptually:

```text id="epvj7v"
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
│  Diagnostics                │
│  Platform Services          │
│  ...                        │
└─────────────────────────────┘
```

The host establishes the runtime context in which NeoCLR services operate.

The normal application model is:

> **One application creates one NeoCLR Host.**

Creating a NeoCLR Host does not mean starting an IL virtual machine. It means creating an instance of the NeoCLR runtime platform.

Runtime functionality is divided into explicit capabilities:

```text id="x0rwc9"
Runtime
├── Metadata
├── Components
├── Memory
├── Execution
├── Tasks
├── Introspection
├── Diagnostics
└── ...
```

These facilities have defined architectural boundaries.

The execution engine may consume memory-management facilities without defining memory management. Metadata does not belong to execution merely because execution consumes metadata.

A service boundary is an architectural boundary and does not imply expensive runtime indirection for every operation.

---

# 5. Common Type System

NeoCLR defines a language-neutral Common Type System.

The CTS describes concepts shared between languages and runtime services independently of the NeoCLR instruction set.

The common model may include:

- primitive values;
- values and references;
- records and structured values;
- classes;
- interfaces/contracts;
- unions;
- functions;
- methods;
- generic types and functions;
- signatures;
- strings;
- arrays and collections;
- callable values;
- managed references;
- tasks;
- `Option`;
- `Result`;
- visibility;
- metadata;
- runtime type information;
- component contracts.

The CTS is an interoperability contract rather than the object model of one particular language.

Individual languages may have richer or substantially different type systems.

A language does not need to expose every language feature through the NeoCLR CTS.

---

# 6. Language-Neutral Metadata

NeoCLR metadata describes the semantic structure of a NeoCLR program independently of its execution representation and source-language paradigm.

Functions are first-class metadata definitions. They are not required to belong to a class.

For example:

```text id="8yufqf"
Namespace System.Math

Function
    Name: Sin
    Parameters:
        value: Float64
    Returns:
        Float64
```

does not need to become:

```text id="tps5h5"
Type: Math
    static Method: Sin(...)
```

merely because an earlier runtime metadata format required executable behavior to belong to types.

Conceptually:

```text id="4p87o1"
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
├── Namespace / Module
└── ...
```

A method is behavior associated with a type rather than the fundamental representation of all executable behavior:

```text id="9xknws"
Callable
├── Function
└── Method
```

This distinction also appears in Introspection. `FunctionInfo` should not require a synthetic `TypeInfo` merely to identify its owner.

---

# 7. Metadata Describes Programs, Not IL

NeoCLR metadata describes programs independently of their execution representation.

```text id="vfsn3h"
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
     ├── Native Code
     ├── Runtime-Provided Implementation
     └── External Implementation
```

A function or method existing in metadata must not imply that it has an IL body.

For example:

```text id="tcfmsu"
Function
    Name: LoadUser

    Signature:
        UserId -> Task<Result<User, LoadError>>

    Implementation:
        Native
```

must be representable.

A `MetadataContext` can therefore understand a program without executing or loading its implementation.

> **NeoCLR IL is an execution format, not the interoperability format of NeoCLR.**

---

# 8. Language Neutrality and Projection

NeoCLR is not the Raven runtime.

Raven is the first language used to develop and validate NeoCLR, but runtime concepts should not require Raven-specific semantics unless those semantics are deliberately adopted as part of NeoCLR.

Conceptually:

```text id="wzxj8h"
             Raven
               │
               ▼
          ┌──────────┐
Other ───►│  NeoCLR  │◄─── Other
language  │  model   │     language
          └──────────┘
               │
               ▼
             Runtime
```

Languages may expose different ergonomic projections over common runtime contracts.

For example:

```text id="5gqu2x"
NeoCLR concept       Possible projection

Task<T>              async/await, Future<T>, Promise<T>
Option<T>            Option<T>, Optional<T>, nullable syntax
Result<T,E>          Result<T,E>, union/result syntax
Iterable<T>          foreach protocol, sequence abstraction
```

A projection does not need to reproduce NeoCLR API spelling exactly, provided interoperability semantics are preserved.

---

# 9. Consumption and Implementation

Consuming a NeoCLR component and implementing a NeoCLR component are separate interoperability problems.

Consumption may look like:

```text id="ny6dt6"
NeoCLR Metadata
       ↓
Projection Generator
       ↓
Rust / C++ / Go API
       ↓
Runtime Bindings
```

Implementation requires additional contracts concerning:

- implementation identity;
- native entry points;
- lifetime;
- errors;
- asynchronous behavior;
- generic instantiations;
- interface/contract implementation;
- value transport;
- callbacks;
- metadata representation.

NeoCLR should not require projection capabilities to be symmetrical.

A language projection may support consuming NeoCLR components before it supports exporting arbitrary components into NeoCLR.

---

# 10. Managed and Native Are Orthogonal

NeoCLR should not treat managed versus native as the primary architectural division.

A program has several independent properties:

```text id="72vl69"
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

A native compiler may generate machine code while using NeoCLR memory management.

A native component may retain references to NeoCLR-managed objects.

A Rust component may use Rust ownership while exposing NeoCLR metadata.

Raven may use NeoCLR IL, managed memory, runtime suspension, and the complete NeoCLR environment.

These models should remain capable of interoperating.

---

# 11. The Common NeoCLR Platform

Above the runtime and foundational libraries is the common NeoCLR application platform.

Its public API uses the `System` namespace.

`System` represents the shared application model of NeoCLR rather than a particular host operating system.

The current namespace vocabulary is:

```text id="iztzg8"
System
├── Collections
│   └── Linq
├── Concurrency
├── Configuration
├── Data
│   ├── Compression
│   ├── Json
│   ├── Xml
│   └── Serialization
├── DependencyInjection
├── Globalization
├── Introspection
├── Localization
├── Logging
├── Networking
│   ├── Dns
│   ├── Http
│   │   ├── Client
│   │   └── Server
│   ├── Sockets
│   └── WebSockets
├── Runtime
│   ├── Reflection
│   └── Emit
├── Security
│   └── Cryptography
├── Streams
├── Storage
├── Tasks
├── Text
└── Time
```

This hierarchy is a design target rather than a commitment that every namespace must exist in the initial release.

Detailed API design remains the responsibility of separate proposals.

---

# 12. Namespace Strategy

Namespaces describe **conceptual ownership**.

They do not describe deployment.

For example:

```text id="uzuf9e"
System.Networking.Http
```

means that HTTP belongs to the networking domain.

It does not imply that HTTP must be in the same assembly or package as sockets, DNS, or WebSockets.

Likewise:

```text id="fluz2f"
System.Data.Json
```

identifies JSON as part of the structured-data domain without implying that JSON and XML must share an implementation or common interface.

Namespace hierarchy should be introduced when it communicates useful domain structure.

It does not need to be symmetrical.

The namespace identifies the **primary conceptual home** of an API rather than every concern in which it participates.

---

# 13. Cross-Cutting Concerns

The API architecture is fundamentally a graph even though namespaces form a tree.

Serialization, for example, may interact with:

```text id="m4hnri"
System.Data.Json
System.Data.Xml
System.Introspection
System.Storage
System.Networking
```

without owning any of them.

Likewise, tasks, streams, logging, text encoding, cryptography, introspection, and other facilities may participate across multiple API domains.

Namespace hierarchy should therefore not be used to encode every API relationship.

Cross-cutting concerns should compose across namespace boundaries.

---

# 14. System.Runtime Foundation

`System.Runtime` is the foundational library/package upon which the rest of the common NeoCLR libraries can depend.

This is distinct from the meaning of the `System.Runtime` namespace.

For example, fundamental types may include:

```text id="5jv49f"
System.Object
System.String
System.Option<T>
System.Result<T,E>
System.RuntimeTypeHandle
```

while physically being supplied by the `System.Runtime` package.

Meanwhile:

```text id="85tz69"
System.Introspection (includes TypeInfo)
System.Runtime.Reflection
System.Runtime.Emit
```

are conceptual API namespaces.

Package ownership and namespace ownership must therefore not be conflated.

A type being supplied by `System.Runtime` does not imply that its namespace must be `System.Runtime`.

---

# 15. Libraries, Packages, and SDKs

NeoCLR distinguishes at least four organizational concepts:

```text id="dbsy01"
Namespace
    conceptual API ownership

Library / assembly
    binary and implementation boundary

Package
    dependency and distribution boundary

SDK / capability pack
    developer-facing aggregation
```

These structures may align where useful but are not required to do so.

In particular:

```text id="osud4e"
Namespace hierarchy
        ≠
Assembly hierarchy
        ≠
Package hierarchy
        ≠
SDK hierarchy
```

This distinction should be preserved from the beginning.

---

# 16. Dependency Architecture

Higher-level NeoCLR APIs form a dependency graph rooted in the foundational runtime libraries.

Conceptually:

```text id="8cpc30"
                   System.Runtime
                        │
              ┌─────────┼─────────┐
              ▼         ▼         ▼
            Text   Collections   Tasks
              │                   │
              ▼                   ▼
          Data.Json           Networking
                                  │
                            ┌─────┴─────┐
                            ▼           ▼
                          Http        Sockets
```

This diagram is illustrative rather than an exact dependency graph.

Actual dependencies should emerge from API requirements.

The important constraint is dependency direction: foundational packages should not acquire dependencies on higher-level capabilities merely for implementation convenience.

---

# 17. Capability Composition

NeoCLR should be modular by architecture without requiring maximal package fragmentation.

Capabilities may be separable when doing so provides meaningful benefits in deployment, dependency management, implementation, or constrained environments.

A program using HTTP should not inherently require XML, localization, an HTTP server, media processing, or unrelated capabilities.

Future capabilities might include:

```text id="gq51b6"
System.Media
System.Graphics
System.Devices
```

Such APIs may be delivered through optional packages or capability packs while remaining part of the common NeoCLR API model.

The exact packaging strategy is a separate concern and may evolve.

---

# 18. Platform Services and Providers

Some common NeoCLR APIs represent services supplied by the execution environment.

Examples include:

```text id="81n1z7"
System.Storage
System.Networking
System.Time
System.Globalization
System.Cryptography
```

NeoCLR defines the common abstraction and its semantics.

The environment supplies an appropriate implementation.

Applications should normally program against the common abstraction rather than selecting Windows, Linux, or another host implementation.

Providers may nevertheless be replaceable where appropriate for:

- testing;
- constrained environments;
- custom hosts;
- application-specific behavior.

Provider extensibility is therefore part of the architecture without making provider selection the normal programming model.

---

# 19. Storage Model

Storage illustrates the provider model.

NeoCLR should not assume that an application has unrestricted access to a native host filesystem.

`System.Storage` defines the NeoCLR storage domain.

A conventional `FileSystem` can be one storage provider, while another environment might expose only a particular application-specific or otherwise constrained storage area.

The same common storage model may therefore operate against:

```text id="e2vrg9"
desktop filesystem
application sandbox
in-memory environment
WASM environment
archive-backed storage
remote-backed storage
constrained host
```

without introducing host-specific common APIs such as:

```text id="at57f5"
System.Storage.Windows
System.Storage.Linux
```

The abstraction belongs to NeoCLR; the implementation belongs to the environment.

`FileSystem` should not be treated as the definition of storage itself. It is one provider capable of exposing filesystem semantics.

Other providers may expose only a particular area or capability. An application should therefore be able to receive access to the storage it needs without implicitly receiving access to the entire host filesystem.

For example:

```text id="h7ptq4"
Storage capability
       │
       ├── FileSystem
       │      └── conventional filesystem
       │
       ├── Application Storage
       │      └── constrained application area
       │
       ├── Package Storage
       │      └── packaged resources
       │
       └── other providers
```

The public API should not manufacture abstractions such as `StorageFile` or `StorageDirectory` merely because the types belong to `System.Storage`.

Natural domain concepts can retain natural names:

```text id="4py3g7"
System.Storage

File
Directory
Path
FileSystem
...
```

Namespace ownership already establishes their conceptual context.

Shared abstractions between different storage providers should be introduced only where they represent genuinely shared semantics.

---

# 20. Host Independence

The common `System` platform should not encode host-platform identity.

NeoCLR should avoid public common APIs such as:

```text id="p33npd"
System.Storage.Windows
System.Networking.Linux
System.Runtime.Posix
```

Host-specific implementation details should remain below the common abstraction.

This does not mean NeoCLR must provide a common abstraction for every feature available on every operating system.

> **Platform independence does not imply platform completeness.**

If a capability cannot be represented naturally and consistently across the common NeoCLR platform, it does not need to be forced into `System`.

---

# 21. Platform-Specific APIs

There may eventually be important host capabilities that cannot be represented naturally within the common NeoCLR platform.

When this occurs, NeoCLR should preserve the common model rather than distort `System` around a particular host.

A possible future organization is:

```text id="1hhtji"
System.*        common NeoCLR platform

Windows.*       possible Windows application model
Linux.*         possible Linux-specific APIs
```

Such SDKs are not an obligation of the NeoCLR project at this stage.

Their existence, ownership, scope, packaging, and implementation are future decisions.

The architectural requirement today is simply that NeoCLR should not prevent such APIs from coexisting with the common platform later.

---

# 22. Platform API Interoperability

If platform-specific APIs are introduced in the future, they should be able to interoperate with common NeoCLR types where their semantics are compatible.

Conceptually:

```text id="g0vwbx"
Platform API
     │
     ↕
projection / adapter
     ↕
System API
```

A platform API may also expose concepts for which no common NeoCLR representation exists.

Those concepts should remain platform-specific rather than forcing artificial abstractions into the common platform.

This follows the same broad philosophy as language interoperability: NeoCLR establishes a stable shared boundary while allowing richer models to exist around it.

---

# 23. Native Interoperability

NeoCLR should permit managed languages to interact with native code without requiring native libraries to participate fully in the managed runtime model.

Conceptually:

```text id="8gqqmb"
Managed NeoCLR Code
        │
        ▼
Native Interoperability Boundary
        │
        ▼
Native Library / Platform
```

Managed code may retain control over how it interacts with native memory and native APIs while recognizing that unmanaged components cannot provide the same memory-safety and lifetime guarantees as NeoCLR-managed objects.

Native interoperability should therefore be first-class without pretending that native code is managed.

The runtime boundary should favor stable identities and contracts rather than exposing implementation-specific runtime objects.

Potential stable identities include:

```text id="i79gda"
RuntimeHandle
ModuleHandle
TypeHandle
FunctionHandle
MethodHandle
ObjectHandle
MetadataToken
```

The native boundary should not expose implementation details such as JIT objects, GC implementation objects, execution frames, or internal loaded-type structures.

---

# 24. Component Boundary

NeoCLR should eventually provide a component boundary through which implementations communicate independently of their execution mechanism.

The component model should support more than object-oriented interfaces.

A component might export:

```text id="r7jrc3"
Functions
Types
Contracts / Interfaces
Constants
Services
```

For example:

```text id="dd69fq"
component Users
{
    func Load(id: UserId)
        -> Task<Result<User, LoadError>>
}
```

does not inherently require a `UserService` class.

A language that naturally models this as a module should be able to do so.

A language that prefers an object or service abstraction may project it accordingly.

The component boundary should therefore represent semantic contracts rather than prescribe a particular source-language paradigm.

---

# 25. Tasks and Asynchrony

NeoCLR defines tasks as an asynchronous computation model independent of threading.

The common model is:

```text id="3dhnke"
Task<T>
```

with runtime suspension as the long-term execution mechanism.

Raven may initially generate async state machines while targeting the same task contract.

For example:

```raven id="gk7mxx"
async func LoadUser(id: UserId)
    -> Task<Result<User, LoadError>>
{
    let data = await database.Load(id)?;
    return User.Parse(data);
}
```

The distinction between asynchronous computation and threading is reflected in the platform namespace model:

```text id="zq8ajw"
System.Tasks
System.Threading
```

rather than:

```text id="mgc9ya"
System.Threading.Tasks
```

A task may ultimately execute or resume on a thread, but threading is an execution concern rather than the conceptual definition of asynchronous computation.

The runtime's Tasks/Suspension capability and the public `System.Tasks` API are related but are not the same architectural concept.

The former provides runtime machinery. The latter defines the public programming model.

---

# 26. Introspection and Runtime Reflection

NeoCLR distinguishes program description from runtime execution capabilities.

```text id="2s21mj"
System.Introspection
```

defines the metadata and introspection model.

It can describe assemblies, modules, types, functions, methods, properties, and other program structures without requiring those assemblies to be loaded for execution.

Two contexts may therefore exist conceptually:

```text id="4l4t0n"
MetadataContext
    external/program metadata

RuntimeContext
    loaded runtime entities
```

Runtime reflection belongs separately under:

```text id="us85wq"
System.Runtime.Reflection
```

and dynamic generation under:

```text id="5ivuc4"
System.Runtime.Emit
```

Reflection capabilities may operate over the introspection model without making runtime execution part of the introspection model itself.

This separation allows compilers, tooling, analyzers, metadata processors, and other applications to understand programs without acquiring runtime execution capabilities.

---

# 27. Error Model

Recoverable failures in NeoCLR APIs should normally be represented explicitly.

Typical APIs should use:

```text id="pg63ve"
Result<T, E>
Option<T>
```

where appropriate.

For example:

```raven id="f0a6bn"
func Parse(value: String)
    -> Result<Value, ParseError>
```

or:

```raven id="mp63vh"
func Find(name: String)
    -> Option<Item>
```

Faults or runtime exceptions should primarily represent conditions that cannot reasonably be handled through ordinary API control flow.

The common error model is part of the language-interoperability story and therefore requires well-defined runtime and metadata semantics rather than being merely a Raven convention.

Languages may project these semantics differently while preserving the underlying contract.

---

# 28. Evolution and Deployment

NeoCLR should avoid coupling its public API organization to its initial deployment model.

Managed platforms have historically evolved from large bundled framework installations toward models including:

- application-local deployment;
- side-by-side versions;
- containers;
- package-based distribution;
- trimming;
- self-contained applications;
- constrained environments.

NeoCLR should assume from the beginning that its deployment model may change.

An API should not belong to `System.Runtime` merely because it happens to ship with the runtime today.

A namespace should not exist merely because a package requires a name.

An assembly boundary should not become a permanent conceptual API boundary.

An SDK should not imply that every component must always be deployed together.

> **The initial distribution is one composition of the platform, not the definition of the platform.**

---

# 29. Evolution Principle

The platform should follow this principle:

> **Design API boundaries for semantic stability and package boundaries for replaceable composition.**

Namespace organization should therefore be capable of surviving changes to:

- package boundaries;
- assembly boundaries;
- SDK composition;
- deployment;
- trimming;
- host implementation;
- runtime implementation.

This allows NeoCLR to evolve without requiring architectural migration merely because an earlier deployment model became obsolete.

---

# 30. Relationship to Windows Runtime

Windows Runtime demonstrates several architectural ideas relevant to NeoCLR.

In particular, it demonstrates that language interoperability can be based upon:

```text id="m48spk"
common semantics
       +
metadata
       +
runtime contracts
       +
language projections
```

rather than requiring every language to target the same virtual instruction set or expose identical source-language APIs.

It also demonstrates the usefulness of organizing platform APIs around recognizable conceptual domains such as networking, storage, globalization, data, media, and devices.

NeoCLR can learn from these decisions without adopting COM, Windows-specific assumptions, WinRT's object model, or its exact namespace taxonomy.

NeoCLR should also not inherit the assumption that cross-language APIs must fundamentally be object-oriented.

The intended model is:

> **Common semantics + metadata + runtime contracts + language projections**

rather than:

> **Every language must look like Raven, C#, or another particular language at the runtime boundary.**

---

# 31. Architectural Constraints

NeoCLR architecture should follow these constraints:

- **The runtime is language-neutral.** Raven is a consumer and proving ground for NeoCLR rather than the definition of its runtime semantics.
- **The CTS is semantic rather than language-specific.** It should not unnecessarily encode the assumptions of one source language.
- **Metadata describes programs rather than IL.** IL is one possible implementation representation.
- **Functions are first-class.** Languages should not need synthetic classes merely to export executable behavior.
- **Memory and execution remain separable.** Managed memory should not be architecturally inseparable from virtual execution.
- **Runtime services have explicit boundaries.**
- **`System` represents the common NeoCLR application platform.** It does not represent the host operating system.
- **Common APIs have NeoCLR-defined semantics.** They should not merely rename Windows, POSIX, Linux, or other host APIs.
- **Platform implementation is hidden where appropriate.** Applications should not normally select an OS-specific provider to perform ordinary platform operations.
- **The common platform is not required to expose every host capability.**
- **Platform-specific SDKs remain possible without being a current NeoCLR obligation.**
- **Namespace and deployment structures remain independent.**
- **Cross-cutting concerns compose across namespace boundaries.**
- **Runtime mechanisms remain separate from higher-level platform policy.**
- **Optional capabilities should be possible without fragmenting the fundamental programming model.**
- **Package boundaries should follow useful composition boundaries rather than dictate API taxonomy.**
- **Native interoperability is first-class without pretending native code has managed semantics.**
- **Consuming and implementing foreign components are separate interoperability problems.**

---

# 32. Non-Goals

NeoCLR does not currently attempt to:

- define complete Windows, Linux, macOS, Android, or other platform SDKs;
- provide common abstractions for every operating-system feature;
- make every `System` API part of the runtime;
- require every `System` API to ship together;
- require package hierarchy to mirror namespace hierarchy;
- require assembly hierarchy to mirror package hierarchy;
- define every future capability package;
- require every language projection to support every NeoCLR feature;
- require symmetric language interoperability;
- force native libraries to adopt NeoCLR memory management;
- force all storage providers to behave like a native filesystem;
- encode every cross-cutting API relationship through namespaces;
- finalize the native ABI, component ABI, or all language projections in the initial implementation.

The goal is to preserve the architectural space necessary for these facilities to evolve without forcing premature commitments.

---

# 33. Initial Implementation Strategy

The initial implementation may primarily support:

```text id="1p2c0a"
Raven
  │
  ▼
NeoCLR Metadata + IL
  │
  ▼
NeoCLR Host
  │
  ├── Execution
  ├── Memory
  ├── Tasks
  └── Runtime Services
```

while still preserving the architecture required for other execution and interoperability models.

Several invariants should therefore exist from the beginning:

- metadata does not assume IL;
- metadata supports free functions;
- functions do not require synthetic containing classes;
- runtime APIs do not expose JIT internals;
- memory management remains conceptually separate from execution;
- runtime services belong to an explicit host;
- runtime identities can be represented through stable handles;
- native implementations remain possible;
- the CTS does not assume object-oriented source languages;
- consuming and implementing foreign components are treated separately;
- `System` APIs do not assume a particular host operating system;
- namespace boundaries do not dictate package boundaries;
- optional capabilities can be introduced without enlarging the foundational runtime.

The initial implementation can therefore be narrower than the architecture without making the architecture narrower than the intended platform.

---

# 34. Design Rules for Future NeoCLR APIs

Every major NeoCLR subsystem should be reviewed using several questions.

> **Have we accidentally made IL, the JIT, the GC, or another particular implementation part of this service's public model?**

> **Have we accidentally required a source language to adopt an object-oriented abstraction where the underlying concept does not require one?**

> **Have we accidentally exposed assumptions of the host operating system through the common `System` API?**

> **Have we confused namespace ownership with assembly, package, SDK, or deployment ownership?**

> **Have we introduced a shared abstraction because the concepts genuinely share semantics, or merely because we wanted a tidy hierarchy?**

> **Could this capability operate in a constrained environment without granting broader host access than it actually requires?**

These questions apply throughout the runtime and application platform.

---

# 35. Summary

NeoCLR is intended to be a **common runtime and application platform**, rather than merely a common virtual machine, common object system, or standard library.

Its conceptual foundation is:

```text id="bbhyg7"
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
        ┌───────────────┼───────────────┐
        │               │               │
      Memory         Execution         Tasks
        │               │               │
        └───────────────┼───────────────┘
                        │
             Foundational Libraries
                        │
                Common System APIs
                        │
              Optional Capabilities
                        │
                  Host Integration
                        │
                 Platform / OS
```

Languages sit around the common semantic model:

```text id="ubq0xz"
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

No language is required to use NeoCLR-managed memory for all of its own objects.

No language is required to become object-oriented in order to participate in NeoCLR.

Likewise, the common application platform does not require every capability to be part of the runtime or every environment to expose the same host facilities.

The commonality exists at the level of **semantics, metadata, runtime contracts, and stable platform APIs**.

The initial goal is not universal interoperability or universal platform abstraction. It is to establish sufficiently neutral boundaries that richer interoperability, capabilities, providers, packages, and application models can be built incrementally rather than retrofitted later.

> **Design semantic boundaries for stability; compose implementations and capabilities as the platform evolves.**