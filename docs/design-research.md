# Research and decisions: improving the .NET platform and CLR

Every roadmap capability and substantive revision of an implemented feature belongs
in this discussion: what .NET/CLR already provides, what problem remains for neoCLR,
and which solution best serves the platform. Familiar APIs and observable behavior
are the starting point. Different syntax or internals are permitted, but difference
alone is not an improvement. More runtime machinery is an option to evaluate, not
an automatic preference. Historical .NET/CLR compatibility accommodations are
context, not constraints: neoCLR may choose a more consistent contract across its
runtime and frontends without reproducing those workarounds.

Apply this process to new work and to changes in existing memory, arrays, collections,
reflection, dispatch, debugger and frontend contracts. Do not treat an implemented preview
choice as settled merely because it exists. Keep published history frozen; describe
reassessment and migration in current design documents and Unreleased entries.

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
