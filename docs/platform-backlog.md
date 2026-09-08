# Platform backlog

All planned capabilities and substantive revisions to implemented behavior follow
the [research and design comparison](design-research.md): establish the .NET/CLR
baseline, evaluate alternatives and justify improvements with evidence. Each roadmap
phase includes that research before its contract is settled.

These are planned capabilities, not implemented features or commitments to a release
date. The goal is to make neoCLR useful as a platform, with familiarity primarily in
C#/.NET APIs and observable behavior, while improving contracts where useful.
This does not require C# syntax, CLR internals or identical runtime representations.
The absence of legacy compatibility requirements gives us room to put mechanisms
in the runtime when that makes them more consistent across languages and libraries.

Preserve the existing foundations: values by default, explicit managed-reference
access, allocation independent of type identity, checked frame lifetimes, a managed
GC heap, and separate native pointers. An Object hierarchy remains optional for
runtime types. Keep Neo updated with small executable demonstrations of each slice;
it remains a concept compiler rather than a prerequisite full language implementation.

## Capabilities to develop

| Area | Intended capability | Decisions before implementation |
| --- | --- | --- |
| Binding immutability and readonly references | Binding rules in the language; restricted reference access enforced by the runtime | Preserve permissions through signatures, storage and dispatch; keep owned fields distinct from referenced targets; no runtime immutable-slot requirement |
| Object-oriented programming | Inheritance, base references, virtual methods and overrides, with familiar construction and member lookup | Base metadata and layout; constructor chaining; override validation; base-value copying or slicing; complete derived-object tracing and lifetime preservation through base views |
| Nullability | Genuinely nullable slots with an explicit absent value, including nullable managed references | Representation and type identity; nullable value versus nullable reference syntax; conversions, access checks and initialization; reflection, verifier and GC rules |
| Delegates and lambdas | [Delegates as the shared runtime callable abstraction](delegates.md); language function values and lambdas build on them, with bound receivers and captured environments | Typed single-target delegates and Func<Void> are implemented; C#-style closure environments; escaping captures and GC roots; delegate equality and possible multicast behavior; whether low-level function pointers are needed |
| Enums and flags | Named integral values and typed flag combinations with familiar .NET-like API behavior | Underlying width/signedness; distinct enum identity; explicit numeric conversions; zero, aliases and unnamed values; flag combination/testing; formatting, parsing and reflection |
| Generic constraints | Base/interface constraints and explicit not-null, not-void and not-reference restrictions | Metadata encoding, substitution and validation; constraint composition; address-mode restrictions versus object-graph restrictions; consistent enforcement for source, IL, reflection and host entry points |
| Runtime async model | Suspension and resumption, potentially with task-based APIs familiar from .NET | Ownership of suspended activations; references across suspension; scheduling and completion; cancellation, ordinary errors and terminal Faults; debugger and GC integration |
| Dynamic dispatch with hooks | Extensible runtime binding for operations whose targets are resolved dynamically | Supported operations and hook contracts; lookup and fallback order; missing-member results; access checks; caching and invalidation; interaction with typed virtual/interface dispatch |
| Fundamental library and framework | A coherent set of base types and useful framework APIs, implemented as scenarios require them | Which contracts belong in metadata/runtime services and which belong in library types; optional Object methods; collections, text, I/O, callable and async APIs; consistent errors, references and cleanup |

## Contract boundaries

**Mutability** separates compiler-enforced binding rules from runtime-enforced
managed-reference access. There is no immediate requirement for immutable runtime
slots. The [placement decision](mutability.md) and [readonly storage signatures](readonly-storage.md)
record that boundary.

**Inheritance** builds on the [object hierarchy design](object-hierarchy.md). A base
reference must preserve the complete derived allocation's identity and reachability.
Inheritance must not introduce an implicit value/reference type split or make every
type derive from Object. Value equality and hashing remain separate from reference
identity. Class syntax in Neo follows the runtime contract rather than defining it
in isolation.

