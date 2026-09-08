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

## Next bounded design slice

1. Define typed static and bound-instance delegates and signature compatibility,
   including readonly, output and explicit managed-reference contracts. Generic
   functions must be closed before binding to an invocable delegate.
2. Define how a delegate retains a managed target. Heap targets must remain GC roots;
   frame targets must not escape their owners. Do not silently copy or promote a
   referenced value to extend its lifetime.
3. Add checked construction and invocation, introspection/debugger information, and
   one library callback consumer with a small Neo projection.
4. Then specify lambda capture modes and escaping environments. Compare equality,
   combining/removing invocation lists, multicast return values and exception behavior
   with .NET before deciding what to adopt or deliberately change.

Exact Delegate/MulticastDelegate hierarchy, variance, invocation opcodes, open-instance
binding and capture syntax remain undecided. Pinning and native callback trampolines
are separate interop work. A pinned .NET behavioral probe and positive/negative
IL/artifact lifetime tests are required before settling these contracts.
