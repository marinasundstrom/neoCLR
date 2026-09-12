# Propagation runtime-library contract

Recorded 2026-09-12. This is the first runtime-library slice for Raven-target propagation,
not support for Raven's `?` operator in the importer yet.

## Contract

`System.Propagatable<TSelf,TOutput,TResidual>` declares two readonly managed-receiver
operations:

- `TryGetOutput(out(true) TOutput& output) -> Boolean`
- `TryGetResidual(out(true) TResidual& residual) -> Boolean`

A true result initializes the destination with the selected channel. A false result
leaves it untouched and does not establish initialization. This reuses neoCLR's
[conditional output contract](union-convention.md), avoiding a fabricated default value
for an inactive payload. The carrier is not consumed or mutated by extraction.

Result<T,E> implements Propagatable<Result<T,E>,T,E>. Option<T> implements
Propagatable<Option<T>,T,Void>. Option's absence channel contains Void; Some<Void> is
still successful output, distinguishable from None. A well-formed carrier has exactly
one channel. Terminal faults are not residual values and are not translated into Result.

Each carrier provides a static `FromResidual` factory: Result<T,E> takes E and returns
Error; Option<T> takes Void and returns None. A compatible residual can reconstruct a
carrier with a different output type. It must not be silently converted to an unrelated
residual type. Error objects/payloads retain their existing copy/reference semantics.

## Enforcement boundary and .NET comparison

Raven currently declares System.IPropagatable<TSelf,TOutput,TResidual> with these two
instance methods and a static factory. Its binder checks the self argument and resolves
concrete carrier methods. NeoCLR uses its normal prefix-free interface name.

Modern C# supports static abstract interface members, including self-typed generic
contracts; see [Microsoft's static interface member tutorial](https://learn.microsoft.com/en-us/dotnet/csharp/advanced-topics/interface-implementation/static-virtual-interface-members)
(primary source consulted 2026-09-12). NeoCLR currently validates instance interface
members only. Consequently FromResidual is a carrier method **outside this interface**,
not an implemented static abstract interface member. TSelf describes the carrier for
compiler protocol selection; runtime interface conformance alone does not enforce that
TSelf equals the implementing type or require the static factory.

This provisional split reuses current dispatch and needs no new opcode, but its cost is
weaker interface-level enforcement. A targeting compiler/importer must check the complete
self/output/residual/factory shape before admitting propagation. Future static abstract
interface support could encode that requirement directly. Do not treat a marker interface
alone as proof that an arbitrary carrier is a valid propagation target.

NeoCLR's true-only initialization contract also differs from C#'s ordinary out parameter
initialization on every normal return. Raven metadata/lowering must account for that
existing runtime distinction instead of assuming a false extraction initialized storage.
No Raven .NET behavior is changed in this slice.

## Validation and next slice

`cargo test --test propagation` covers interface dispatch, both Result channels,
non-overwriting misses, Option<Void> success/absence, compatible residual reconstruction
with a different output type, artifact round-trip, and rejection of unproven output reads
by both verification and execution. Existing union output, library, generic-bound,
Clonable and adapted Raven collection checks also pass.

The Raven experiment now exposes matching Result declarations and selects the protocol
through target project properties. `samples/library-propagation.rvn` compiles and runs
`Result<int, OverflowError>` extraction and early return. The importer checks exact
carrier signatures and the interface shape. Adapters initialize Int32/OverflowError
out destinations on both paths to satisfy ordinary CLI out semantics, then call the
runtime's conditional extraction methods. This safe bounded defaulting is not a rule
for arbitrary payloads.

Raven's selected-target lowering omits implicit exception capture. The emitted
invalid-carrier `throw null` sentinel becomes a terminal neoCLR fault; arbitrary thrown
values and exception regions remain rejected. Carrier `initobj` scratch locals remain
uninitialized until a valid case is assigned, rather than fabricating a union case.

`verify_project.py --collections` includes the executable propagation sample alongside
existing fundamentals and stale-output rejection. `--interfaces` also checks malformed
protocol metadata and rejects throwing a non-null-sentinel value. Raven's default .NET
propagation tests remain unchanged and pass.

Option<Int32> and Result<Void,OverflowError> now execute success and early residual
return, including a discarded Void-output propagation statement. The sample workflow
uses ordinary ArrayList constructors and iteration. The [Void decision](void-semantics.md)
separates unit values from no-result calls. Signature validation rejects raw CLI VOID
in storage before Cecil member resolution. General payload support remains bounded;
this is not arbitrary union import. Defer cleanup remains separate.
