# Neo slice plan

The [library-focused next preview](library-preview.md) prioritizes shared type
relationships, interface inheritance and class inheritance to support useful library
contracts and their Neo projection. [Interface inheritance](interface-inheritance.md)
and [class inheritance/virtual dispatch](class-dispatch.md) are implemented.

All planned capabilities and substantive revisions to implemented behavior follow
the [research and design comparison](design-research.md): establish the .NET/CLR
baseline, evaluate alternatives and justify improvements with evidence. Each roadmap
phase includes that research before its contract is settled.

The [Neo foundation](neo.md) and the completed slices below are implemented. The
[grammar](neo-grammar.md) tracks shipped syntax, with future features called out
separately.

Neo is a small companion compiler for testing and explaining neoCLR. Maintain it as
the runtime changes: update affected lowering, runnable examples, tests and grammar
in the same slice. Prefer ordinary CLR-like IL and existing library contracts; add
runtime instructions only when a concrete semantic need justifies the deviation.
The current compiler is not intended to become a complex, full-fledged compiler.
Every addition should make a runtime capability easier to exercise or explain.

## 1. Structured control flow — completed

Implemented if/else, while, integer-range for, loop, break/continue, primitive
comparisons and short-circuit Boolean operators with existing IL branches.
`0..9` includes both endpoints; `0..<10` excludes the upper endpoint. Bounds evaluate
once, empty ranges skip the body, and inclusive Int32.MaxValue does not wrap.

Block names do not escape or shadow active names. Branch return checking is
implemented; loop termination analysis stays conservative. Storage remains frame-lived,
so taking addresses of block-local values is rejected. References to outer locals and
heap objects remain available; deterministic block cleanup is not claimed.

[Acceptance tests](../tests/neo_control_flow.rs) cover ranges, nested transfers,
short-circuit effects, scope/lifetime diagnostics and instruction limits. Run the
[control-flow example](../examples/source/control-flow.neo) to print and return 21.
[Iterable/Iterator library protocols](common-interfaces.md) are now implemented;
Neo currently consumes them with while. Iterable for lowering with guaranteed disposal,
custom steps and broader collection syntax remain future.

## 2. Patterns and union-aware match — completed

Implemented match expressions and statements with case payload bindings, discarded
payloads, payload-free cases and a wildcard. Both forms require exhaustive coverage
and reject duplicate/unreachable arms. Expressions have one exact result type;
statement blocks allow actions, return and loop transfers. Scrutinees evaluate once.

Closed generic annotations and typed System calls expose Option/Result. Coverage
requires bundled System's union marker, constructor cases and typed public accessors;
matching lowers to existing library calls and branches. Payload bindings are ordinary
copies scoped to their arm. Runtime provenance checks still reject indirect frame
escapes; heap-backed field references can remain valid match results.

[Acceptance tests](../tests/neo_match.rs) exercise parsing, nested Option/Result input
and EOF, type/coverage failures and lifetimes. The [match example](../examples/source/match.neo)
prints an error message and 42. Guards, nested destructuring patterns, generic source
unions and subtype patterns remain future work; nested unions currently use nested
matches. No general open-world exhaustiveness contract is claimed.

## 3. A bounded console calculator — completed

The [calculator](neo-calculator.md) combines input, parsing, matching, loops and
reference parameters. It reports recoverable parse/division errors, permits retries,
and stops at EOF or its explicit input cap. The only additional compiler support is
explicit byte-to-int conversion and ordinary System instance calls for existing
library operations. No new runtime service or opcode was needed.

[Acceptance tests](../tests/neo_calculator.rs) cover representative and malformed
input, resource bounds, I/O failures, CLI execution and a separate GC-pressure
workload. Existing GC monitoring is adequate; expand it when a workload exposes a
concrete diagnostic gap.

## 4. Type inspection — completed

`typeof(T)` produces the existing System.Type descriptor through ldtoken and
GetTypeFromHandle. Aliases, records, T& and closed generic signatures use existing
metadata identities. Read-only System properties expose Name and GenericArgumentCount
through their declared getters; ordinary calls expose Equals and GetGenericArgument.
No instance of T, boxing, dynamic GetType behavior or new runtime opcode is involved.

