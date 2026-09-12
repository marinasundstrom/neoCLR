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

Next expose matching declarations in the experiment profile, configure Raven's target
protocol name while preserving its .NET default, and admit emitted calls with exact
contract/signature checks. Verify success, early return, incompatible residual rejection,
and Void payloads through the actual Raven program and editor workflow before marking
propagation complete in the preview acceptance matrix. Defer cleanup remains separate.
