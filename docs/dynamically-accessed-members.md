# Dynamically accessed members (development)

The native backend recognizes `System.Diagnostics.CodeAnalysis.DynamicallyAccessedMembersAttribute`
on `System.Introspection.TypeInfo` parameters, return values, fields, properties and descriptor method receivers. This is an initial subset of the
.NET contract, not general trim analysis. It adds inferred roots to the same
retention pipeline used by [project retention files](native-retention.md).

```raven
import System.Diagnostics.CodeAnalysis.*

func Read(
    [DynamicallyAccessedMembers(
        DynamicallyAccessedMemberTypes.PublicParameterlessConstructor |
        DynamicallyAccessedMemberTypes.PublicProperties)] type: TypeInfo
) -> Result<Object, JsonError> {
    return JsonSerializer.Deserialize("{\"Text\":\"retained\"}", type)
}
```

The [executable consumer](experiments/dynamic-access/sample/Main.rvn) passes
`typeof(Message)` to this annotated parameter. Keep annotations on forwarding
parameters as well as the ultimate reflective API. A bare `Object` parameter does
not communicate the type that reflection will inspect.

## Current support

- Supported requirements: `None`, `PublicParameterlessConstructor`,
  `PublicConstructors`, `PublicProperties`, and their combinations. The enum
  exposes the conventional .NET values through `Interfaces` and `All`; unsupported
  flag requests fail with `DAM1001` rather than acting as inert annotations.
- Valid targets are currently `TypeInfo` parameters, return values, fields and properties. Native
  analysis validates each target identity and type; malformed and unsupported contracts fail with `DAM1001`.
- Exact nongeneric nominal type tokens are recognized through the runtime's
  `RuntimeContext.GetTypeInfoFromHandle` and `RuntimeTypeInfo.FromHandle` bridges.
  Locals, argument forwarding, casts and control-flow joins preserve known facts.
  Joins retain the union of up to 64 known types. Address-taken mutable slots are excluded from fact tracking
  conservatively; unrelated locals and arguments retain their facts.
- Inferred roots preserve public constructor metadata/invocation or public property
  metadata/getters/setters as requested. The current constructor policy conservatively
  retains all supported public constructors even for a parameterless-only request;
  this can increase native image size. Property retention does not itself request
  construction. Existing backend shape/access checks remain authoritative.
- The pass follows entry-point calls and bound functions, checking declared parameter contracts at interface calls.
  Unused direct callers do not root their types. Dynamic implementation bodies are
  not traversed before specialization (`DAM2004`); reflection inside them needs
  explicit policy where not otherwise proven. Nongeneric nominal class/interface
  parameter compatibility is checked separately (`DAM2005`), as are nongeneric
  reference-class overrides (`DAM2006`). Generic mappings and implementation-body
  traversal remain gaps. Public constructors and instance property accessors selected by
  inferred requirements are additional analysis roots. Their direct calls are
  analyzed repeatedly until no new bodies are found, so reflection inside a retained
  constructor/accessor can infer further types. Cycles terminate because the root set
  only grows within the finite input module. Private constructors, unrequested member
  categories, static/indexed properties and unrelated methods are excluded.
- `DAM2001` reports unknown or insufficient input requirements, `DAM2002` reports
  unsupported target shapes, and `DAM2003` reports indirect use of an annotated
  function. These warnings do not establish successful retention. Use a qualified
  explicit policy for such cases and test the native consumer; explicit policy
  does not pretend to prove data flow or suppress those diagnostics.

The report's `dynamicAccess` entry records inferred source rows/flags and
warnings. The backend also writes warnings to stderr. Project-build logs retain
stderr in `build.json`. Automatically inferred roots merge with validated explicit
policies; they do not remove explicitly requested capabilities. Executable bodies
admitted by explicit policies also seed the analysis. The native construction,
constructor and property binders validate the policy on a temporary copy and provide
the exact admitted body rows. Metadata-only policies do not seed bodies or acquire
invocation rights; the final union binds the original services once. Inferred reflection
requires the existing `--compile-system` and `--reference-arena` runtime context.

