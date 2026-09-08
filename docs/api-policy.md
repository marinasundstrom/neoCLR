# Familiar concepts, deliberate platform contracts

All planned capabilities and substantive revisions to implemented behavior follow
the [research and design comparison](design-research.md): establish the .NET/CLR
baseline, evaluate alternatives and justify improvements with evidence. Each roadmap
phase includes that research before its contract is settled.

Use familiar .NET APIs and concepts wherever they fit neoCLR's intended model. We do
not need to reproduce CLR/.NET legacy structures, historical restrictions, or every
compatibility behavior. Deliberate improvements to structure and behavior are valid
platform decisions, not merely unavoidable exceptions to a compatibility requirement.
Familiarity is a goal independent of implementation:
a platform-written function, native runtime call, interpreter operation, and future
JIT implementation should expose the same documented consumer contract.

Prefer established namespaces, type/member names, overload families, parameter
order, and observable behavior when their contracts remain appropriate.
Implement a small coherent subset first instead of inventing unrelated substitutes
for missing framework functionality. A missing implementation remains a documented
gap, not a reason to redefine the API silently.

Intentional changes include Result-based recoverable errors, Option-based absence,
first-class Void, explicit ownership/sharing, free functions, and interface names
without the `I` convention. These need explicit API mappings. Implementation details
alone do not justify additional consumer-visible differences.

## Library scope and language projection

The long-term target is the expected breadth and role of the .NET foundational
library, adapted to neoCLR rather than exact API or binary compatibility. The
[library-focused preview plan](library-preview.md) prioritizes interface/class
inheritance where reusable library contracts require it, then useful APIs and
end-to-end examples. Runtime/library contracts and Neo syntax are separate design
layers: other frontends must be able to consume the same APIs. Familiarity includes
documented behavior, not just familiar member names.

## Current examples

For each intentional departure, state the problem it solves, the resulting contract,
and the migration impact. Keep compatible behavior where useful, but do not add a
legacy abstraction solely because .NET exposes it. Use the current preview plan to bound implementation while
leaving room for broader library coverage as the runtime develops.

| API | Familiar surface | Deliberate change or current limitation |
| --- | --- | --- |
| `System.Console.WriteLine(string/int32)` | Familiar name and overloads | Produces the real Void value; console buffering is a prototype limitation |
| `System.Int32.Parse(string)` | Familiar parsing entry point | Returns Result instead of throwing; culture and whitespace coverage remain incomplete |
| `System.Math.Abs(int32)` | Familiar name and numeric result on success | Returns Result for the overflow case |
| `System.Int32.Divide(int32,int32)` | Uses the familiar Int32 domain | Experimental neoCLR extension for recoverable division, not a claim of a matching .NET member |
| `neoCLR.Runtime.*` | Internal implementation boundary | Not intended as consumer-facing replacements for System APIs |

The current assembler models declaring types and static/instance methods alongside
free functions. Primitive members belong to canonical System type definitions.
Ordinary receivers are value snapshots; methods can explicitly declare byref
receivers when they must access the original managed slot. Basic visibility, assembly references and explicit borrowed
interface dispatch are implemented, as are managed base views, class virtual dispatch
and abstract records. See [class contracts](class-dispatch.md) for current restrictions.

The Divide helper is a proof-of-concept extension whose final API location remains
open. Existing .NET APIs should be preferred when an appropriate equivalent exists.
BCL error/absence adaptations should be documented alongside behavior and migration
examples rather than buried in runtime implementation notes.

## Runtime implementation declarations

Use the recognizable `MethodImpl`/`InternalCall` metadata mechanism for functions
provided by native runtime code. The public library remains written for neoCLR and
calls declared helpers at an explicit boundary. A future source compiler should be
able to accept the familiar attribute form and lower it to the implementation flags.
Runtime binding is separate from public API naming and from native interop imports.
See [runtime library](runtime-library.md) for the implemented mapping and limits.

As APIs are added, tests should cover ordinary observable .NET behavior and the
intentional neoCLR adaptations separately. Do not promise compatibility for a
member until its supported inputs, outputs, errors, and other effects are defined.

## Collection namespace and naming

Place generic collections directly in System.Collections. Generic arity already
distinguishes type identities; a separate Generic namespace is unnecessary. The
preview implements System.Collections.ArrayList<T> with a managed T[]& backing
array and GC lifetime. T can itself be a managed-reference type. It implements List<T>, without an I prefix, through
[explicit borrowed interface references](interfaces.md).
See [ArrayList](array-list.md) for the concrete API and current element limits.

## References in ordinary APIs

Use typed managed T& parameters when an API needs access to caller storage, and
explicit byref receivers when a method must mutate the original value. Keep raw
pointers for APIs deliberately concerned with native memory or interop. Ordinary
by-value inputs remain useful where copying is the intended contract; a managed
reference does not imply counted ownership, allocation or retention.

Out parameters declare assignment on every normal return. Boolean TryGet methods
can use out(true) for assignment only on success, avoiding invented default values
for arbitrary T. System.Option/Result case extraction now follows that contract.
See the [Raven-like reference guide](references-in-pseudocode.md).

Reflection member descriptors share an abstract MemberInfo base for common metadata
and expose readonly managed readers. This is a concrete use of shared storage and
base views; independent capabilities should use interfaces when needed. See
[the API and .NET comparison](reflection-hierarchy.md).

## Runtime guarantees and language policy

The runtime owns shared correctness rules: valid initialized storage, declared
nullability, reference lifetimes, readonly access, construction completion and typed
default semantics. No compiler, handwritten IL or artifact may bypass those rules.
Nullability is still planned; current managed references have no null/default value.

A language owns declaration syntax, binding immutability, inference, early diagnostics,
field-initializer lowering and constructor synthesis. Another language may insert more
initialization than Neo does, provided its resulting IL satisfies the same contracts.
Do not introduce runtime metadata solely to enforce a Neo syntax/synthesis preference.

For example, Neo's [class synthesis rule](classes-and-defaults.md) requires initializers
for all fields when no init is declared. That is compiler policy. Requiring construction
to finish with every field initialized is runtime enforcement. A field can be initialized
with default(T) only if T actually has a valid runtime default. Never invent a zero
address for a non-nullable reference merely to satisfy an initialization obligation.

C# also performs constructor synthesis and inference in the compiler; .NET reference
nullability annotations do not impose equivalent runtime non-nullability. The deliberate
neoCLR difference is enforcing reference/null-state intent across languages, with
representation, verification and GC costs recorded in [the nullability design](nullability.md).

Field completion is the mandatory construction boundary, including conditional
assignments and early returns. Local/temporary definite assignment can be handled
primarily by language analysis and verification. The current interpreter still checks
invalid reads and reference use; future verified backends may remove checks proven
unnecessary. This does not introduce a universal local-slot initialization feature
or relax managed-reference lifetime validity. Nullable metadata is a separate slice.


Delegate APIs retain Invoke and Func's input/result ordering. Because Void is an
ordinary generic argument, Func<T,Void> replaces Action<T>; there is no separate
Action family. This reduces duplicate API families but requires a mapping when
porting .NET APIs. Closures will use compiler-generated captured environments and
delegates, following the C# model. See [the contract](delegate-contract.md).
