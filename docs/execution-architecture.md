# Platform execution and compilation architecture

Status: architectural requirements and proposed implementation sequence. Interpretation
is implemented; JIT compilation, native AOT, a high-level compiler, and a general
embedding API are not. This document does not introduce executable backend flags or
commit to a native code generator, binary schema, or public hosting ABI.

Interpretation, JIT, and native AOT are platform-wide architectural concerns. They
influence the type system, metadata, instructions, linking, verification, library,
and runtime services. Hosting is a separate consumer of these contracts, not the
place where execution-mode semantics are defined.

## Shared semantic contract

neoCLR defines observable behavior independently of the backend. Shared contracts
include type/member identity, value and generic semantics, calls and returns, storage
and initialization, numeric operations, native interop, Errors and Faults, and
explicit runtime services. The interpreter is an implementation and a conformance
tool; its Rust representations are not the specification.

The assembler and future high-level compiler produce the same platform metadata/IL.
They can have separate front-end representations and need not communicate through
assembly text. Verification and resolution should operate on that common platform
model. Native backends may lower it into their own machine-oriented representation.

```text
neoIL assembler -----------+
                           +--> metadata/IL --> resolution + verification
future language compiler --+                         |
                                  +------------------+-------------------+
                                  |                  |                   |
                             interpreter      JIT compilation     native AOT build
                                  |                  |                   |
                                  +-------- defined runtime services -----+
```

This is a logical architecture, not a demand for identical runtime machinery.
Resolution may happen at build time, load time, or invocation time under a declared
profile. Native code can specialize, inline, and eliminate operations when their
observable semantics are preserved. Values may be represented differently inside a
backend, but interoperability boundaries require explicit representation agreements.

## Mode capabilities and artifacts

| Mode | Compilation/execution responsibility | Initial architectural requirement |
| --- | --- | --- |
| Interpreter | Execute supplied supported IL | Resolve modules and callable signatures without assuming a native compiler exists |
| JIT | Generate native code during execution | Define compilation requests, generic instantiation, code lifetime, and runtime service bindings |
| Native AOT | Generate native code before execution | Identify compilation roots, resolve dependencies and required instantiations, and emit explicit runtime/native imports |

Recommend two early AOT outputs: a standalone executable and a native library with
explicit exports. An AOT executable may link a runtime support library; native AOT
does not imply freestanding execution or absence of runtime services. A future
freestanding profile would be a separate requirement.

Capabilities such as runtime compilation, IL interpretation, metadata discovery,
and loading additional precompiled modules must be distinct. Loading a native module
does not itself require a JIT. Initial AOT can use a statically closed set of code
roots while later designs consider compatible precompiled extensions.

Unavailable capabilities must produce explicit diagnostics. There must be no silent
fallback that embeds an interpreter/JIT into an AOT artifact. A hybrid profile could
include them deliberately. Neither one mode per process nor one mode per context is
settled; mixed execution needs call/representation/code-lifetime contracts before it
can be supported. The architecture must not reduce modes to a hosting-API setting.

## Decisions needed before a native backend

### Identity and references

Types and members need definition identities distinct from display names and source
aliases. References must retain their declaring module/type and generic context.
Closed instantiations must not replace definition identity: two declared overloads
can acquire identical substituted parameter types and still denote different members.
Linking must map those references deliberately. This is the next shared prerequisite
for interpretation, compilation, tooling, and embedding.

Free functions remain functions without a required synthetic type container in the
semantic model. Preserve familiar CLI concepts where they fit; do not conflate a
CLI-like file with ordinary .NET execution compatibility. See [format direction](format-direction.md).

### Representation and calling conventions

Distinguish portable signatures from target layouts and call ABIs. Specify target
pointer width, alignment, record layout, argument passing, return values, and native
interop boundaries. Different architectures need not share byte layouts. Compiled
and interpreted calls on the same target need adapters or an agreed ABI if they mix.

Void remains an inhabited semantic value. A native ABI may carry no physical bytes
for that value while preserving its logical return/stack contract at IL boundaries.
Likewise, value semantics do not promise physical native-stack storage for every
value. Explicit pointer identity and lifetime remain observable constraints.

The current interpreter uses host record objects and per-call method substitution.
Those are prototype techniques, not requirements for compiled code. Avoid exposing
them as the permanent object ABI or forcing runtime dictionary lookups merely because
the interpreter currently resolves names on invocation.

### Generics

