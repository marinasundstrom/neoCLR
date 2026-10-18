# Reflection

Create objects, invoke methods, and read or write members discovered at runtime.

[Introspection](../introspection/) describes a program's types and members.
Import `System.Runtime.Reflection.*` to add execution extensions to those descriptions.
Expected validation failures are `Result` values, so callers can inspect an error or
propagate it with `?`.

<a id="development-dynamic-arrays"></a>

## Dynamic arrays

Separately compiled JSON mapping can use `ArrayReflection` to read managed vector
lengths and boxed elements, or construct a vector from a runtime-backed array descriptor.
Reference elements retain identity. These bounded operations use terminal Faults for
invalid inputs and do not support custom value elements or multidimensional arrays.
See the [API contract](/docs/reflection.html#dynamic-managed-arrays-development-2026-10-07).
Use matching development libraries.

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

Development property getters also support boxed nongeneric value records. The getter
borrows the rooted box, preserving getter side effects while keeping the original
unboxed value separate. Record-valued properties return boxed copies. Existing public
visibility and native retention rules apply; value-record setters remain unsupported.


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

Method and instance-member execution supports public IL methods and fields/properties on nongeneric
reference classes, with references and built-in scalar values. Constructor selection
requires exact scalar types or assignable references; it does not rank multiple matches.
There is no binder coercion, optional-argument completion, byref/out support or arbitrary value-type execution. Generic method/instance-member execution and independent execution contexts remain
future work. Static fields have the separate owner restrictions described below.

[Read the API guide](../../docs/reflection.html) ·
[Type extensions](xref:System.Runtime.Reflection.TypeReflectionExtensions) ·
[Method extensions](xref:System.Runtime.Reflection.MethodReflectionExtensions) ·
[Field extensions](xref:System.Runtime.Reflection.FieldReflectionExtensions)

Attribute inspection belongs to [Introspection](../introspection/#development-case-inspecting-route-declarations-at-startup):
MemberInfo and ParameterInfo expose constructor data without executing attribute
constructors. Read it once during startup and cache the validated mapping.


<a id="development-case-preparing-union-constructors-for-routes"></a>

## Case: preparing union constructors for routes

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
with preparation, repeated construction and rejection checks. The [attributed HTTP case](/cases/http-server/#case-attributed-item-routes)
combines this execution support with startup schema validation and cached route bindings.
See [constructor execution](xref:System.Runtime.Reflection.ConstructorReflectionExtensions)
for signatures and limitations.

<a id="development-fields-and-type-initialization"></a>

## Fields and type initialization

Field descriptions include instance/static storage and Int32 constants, with
`IsInitOnly`, `IsLiteral` and `ConstantValue`. `TypeInfo.TypeInitializer` and
`IsBeforeFieldInit` describe initialization without running it. Public static field
reflection uses normal initialization guards; readonly writes return AccessDenied.
JIT/AOT use explicitly retained metadata and operations (reflection-roots schema 7).
See the [API contract](/docs/reflection.html#fields-and-type-initializers-development)
for exact value/owner restrictions and the distinction between Result errors and
terminal initializer faults.

Development native `TypeInfo.BaseType` queries now use the interpreter's metadata
recipe for closed type tokens present in the compiled program. The reference-record
consumer verifies explicit Object ancestry across interpreter, JIT and AOT.

Development builds also preserve imported `ValueType` ancestry for application
structs and record structs, including boxed ancestry tests across interpreter, JIT
and AOT. Application enum ancestry now also retains the Enum/ValueType/Object chain, with
exact boxed enum identity across all three modes. Intrinsic primitive and core-enum ancestry are also validated against a rebuilt
source library set; ordinary local value-record authoring remains separate work.
