# Custom-attribute metadata

The original subset supports parameterless markers on type and method/function
definitions. Development adds String (including null), Int32 and Boolean constructor
arguments and scoped imported member/parameter targets. This provides metadata for library conventions without special runtime
type categories. Attributes are data; assembling, loading, linking, and running an
annotated method do not instantiate attributes or execute their constructors.

## Assembly and metadata

```text
.type MarkerAttribute
    .method instance .ctor() -> Void
        ldvoid
        ret
    .end
.end

.type Box<T>
    .custom instance MarkerAttribute::.ctor()
    .field Value T
    .method instance Get() -> T
        .custom instance MarkerAttribute::.ctor()
        ldarg this
        ldfld Box<T>::Value
        ret
    .end
.end
```

`.custom` inside a type body attaches to that type, including when placed after a
field declaration. Inside a method or free function it attaches to that function
and must precede instructions and labels. Development `.custom token N ...` selects a retained field, property or parameter
source token belonging to the enclosing definition. Module, return-value and
generic-parameter targets remain unsupported. Attribute type names are exact;
there is no automatic Attribute suffix lookup.

TypeDef and Function each have an optional `custom_attributes` list. Each entry
contains a `constructor` FunctionRef with explicit owner, instance flag, name, and
parameter signature. The enclosing definition supplies the parent identity.
Empty lists are omitted from serialized modules, so old modules remain unchanged.
Older readers reject nonempty lists rather than silently ignoring their metadata.
There is no binary CLI custom-attribute blob writer yet.

This follows the constructor-reference direction of
[.NET custom-attribute metadata](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadata.customattribute?view=net-10.0).
Development argument data follows the constructor signature:

```text
.custom instance RoutePatternAttribute::.ctor(String) = [{"String":"/items/{id}"}]
```

Int32 and Boolean use `{"Int32":42}` and `{"Boolean":true}`; a null string uses
`{"String":null}`. Old parameterless syntax is unchanged. The optional arguments
and target_token JSON members default to empty/None. New data requires an updated
reader. No named arguments, raw CLI blob writer or implicit instantiation is added.

## Validation and constructor members

The referenced constructor must resolve on a closed record type, be an instance
member named `.ctor`, match every supported argument exactly, and return Void. Missing owners,
open types, wrong arity, static methods, other member names, unsupported arguments,
and unresolved constructors are rejected. Forward references and references into the
linked System library are supported. The definition still undergoes ordinary method
validation even though applying its attribute does not execute its body.

`.ctor` is now an accepted member name. Its declaration must be instance and return
Void; this change supplies constructor identity for metadata. `newobj Type` retains
its existing field-based record construction and does not automatically invoke a
`.ctor`. Explicit ordinary calls to a constructor still follow existing value-receiver
semantics. General constructor initialization, visibility, and verification rules
remain separate work.

Attribute references stay closed even on generic definitions; `Marker<Int32>` can
be used, but `Marker<!0>` cannot. Specializing an annotated method preserves its
attribute metadata without substituting it. No System.Attribute inheritance check
is imposed because the platform does not yet implement that hierarchy. Repeated
attributes are retained in declaration order; AttributeUsage, multiplicity, inherited
attributes, and compiler-specific target restrictions are not implemented.

## Library conventions

The platform-written System library now defines
System.Runtime.CompilerServices.UnionAttribute with an ordinary parameterless
constructor. It carries no VM behavior. A marker alone neither validates a union
member pattern nor changes layout, allocation, copying, or instruction execution.
The full [union convention](unions-and-enums.md) still needs its construction,
typed-access, and storage contracts.

InternalCall implementation flags and P/Invoke import metadata remain their existing
explicit mechanisms; arbitrary custom attributes do not enable native execution.
The development [attribute introspection API](attribute-introspection.md) exposes
descriptive snapshots on MemberInfo and ParameterInfo. `examples/attributes.neoil` demonstrates
annotations on a generic type, its method, and a free function, with a constructor
that would Fault if executed, proving the example uses metadata without instantiation.


## General metadata and AttributeUsage direction (2026-10-10)

Author direction, 2026-10-10: TestAttribute discovery is a consumer of general
custom attributes, including AttributeUsageAttribute. Support applicable declaration
kinds rather than designing a methods-only testing mechanism. This extends the
[testing direction](../runtime/raven/tests/README.md), not its execution contract.

### Implemented metadata foundation

