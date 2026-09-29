# Function types

## Delegates evolved

Function types evolve the familiar delegate model: a callable object holds a target
method or function, retains its receiver or captured environment when needed, and
calls it through Invoke. The signature itself becomes a structural type.

For a signature such as `(int) -> int`, matching parameter types, return type and
reference-passing contracts mean the same Function type. Callers can share that
contract directly, without declaring a named delegate type. A Function object is
an instance of the signature, bound to a particular target.

**Development API:** available in development builds after Preview 11, using the
matching runtime and Raven toolchain.

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
names and namespaces. Collections can narrow descriptor results with
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
