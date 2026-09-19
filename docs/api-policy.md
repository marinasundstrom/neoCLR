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
first-class Void, invariant mutable arrays, free functions, and interface names
without the `I` convention. These need explicit API mappings. Implementation details
alone do not justify additional consumer-visible differences.

## Namespaces for utility functions (2026-09-14)

Prefer namespace functions when an API only groups operations and does not need a
type contract. The author selected `System.Math` as the first example: a namespace
can contain functions, constants where the language supports them, and related
types. Do not introduce a public utility class solely to act as a namespace.
Keep classes when instances, state, polymorphism, or another actual type contract
justify them. This is a guideline, not a blanket conversion of existing APIs.

Separate the source API from its binary representation. For now use Raven's existing
.NET namespace-function contract: static methods in a compiler-generated CLI type,
identified by the target core's `System.Runtime.CompilerServices.TopLevelAttribute`.
The generated container is an emission detail, not the intended source-level API.
Do not require new metadata tables or infer namespace membership from a container
name. The existing [namespace-member probe](experiments/raven-target/NamespaceMemberProbe.cs)
checks referenced function imports, overloads, completion, accessibility and rejection
of unmarked lookalike containers. neoIL free functions and CLI method containers
remain distinct representations; importing must preserve their documented mapping.

Compared with exposing a .NET-style static utility class, this gives the namespace
room for related types and avoids making a type part of the API merely to group
functions. It does not imply a performance benefit. The cost is a different source
contract for languages without namespace functions, plus deliberate handling of
generated containers in metadata, reflection and tooling. Such languages may need
to call the emitted static methods. Preserve normal CLI metadata for now rather
than introducing another binary format to hide that difference.

The Math migration must validate both qualified calls and `import System.Math.*`,
including a related type in that namespace. A namespace and the old public
`System.Math` type cannot simply be presented as interchangeable source contracts.
Record the API migration separately from replacing method bodies with Raven code;
this guideline alone does not claim that the migration has been implemented.

## Library scope and language projection

The long-term target is the expected breadth and role of the .NET foundational
library, adapted to neoCLR rather than exact API or binary compatibility. The
[library-focused preview plan](library-preview.md) prioritizes interface/class
inheritance where reusable library contracts require it, then useful APIs and
end-to-end examples. The [current API plan](runtime-api-plan.md) sequences additions by concrete need.
Runtime/library contracts and Raven syntax are separate design
layers: other frontends must be able to consume the same APIs. Familiarity includes
documented behavior, not just familiar member names.

## Design for modern language consumers

The author's 2026-09-14 direction distinguishes application freedom from standard
library design. Applications may use callbacks and other supported programming
patterns. Public runtime class-library APIs in System namespaces should present
coherent contracts for modern programming languages, consumable across frontends.
Evaluate the consumer's use, composition and result handling before choosing the
implementation mechanism or reproducing historical API families.

For ordinary asynchronous operations, default to the selected task-based direction: Task<T>
represents the operation and Task<Void> represents completion without a payload;
the runtime eventually supplies suspension and resumption. Fallible asynchronous
operations carry Result inside Task (Task<Result<T,E>> under the current explicit
error-type contract). Await yields the Result; recoverable errors remain values
handled through Result, without a separate task-exception channel. Cancellation and
terminal Fault handling require their own contracts. Do not add parallel
callback-completion or legacy async API families solely for .NET compatibility.
Some public runtime APIs are better expressed through callbacks; choose callbacks
when they best express the specific contract. This includes, but is not limited to,
predicates and event handlers. Task-based completion is the normal async pattern,
not a blanket requirement for every runtime API or a prohibition on delegates.
Internal callbacks and host adapters need not determine the public async surface.

