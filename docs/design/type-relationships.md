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

## Experimental composition relationship core (2026-10-14)

On `codex/structural-union-intersection`, the shared relationship core delegates
closed union/intersection assignment to the interpreter's existing composition
rules. Every union alternative must satisfy the target; every intersection target
bound must be guaranteed. Assignment to a union may select an alternative, and an
intersection source may supply a guaranteed bound. Closed generic wrappers remain
invariant. This is the bounded existing entailment proof, not a new distributive
normalizer or general structural member-shape solver.

Composition validation precedes equality and Object shortcuts, including nested
generic arguments. Invalid/value bounds and disabled experimental signatures cannot
obtain a positive result merely by comparing to themselves. Strict subclass returns
false when either operand is a direct composition: assignment guarantees do not
create a nominal base declaration. In contrast to .NET, these native structural
signatures have no CLI type declaration; the benefit is preserving their guarantees
without inventing nominal ancestry, at the cost of an experimental native contract.

This slice changes the internal relationship core only. Guest union/intersection
TypeInfo descriptors and handle construction remain gated, as do composition JIT/AOT
paths. Existing mainline nominal and Function behavior is retained.
