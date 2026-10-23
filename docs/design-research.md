# Research and decisions: improving the .NET platform and CLR

Every roadmap capability and substantive revision of an implemented feature belongs
in this discussion: what .NET/CLR already provides, what problem remains for neoCLR,
and which solution best serves the platform. Familiar APIs and observable behavior
are the starting point. The author reaffirmed on 2026-09-27 that .NET is the ergonomic
target, not an exact API-shape requirement. Learn from other frameworks and let
neoCLR’s UTF-8 and text model inform fundamental API contracts.

On 2026-10-10, the author sharpened the default: “We should stick to .NET structure
and semantics unless we have good reason to diverge”. Adopt familiar .NET API
organization, naming, defaults and observable behavior when they fit. A deliberate
variation needs a concrete platform or user benefit, explicit costs and validation;
an unimplemented feature is a gap rather than a reason to invent different semantics.
This updates the default without requiring identical CLR internals or undoing justified
UTF-8 platform choices. Preserve earlier decisions and reassess them when affected.

On 2026-10-20 the author explicitly extended this default to runtime execution and
ABIs. Compare calling conventions, argument/return passing, value layout, dispatch
and interop against the corresponding .NET/CLR contract before choosing a different
one. Identify the relevant target/platform and implementation layer; this direction
is not a claim of existing .NET binary compatibility. Existing neoCLR-specific ABI
choices remain subject to review, with reasons, costs, validation and migration
recorded for retained differences. No wholesale ABI replacement is implied by this
policy update; assess affected contracts as part of their implementation work.
The author clarified that records should remain ordinary classes/structs with
compiler-generated semantics and interfaces; do not introduce a separate record
runtime representation or ABI to compensate for incomplete compiler synthesis.

Different syntax or internals are permitted, but difference
alone is not an improvement. More runtime machinery is an option to evaluate, not
an automatic preference. Historical .NET/CLR compatibility accommodations are
context, not constraints: neoCLR may choose a more consistent contract across its
runtime and frontends without reproducing those workarounds.

Apply this process to new work and to changes in existing memory, arrays, collections,
reflection, dispatch, debugger and frontend contracts. Do not treat an implemented preview
choice as settled merely because it exists. Keep published history frozen; describe
reassessment and migration in current design documents and Unreleased entries.

## Choose runtime services or foreign libraries deliberately

Author direction, 2026-10-12: for features that need implementation outside managed
code, explicitly consider both a runtime-owned service invoked through InternalCall
and a separate native library invoked through the foreign-function interface.
Neither placement is the default simply because the implementation is native.
Keep ordinary managed implementations in the comparison where they meet the need.

InternalCall is appropriate to consider when the operation needs privileged access
to runtime invariants: allocation and GC, managed object representation, execution
state, thread attachment or tightly coordinated runtime lifecycle. Account for the
cost of enlarging the trusted runtime, adding bindings across execution modes, and
coupling implementation changes to runtime releases.

A separate native library is appropriate to consider when a feature can expose a
well-defined ABI with explicit ownership and error contracts. Evaluate independent
packaging/versioning, reuse, replacement and optional deployment against the costs
of library resolution, ABI compatibility, marshalling, callbacks, lifetime management
and target distribution. Foreign code may itself be managed by another runtime;
the distinction concerns implementation ownership and guarantees, not its language.

For each decision, record:

- Required runtime privileges and whether a stable ABI can avoid exposing runtime
  internals; identify GC roots, thread entry, callback and shutdown obligations.
- Public managed API and behavior, including errors and cancellation, independently
  of which implementation provider supplies them. A safe public wrapper must uphold
  its guarantees whichever provider it uses.
- Packaging, version compatibility, optional dependencies, supported platforms and
  interpreter/JIT/AOT availability. Current FFI gaps count as implementation work,
  not evidence that a feature must permanently live inside the runtime.
- Boundary costs and relevant measurements when performance motivates placement;
  neither InternalCall nor a foreign library is assumed faster without evidence.
- The .NET API/runtime/library baseline, alternatives, selected tradeoffs and focused
  validation, following the research requirements below. Keep provisional placement
  and the conditions for reconsidering it explicit.