The [source example](../examples/source/typeof.neo) and [tests](../tests/neo_typeof.rs)
cover descriptor identity, generic arguments, return lifetime, invalid operands,
property access boundaries and CLI execution.

## Managed-reference source access

Managed references are read and written automatically in Neo. Examples use
`age = age + 2`, with the compiler providing the underlying loads/stores. Explicit
reference formation/forwarding remains available through `&`; mutable reference
rebinding copies a reference-valued right-hand side. Raw native pointers retain
separate explicit low-level semantics and are not yet exposed by Neo. The
[managed-access tests](../tests/neo_managed_access.rs) cover contexts, copies, forwarding,
retargeting and lifetime checks.

## Managed arrays implemented

Neo now demonstrates owned array copying, managed heap arrays, indexed mutation,
Length and element-reference forwarding. The [array contract](managed-arrays.md)
keeps CLR newarr semantics and adds an explicit owned construction operation.
Reference elements are implemented. Slices and pinning remain future decisions.

## Interface demonstration implemented

Neo now supports interface declarations, record conformance and instance methods,
explicit `as Contract&` projections, implicit concrete-to-interface reference
conversions in typed contexts, and virtual interface dispatch. Names follow the usual
Neo convention without an `I` prefix. The [interface guide](neo-interfaces.md) and
[source example](../examples/source/interfaces.neo) demonstrate frame and heap
receivers, mutation, caller-view forwarding and heap retention without boxing.
Tests cover invalid conformance, local escapes, reference parameters/results and GC
pressure. The slice uses the existing runtime interface operations and managed receivers.

Choose further source support around concrete scenarios: adapting bundled System
contracts, receiver modes, properties or generic declarations. Interface inheritance,
variance and general dynamic casts remain separate work. Checked local array extents
and optional braced heap initializers are implemented, as described in
the [array contract](managed-arrays.md). Bundled single-index Item properties now
project through bracket syntax for getters and setters.

## Later platform slices

Use the completed programs to select the next runtime capability and add a small Neo
demonstration with it. The [optional object hierarchy](object-hierarchy.md) remains
a later platform area after the memory foundation and interface demonstration: first establish base
layout/views and tracing of the complete derived allocation, then dispatch and common
library methods as needed. No mandatory Object root; value equality and hashing must
remain independent of whether a value is accessed as T or T&. Reference identity is
a separate operation. Do not add inheritance syntax before those contracts are ready.

[Pinning and native interop](pinning.md) remain a separate bounded design effort once
an interop scenario requires them. [Library ports and compiler bootstrapping](neo-bootstrapping.md)
are possible later exercises, not prerequisites or reasons to grow every compiler
feature now.

## Interactive inspection implemented

The [terminal debugger](debugger.md) launches paused, supports stepping and breakpoints,
and inspects live guest frames, managed heap objects and tracked native storage. Neo
emits source sequence points and slot labels. Further UI/editor integration and richer
source-level debugging should build on this execution-control and snapshot boundary.

## Runtime library reference contracts reviewed

The [API design document](api-design.md) inventories values, managed references,
receivers, outputs, and native storage contracts. List/ArrayList now require managed
reference receivers, and Neo honors library receiver and property metadata. The
collection now holds a managed T[]& backing array, supports reference elements, and
uses GC rather than Free. Its capacity allocation and copy rules are documented in
[ArrayList](array-list.md).

## Reflection introspection — completed

Implemented [reflection introspection](reflection.md) through ordinary System.Type
queries and independent MethodInfo, FieldInfo, PropertyInfo and ParameterInfo records.
The [Neo example](../examples/source/reflection.neo) exercises field/method/property
arrays, parameter types, and closed managed-reference generic arguments using existing
syntax. Public defaults, explicit BindingFlags filtering, accessor options and type
shape queries are documented and tested. Constructors and module free functions are
excluded from Type.GetMethods; FunctionInfo remains reserved for future module queries.

