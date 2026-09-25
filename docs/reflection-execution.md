# Runtime-backed reflection: construction checkpoint

Development implementation, 2026-09-25. This is the private interpreter foundation
for the [mapped JSON report](json-dom-design.md#next-investigation-one-mapped-report--2026-09-25).
It is not yet a public System.Runtime.Reflection API or a JsonSerializer mapping
feature. Introspection remains descriptive; there are no new operations on TypeInfo.

## Implemented boundary

Two private InternalCall services are admitted by exact signature:

| Service | Signature | Behavior |
| --- | --- | --- |
| neoCLR.Runtime.ReflectionConstructionCheck | (System.RuntimeTypeHandle) → Int32 | Inspect whether construction is supported; never run the constructor |
| neoCLR.Runtime.ReflectionConstruct | (System.RuntimeTypeHandle) → System.Object | Revalidate and execute the public parameterless class constructor |

Check returns 0 for support, 1 for an unbound or noncanonical identity, 2 for an
unsupported type shape, 3 for denied access and 4 for no supported parameterless
constructor. These numbers are private bridge details, not a proposed public error
union. Direct misuse of the execution service faults. A future Result-based facade
can translate validation failures; executed user-code Faults remain terminal.

The supported target is a concrete, nongeneric reference class assignable to Object
with a public parameterless IL constructor and publicly accessible containing types.
Generic classes, value types, abstract classes, interface/array/intrinsic shapes,
nonpublic construction and constructor-argument binding are outside this checkpoint.
Construction never synthesizes missing constructors or bypasses initialization.

Resolve the handle's full definition identity in the current loaded module set and
check its canonical round trip; do not bind by display name. Reuse normal access
checks, including containing-type visibility. This is not an independent-context
identity policy: RuntimeContext still represents the current loaded program. Public
provider validation and metadata-only descriptor failures belong in the next facade
slice; no arbitrary MemberInfo index is accepted here.

## Execution and lifetime

The service creates a private two-instruction interpreter adapter: ordinary `newobj`
for the resolved constructor followed by `ret`. It runs on the current invocation's
frames, heap, instruction limit, cancellation and scheduler context. Constructor
chaining, field initialization, user Faults and rooting therefore follow the same
path as direct construction. The adapter consumes a frame and instruction steps;
it is not a zero-cost reflection mechanism or a nested interpreter invocation.

Reachability reports ReflectionExecution plus type inspection, frame allocation and
managed heap services. Its static call graph does not enumerate dynamically selected
constructors. A future pruner/AOT backend must retain or explicitly close that target
set and provide an execution strategy; it must not treat the graph as complete.
The interpreter currently retains the loaded definitions.

## .NET comparison and alternatives

Use the existing [Activator.CreateInstance baseline](https://learn.microsoft.com/en-us/dotnet/api/system.activator.createinstance?view=net-10.0)
from the JSON design research reviewed 2026-09-25: creation through a Type invokes
a constructor, with broader overload/type support than this checkpoint. neoCLR
retains that basic behavior and restricts admission while proving execution and
lifetime. That restriction reduces implementation scope but excludes valid .NET
scenarios; it is not claimed as an API improvement.

Direct host allocation/field writes would be smaller but bypass constructor code
and its invariants. A nested invocation would duplicate ownership and scheduling
boundaries. Reusing the existing frame path avoids those semantic splits, at the
cost of an adapter frame and runtime resolution. Public names, Result error cases,
property access and mapping remain separate work; no emission/metadata convention
or Raven compiler setting changes here.

## Validation and next slice

`tests/reflection_construction.rs` covers actual constructor effects, serialization,
wrong signatures, unsupported/private/missing constructors, foreign type identities,
constructor Faults, frame limits, collection during construction and service analysis.
The existing constructor tests continue to exercise direct construction invariants.

Next implement checked property execution using explicit accessor metadata: preserve
setter/getter code and virtual dispatch, validate receiver/value types and visibility,
and root arguments throughout the call. Then expose the smallest Result-based Raven
extensions in System.Runtime.Reflection with API reference coverage and a compiled
consumer. Only then use those operations for the mapped JSON round trip. Field access,
coercion, private binding and arbitrary method invocation remain separate candidates.

Local validation: eight reflection-specific checks (seven integration cases plus
one identity-binding unit test) and eight existing constructor tests pass. API
snapshot validation passes unchanged. The Introspection/Web feature pages still
accurately describe reflection and object mapping as future public capabilities;
no website build or publication was run.
