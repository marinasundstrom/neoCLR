# Compatibility and innovation strategy

neoCLR balances two goals: improve the parts of the CLR model that constrain useful
languages, and remain recognizable enough that existing .NET knowledge transfers.

Keep familiar concepts when they still express the intended behavior: metadata
tables, type and member signatures, overload identity, visibility, property
accessors, CLI-like stack effects, and ordinary constructor/call patterns. A language
or migration tool should be able to map these concepts without inventing a second
semantic model.

The [current platform direction](platform-direction.md) retains .NET's useful value
and reference type categories. Change a contract only when a concrete scenario and
comparison justify its costs. Result/Option APIs, generic Void and invariant mutable
arrays are deliberate differences. Modern UTF-8 text APIs, bounded memory views and
nullability metadata are areas for evaluation, not reasons to replace the type model.

Every deliberate deviation belongs in the IL and metadata references, with its stack
effect, type rules, fault behavior, and version status documented. Temporary
compatibility forms are labeled as bootstrap and must have a removal plan. Preview
artifacts may break, but the break should be visible and mechanically diagnosable.

This keeps migration incremental: familiar code can target the stable common subset,
while language authors can opt into lower-level or more expressive neoCLR services.

## Planned Raven integration experiment

The [Raven target experiment](raven-target-experiment.md), planned 2026-09-12, tests
an existing compiler against neoCLR's metadata, binary artifact path and own System
library. Begin by inspecting Raven's emitter before selecting an encoding. This is
planned source/compiler integration with deliberate migration differences, especially
Result-based errors; it is not a promise to run unchanged .NET assemblies.

## Typed operations and deliberate low-level access

neoCLR retains typed signatures, values and ordinary checked operations alongside
explicit pointer reinterpretation and native calls. Type erasure through Void* is
intentional: the caller supplies the missing payload-type and lifetime contract.
Allocation tracking can diagnose some invalid accesses without proving that an
arbitrary pointer cast describes the stored value. Do not claim universal type or
memory safety across these operations.

Higher-level languages may restrict those operations, distinguish safe and unsafe
code, or project managed ownership services. Those language policies must not become
unavoidable VM restrictions that eliminate deliberate low-level access. This does
not require removing the interpreter's existing checks or promising unrestricted
foreign-memory access before its native boundary is implemented.

System.Value is scheduled for [retirement](value-storage.md#retirement-decision)
after its library callers have ordinary storage and explicit-reference replacements.
A permanent runtime-owned arbitrary-value container is not the intended foundation.
