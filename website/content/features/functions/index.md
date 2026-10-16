# Function types

## Delegates evolved

Function types evolve the familiar delegate model: a callable object holds a target
method or function, retains its receiver or captured environment when needed, and
calls it through Invoke. The signature itself becomes a structural type.

For a signature such as `(int) -> int`, matching parameter types, return type and
reference-passing contracts mean the same Function type. Callers can share that
contract directly, without declaring a named delegate type. A Function object is
an instance of the signature, bound to a particular target.

**Development API:** use a matching development runtime and Raven toolchain.

## Pass behavior by its signature

A callback lets a caller choose an operation. In this executable example, adding
one and doubling both fit `(int) -> int`, so the same Apply function accepts either:

```raven
{{FUNCTION_OPERATIONS}}
```

```raven
{{FUNCTION_BINDINGS}}
```

These excerpts come from the [executable callback contract](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/function-types/Callbacks.rvn);
Check is its assertion helper.

The methods differ, but their Function type is the same. Parameter names, method
names and declaring types do not contribute to signature identity. Ordered parameter
types, return type and reference-passing contracts do. Nominal component types keep
their own identities. Calling the object uses its synthesized, typed `Invoke` method.

## Value bindings and target inspection

Function objects have value equality: separately created bindings to the same
closed target and receiver compare equal. Different receivers or capture environments
remain distinct, even when their current contents match. Equality does not compare
computed results, and mutable captures remain mutable.

The read-only synthesized `Function` property returns a `MethodInfo` describing the
bound target. It is distinct from the Function type's synthesized Invoke descriptor.
The descriptor identifies the method independently of the bound receiver.

## Inspect the shape

[FunctionTypeInfo](xref:System.Introspection.FunctionTypeInfo) describes a specific
signature and directly exposes Parameters and ReturnType. InvokeMethod provides
its member view. GetMethods discovers Invoke, the Function getter and Object overrides; GetProperties
discovers the read-only Function property. Synthetic members have no declared module
or metadata token. [TypeInfo](xref:System.Introspection.TypeInfo).IsFunctionType
identifies Function shapes; IsNominalType is false.

Structural types can have members and extension members without having names.
[NominalTypeInfo](xref:System.Introspection.NominalTypeInfo) carries declaration
names and module paths. Collections can narrow descriptor results with
`module.GetTypes().OfType<NominalTypeInfo>()`.
See [introspection](../introspection/) and the [Function API reference](../../docs/functions.html)
for exact contracts and limits.

## Function objects and Object

Function types inherit Object while remaining structural. Object views support
GetType, value Equals/GetHashCode, and casts back to the matching signature.
ToString displays the bound target's qualified name and closed signature without
printing captured values. Separate equal bindings have distinct reference identity;
copies retain it.

Typed invocation and property access use the Function object's bound target.
Null member access raises NullReference.


### Function-kind constraints (development)

On Raven `codex/neoclr` with neoCLR `main`, Raven supports `where T: func` for native
generic classes and functions/methods. It restricts a generic argument to a structural function signature;
nominal delegates and objects with `Invoke` do not qualify. The constraint does not
supply an invocation signature to the generic body. A tested function slot stores
and returns a function that the consumer invokes after its signature is concrete.

Raven distinguishes native function signatures from nominal delegate symbols,
including generic substitution and imported callback signatures. Normal Raven/.NET
mapping is unchanged, and the .NET target rejects `func` constraints. A separately
compiled provider and consumer forward a callback through a constrained generic
function and return 42 in interpreter/AOT/JIT. Signature bounds such as
`where T: () -> T2` remain a gap.
Function-list/observer APIs remain deferred; provisional dynamic invocation is described below. This is integrated development work, not a published release capability.

### Provisional dynamic invocation (development)

A native primitive now accepts a function reference and an Object argument array,
validates/converts the arguments and invokes its existing bound target. The public Raven
FunctionReflection consumer demonstrates interpreter/JIT/AOT execution. Scalar results are
boxed, reference identity is retained, and no-result calls return null. Count/type
mismatches reject before invocation; target faults propagate. Exact boxed scalar
and supported reference signatures are the initial profile; ref/out, coercion and
arbitrary value marshalling remain gaps. The provisional class-library `System.Runtime.Reflection.FunctionReflection.Invoke`
now returns `Result<Object?, ReflectionError>` for input validation. Target faults
still propagate; generic overloads remain future work. Function-list/event policy remains separate.

The same source sample checks Object.GetType against the exact structural function
type and dynamically passes a function through another function as an argument and
result. Native `typeof` tokens preserve function signatures. This qualifies runtime
type identity; it does not imply every reflective member query is supported by AOT.


On the neoCLR compiler line, Raven type patterns support function signatures:
`if obj is () -> T { }` tests a signature using an in-scope generic parameter, while
`() -> T callback => callback()` captures and invokes it in a match arm. The native
sample validates these forms in interpreter/JIT/AOT. The shared Raven parser also
supports the .NET target, using its standard delegate representation.


The retained native sample also checks that `TypeInfo.IsInstanceOfType` agrees with
function-type `is` patterns, including closed generic signatures, null, nominal
objects and mismatched signatures. Reflection uses the existing exact native
function signature relationship.


The public function invocation consumer imports the fresh class-library API and
checks wrong argument counts/types, null and non-function receivers, plus successful
null reference results. Validation errors are ordinary Result values; no event or
multicast policy is implied.
