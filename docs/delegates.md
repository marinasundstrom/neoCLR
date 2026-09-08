# Delegate direction

Delegates are the planned runtime abstraction for typed callable values. Ordinary
functions remain executable definitions; neoCLR will not add a competing runtime
model in which every function is an object. Languages can nevertheless offer function
values, method-group conversions and lambdas by building on delegates. This is a
direction decision, not implemented delegate support.

This follows the [.NET delegate model](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/delegates/)
(primary source consulted 2026-09-08): delegates represent compatible methods,
including static and instance methods, and C# can lower lambdas into delegates.
C# syntax and closure lowering belong to the language; callable signatures, invocation
checks and managed target lifetime belong in the platform. Familiarity does not require
copying every current CLR implementation detail.

A separate universal function-object representation would overlap with this mechanism
and add interoperability choices for languages and libraries. Delegates give them a
shared callback contract. The tradeoff is committing to a common callable abstraction;
we still need to specify its representation and measure allocation/invocation costs.
This decision does not imply that lambdas always allocate, or that delegates require
unmanaged function pointers.

## Contract checkpoint and next slice

The [typed delegate contract and runtime audit](delegate-contract.md) now include a
pinned .NET probe and executable neoCLR interface-adapter comparisons. Guest delegate
support remains unimplemented. The preferred first scope is static/closed generic
functions and heap-backed managed receivers, with exact invocation contracts and no
implicit receiver copying or promotion. Frame values remain usable as ordinary
reference arguments; capturing them requires later aggregate lifetime work.

Next implement checked binding and invocation, GC tracing, closed reachability,
debugger information and one library consumer with a small Neo projection. Then
address scoped captures and language lambdas. The contract records acceptance checks
and remaining equality, multicast, reflection, hierarchy and instruction choices.
Native callback trampolines remain separate interop work.
