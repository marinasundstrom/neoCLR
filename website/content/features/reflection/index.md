# Reflection

Create objects, invoke methods, and read or write members discovered at runtime.

**Development API.** Use matching development compiler, library and runtime artifacts.
The new constructor, method and field operations are not included in Preview 10.

[Introspection](../introspection/) describes a program's types and members.
Import `System.Runtime.Reflection.*` to add execution extensions to those descriptions.
Expected validation failures are `Result` values, so callers can inspect an error or
propagate it with `?`.

## Construct and invoke

This excerpt comes from the executable reflection consumer. `Model` has a public
constructor taking an integer, an instance `Add` method and a static `Twice` method.
Typed construction checks the requested result type before executing the constructor.

```raven
{{REFLECTION_CONSTRUCTION_SAMPLE}}
```

`TypeInfo.GetConstructors()` returns descriptions of declared public instance
constructors. Each `ConstructorInfo` exposes parameters, accessibility, its declaring
type and metadata identity. Discovery does not execute the constructor.

## Read and write members

Fields access object storage. Properties execute their getter or setter, preserving
ordinary virtual dispatch and application behavior. Both use nullable object values
and exact boxed scalar types.

```raven
{{REFLECTION_FIELD_SAMPLE}}
```

The sample's `Field` and `Method` helpers select descriptors by name. The complete
[downloadable consumer](../../samples/reflection-members.rvn) includes their definitions,
imports, model and rejection checks. Follow the [development setup guide](../../try/#development)
to compile and run it.

## Explicit failures

Invalid receivers, incompatible arguments, ambiguous constructors and denied access
produce `ReflectionError` cases. Read-only field writes and private execution are denied.
An invoked method returning no value succeeds with a null object result; field writes
return unit. Faults raised by application code remain terminal, and mutations are not
rolled back.

The current scope is public IL methods and instance fields/properties on nongeneric
reference classes, with references and built-in scalar values. Constructor selection
requires exact scalar types or assignable references; it does not rank multiple matches.
There is no binder coercion, optional-argument completion, byref/out support, static field
storage or arbitrary value-type execution. Generic classes and independent execution
contexts remain future work.

[Read the API guide](../../docs/reflection.html) ·
[Type extensions](xref:System.Runtime.Reflection.TypeReflectionExtensions) ·
[Method extensions](xref:System.Runtime.Reflection.MethodReflectionExtensions) ·
[Field extensions](xref:System.Runtime.Reflection.FieldReflectionExtensions)

Development attribute inspection belongs to [Introspection](../introspection/#development-case-inspecting-route-declarations-at-startup):
MemberInfo and ParameterInfo expose constructor data without executing attribute
constructors. Read it once during startup and cache the validated mapping.


## Development case: preparing union constructors for routes

Read route attributes and select constructors at startup, then keep those descriptors
for the server lifetime. The tested `ItemRouteFactory` finds the `/items/{id}` case,
checks its Int32 payload and retains both its case and union carrier constructors.
Each request can construct an ordinary union and dispatch with `match`:

```raven
{{UNION_CONSTRUCTION_SAMPLE}}
```

`ConstructorInfo.Invoke` executes the exact retained constructor. Nongeneric value
records, including admitted union cases and carriers, are supported through this
extension. Access and argument checks run on every invocation; discovery and overload
selection do not. TypeInfo activation keeps its reference-class-only contract.

The [downloadable cases](../../samples/http-json.zip) include `union-construction`
with preparation, repeated construction and rejection checks. This supplies the
construction prerequisite; general route-schema binding remains under development.
See [constructor execution](xref:System.Runtime.Reflection.ConstructorReflectionExtensions)
for signatures and limitations.
