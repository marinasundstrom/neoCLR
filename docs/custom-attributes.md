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


### Enum and named payload foundation (2026-10-10)

Implemented in the host metadata layer: fixed arguments preserve the nominal type
of Int32-backed enums; named String/Int32/Boolean field or property values preserve
member name, kind, type and value. Native records have an optional `named_arguments`
array. CLI projection writes the standard named count and field/property entries;
constructor signatures encode enum identity instead of erasing it to Int32.
Omitting the native array preserves old marker/scalar records. Readers must be
updated for records that contain it; no published format compatibility is claimed.

Owned named members are checked for an exact type and a public writable instance
field or public instance read/write non-indexed property. The linked runtime checks
resolved declarations without executing constructors, getters or setters. External
references require matching dependency metadata at link time. Inherited named
members, broader primitive types, named enum values, arrays and System.Type values
remain implementation gaps relative to .NET. AttributeUsage target/multiplicity
policy is not yet enforced by this payload layer. Existing record-shaped native
attributes do not yet require a System.Attribute base class.

The host APIs expose both argument groups. Guest CustomAttributeData still has its
previous constructor-argument-only layout and explicitly rejects named-data queries
rather than returning incomplete data. Extending that descriptor, the library
reference, compiler emission/import and AOT retention are subsequent slices.

