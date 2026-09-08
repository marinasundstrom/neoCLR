# Neo slice plan

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
General iteration protocols, custom steps and broader collection syntax remain future.

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
prints an error message and 42. Guards, nested destructuring patterns, user-declared
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
rebinding uses an explicitly addressed right-hand side. Raw native pointers retain
separate explicit low-level semantics and are not yet exposed by Neo. The
[managed-access tests](../tests/neo_managed_access.rs) cover contexts, copies, forwarding,
retargeting and lifetime checks.

## Managed arrays implemented

Neo now demonstrates owned array copying, managed heap arrays, indexed mutation,
Length and element-reference forwarding. The [array contract](managed-arrays.md)
keeps CLR newarr semantics and adds an explicit owned construction operation.
Reference elements, slices and pinning remain scoped future decisions.

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
variance and general dynamic casts remain separate work. Fixed-length array
annotations and braced initializers are also a separate refinement, described in
the [array contract](managed-arrays.md).

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
native collection backing store remains a separate migration.

## Next slice: reflection introspection

Extend [System.Type inspection](type-inspection.md) to enumerate fields, methods,
and properties. Keep the familiar .NET `System.Reflection` descriptor names and
`Type.GetFields()`, `GetMethods()`, and `GetProperties()` where they fit. The official
[GetFields](https://learn.microsoft.com/en-us/dotnet/api/system.type.getfields),
[GetProperties](https://learn.microsoft.com/en-us/dotnet/api/system.type.getproperties),
and [BindingFlags](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.bindingflags)
contracts are the comparison baseline; document every supported subset or deviation.

The bounded implementation should cover:

- `FieldInfo`: Name, DeclaringType, FieldType, and represented visibility/storage flags.
- `MethodInfo`: Name, DeclaringType, ReturnType, static/instance information, and
  `GetParameters()` returning parameter names, positions, types and output contracts.
  Keep constructor enumeration distinct from ordinary methods.
- `PropertyInfo`: Name, DeclaringType, PropertyType, index parameters, CanRead/CanWrite,
  and getter/setter descriptors using existing property metadata.
- Type shape: namespace/full-name policy, generic arguments, implemented interfaces,
  and element type plus IsArray/IsByRef/IsPointer where supported. Audit the existing
  qualified Type.Name contract before introducing CLR-like Name/FullName behavior.
- neoCLR-specific facts: managed-reference signatures and reference receiver mode.
  Expose these without classifying the underlying definition as inherently a value
  type or a reference type.

Start with public member enumeration. Define a supported BindingFlags subset for
explicit filtering; reject unsupported flags instead of silently ignoring them.
There is no inherited-member enumeration until inheritance exists. Document order,
overload identity, visibility, generic substitution and descriptor lifetime. Inspection
must not invoke property getters or grant mutation/private invocation access.

Return ordinary typed descriptor arrays, choosing value results by default and using
managed heap references only where the API deliberately promises shared storage.
Descriptors describe metadata; they must not capture guest object references or become
hidden GC roots. Preserve module/revision/definition identity and avoid recursive
unbounded snapshots when a type's members refer back to the type.

The end-to-end Neo example should enumerate a record's fields and methods, and a
library type's properties and method parameters, using existing arrays, loops and
ordinary property access. Include closed generic types and managed-reference signatures.
Keep Neo updates bounded to that scenario. Add source/artifact round-trip tests,
visibility checks and a GC-pressure test for returned descriptor arrays.

For inspection starting from an object/reference expression, explicitly distinguish
its declared type from the concrete type behind a managed interface view. Design a
reference-aware type lookup that does not require an Object base or boxing. Do not
pretend the existing `TypeOf<T>.Of(T)` provides dynamic type discovery. Reflective
GetValue/SetValue, Invoke, construction, attribute instantiation and metadata mutation
remain subsequent work after enumeration and type discovery are sound.

## Following slice: ordinary and output reference parameters

Clarify and demonstrate the difference between an ordinary `Foo&` parameter and the
proposed Neo spelling `out Foo&`, using the runtime's existing output contracts.
`Foo&` grants access to an initialized value. An output reference permits
uninitialized caller storage and requires the callee to assign it before a normal
return; the caller can then rely on initialization. Both use one managed reference,
with the same storage identity, frame/heap provenance and lifetime restrictions.
`out` is not a second reference layer or an ownership transfer.

Cover assignment tracking, reads before assignment, aliases and interior fields,
forwarding to other output parameters, and why an unrelated field write does not
fulfill an output obligation. Distinguish ordinary outputs from existing conditional
outputs (`out_when_true`) before choosing which syntax Neo should expose. The
[reference-slot contract](reference-slots.md) is the runtime baseline. Neo does not
yet expose output parameter declarations or uninitialized local bindings; implement
only the bounded source support needed for an end-to-end example after documenting
these rules.
