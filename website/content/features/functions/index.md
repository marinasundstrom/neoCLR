# Function types and objects

A Function type describes a callable signature. A Function object binds a particular
method or function to that signature, retaining its receiver or captured environment
when needed. The same signature means the same structural Function type.

**Development API:** requires the matching development runtime and Raven toolchain.
This model replaces delegates in development artifacts and is not part of the
published Preview 11 contract.

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
It retains method identity, not the bound receiver. This transitional property will
continue to return MethodInfo until a broader function-info model is designed.

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

## Relationship to delegates

The binding mechanism remains similar to a single-target delegate. The difference
is structural signature identity: callers do not need to share a named delegate
declaration. .NET delegates are also represented in its type system; neoCLR makes
the signature itself the Function type. This offers signature-based compatibility,
while giving up the existing Delegate class hierarchy and its common callable contract.

Function types inherit Object while remaining structural. Object views support
GetType, value Equals/GetHashCode, and casts back to the matching signature.
ToString displays the bound target's qualified name and closed signature without
printing captured values. Separate equal bindings have distinct reference identity;
copies retain it. This does not imply Object inheritance for every structural family.

There is no multicast invocation list. Dynamic execution of synthetic reflection
descriptors is not supported. Typed invocation and property access are supported.
Null access raises NullReference. The current balance of structural types and delegate-like binding
is provisional.

## Open direction

A common base type or interface could express “accept any function/method,” useful
for inferred request handlers. A future FunctionInfo interface might cover methods
and module functions; it is not introduced now. We are also considering whether a
fuller functional object model should represent the function itself beyond its
binding and signature, and how introspection should construct structural types.

Named nominal Function types may eventually inherit explicitly eligible Function
shapes. Matching signatures would not make distinct nominal Function types directly
interchangeable; structural types would not become generally inheritable.
See the [open questions](../../proposals/#function-types-and-nominal-type-information).