This follows the existing [.NET TAP comparison](platform-direction.md#async-and-time-preserve-the-application-model)
and [design research process](design-research.md). A focused task-based surface
reduces overlapping completion contracts for consumers; the cost is adaptation when
porting callback-based code or integrating callback-based hosts. Exact task members,
adapters and suspension contracts remain future design work. Broader API choices
must justify their own benefits and costs; modern language support alone does not
establish that a new API is better.

## Async naming decision (2026-09-14)

Use ordinary operation names for asynchronous System APIs. The Task return type
communicates asynchronous completion; an Async suffix is unnecessary. For operations
that may wait, async is the ordinary surface and an explicitly synchronous or blocking
alternative is the exception, added only for a concrete need.

For example, a proposed Read(...) -> Task<Result<Bytes,ReadError>> may coexist with
ReadBlocking(...) -> Result<Bytes,ReadError> when a blocking implementation is useful.
Use Blocking when the call actually waits; Sync may identify a synchronous alternative
where that is the more accurate contract. This does not require suffixing ordinary
in-memory computations or making every library method return Task. Do not distinguish
paired methods solely by return type. The language async keyword and runtime method
metadata are separate questions unaffected by this naming choice.

.NET TAP uses Async names and preserves existing synchronous APIs when adding task
counterparts. neoCLR need not carry that compatibility burden. The author identifies
the existing synchronous surface as a major historical reason for .NET's convention;
the documented coexistence guidance supports that rationale without establishing
Microsoft's complete historical motivation. See the [comparison](async-api-design.md).
The benefit is a consistent ordinary name for the preferred operation; the cost is
less .NET naming familiarity and dependence on signatures/tooling to identify a task
return at an unawaited call site. Porting .NET code may require renaming calls.

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
Ordinary class/value receiver behavior follows the selected CLR-like type categories;
explicit byrefs address storage. See the [Raven application contracts](raven-application-types.md)
for the current imported surface and restrictions. Earlier Neo/address-mode examples
are historical and do not define current frontend requirements.

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

## Query operator naming direction (2026-09-19)

The author supplied this naming principle:

> Prefer terminology that has converged across modern languages for fundamental
> iterable operations; retain .NET terminology where it is already broadly
> conventional or materially clearer.

Apply it to the existing query API after Preview 8, before Task/async contract work.
This is not a mechanical adoption of Rust or JavaScript terminology.
Keep Raven's existing method casing; this is terminology, not a casing redesign.

The first post-release slice uses `Map` for element projection (Preview 8: `Select`) and
`Filter` for predicate selection (Preview 8: `Where`), with initial capitals as the
author confirmed. Preserve deferred execution,
order, callback counts and iterator cleanup semantics while changing those names.
Do not rename already clear operators merely to differ from .NET. Review terminal
names against their Option/Result and cardinality contracts, not only a synonym list.
For future flattening, `FlatMap` is preferred over `SelectMany`. Do not automatically
choose `Fold` over `Reduce`, or `Drop` over `Skip`: names must follow the selected
semantics, including seed and empty-input behavior where applicable. Those operators
were not implemented by the naming decision alone. The subsequent basic-operator
slice implements FlatMap and retains Skip. The author selected Fold for explicitly
seeded, left-to-right accumulation; empty input returns the seed. Seedless reduction
remains separate. See the [mapping table](raven-query-api.md#net-operator-mapping).

[.NET Enumerable](https://learn.microsoft.com/en-us/dotnet/api/system.linq.enumerable)
provides the current comparison. Conventional terminology also appears within .NET:
[F# sequences](https://learn.microsoft.com/en-us/dotnet/fsharp/language-reference/sequences)
use map/filter/fold, while [Rust iterators](https://doc.rust-lang.org/std/iter/trait.Iterator.html)
provide map/filter/flat_map/fold/reduce (primary sources consulted 2026-09-19).
These names can make the library easier to recognize across languages; the cost is
source migration and reduced familiarity for callers expecting LINQ spellings.
Equivalent names do not imply identical ownership or empty-sequence behavior.

Validate the chosen API through source compilation, emitted metadata, editor
completion and existing query execution tests. Update samples and the feature page
together; document the migration rather than presenting new spellings as Preview 8
capabilities. This slice removes the old names without aliases and retains System.Linq;
a namespace redesign is separate work.

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

## Compositional type and parameter contracts

Follow the [type-design guide](type-design.md): evaluate a type's fields and operation
guarantees together. An outer value can share reference-valued fields, and T& alone
does not grant retention or ownership. Specify copying, mutation, retention, readonly
limits and disposal independently when designing library APIs.