**Nullability** follows the [explicit signature direction](nullability.md): a special
null state rather than zeroed bytes, with Option preferred for domain optionality.
It is a runtime representation and validation feature, not just a
compiler warning annotation. A null value must be distinct from an uninitialized
slot and from Void. Decide its relationship to Option<T> without silently treating
all three as interchangeable. In particular, distinguish a nullable reference from
a reference to a nullable value. Native null pointers retain their separate interop
meaning. Not-null constraints depend on these definitions.

**Delegates** should expose typed managed callables without requiring users to handle
raw pointers. A captured reference cannot outlive its target merely because a closure
holds it. Specify escaping captures and receiver retention before adding lambdas that
can escape a call. Low-level function pointers are a possible supporting mechanism,
not a settled requirement; managed callable identity and native callback lifetime/ABI
are separate concerns.

**Enums and flags** should provide named integral values and a flags designation,
with typed bitwise combination, removal and testing. Decide underlying integral types,
explicit conversions, zero/default values, duplicate names for one value, and treatment
of unnamed values and unknown bits. Define formatting, parsing and reflection behavior
alongside the metadata and verifier rules. Keep enums distinct from tagged unions
such as Option/Result; flag combinations are not separate payload-bearing cases.
A reflection-options example is a useful first consumer. An enum must not require
Object inheritance merely to have runtime type identity. Source syntax remains open.

**Generic constraints** now have an initial runtime-enforced
[notvoid/notreference subset](generic-constraints.md). These restrict the outermost
argument; notreference excludes managed reference forms, not native pointers or fields
containing references. Base/interface contracts and constrained-member lookup are next;
notnull depends on nullable metadata. Stronger symbolic implication checking and
constructor constraints remain separate work.

**Async** requires a lifetime decision before surface await syntax. Suspending a
callee while it holds a reference into a caller frame must either retain an eligible
owning activation or be rejected by a defined rule. Suspension must not implicitly
invalidate references or silently promote existing locals. Task-based completion is
a candidate projection, not yet the chosen runtime representation. Decide how async
completion carries Result/error values and terminal Faults within neoCLR's existing
error model rather than assuming a guest exception system. Suspension alone does
not imply parallel execution or a settled threading model.

**Dynamic hooks** extend runtime binding; existing virtual dispatch remains the
mechanism for statically declared polymorphic contracts. Define which operations can
be intercepted and how they preserve accessibility, argument types and lifetime
checks. Explicit metadata or runtime-service contracts should identify hooks rather
than assigning hidden behavior to an arbitrary method name.

**The framework** should grow into a useful, coherent platform through real programs.
Candidate needs include Object's conventional methods where a base is useful,
collection contracts and implementations, text and formatting, streams/files,
delegates, task APIs and resource cleanup. These are areas to select from, not a
promise to reproduce the entire .NET library. Follow the [API design](api-design.md)
and [API policy](api-policy.md), retaining familiar namespaces, names and contracts
where they fit the value/reference model.

## Suggested sequence and acceptance

1. Keep binding immutability in the language and complete shared readonly reference contracts.
2. Establish inheritance and nullable-slot contracts. They affect layout, tracing,
   assignability, reflection and the initial base-library design.
3. Explore enums and flags as a bounded metadata/library slice; it can move earlier
   because its integral representation need not wait for inheritance or nullability.
4. Add generic constraint metadata around those contracts. Address-mode and Void
   constraints can be explored independently, but their scope must be explicit.
5. Add managed delegates and closure lifetimes, with a small callback scenario.
6. Design suspension and implement one end-to-end async scenario, including GC and
   debugger inspection of suspended execution.
7. Add dynamic hooks around a concrete use case after member lookup and dispatch
   rules are stable enough to extend.

This is a proposed dependency order, not a fixed implementation schedule. Grow the
framework throughout these slices as each scenario needs it. For every selected
slice, document the contract, implement runtime enforcement and metadata support,
add the relevant verifier/GC/reflection/debugger integration, and demonstrate it in
Neo with source/artifact tests. Keep unsupported behavior explicit. Introducing a
runtime primitive is appropriate when it provides a shared enforceable capability;
ordinary library policy can remain ordinary library code.
