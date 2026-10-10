# Introspection and Object contracts

[TypeInfo](xref:System.Introspection.TypeInfo) describes a type's shape, generic
arguments and supported member queries in the loaded program. Named declarations
also provide [NominalTypeInfo](xref:System.Introspection.NominalTypeInfo), including
the declaration metadata shared with [MemberInfo](xref:System.Introspection.MemberInfo).

These APIs inspect metadata. Use the separate [reflection operations](reflection.md)
to construct objects, invoke methods or access fields within the supported subset.

<a id="development-nominal-and-structural-type-descriptors"></a>

## Nominal and structural type descriptors

TypeInfo does not inherit MemberInfo. Common code uses DisplayName and
IsNominalType; only a NominalTypeInfo view exposes Name, FullName, Namespace,
module/token, declaring type and custom attributes. Arrays and structural Function
shapes do not implement NominalTypeInfo or MemberInfo. Named declarations, including
constructed generic types, do. General shape and member queries remain on TypeInfo.
Use NominalTypeInfo for declaration metadata and DisplayName for diagnostic text.

## Type identity

Repeated queries can allocate different descriptors of the same type. Both runtime descriptor providers
apply represented-type identity through both `Equals(TypeInfo)` and
`Object.Equals(Object?)`. Generic arguments, element shape and loaded definition
identity participate; a name or metadata token alone is insufficient.

Typed equality implements `EquatableTo<TypeInfo>.Equals(TypeInfo)`, with no nullable
operand. Object equality is the separate null-aware reference boundary: null and
unrelated objects compare false. `Object.ReferenceEquals` still compares descriptor
allocations. This follows the distinction between type identity and wrapper identity
in .NET's Type APIs, without requiring CLR wrapper caching.

`GetHashCode`, called through Object, hashes the represented DisplayName with System.HashCode.
Equal represented types have equal hashes. Distinct definitions with matching display
names may collide; hashing never substitutes for equality. Hashes are not persistent
identifiers and may change between runtime versions. `ToString` returns the represented
FullName, rather than the runtime wrapper class name.

```raven
let first = typeof(int)
let second = typeof(int)
let view: Object = first
// Both compare the represented type, despite separate descriptor allocations.
first.Equals(second)
view.Equals(second)
```

The executable integration fixture also checks constructed generic types, arrays,
wrong types, null at the Object boundary, HashMap callbacks and retention through GC.
There is no general default comparer or cross-program descriptor identity contract.

## Logical modules (development, 2026-10-10)

`RuntimeContext.Current.ExecutingAssembly.GetModules()` now returns flat logical
modules in ordinal name order, including explicitly declared empty modules. A module
is a namespace of declarations within an assembly; dotted names do not introduce
parent links or recursive membership. `ModuleInfo.GetTypes()` returns retained types
owned by the exact name, including nested types, with their original declaring-type
ownership. Older images without a declaration table project retained types, free
functions and constants; they cannot recover undeclared empty modules.

Type, member and parameter `Module` properties identify the same logical owner.
Module identity remains full assembly identity plus logical name, within the loaded
program. **Migration:** `ModuleInfo.MetadataToken` is removed. Physical image rows
and their token scopes are separate metadata facts. Remaining type/member/parameter
tokens describe physical definitions; pairing one with a logical module is not a
replacement for physical scope information. No guest resolve-by-token API is provided.
Rebuild the source libraries, reference assembly and runtime together.

Interpreter traversal is qualified by the [Raven consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/guest-modules/README.md).
AOT supports `NominalTypeInfo.Module` for explicitly retained nominal type definitions
through the existing private reflection-roots policy, reading the resulting Name,
and following Module.Assembly to AssemblyInfo.Name/FullName. Assembly ownership
is retained only for the exact module/assembly pairs reached from retained types.
Unretained types fault with an explicit metadata-retention error. This does not grant
constructor/method invocation. Explicit schema-4 moduleCatalogs roots additionally
enable AssemblyInfo.GetModules for the complete declared names, including empty
modules. Without that policy the call faults, even when some type owners are retained.
Native RuntimeContext.ExecutingAssembly, AssemblyInfo.GetTypes, Module.GetTypes and
module Object.Equals dispatch are not qualified by this slice. Guest module function
enumeration and context-wide assembly discovery remain open.

