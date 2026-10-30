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

See [the neoIL walkthrough](../neoil/) for the underlying signature, binding and invocation instructions.

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


<a id="function-kind-constraints-development"></a>

### Function-kind constraints

Raven supports `where T: func` for native generic classes and functions/methods.
Only structural function signatures satisfy the constraint; nominal delegates and
objects with an Invoke member do not. A function-kind constraint allows storage and
forwarding but does not supply a callable signature to the generic body.
Signature bounds such as `where T: () -> T2` remain unsupported.

Native signature substitution and imported callbacks retain their function types.
The .NET target keeps its standard delegate representation and rejects `func`
constraints. Function-list, event and multicast policies remain undecided.

<a id="provisional-dynamic-invocation-development"></a>

### Provisional dynamic invocation

`System.Runtime.Reflection.FunctionReflection.Invoke` accepts a function reference
and an Object argument array. It returns `Result<Object?, ReflectionError>` after
checking the receiver, argument count and supported argument types. Invalid input
is rejected before execution. Target faults remain terminal.

Scalar results are boxed, reference results retain identity, and no-result calls
return null. Exact boxed scalar and supported reference signatures work in the
interpreter, JIT and AOT. Ref/out, coercion, arbitrary value marshalling and generic
helper overloads remain unsupported. The API is provisional.

### Function type patterns

`if obj is () -> T { }` tests a signature using an in-scope generic parameter.
A match arm `() -> T callback => callback()` captures and invokes the function.
Native `typeof` and `TypeInfo.IsInstanceOfType` use the same exact signature
relationship, including closed generic arguments. Null and mismatched signatures
do not match. The shared Raven syntax also works on .NET using standard delegates.

Functions can be arguments and results of other functions. Reflective invocation
preserves their signature identity; native metadata retention still determines
which reflective member queries are available.

### Callback overload inference (development)

The neoCLR compiler compares declared structural callback signatures when generic
overloads accept the same constructed callback type. A function returning
`Task<T>` selects Task.Run's async-unwrapping overload without an explicit result
type, matching ordinary .NET delegate selection. Conflicting specificity remains
ambiguous. See the [compiler qualification](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/task-overload-20261028/README.md).

## Generic capabilities

See [Generics](../generics/) for type parameters, function constraints,
return-type projection and methods whose whole signature materializes from a
function-constrained generic argument. These are development capabilities.
