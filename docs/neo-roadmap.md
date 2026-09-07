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

## Later platform slices

Use the completed programs to select the next runtime capability and add a small Neo
demonstration with it. The [optional object hierarchy](object-hierarchy.md) remains
the next planned platform area after the memory foundation: first establish base
layout/views and tracing of the complete derived allocation, then dispatch and common
library methods as needed. No mandatory Object root; value equality and hashing must
remain independent of whether a value is accessed as T or T&. Reference identity is
a separate operation. Do not add inheritance syntax before those contracts are ready.

[Pinning and native interop](pinning.md) remain a separate bounded design effort once
an interop scenario requires them. [Library ports and compiler bootstrapping](neo-bootstrapping.md)
are possible later exercises, not prerequisites or reasons to grow every compiler
feature now.