Validation: the 168-group host suite and nine Rust attribute tests pass. A C#-compiled
AttributeUsage annotation is decoded identically to CLR CustomAttributeData for its
enum value and named options. Native PE/NEOX and CLI projection retain the data;
invalid member names/kinds/types and duplicates reject. The native executable with
faulting attribute constructors/accessors verifies and returns 42. See the
[host API details](../api-docs/experimental-metadata.md#enum-and-named-attribute-data-development-2026-10-10).


### Raven import and usage binding (2026-10-10)

The shared Raven compiler source now exposes imported attributes on types,
constructors, methods/module functions, fields, properties and parameters using its
ordinary AttributeData contract. Enum identity and named values survive mapping.
Existing binder checks enforce imported AttributeUsage targets and repetition,
including inherited policies and directly replaced policies with fresh defaults.
Inherited is preserved as data, without adding runtime inherited-attribute queries.
Malformed constructor/named-member metadata is rejected with RAVT003 before binding.
A policy lookalike in another namespace is not treated as System.AttributeUsageAttribute.

The host reader also inspects bounded nongeneric CLI instance/static nominal
signatures, so the explicit primitive bootstrap's attribute constructors can be
validated without executing them. Previously its logical signature API admitted
only static CLI value signatures. Native signatures remain authoritative.

Validation uses metadata-authored libraries and a separate Raven consumer: nine
usage scenarios, three malformed payloads and supported member inspection; 20
ordinary Raven AttributeUsage tests, native flags and runtime async-symbol regressions
pass. The 168-group host metadata suite passes. Source attribute emission is still
rejected by the native emitter. The bootstrap fixture does not establish strict
native Attribute inheritance enforcement. These results do not qualify guest named
data, discovery, AOT retention or a rebuilt development bundle.

Tested compiler: Raven `51da30ea7` on the shared integration line
`codex/source-object-metadata-resolution`, with this checkout's metadata project.
The explicit CLI bootstrap is `target/library-scopes-final/bundle/lib/Core.dll`,
SHA-256 `132bbb0932d5903cdca1c66a18cba68ac79299973cec13591bc4af885eee6e6f`.
That bundle's compiler remains `494dede84`; this source-import qualification is
not a claim that the older bundled compiler implements it.

## Native AOT inspection — development 2026-10-10

The author explicitly requires custom attributes with AOT. The first
[retained native gate](experiments/native-attributes/README.md) now passes on macOS
ARM64 with interpreter parity. Private reflection-roots schema 3 separates
`customAttributes` retention from constructor/accessor invocation. Fixed arguments,
including Int32 enums, are materialized using the VM's metadata recipe; source
member tokens survive executable lowering. User constructors are neither called
nor rooted. Windows x64 has a matching workflow gate; qualification is pending.

This is native runtime support, not native source annotation emission. The fixture
attaches annotations after compiling its Raven consumer; the bundled compiler
remains unchanged. Guest named-data inspection and automatic discovery remain open.
See the gate's retention design, tradeoffs, provenance and exact bounds.


## Raven source annotation emission (development 2026-10-10)

The shared Raven integration line now emits bound source annotations for supported
types, interfaces/enums, functions, constructors, methods, fields, properties/accessors
and parameters. String (including null), Int32, Boolean and Int32 enum fixed values
and primitive named field/property values round-trip through native metadata. Local
constructors in a co-owned Attribute hierarchy and imported constructor identities
are both covered. Unsupported targets or payloads diagnose before publishing output.
Ordinary property attributes no longer leak onto backing fields; explicit field
attributes and field-only Raven storage retain their intended targets.

Validation uses the native attribute probe and 21 ordinary .NET AttributeUsage tests;
flags marker regression also passes. This is compiler-source support, not yet an
updated toolchain bundle: the bundle remains at Raven 494dede84. No Runtime Contract
configuration changes. See [bridge ownership and bounds](raven-cli-bridge.md#native-source-annotations-2026-10-10).

The metadata-authored AOT gate now passes on macOS ARM64 and Windows x64. The
[Windows evidence](experiments/native-attributes/windows-validation.json) records nine
verified input hashes (with Windows CRLF checkout normalization where applicable),
matching native/interpreter success, interpreter live=0, KERNEL32-only native
imports and explicit failure without retention. Two build executables were not
archived for independent rehashing. This gate still attaches attributes after source
compilation; compiler-produced annotation AOT qualification is a separate next step.

Before returning to TestAttribute discovery, complete separate-library Attribute
inheritance, runtime usage declarations and guest named-data support. Arrays/type
constants, wider primitives, inherited named members, named enum arguments and
assembly/module/return/generic-parameter targets remain gaps. Test discovery needs
retention and invocation adapters as well as data. The author-selected follow-up
after that framework is completion of modules in Introspection and RuntimeContext.

Compiler evidence: Raven `dfaa76145` on shared integration branch
`codex/source-object-metadata-resolution`, using the explicit Core.dll bootstrap
SHA-256 `132bbb0932d5903cdca1c66a18cba68ac79299973cec13591bc4af885eee6e6f`.


## Named guest inspection (development 2026-10-10)

The source runtime now exposes GetNamedArguments and immutable
CustomAttributeNamedArgument descriptors (MemberName, IsField, TypedValue). The
shared interpreter/AOT recipe preserves metadata order, String/null, Int32 and
Boolean values, without executing constructors or assignments. The expanded native
consumer tests both target kinds, empty sequences, exact types and copied sequence
isolation; attribute accessors deliberately fault if invoked and are not native roots.

This follows the host metadata and .NET data model. MemberInfo resolution on the
named descriptor remains an explicit API gap, not a different semantic contract.
Wider/type/array/named-enum constants remain unsupported. Match the updated source
runtime library and runtime; older fixed-only libraries still inspect fixed values
but reject named data explicitly. The metadata/retention format and Runtime Contract
options are unchanged. The source-built macOS ARM64 consumer passes in both modes;
The [Windows action](https://github.com/marinasundstrom/neoCLR/actions/runs/38045805683) succeeds at 96ad94e2; archived input hashes have not yet been independently checked.

The next bounded work is runtime AttributeUsage/AttributeTargets declarations. The
separate-library Attribute base requires a coordinated external-base reference,
constructor validation and compiler-emission slice; it has not been bypassed.
Test discovery and the later module-model completion remain in the selected order.


## Runtime usage declarations (development 2026-10-10)

Source-owned AttributeTargets and sealed AttributeUsageAttribute now provide .NET
flag values, read-only ValidOn, AllowMultiple=false and Inherited=true defaults.
Both options are writable; inherited guest queries remain a gap. The
[runtime consumer](experiments/attribute-usage/README.md) checks these contracts in
interpreter/AOT and requires RAV0502 for an invalid imported target. Windows CI runs
the same gate; its qualification remains pending.

The prepared primitive bootstrap removes duplicate usage declarations and their
type-level usage annotations; ordinary .NET lookup is unchanged. Source ownership
rejects duplicates. Raven 0f09c350a fixes recursive validation of self-described
AttributeUsage by binding immutable data before enforcing policy; 24 focused .NET
tests pass. The development CI compiler pin advances to that revision. Full
compiler-produced annotation AOT inspection and external Attribute bases remain
open before test discovery, followed by the requested module model work.