Retain type parameters, constraints when available, and instantiated references in
the common model. Select native specialization/code-sharing policy separately.
Full specialization, shared implementations with type information, and combinations
remain options; the platform has not selected one.

For an initial AOT experiment, explicitly identify required closed instantiations
from reachable code and declared roots. Unbounded or dynamically selected instantiations
need a diagnostic or an explicitly supported capability. Reflection and exported
generic APIs must not depend on code that the build accidentally omitted.

### Runtime services, failures, and optional memory management

Define allocation/free, native calls, host services, module initialization, and Fault
reporting as explicit contracts usable by all backends. Calls need an ABI for normal
values as well as terminal Fault propagation across compiled/interpreted/native
boundaries. Guest Result values are ordinary data and are not the Fault channel.
Do not introduce guest exceptions to obtain a convenient native implementation.

Fault containment, context reuse, and cleanup after partial work remain open. The
hosting proposal recommends returning control to the host for an invocation Fault,
but this is not a promise of rollback or safety after arbitrary native failures.
Resolve this before making a stable public invocation ABI.

Allocation remains separate from construction and ownership. Optional reference
counting or collection must have explicit protocols. If a future profile permits
moving collection, compiled code would need the agreed roots/relocation/pinning and
barrier contracts; unrestricted stable raw addresses cannot silently become movable.
Neither JIT nor AOT should require a universal GC or count header on every value.

### Verification, checks, and observability

The existing verifier analyzes metadata and IL independently of invocation. It should
remain usable by every backend. Its current opt-in status and limitations are in
[verification](verification.md). Whether verification is mandatory for a particular
profile is still a decision, not a different language semantics for each backend.

Classify checks before generating optimized code: which failures are required platform
behavior, which operations require verified preconditions, and which interpreter side
tables are diagnostics for raw pointers. Do not quietly erase a required Fault in AOT,
or require every backend to emulate a diagnostic representation that was never part
of the contract. Shared tests must distinguish these categories explicitly.

Debug/source mappings and runtime diagnostics should follow module/member identities,
with backend-specific instruction/native-code locations as additional information.
Full debugger integration is later work; retaining a route back to source should not
be an afterthought of the high-level compiler.

## Hosting is an independent requirement

neoCLR should be embeddable through a deliberate hosting API, eventually with a
versioned native boundary suitable for applications written in other languages.
The existing Rust run helpers are a starting point, not that final ABI.

A minimal hosting milestone should load a module, resolve a function by identity and
signature, invoke it repeatedly with typed inputs, return values/Faults, and provide
host services. Value/handle lifetimes, resource limits, cancellation, callbacks,
reentrancy, threading, and teardown require explicit contracts. A versioned C ABI
with opaque handles is a recommendation, not an implemented choice.

The CLI and language tooling should consume hosting/runtime services rather than
require special executable-only semantics. AOT exports and interpreted invocations
should share semantic contracts without pretending all internal value layouts or
entry mechanisms are identical. General native library exports can also exist
without embedding the interpreter.

## Proposed implementation sequence

1. Stabilize module/type/member identities and references, including generic overloads.
   Keep the verifier aligned with this model; complete type/access contracts before
   finalizing constructor or mutation lowering.
2. Specify invocation, target layout, failure, and runtime-service boundaries. Build
   a minimal embedding experiment using the interpreter, without freezing a broad ABI.
   Perform a small binary metadata/read-back experiment to test tooling assumptions.
3. Develop the future language compiler as another producer of the same metadata/IL.
   Migrate a few library functions/types incrementally; compiler self-hosting can wait.
4. Run a deliberately small native AOT experiment: one executable printing HelloWorld,
   and one exported scalar function invoked by a C host. State supported instructions,
   architectures, runtime dependencies, and generic roots. Use the same semantic tests
   as the interpreter for that subset.
5. Expand native compilation using library workloads and conformance results. Introduce
   JIT compilation through the shared native/runtime contracts when useful. This is
   a proposed implementation order; JIT remains an architectural target from the start.
6. Consider mixed execution, broader dynamic discovery, optional ownership/GC protocols,
   richer native interop, and runtime async after their concrete contracts are ready.

For each backend, test values, copies, calls, generics, initialization, explicit memory
operations, numerical behavior, Errors, and required Faults on supported platforms.
Pair future language-generated modules with handwritten IL fixtures. The aim is one
platform with several implementations, not an interpreter language and a separately
specified compiled language.