## Annotated runtime APIs

JSON's `Deserialize(..., TypeInfo)` overloads declare public parameterless
construction and public properties. `ObjectMapper.Read`/`ReadValue` carry the same
requirements. DI's `Add`/`AddKeyed` implementation parameters and
`TryAdd`/`TryAddKeyed` forwarding parameters declare public constructors, with the
matching `ServiceCollection` interface and `DefaultServiceCollection` declarations.
`ConstructorPlan.Prepare` declares the same constructor requirement.

The first slice does not infer requirements through registration storage or async
state-machine fields. Such internal flows can still warn even when the caller's
known type supplied sufficient roots. Nested object-property types, callbacks,
reflection-discovered types and recursive object graphs are not automatically
closed over. The retained-body follow-up handles explicit known type tokens within
those bodies, not types recovered from property descriptors or arbitrary storage. Their needed types may require explicit policies.

## .NET comparison and remaining work

.NET's [annotation guidance](https://learn.microsoft.com/en-us/dotnet/core/deploying/trimming/fixing-warnings)
uses member requirements on type-bearing values and generic parameters, propagated
through callers. The [member flag values](https://learn.microsoft.com/en-us/dotnet/api/system.diagnostics.codeanalysis.dynamicallyaccessedmembertypes?view=net-10.0)
are reused here. neoCLR uses its existing `TypeInfo` contract rather than introducing
another type descriptor. This preserves familiar intent while reusing current
native retention and invocation checks.

The pass reuses typed verifier stack effects for relevant caller bodies. This adds
build-time analysis and metadata storage; it does not change interpreter behavior
or introduce runtime code generation. More conservative retention and warnings
are costs of the initial implementation, not improvements in .NET semantics.

Generic-parameter annotations, string type-name resolution, arbitrary generic target retention, richer member flags,
class/base annotation inheritance and complete interprocedural inference remain
unimplemented. Nongeneric nominal reference-class parameter override compatibility
is checked; generic/value-type virtual contracts remain unqualified. Do not treat an annotation as a blanket no-trim promise.
Future work must extend metadata transport, flow analysis, runtime API annotations,
negative diagnostics and reference documentation together.

## Metadata ownership

Raven transports the ordinary parameter custom attribute, its enum argument and
its exact parameter token. No new Runtime Contract mapping or compiler emission
rule is required. The native backend owns analysis and inferred retention; the
runtime library owns the declaration and API annotations. Existing source-name
origins identify the attribute and handle bridges. The emitted native attribute
records are retained independently of guest reflection admission. This does not
add a temporary CLI pseudo-attribute or make a bridge encoding a platform rule.

## Retained-body follow-up (2026-10-28)

The initial pass analyzed only direct entry-point reachability. That left calls made
inside reflection-retained constructors/accessors outside analysis. Following these
bodies to a fixed point brings this subset closer to .NET's dependency propagation
without inventing new attribute semantics. The implementation conservatively considers
all public constructors for the existing constructor policy; unsupported reflection
shapes remain subject to backend admission. Re-running the bounded flow pass costs
additional compilation work; no performance improvement is claimed. The subsequent explicit-policy follow-up also seeds bodies admitted for execution
by a validated policy, while retaining metadata-only requests as descriptive data.
See [native before/after and regression evidence](validation/dynamic-access-closure-20261028/README.md).

## Explicit-policy analysis seeds (2026-10-28)

A type can enter execution through a project retention file even when its caller's
TypeInfo flow is unknown. Public parameterless constructors, admitted constructor
invocation bodies and admitted property accessors now participate in annotation
analysis. Reusing the code-generation binders avoids a separate interpretation of
access checks, policy versions and invocation rights. Known dependencies inside
those bodies can therefore be inferred using the same bounded closure as annotation
roots. Validation and binding on a disposable copy add build-time work and allocation;
this is a correctness extension, not a performance claim.

Generic-owner body specialization remains outside this analysis; the existing
constructor binder does not supply those as ordinary body seeds. Explicit policies
still do not prove unknown type flow or suppress DAM2001/DAM2004 diagnostics. See
[policy-seed qualification](validation/dynamic-access-explicit-20261028/README.md).

## Interface parameter compatibility (2026-10-28)

`DAM2005` warns when corresponding parameter requirements differ between an interface
method and its selected class implementation. Absent annotations count as no required
members. Missing, weaker and stronger requirements are mismatches: callers through
the interface and concrete class must see the same contract. This follows the .NET
[IL2092 rule](https://learn.microsoft.com/en-us/dotnet/core/deploying/trimming/trim-warnings/il2092)
and [runtime diagnostic reference](https://github.com/dotnet/runtime/blob/main/docs/tools/illink/error-codes.md#il2092-trim-analysis-dynamicallyaccessedmembertypes-in-dynamicallyaccessedmembersattribute-on-the-parameter-parameter-of-method-method-dont-match-overridden-parameter-parameter-of-method-base-method-all-overridden-members-must-have-the-same-dynamicallyaccessedmembersattribute-usage).

The bounded check uses original source identities and existing interface mapping
resolution, including explicit implementations and inherited nongeneric interfaces.
It checks concrete nongeneric nominal reference classes and nongeneric interface
methods throughout the supplied load set, including unused classes. It adds warnings
to `dynamicAccess` and stderr without adding retention roots or changing dispatch.
The cost is extra metadata traversal; no performance gain is claimed.

Generic classes/interfaces/methods, value-type implementations, abstract class mappings
remain outside the interface check. Nongeneric reference-class overrides are checked
separately below. `DAM2004`
still marks the separate implementation-body traversal gap. A mismatch is a diagnostic,
not proof that inferred retention satisfies the implementation. See
[matching/mismatching native consumers](validation/dynamic-access-interfaces-20261028/README.md).

## Class override parameter compatibility (2026-10-28)

`DAM2006` reports unequal parameter requirements between a class override and its
nearest inherited declaration. This uses the same exact-match rule as the interface
check and .NET IL2092. Absent annotations mean zero requirements; weakening or
strengthening a requirement is a mismatch. Every override in a chain is checked
against its nearest declaration, including abstract/reabstracted declarations.

The pass uses the existing inheritance slot lookup shared with verifier validation.
It checks nongeneric nominal reference-class methods, including closed generic
base classes. The shared slot lookup substitutes each ancestor before matching the
parameter signature; generic derived classes and generic methods remain excluded.
It inspects the supplied load set, without rooting bodies, widening retention or
changing virtual dispatch. Open generic owners/methods, value-type overrides and
runtime-owned implicit Object adaptations remain outside this check. The latter
currently expose no TypeInfo parameter contracts. DAM2004 still describes the
separate implementation-body traversal gap.

This adds metadata traversal, not runtime code. No performance improvement is claimed.
[Override qualification](validation/dynamic-access-overrides-20261028/README.md)
compares matching and mismatching native consumers and checks existing inheritance
behavior. Runtime Contract settings, attribute transport and public APIs are unchanged.

## Closed generic base classes (2026-10-28)

DAM2006 now includes an override on a nongeneric derived class whose base is a
closed generic type, such as `DefaultReader : Reader<int>`. The existing verifier
slot lookup substitutes the base's type arguments before selecting the overridden
method, including overloads and multiple ancestry levels. The earlier blanket skip
for constructed ancestors is removed. A verified nongeneric derived type provides
a closed ancestry context; this does not add annotations on generic parameters.

The same exact-match rule and diagnostic behavior apply. TypeInfo parameter
requirements remain the supported annotation target. Generic derived classes,
generic methods and generic interface mappings remain outside the check. No new
retention roots, runtime behavior, public APIs or compiler encodings are introduced.
[Qualification](validation/dynamic-access-generic-base-20261028/README.md) covers
substituted signatures, overload distinction, nested substitutions and native
matching/mismatching controls. No performance improvement is claimed.

## Return contracts (2026-10-28)

`[return: DynamicallyAccessedMembers(...)]` declares requirements on a TypeInfo
result. Reachable implementations check every return path: known type tokens infer
roots, annotated inputs can satisfy the result, and unknown or weaker values warn
with DAM2001. Callers receive the declared guarantee, including through locals and
joins; a warning in the producer is not proof of successful retention. Interface
and virtual-override checks now compare return flags as well as parameter flags,
within the same bounded owner/method scope. This follows .NET's IL2063/IL2093
[rules](https://github.com/dotnet/runtime/blob/main/docs/tools/illink/error-codes.md).

The [factory consumer](experiments/dynamic-access/returns/Main.rvn) returns
`typeof(Message)` and deserializes JSON with no explicit retention file.
Raven emits return annotations with a separate optional method-origin `return_token`
(Param-table identity); ordinary parameter ordinals do not change. The metadata
reader, reference projection and compiler importer preserve that association.
Missing, zero, duplicate and wrong-table return identities reject. Old artifacts
without return annotations omit the new field. Older strict native readers reject
artifacts using it; rebuild the compiler, metadata library and backend together.
Runtime Contract configuration and ordinary .NET emission remain unchanged.

The author has also requested fields/properties, generic parameters, receivers and
all member categories. Those remain implementation work; this return extension does
not make currently rejected flags usable. See [qualification](validation/dynamic-access-returns-20261028/README.md).

## Field and property contracts (2026-10-28)

Annotated instance/static TypeInfo fields check values stored by constructors,
ordinary stores and record construction. Reads provide the declared member
requirements. Inherited fields use their declaring field identity; structural tuple
storage has no nominal field annotation. Property requirements apply to getter
returns and the setter's value parameter, including indexers. A getter consisting
of a direct field read also establishes that field's contract, so ordinary
field-backed and auto-properties check constructor writes without extra annotations.
Computed getters still have their return paths checked. Unknown or weaker inputs
warn with DAM2001; taking an annotated field's address warns with DAM2007 because
indirect writes are not tracked. Warnings remain visible even if other inputs root
the same type.

Static field access and strict type-entry triggers include the corresponding type
initializer in the analysis graph; unused initializers are not blanket roots.
This mirrors .NET's write-validation/read-guarantee model described in its
[ILLink diagnostics](https://github.com/dotnet/runtime/blob/main/docs/tools/illink/error-codes.md).
The older [reflection-flow design](https://github.com/dotnet/runtime/blob/main/docs/design/tools/illink/reflection-flow.md)
is background rationale, not an assertion that every proposed flow is implemented.

HTTP JSON TypeInfo overloads now carry the mapper's constructor/property contract.
JsonClientRead's stored type and constructor carry the same contract through callback
storage. Collections, arbitrary computed backing associations, indirect writes and
compiler-generated async state-machine storage are not automatically inferred.
This adds build-time typed-flow work; no performance improvement is claimed.
[Native auto-property and focused storage qualification](validation/dynamic-access-storage-20261028/README.md)
records the supported scope. Generic parameters, receivers and the remaining member
categories are still the next requested layers.

## Descriptor receiver contracts (2026-10-28)

A method-level DAM annotation on an instance method of TypeInfo or a type implementing
TypeInfo specifies the requirement on `self`. Known caller type values infer roots;
unknown receiver values warn with DAM2001. Within the method, `self` carries that
requirement. Static methods, constructors and unrelated instance owners reject this
placement with DAM1001. Interface/override checks compare receiver flags as well as
parameter and return flags within their existing scope, matching the .NET
[IL2041/IL2094 rules](https://github.com/dotnet/runtime/blob/main/docs/tools/illink/error-codes.md).

Parameterless TypeInfo.GetConstructors/GetProperties and their runtime implementations
now declare PublicConstructors/PublicProperties. The BindingFlags overloads await
broader member-category support rather than declaring an insufficient public-only
contract. The [native direct-introspection consumer](experiments/dynamic-access/receiver/Main.rvn)
needs neither user-authored annotations nor a retention file. [Qualification](validation/dynamic-access-receiver-20261028/README.md)
records inferred flags 515 and matching counts. Runtime Contract settings and metadata
transport are unchanged; method attributes already carried this information.