This is an architectural review criterion, not a mandate to move existing services
or introduce a new registration API. A runtime-owned adapter may delegate to a
separate native component when that separation is justified. See the existing
[provider distinction](native-interop.md#runtime-internal-calls-and-implementation-options)
and [trust/ownership contract](design/native-import-export-contract.md#trust-and-ownership-across-the-boundary).

## Value contracts without unnecessary materialization

Author direction, 2026-10-23: value objects are consumer-facing contracts that
establish well-formed inputs and provide structure. Their presence in an API does
not require internal HTTP, URI, path or other processing to allocate equivalent
wrappers around every primitive value. Prefer validated primitive representations
internally where a structured object adds no value. Materialize lazily when a
consumer requests structure, retain values already supplied by callers, and cache
stable formatting or a bounded known-value catalog when beneficial.

The author cited the observed HTTP server cost of storing URI value objects as a
concrete motivation: requests should retain already known URI component strings
and project a Uri on demand. Construct from those validated components where
possible, instead of formatting and reparsing an equivalent URI. A contract value
may be a class, struct or record of either kind; choose representation separately
from materialization timing and account for copying and ownership as well as GC.
This discussion reports the prior observation, not a new benchmark measurement.
The author clarified that multiple allocations inside a value object are permitted:
there is no fixed allocation ceiling. Optimize avoidable work and copies based on
measurement, while allowing useful component storage and ordinary GC costs.

Keep validation and observable semantics consistent between the primitive and
structured paths. Account for ownership and lifetime when borrowing primitives;
avoid unbounded interning of external input. Measure allocations and performance
for the actual hot path, including conversions introduced by the measurement
itself. This direction does not authorize weakening contracts or replacing every
existing value object with a primitive-only public API.

## Required design comparison

For each substantive slice, record these points in its design document. A small
refinement may use a short paragraph; a cross-cutting runtime change needs alternatives
and executable evidence. Routine fixes may link the existing comparison and explain
why the fix restores that contract rather than repeat the research.

1. **Problem and scenario.** Give a concrete program or API need. Distinguish a
   missing neoCLR implementation from a limitation of .NET itself.
2. **.NET baseline.** Separate C# language rules, emitted metadata/IL, CLI requirements,
   current runtime implementation, and library policy. Cite primary sources and
   identify the relevant version, commit or specification section and retrieval date.
   Mark proposals as proposals; do not confuse them with shipped behavior.
3. **Alternatives.** Compare adopting the .NET behavior, adapting it to neoCLR's
   value/reference model, and adding a new mechanism where justified. Evaluate
   language, metadata, runtime and library placement independently. Include relevant
   other platforms, .NET API feedback and alternative .NET ecosystem projects using
   the review scope below; keep the .NET baseline explicit.
4. **Benefits and costs.** Consider correctness, enforceability across frontends,
   usability, implementation complexity, allocation/copying cost, GC/lifetime effects,
   interop, reflection, debugging and eventual JIT/AOT support. Benchmark performance
   claims; the absence of legacy requirements does not demonstrate a speedup.
5. **Decision and uncertainty.** State the preferred contract, why alternatives fit
   less well, deliberate deviations and their migration impact. Label evidence gaps,
   experiments and assumptions. A preferred design can remain provisional.
6. **Validation.** Specify positive and negative source/IL/artifact scenarios, including
   attempts to bypass compiler-only restrictions. Where behavior is meant to match,
   use small .NET comparison programs on a recorded toolchain. Check runtime-source
   claims against a pinned revision when relevant.

## Scale the decision to its consequences

Not everything is worth changing. Keeping the existing behavior is an explicit
alternative, including when evidence of a better design is weak.

Small, reversible prototypes may begin immediately to answer a bounded question.
State the hypothesis, comparison and success/failure criteria; label the implementation
experimental and avoid presenting its API as a settled platform contract. Experiments
are a way to obtain input, not a shortcut around evaluating the result.

Before adopting a larger decision affecting type identity, metadata, lifetime safety,
GC, cross-language behavior or widely used APIs, document the problem, existing
behavior, outside experience, alternatives and compatibility costs. Use a reduced
prototype or comparison program where it can answer the uncertainty. Identify remaining
unknowns and whether they are acceptable; do not silently turn an experiment into policy.

Before production implementation of a selected contract, resolve its essential safety
and interoperability questions. Research depth follows consequences and uncertainty,
not a fixed document size or an approval ritual. This does not add a permission gate
or require external feedback to arrive before any useful work can proceed. Reopen
choices when tests, measurements or better evidence contradict them.

## Broader API review scope (2026-09-13)

The author directs API reviews to consider other platforms, discussion of .NET APIs,
and independent projects that offer different approaches within the .NET ecosystem.
The purpose is to learn from demonstrated designs and user experience, not restrict
research to official .NET documentation or assume a replacement library is better.

For a new API family or substantive redesign, gather four kinds of evidence:

1. **Current .NET contract and rationale.** Read documentation, implementation and
   relevant API-review decisions. Check whether the criticized behavior still applies
   to the current baseline rather than only .NET Framework or an older release.
2. **Experience and criticism.** Inspect relevant issues, discussions, review comments
   and first-hand experience reports. Capture the concrete scenario, competing views,
   maintainer response and current resolution. Cite the specific comment when relying
   on it. Attribute opinions; popularity, an open issue or a rejected proposal does
   not establish a defect or the best solution.
3. **Other platforms.** Compare at least one relevant design using its own primary
   documentation and examples. Separate language conveniences from runtime guarantees
   and library policy. Account for ownership, GC, Unicode and concurrency assumptions
   that may not transfer to a managed CLI-like platform.
4. **Alternative .NET projects.** Where relevant, inspect a project that addresses
   the same problem: its rationale, public contract, tests, limitations and migration
   story. Identify whether it solves the issue entirely as a library, needs language
   support or exposes an actual runtime limitation. A library solution is evidence
   against adding unnecessary runtime machinery.

Select sources by the problem, not by a fixed list of fashionable platforms. If no
useful comparison or independent project is found, record that gap instead of inventing
an analogue. Routine fixes can reuse an existing review; this is not an exhaustive
survey requirement for every overload or maintenance commit.

Record a compact comparison in the feature document: source/version/date, status
(shipped, proposal, rejected, superseded or opinion), concrete benefit, cost, portability
to neoCLR and an example/test needed to validate the claim. Include counterevidence
and reasons to keep .NET behavior. Verify performance claims with comparable workloads;
repository marketing and anecdotes are hypotheses until checked.

Initial entry points checked 2026-09-13, not completed evaluations:

- [.NET API review process](https://github.com/dotnet/runtime/blob/main/docs/project/api-review-process.md)
  provides context for proposals and design discussions. Read the relevant issue's
  history and final outcome before treating a review comment as platform policy.
- [Noda Time 3.2 design philosophy](https://nodatime.org/3.2.x/userguide/design)
  is a useful date/time comparison: explicit domain distinctions, injectable clocks
  and documented tradeoffs. Compare it with modern .NET TimeProvider and our bounded
  date/time needs; this is not a decision to copy Noda Time's whole API.
- [Rust's str documentation](https://doc.rust-lang.org/std/str/)
  is an entry point for valid UTF-8 and decoding contracts. Inspect its concrete
  operations for the text review; do not import Rust's ownership model merely to
  borrow an API idea.

## Starting evidence and limits

Primary sources consulted on 2026-09-08:

- [ECMA-335 publication page](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
  identifies the CLI specification. Use the applicable partition and section for
  individual metadata/IL claims; an old CLI edition alone does not establish all
  behavior of today's .NET runtime.
- [Microsoft's readonly reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/readonly)
  distinguishes readonly fields, struct members and reference access. A readonly
  reference-type field cannot be reassigned after construction, but its target can
  remain mutable. This supports separating binding/storage restrictions from target
  access; it does not prove neoCLR's proposed runtime capability representation.
- [Microsoft's nullable reference types guide](https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/null-safety/nullable-reference-types)
  describes C# nullable reference annotations as compile-time analysis that does not
  change the runtime type. neoCLR's proposed enforced nullable-slot distinction must
  therefore be evaluated as an intentional change, not described as reproducing that
  C# feature. This observation does not describe nullable value types.

This is initial evidence, not a completed comparative study of the backlog. The
[readonly input-parameter comparison](readonly-parameters.md) adds a reproducible
.NET 10 probe and a bounded implementation decision. Broader mutability and each
later area still need their own evidence and cost assessment.

## Research tasks attached to the roadmap

| Area | Comparison to complete before settling the contract |
| --- | --- |
| Immutable bindings and readonly access | Compare C# readonly/ref rules, CLI storage/address restrictions and current runtime enforcement. Evaluate compiler-only, verifier-backed and mandatory runtime capabilities, including aliasing and defensive-copy behavior. |
| Inheritance | Compare base layout, construction, virtual slots and casts; retain the selected CLR-like type categories and assess any remaining object-hierarchy differences. |
| Nullability | Compare reference annotations, nullable value representation, generic/default initialization and reflection; test the cost and usefulness of enforced nullable storage. |
| Enums and flags | Compare underlying types, flag operations, unnamed values, conversions, formatting and reflection; justify deviations rather than redesigning familiar options APIs. |
| Generic constraints | Compare existing metadata/runtime constraints with C#-only restrictions; define how proposed address-mode, null and Void constraints differ. |
| Delegates and lambdas | Follow the [delegate direction](delegates.md): compare managed delegate invocation, capture lowering, lifetime/GC behavior and function-pointer facilities. Existing callables use delegates; the reopened function-type review compares alternatives without selecting a replacement. The first implementation uses checked binding and ordinary Invoke, with Func<Void> replacing Action. Validate later closure and multicast choices. |
| Async | Compare current .NET async implementations and relevant proposals on pinned versions; distinguish API/task behavior from state-machine or runtime-suspension mechanisms. |
| Dynamic hooks | Compare .NET dynamic binding and extensibility with ordinary virtual dispatch, then select a concrete problem that justifies runtime hooks. |
| Framework and existing features | Map each affected API to its .NET behavior and assess deliberate neoCLR differences, including copying, allocation, errors and reference contracts. |

These tasks are part of each [roadmap phase](roadmap.md), not a separate promise to
implement every candidate. Keep the [API policy](api-policy.md) and feature-specific
documents aligned with the resulting decisions.

The [readonly receiver refinement](readonly-parameters.md#readonly-instance-receivers)
reuses the input-capability evidence and compares C# readonly instance-member behavior.
It records enforced receiver metadata, exact interface matching, collection getter
migration and the continuing shallow-access and partial-verifier limits.
