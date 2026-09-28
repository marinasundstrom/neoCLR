<a id="runtime-reflection-development"></a>

# Runtime reflection

Import `System.Runtime.Reflection.*` to execute a small set of operations on
runtime-backed `System.Introspection` descriptors. Introspection itself remains
descriptive. These provisional extensions also support the existing bounded JSON object mapper.
See the separate [Reflection feature page](/features/reflection/) for a walkthrough.

- `TypeInfo.CreateInstance() -> Result<Object, ReflectionError>` invokes a public
  parameterless constructor on a concrete, nongeneric reference class.
- `PropertyInfo.GetValue(receiver: Object?) -> Result<Object?, ReflectionError>`
  invokes a public instance getter and boxes scalar values.
- `PropertyInfo.SetValue(receiver: Object?, value: Object?) -> Result<(), ReflectionError>`
  invokes a public instance setter. It returns completion after the setter finishes.

Use Result propagation when the caller should return the same expected failure.
The [Type extensions](xref:System.Runtime.Reflection.TypeReflectionExtensions),
[Property extensions](xref:System.Runtime.Reflection.PropertyReflectionExtensions)
and [ReflectionError](xref:System.Runtime.Reflection.ReflectionError) document the
individual contracts.

## Values and execution

Properties must be nonindexed and declared on nongeneric reference classes.
Compatible derived receivers are supported, and ordinary virtual dispatch selects
the implementation. Getters and setters execute user code; reflection does not
read or overwrite a backing field directly.

Reference values accept compatible objects or null. Scalar values require their
exact boxed built-in type. There is no numeric coercion, null-to-default conversion,
custom struct/enum/union value mapping, private binding or static-property support.
These limits are narrower than .NET PropertyInfo's binder and indexer overloads.
Null is represented by a nullable object result because this API observes existing
object storage; it does not prescribe how application or JSON models express absence.

Metadata that is not backed by the current runtime returns UnboundMetadata. Unsupported
shapes, access restrictions, missing constructors/accessors and invalid receiver/value
arguments return explicit error cases. Failures raised inside a constructor or accessor
remain terminal runtime Faults, including allocation and execution limits. Setters may
have side effects before a fault; operations are not transactional.

Only the current loaded program is supported. Dynamic loading, separate execution
contexts, generic-class invocation and static-field access remain future work. These
extensions do not make every introspection descriptor executable.

Development FunctionTypeInfo.InvokeMethod and TypeInfo.GetMethods expose a synthesized
Function Invoke descriptor. It has no declaration index, token or module; optional
properties return None. MethodReflectionExtensions.Invoke returns UnboundMetadata
for it. Use typed Function invocation to execute a binding. Declared MethodInfo
DefinitionIndex and MemberInfo/ParameterInfo metadata now require Option matching.

## Imported artifact access information

The Rust host's `metadata_origin::MetadataOrigin` adds optional `publicly_visible`
for type origins and `member_access` for method origins. The former includes all
containing types. `SourceAccess` preserves Public, Private, Assembly, Family,
FamilyOrAssembly, FamilyAndAssembly and CompilerControlled. Fields on the wrong
kind of origin are rejected. Reflection requires explicit public source access on
imported definitions, in addition to runtime checks; older origins lacking these
fields are denied. Definitions without source origins use ordinary runtime access.
Rust callers constructing MetadataOrigin literals must supply the new optional fields.
Newly imported artifacts need the matching runtime. These host/artifact contracts
are not additional guest APIs or permissions to bypass normal runtime access checks.

## Constructor arguments, methods and fields (development)

[ConstructorInfo](xref:System.Introspection.ConstructorInfo) describes a constructor
through MemberInfo identity, accessibility and GetParameters; GetConstructors defaults
to public instance declarations. Constructors are not inherited or included in GetMethods.
Adding ConstructorInfo extends the closed MemberInfo family: update exhaustive matches.

[TypeReflectionExtensions](xref:System.Runtime.Reflection.TypeReflectionExtensions)
adds `CreateInstance(params Object?[])` and `CreateInstance<T>(params Object?[])` returning
Result<Object, ReflectionError> and Result<T, ReflectionError>. The described type is
constructed; T is a checked result view, validated before invoking user code.

[MethodReflectionExtensions](xref:System.Runtime.Reflection.MethodReflectionExtensions)
adds `Invoke(Object? receiver, params Object?[] arguments) -> Result<Object?, ReflectionError>`.
Use null for static methods and a compatible non-null object for instance methods.
Void-returning methods produce a successful null. Virtual calls dispatch normally.

[FieldReflectionExtensions](xref:System.Runtime.Reflection.FieldReflectionExtensions)
adds `GetValue(Object?) -> Result<Object?, ReflectionError>` and
`SetValue(Object?, Object?) -> Result<(), ReflectionError>`. Fields must be public
instance fields; source read-only fields cannot be assigned. Private helper visibility
does not grant permission. New origins retain field_access and field_readonly arrays;
old imported fields without those admission flags cannot execute through reflection.

Arguments require exact boxed built-in scalars or assignable references/null. No numeric
coercion, optional defaults, byref/out, generic-class execution, custom value payloads or
params expansion of the selected target is provided. Multiple compatible constructors
return AmbiguousConstructor; none returns MissingConstructor. Invalid method arguments
return InvalidArguments. Incompatible T returns InvalidResultType. UnsupportedMethod
and UnsupportedField distinguish unsupported metadata shapes. User Faults stay terminal.

## Retained constructor invocation (development)

`ConstructorReflectionExtensions.Invoke(self: ConstructorInfo, params arguments: Object?[])
-> Result<Object, ReflectionError>` invokes the selected constructor directly.
Import `System.Runtime.Reflection.*` and call `constructor.Invoke(arguments)`.
Keep the descriptor after startup discovery to avoid overload selection on each call.

This extension supports concrete nongeneric reference classes and value records,
including admitted standard union cases and carriers. Arguments accept exact boxed
built-in scalars, exact boxed nongeneric value records, or assignable references/null.
Values are boxed on return. Construct a union case first, then pass that boxed case
to the selected carrier constructor. No implicit boxed case-to-carrier conversion,
coercion, enum construction, generic value construction, optional defaults, byref/out
arguments or private execution is supplied. The TypeInfo.CreateInstance overloads
retain their existing reference-class-only contract.

UnboundMetadata, UnsupportedType, AccessDenied, MissingConstructor and
InvalidArguments distinguish validation failures. Argument validation precedes
execution; constructor faults remain terminal. Descriptor identity resolves within
the current loaded program; access and arguments are rechecked per invocation.
This retains constructor selection, not a compiled execution plan.
See the [tested union construction case](/features/reflection/#development-case-preparing-union-constructors-for-routes).


`ConstructorInfo.Invoke(arguments: Sequence<Object?>)` copies a dynamic argument
collection before using the same retained-constructor invocation contract. This
supports cached route bindings without exposing private array-allocation services.
The caller controls collection mutation while it is copied. It adds no overload
selection or coercion, and provider/constructor Faults remain terminal.
