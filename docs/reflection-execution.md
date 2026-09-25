# Runtime-backed reflection: construction and property checkpoints

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
slice; property indices are checked against the resolved declaring type.

## Property execution checkpoint

Four additional private services use the same canonical type identity checks:

| Service (neoCLR.Runtime prefix) | Parameters | Return |
| --- | --- | --- |
| ReflectionPropertyGetCheck | RuntimeTypeHandle, Int32, Object | Int32 |
| ReflectionPropertySetCheck | RuntimeTypeHandle, Int32, Object, Object | Int32 |
| ReflectionPropertyGet | RuntimeTypeHandle, Int32, Object | Object |
| ReflectionPropertySet | RuntimeTypeHandle, Int32, Object, Object | inhabited Void |

The integer argument is the declared property's metadata index, matching
PropertyInfo.DefinitionIndex, not a position in a filtered query. Check returns
0 for support, 1 for unbound identity, 2 for unsupported shape, 3 for access denied,
4 for invalid property, 5 for missing accessor, 6 for invalid receiver and 7 for
invalid value. Checking never invokes accessor code. Execution revalidates;
invalid direct use faults. These codes are private, not public error contracts.

Support is limited to nongeneric reference-class owners, public instance IL accessors
and nonindexed properties. Abstract owners are allowed with compatible derived
receivers. The adapter invokes the actual accessor through ordinary virtual dispatch;
it does not write backing fields. Static/native accessors and value-type receivers
remain outside this checkpoint.

Property values can be references or built-in numeric, Boolean and Char scalars.
Get boxes scalars; Set requires their exact boxed type. Reference assignment accepts
compatible values and null. Null scalar values and numeric coercion are rejected.
This does not establish JSON null/Option mapping or nullable-annotation policy.
Custom structs, enums and union value payloads remain unsupported.

Compared with .NET [PropertyInfo.GetValue](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.propertyinfo.getvalue?view=net-10.0)
and [SetValue](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.propertyinfo.setvalue?view=net-10.0)
(reviewed 2026-09-25), this preserves accessor execution and object-shaped values but
omits binder/indexer support. In particular, .NET permits null to set a value-type
property to its default; this prototype rejects it. Exact values keep the initial
contract explicit and small, at the cost of that compatibility. Accessor Faults stay
terminal instead of being wrapped as TargetInvocationException. User setter side
effects are not transactional.

Adapters use normal frames and GC roots for receivers, arguments and boxed results.
They consume frame/instruction budget and may allocate. Public descriptor/provider
validation remains necessary in the facade; this is not a public arbitrary-index API.

## Execution and lifetime

The service creates a private two-instruction interpreter adapter: ordinary `newobj`
for the resolved constructor followed by `ret`. It runs on the current invocation's
frames, heap, instruction limit, cancellation and scheduler context. Constructor
chaining, field initialization, user Faults and rooting therefore follow the same
path as direct construction. The adapter consumes a frame and instruction steps;
it is not a zero-cost reflection mechanism or a nested interpreter invocation.

Reachability reports ReflectionExecution plus type inspection, frame allocation and
managed heap services. Its static call graph does not enumerate dynamically selected
constructors or accessors. A future pruner/AOT backend must retain or explicitly close that target
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
and mapping remain separate work; no emission/metadata convention
or Raven compiler setting changes here.

## Validation and next slice

`tests/reflection_construction.rs` covers actual constructor effects, serialization,
wrong signatures, unsupported/private/missing constructors, foreign type identities,
constructor Faults, frame limits, collection during construction and service analysis.
The existing constructor tests continue to exercise direct construction invariants.

`tests/reflection_properties.rs` adds eight passing cases covering accessor effects,
virtual dispatch, reference/null round trips, exact scalar boxing, receiver/value
rejection, missing/private/indexed/static accessors, GC during a setter, terminal
accessor Faults, frame limits and service signature checks. The seven construction
integration cases and identity-binding unit test pass after sharing identity resolution.

Next expose the smallest Result-based Raven extensions in System.Runtime.Reflection
with API reference coverage and a compiled consumer. Only then use those operations
for the mapped JSON round trip. Field access, coercion, private binding and arbitrary
method invocation remain separate candidates.

The Introspection/Web feature pages still accurately describe reflection and object
mapping as future public capabilities. This private checkpoint changes no public
Raven API or reference snapshot; no website build or publication was run.
