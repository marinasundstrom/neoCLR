# Type relationships

Author-directed development, 2026-10-14. TypeInfo exposes AssignableTo,
AssignableFrom, IsSubclassOf and IsInstanceOfType. The first three require a
non-null descriptor. Instance testing accepts Object? and returns false for null.

## Comparison and scope

The semantic reference is [.NET Type.IsAssignableFrom](https://learn.microsoft.com/en-us/dotnet/api/system.type.isassignablefrom),
[IsSubclassOf](https://learn.microsoft.com/en-us/dotnet/api/system.type.issubclassof)
and [IsInstanceOfType](https://learn.microsoft.com/en-us/dotnet/api/system.type.isinstanceoftype).
Assignment includes identity, ancestry, implemented interfaces and boxing. It is
not a numeric or user-conversion test. Subclass is strict and excludes interfaces.

The author chose AssignableTo/AssignableFrom spellings and explicitly required
non-null type arguments. Unlike .NET's null-to-false assignment convenience, Raven
can reject absent operands at the call site. The cost is a deliberate API difference;
callers must resolve optional metadata before comparing it. Object instance testing
retains meaningful null-to-false semantics.

The runtime currently supports invariant closed generic types and arrays. Variance,
array covariance and generic-parameter constraint queries remain gaps relative to
.NET, not a new permanent variance contract. Function identity is structural;
Function values can be used as Object. The implicit Object root includes boxed
values without requiring a synthetic System.ValueType descriptor.

## Implementation boundary

Managed descriptors extract their opaque handles; the runtime owns relationship
semantics. Interpreter services and native-metadata lowering share
`src/type_relationships.rs`. JIT and AOT derive a closed relation over admitted
type-token producers using original semantic identities before private backend
renaming. No hierarchy descriptor arrays are allocated to answer a query. Code size
and native compilation work grow with the admitted types and true relationships;
large images may eventually justify compact runtime tables.

ReflectionAssignable reuses the existing service and now handles exact value-type
identity and boxing consistently. ReflectionSubclass is a new internal service.
These are typed runtime-service calls in native metadata, not an encoded public CLI
type. The aggregate API assembly is only a documentation/compiler bridge. Existing
compiler typeof/runtime-context contracts remain unchanged. No additional reflection
execution rights or metadata discovery roots are granted by these predicates.

## Validation limits

The Preview 15 Raven SDK used for the managed suite rejects a structural Function
`typeof` token and direct callback-to-Object conversion in the additional consumer
fixture (NEOMETA003). Function relationship rules are covered directly in runtime
unit tests; this does not claim that those two compiler paths are fixed. All three
managed descriptor classes implement the public predicates. The ordinary Function
compiler/backend tests remain separate from this API change.