The host metadata library now authors, reads and exposes attributes for methods
(including constructors and assembly-level functions), fields, properties and
parameters, alongside existing type attributes. CLI, native PE/#Neo and NEOX
snapshots retain data without invoking attribute constructors or annotated code.
Native-to-CLI reference projection preserves these attributes. Attribute identity
includes its assembly; display names alone are not binding identities.

This is a metadata facility, not yet Raven source emission or guest runtime discovery.
Read the [complete host API contract](../api-docs/experimental-metadata.md#member-custom-attributes-development-2026-10-10).
Introspection returns **declared** data; inherited lookup is a separate future policy.
The existing bounded payload supports String, Int32 and Boolean fixed arguments.
Enum constants, named arguments, arrays and type-valued arguments are not yet admitted
by the native authoring profile. No AttributeUsage enforcement is claimed yet.

No new native format field is needed for this slice: enclosing types carry field
and property annotations through their validated metadata target tokens; enclosing
functions carry parameter annotations. Method attributes have no target override.
Readers reject targets outside their owner and excessive counts. Tokens are local
metadata addresses, not durable test IDs. Parameter markers keep their existing
semantics and remain distinguishable from ordinary annotations. Matching updated
host readers are required; earlier bounded readers reject the expanded profile.

### AttributeUsage contract to implement next

The intended contract follows the .NET baseline:

- Attribute classes derive from System.Attribute. AttributeUsageAttribute belongs
  on attribute classes and takes AttributeTargets, with AllowMultiple and Inherited
  Boolean options. The default policy is All, false, true respectively.
- Method targets include constructors only through Constructor, and ordinary
  functions/methods through Method. Module-level functions do not require a fixture
  class or a new testing-specific target kind.
- Preserve and validate usage across native compilation and reference import, not
  only when the attribute declaration is in the current compilation. Resolve the
  core usage marker by identity. A same-named user type is not a policy marker.
- Reject invalid targets and repeated single-use attributes during source binding.
  Define equivalent authoring validation with explicit dependencies; metadata-only
  loading must not execute constructors to determine policy. Unknown or malformed
  policy must not quietly fall back to defaults.
- Preserve inheritance intent. Declared-only inspection remains declared-only;
  inherited queries must explicitly traverse supported base/override relationships
  and apply the selected multiplicity policy. Do not copy inherited annotations
  into emitted declaration metadata.

First qualify enum fixed arguments and named Boolean arguments needed by the real
usage attribute. Then add runtime declarations and native compiler emission/import,
including positive and negative cross-assembly tests. Keep general constructor
execution separate from metadata inspection.

Cover types, constructors, functions/methods, fields, properties and parameters.
Assembly/physical-module annotations, return values, generic parameters, enum literal
fields and events need an explicit inventory and representation where applicable;
this slice does not claim them. Logical declaration modules are not physical CLR
Module rows. Do not conflate those owners merely to reuse AttributeTargets.Module.

TestAttribute will be a normal Method-targeted attribute with a selected repetition
and inheritance policy. Its exact policy and source syntax must be compiled and
qualified before documentation presents it as working. Guest assembly-function
introspection and AOT metadata/body retention remain separate discovery gates.

### Comparison and tradeoffs

Reviewed 2026-10-10: the
[C# attribute specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/attributes)
and [AttributeUsageAttribute reference](https://learn.microsoft.com/en-us/dotnet/api/system.attributeusageattribute?view=net-10.0)
define target selection, multiplicity, inheritance and positional/named arguments.
CLI records data independently of constructor execution. We retain that separation
and .NET-style metadata identity. The current native payload is deliberately smaller;
that restriction blocks ordinary AttributeUsage until enum/named arguments are added.
It is a bridge limitation, not a desired permanent platform rule.

Hard-coding a TestAttribute name would be cheaper but would not support serializers,
routing or other metadata consumers. General metadata support costs validation,
reader/writer compatibility and retained AOT data. There is no performance claim.

### Validation

`dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --member-attributes`
checks CLI/.NET inspection, native PE and NEOX round trips, reference projection,
classless discovery, immutable loaded lists and rejected malformed payloads/targets.
The fixture has attribute constructors and annotated functions that fault if invoked;
inspection completes without executing them. The full 168-group metadata regression
suite passes. The parameter-array gate separately combines ordinary parameter
annotations with ParamArrayAttribute and checks projection does not duplicate it.
The generated native image also passes neoCLR verification and returns 42 from its
unannotated entry point. This does not qualify native guest automatic discovery.
