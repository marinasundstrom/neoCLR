# Runtime reflection — development

Import `System.Runtime.Reflection.*` to execute a small set of operations on
runtime-backed `System.Introspection` descriptors. Introspection itself remains
descriptive. These provisional extensions support the next JSON object-mapping
experiment; object serialization is not implemented yet.

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
contexts, arbitrary method invocation and field access remain future work. These
extensions do not make every introspection descriptor executable.

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