The unstable Rust backend bridge
`neoclr::native_metadata::type_module(&Module, &metadata::Type) -> Result<Value, Fault>`
produces the same module descriptor recipe without allocating a guest wrapper or
executing user code. Unknown types, absent/ambiguous catalog scope or invalid ownership
return Fault. It is an in-repository compiler helper, outside the guest API and
RavenDoc type selection; native snapshot factories materialize its result under
explicit retention.

The same unstable backend bridge provides
`module_assembly(&Module, identity: &str, name: &str) -> Result<Value, Fault>` and
`assembly_name(&Module, identity: &str) -> Result<Value, Fault>`. These validate
against the loaded catalog; missing assemblies or invalid module ownership return
Fault. The AOT adapter admits only retained ownership pairs/full assembly identities,
otherwise faults with `native assembly ownership metadata was not retained`.
These helpers do not enumerate assemblies, load dependencies, or grant invocation.
`assembly_modules(&Module, identity: &str) -> Result<Value, Fault>` returns the
complete declared logical-module recipe, rejecting absent assemblies and legacy
metadata without an explicit declaration table. The AOT adapter requires a matching
catalog root, otherwise faults with `native assembly module catalog was not retained`.
See the [private retention policy](https://github.com/marinasundstrom/neoCLR/blob/main/docs/declaration-modules.md#native-module-catalog-retention--development-2026-10-10).
Catalog retention admits module ownership/name queries, not the contained types or
callable bodies. No guest signature or RavenDoc type-selection change is required.

## Assembly and module identity

[AssemblyInfo](xref:System.Introspection.AssemblyInfo) compares the full assembly
identity in the current loaded catalog. [ModuleInfo](xref:System.Introspection.ModuleInfo)
compares that identity together with its module name. Catalog validation ensures
these keys are unique within one loaded program. Short assembly names, module names
alone and metadata tokens are not sufficient keys. Repeated queries can allocate
separate, equal wrappers; ReferenceEquals still compares allocations.

These contracts apply through Object.Equals(Object?). Null and other descriptor kinds
compare false. GetHashCode hashes the same identity components with System.HashCode;
collisions are allowed and hashes must not be persisted. Assembly display returns
FullName; module display returns Name. This does not introduce nullable typed equality
or a new EquatableTo interface on these descriptors.

.NET Assembly/Module describe loaded runtime entities, with loader contexts and
runtime-specific identity. neoCLR currently has one descriptive loaded catalog and
no dynamic assembly loading. Its full-name-based assembly equality is scoped to that
model, not a promise of CLR loader equivalence. Additional load contexts would require
an additional identity component. String hashing currently creates encoding temporaries.

The executable fixture checks repeated wrappers, Object dispatch and HashMap callbacks
under collection pressure. A separate catalog regression uses assemblies with matching
short names and module names but different full identities, plus multiple modules in
one assembly.

## Field, method and property identity

[FieldInfo](xref:System.Introspection.FieldInfo), [MethodInfo](xref:System.Introspection.MethodInfo)
and [PropertyInfo](xref:System.Introspection.PropertyInfo) implement Object equality
using descriptor kind, closed declaring-type identity and definition index. Two queries
for the same declaration compare equal even with separate wrappers. Different generic
owners, different declarations and different descriptor kinds compare unequal. Null
and unrelated objects compare false. There is no new nullable typed equality operand.

Their hashes combine kind, declaring DisplayName and definition index. Equal members
have equal hashes; distinct type definitions with matching names may collide. Neither
the hash nor DefinitionIndex is a persistent key. ToString returns the member Name;
it does not format a .NET-style signature or uniquely distinguish overloads.

Current queries enumerate retained declarations on the requested type. They do not
walk base types, even without DeclaredOnly; callers can inspect BaseType explicitly.
The closed declaring owner is used for identity; no ReflectedType is exposed. This
is narrower than .NET reflection, where reflected context can participate in equality.
Generic method-definition queries still fault; the current equality contract does
not add generic method instantiation support. Property accessor queries preserve the
same method definition identity as direct method queries.

All three interfaces have generated member reference coverage. Queries describe
metadata only: they do not read fields, invoke methods or execute accessors. Public
application property metadata projection remains limited.

## Parameters

[ParameterInfo](xref:System.Introspection.ParameterInfo) describes explicit method
parameters and property index parameters. Object equality uses the closed declaring
type, owner kind (method or property), definition index and zero-based position.
Repeated queries compare equal; matching names/types/positions from different members
do not. A property index parameter differs from its getter/setter parameter, including
when their metadata token is the same. Tokens can be zero and are not equality keys.

GetHashCode combines the same owner components and position, hashing the declaring
FullName. Distinct definitions with matching display names may collide. Hashes/indexes
are not persistent IDs. ToString returns Name, which can be empty for missing names
and current property index snapshots. Null and other object kinds compare false.
ReferenceEquals still compares wrapper allocations; no nullable typed equality is added.

Ownership is retained internally as a compact key rather than a member/parameter
object cycle. This adds a type descriptor and two integers to each runtime snapshot.
There is no public Member property yet. .NET ParameterInfo exposes Member and Position,
but its base class does not define owner-based Object equality; this is a deliberate
neoCLR snapshot contract. A future Member property needs direct, scoped resolution
without recursively materializing member/parameter graphs or enumerating unrelated
members. Return parameters and optional/default values remain absent. Development attribute
inspection is described below.

ParameterInfo and [BindingFlags](xref:System.Introspection.BindingFlags) have
generated member coverage. Binding
flags select visibility and instance/static categories; they do not enable inherited
traversal or grant member invocation access.

<a id="type-classification-flags-development"></a>

## Type classification flags

TypeInfo exposes IsAbstract, IsOpen, IsClosedHierarchy, IsUnion, IsEnum and
IsValueType. IsOpen means unrestricted inheritance/implementation, not generic
openness. A closed family reports IsClosedHierarchy, while an ordinary leaf reports
neither IsOpen nor IsClosedHierarchy. IsUnion identifies nominal union declarations
(including constructed Option/Result), not their cases or ad-hoc union expressions.
ClosedHierarchy describes the direct family; permitted open subtypes may themselves
have descendants. These are descriptive metadata queries, not added raw-IL enforcement.


<a id="runtime-execution-development"></a>

## Runtime execution

Import System.Runtime.Reflection for [checked construction and property execution](reflection.md).
The descriptor interfaces remain descriptive. Execution accepts runtime-backed
providers, returns Result for validation failures, and preserves terminal user Faults.
Application instance properties and their original tokens are now projected; static
properties remain outside the importer and init-only setters are omitted from assignment.
Method/accessor visibility queries use retained source accessibility when available.

Development constructor discovery adds `ConstructorInfo` to the `MemberInfo` family.
`TypeInfo.GetConstructors()` selects public instance constructors declared on the
type; the flags overload allows descriptive visibility selection. Constructors
expose parameters and visibility, without a method return type. See the
[Reflection API guide](reflection.md) for execution limits.


### Interface method flags (development)

For admitted application interfaces, public instance defaults are virtual and have
bodies (`IsAbstract` is false). Ordinary static helpers report `IsStatic` true and
`IsVirtual`/`IsAbstract` false. Private helper visibility remains private; helpers do
not create an implementation obligation on a class. This behavior is included in Reflection describes these methods; it does not enable general
invocation of interface bodies or bypass their access checks.


Explicit application interface method bodies retain their qualified source names and
private visibility. Query them using NonPublic, Instance and DeclaredOnly flags.
Their IsVirtual flag is false: neoCLR represents explicit interface mappings separately
from class virtual slots, unlike the CLI private/final/virtual emission flags. Ordinary
public method queries exclude these private bodies. This development support does not
add GetInterfaceMap or general reflection invocation of interface implementations.

<a id="attribute-data-development-after-preview-10"></a>

## Attribute data

`MemberInfo.GetCustomAttributesData()` and `ParameterInfo.GetCustomAttributesData()`
return `Sequence<CustomAttributeData>` for directly declared, retained attributes.
Types inherit this method through MemberInfo; fields, properties, constructors and
methods use the same contract. No constructor or setter runs, and attributes are
not inherited or merged. Multiple declarations retain metadata order; retrieval
does not deduplicate them. Raven validates AttributeUsage on source declarations. An unannotated member/parameter returns an empty sequence.

[CustomAttributeData](xref:System.Introspection.CustomAttributeData) exposes
`AttributeType: TypeInfo`, `Constructor: ConstructorInfo` and
`GetConstructorArguments() -> Sequence<CustomAttributeTypedArgument>`.
[CustomAttributeTypedArgument](xref:System.Introspection.CustomAttributeTypedArgument)
exposes `ArgumentType: TypeInfo` and `Value: Object?`, with an exact boxed String,
Int32 or Boolean value (or null String). The argument sequence preserves constructor
order and is copied on every call; descriptor objects expose no mutation.

Retention currently covers application-defined attributes in imported modules and
the existing union marker. Compiler-only annotations and external framework
attributes without admitted runtime definitions are not exposed. Unsupported user
attribute constants and named arguments fail import. Attribute constructor bodies
still need normal bridge admission. Runtime null constants are supported; Raven's
nullable-string attribute constructor emission remains a known source limitation.
There is no `GetCustomAttributes` instantiation API or inheritance option yet.

Development AOT now supports explicit custom-attribute data retention on macOS ARM64 and Windows x64,
using the same descriptor/argument contracts. Int32-backed enum arguments retain
nominal ArgumentType and expose their underlying boxed Int32 Value. Private reflection
roots select metadata independently from invocation; unretained owners fault, while
retained unannotated targets return empty sequences. Named String/Int32/Boolean field/property arguments are exposed by the matching
development runtime library. This is a metadata-authored native gate, not compiler source-annotation
integration or general discovery. Focused compiler-source emission checks pass independently.
[Native validation and limits](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-attributes/README.md).

Cache validated descriptions during startup for repeated mappings. Dynamic union
case construction is a separate next step; this API alone does not install a route
mapper.


`TypeInfo.IsVisible: bool` reports public visibility including containing types,
element types and generic arguments. Missing imported visibility fails closed.
Native definitions use declared visibility; primitives and generic parameter
descriptors are visible. This is descriptive information, not invocation permission:
open types and private members remain subject to their execution restrictions.
The route-mapper case uses it to reject nonpublic schemas during preparation.

<a id="attribute-constructor"></a>

## Attribute base constructor

`protected Attribute()` initializes a derived attribute. Only constructors of derived
attribute classes may invoke it. `System.Attribute` is abstract; this constructor does
not create attribute instances for metadata discovery. Use introspection to inspect
attribute data without executing constructors.

[Attribute type reference](xref:System.Attribute)


### Named attribute arguments (development 2026-10-10)

`CustomAttributeData.GetNamedArguments() -> Sequence<CustomAttributeNamedArgument>`
returns assignments in metadata order without executing constructors, setters or
other annotated code. Each call returns a fresh sequence; replacing entries in an
array obtained by casting it does not change later results. An attribute without
named assignments returns an empty sequence.

| Member | Contract |
| --- | --- |
| `CustomAttributeNamedArgument.MemberName: string` | Exact field/property name. |
| `CustomAttributeNamedArgument.IsField: bool` | True for a field, false for a property. |
| `CustomAttributeNamedArgument.TypedValue: CustomAttributeTypedArgument` | Declared primitive type and boxed constant; null String stays null. |

String, Int32 and Boolean named values are supported in interpreted and retained
native execution. The metadata reader validates the target member and value before
inspection. The .NET names and typed-value model are preserved; its `MemberInfo`
property is a remaining gap, as are named enums, type/array constants and wider
primitives. There are no public constructors or mutation APIs on these descriptors.
Use matching runtime/library artifacts: the descriptor layout now includes the
named snapshot. Older libraries retain fixed-only inspection and explicitly reject
named data. See the [tested consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/native-attributes/README.md).
