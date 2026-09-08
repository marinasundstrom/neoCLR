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
reflection, dispatch, debugger and Neo contracts. Do not treat an implemented preview
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
   language, metadata, runtime and library placement independently. Other languages
   can supply evidence, but do not substitute analogy for a .NET comparison.
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

Before implementation, resolve the questions that affect the slice's contract.
Research depth should follow uncertainty and consequences; this is not an extra
permission gate. Continue authorized work once the comparison supports the choice.
Reopen the decision when tests, measurements or better evidence contradict it.

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
| Inheritance | Compare base layout, construction, virtual slots and casts; identify adaptations required by value-default types and optional Object ancestry. |
| Nullability | Compare reference annotations, nullable value representation, generic/default initialization and reflection; test the cost and usefulness of enforced nullable storage. |
| Enums and flags | Compare underlying types, flag operations, unnamed values, conversions, formatting and reflection; justify deviations rather than redesigning familiar options APIs. |
| Generic constraints | Compare existing metadata/runtime constraints with C#-only restrictions; define how proposed address-mode, null and Void constraints differ. |
| Delegates and lambdas | Compare managed delegate invocation, capture lowering, lifetime/GC behavior and function-pointer facilities; determine which mechanisms neoCLR actually needs. |
| Async | Compare current .NET async implementations and relevant proposals on pinned versions; distinguish API/task behavior from state-machine or runtime-suspension mechanisms. |
| Dynamic hooks | Compare .NET dynamic binding and extensibility with ordinary virtual dispatch, then select a concrete problem that justifies runtime hooks. |
| Framework and existing features | Map each affected API to its .NET behavior and assess deliberate neoCLR differences, including copying, allocation, errors and reference contracts. |

These tasks are part of each [roadmap phase](roadmap.md), not a separate promise to
implement every candidate. Keep the [API policy](api-policy.md) and feature-specific
documents aligned with the resulting decisions.