Managed references now support GetType discovery of their concrete live target,
including through interface views, without boxing, requiring Object, or retaining
the target. Reflective execution and metadata
mutation remain separate future work.

## Ordinary and output reference parameters — completed

Neo exposes unconditional `out name: Foo&` declarations, `out destination` call-site
arguments and typed uninitialized var declarations. It also consumes existing
conditional library output metadata. The [output guide](neo-outputs.md) explains
initialization, forwarding, interface dispatch, reference targets versus bindings,
and the division between static caller checks and runtime callee obligations.
Conditional output declarations and more complete static alias analysis remain future.

## Platform backlog projection

The [platform backlog](platform-backlog.md) records planned inheritance, nullability, enums/flags,
delegates/lambdas, generic constraints, runtime async, dynamic hooks and framework
growth. Add bounded Neo syntax and executable examples alongside each selected
runtime slice; these plans do not expand Neo into a full compiler project. Runtime
contracts and cross-layer lifetime checks come before convenient surface syntax.

Explore [binding immutability and readonly access](mutability.md) before expanding
inheritance and closure syntax. Neo projects runtime-enforced storage and reference
permissions; let binding immutability must remain distinct from target mutability.

The [readonly input-parameter slice](readonly-parameters.md) now enforces restricted
managed access at runtime, with Neo declarations and ParameterInfo.IsReadOnly.
Readonly instance receivers are also implemented, with MethodInfo.IsReadOnly.
[Readonly storage and return signatures](readonly-storage.md) now preserve declared
permissions. The verifier remains conservative about aliases and lifetime provenance.

## Generic functions and the next independent building blocks

[Generic free functions and static methods](function-generics.md) are implemented in
the runtime and Neo, with argument-based inference and explicit type arguments.
Inference keeps emitted runtime contracts precise and preserves managed-reference
bindings. Neo should avoid redundant syntax while keeping value/reference intent
clear; other languages may project the same platform differently.

Keep the upcoming work separate:

1. [Ordinary classes and default(T)](classes-and-defaults.md) are implemented with
   body-declared fields, field initializers and bounded constructor synthesis. Broader
   automatic initialization depends on runtime nullability/defaultability; reference
   slots must never receive an invalid reference as a default.
2. [Runtime delegates](delegates.md), contextual Neo method-group conversion, Func
   (including Void results) and Array.ForEach are implemented. Ordinary functions
   remain definitions; nullable metadata remains separate.
3. Contextual lambdas and shared managed capture cells are implemented, including
   escaping/nested closures and runtime rejection of frame-reference captures.
   Next evaluate broader inference and capture allocation costs; nullable metadata,
   multicast and variance remain independent decisions.

These are bounded explorations, not a commitment to turn Neo into a full compiler.

## Source union declarations and remaining case projection

[Non-generic source unions](neo-unions.md) now generate separate cases and carrier
constructors, with exact case conversions and exhaustive matching.
[Case imports](neo-case-imports.md) now support unqualified constructor/type names
for source and marked bundled unions. Next investigate generic source type declarations
and arbitrary external assembly discovery under
[the Raven-style projection](result-construction.md#case-projection-ravens-model).
Use [type-design guidelines](type-design.md) to review the full field and operation
contracts when selecting value/reference use.

[Explicit library constructor calls](neo-library-constructors.md) now supply the
ordinary metadata path for standalone generic cases and explicit carrier construction.
Argument-based generic constructor inference and bundled case imports now use that path.

[Implicit case-to-carrier conversion](case-to-carrier-conversion.md) now uses marked
bundled carrier constructors with exact case types. The Raven source comparison records
which broader inference and conversion behaviors remain deferred.

The [order-workflow usability pass](experiments/reference-experience/README.md#current-usability-pass-2026-09-09)
now combines imported inferred cases, explicit nested domain-error conversion and
conditional bindings. Reference identity, receipt snapshots and rejected-purchase side
effects are covered together. Next investigate generic source type/union declarations;
error propagation syntax and richer patterns remain later usability questions.
