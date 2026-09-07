# Compatibility and innovation strategy

neoCLR balances two goals: improve the parts of the CLR model that constrain useful
languages, and remain recognizable enough that existing .NET knowledge transfers.

Keep familiar concepts when they still express the intended behavior: metadata
tables, type and member signatures, overload identity, visibility, property
accessors, CLI-like stack effects, and ordinary constructor/call patterns. A language
or migration tool should be able to map these concepts without inventing a second
semantic model.

Change a concept when the old rule is a fundamental limitation, such as mandatory
value/reference categories, implicit ownership, exception-only failure, or a closed
high-level memory model. The replacement must have an explicit contract and a clear
mapping to familiar metadata where possible. `Result<T,TError>`, typed pointers,
array views, companion cases, and explicit allocation are examples.

Every deliberate deviation belongs in the IL and metadata references, with its stack
effect, type rules, fault behavior, and version status documented. Temporary
compatibility forms are labeled as bootstrap and must have a removal plan. Preview
artifacts may break, but the break should be visible and mechanically diagnosable.

This keeps migration incremental: familiar code can target the stable common subset,
while language authors can opt into lower-level or more expressive neoCLR services.

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
