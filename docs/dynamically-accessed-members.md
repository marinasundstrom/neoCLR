# Dynamically accessed members (development)

The native backend recognizes `System.Diagnostics.CodeAnalysis.DynamicallyAccessedMembersAttribute`
on `System.Introspection.TypeInfo` parameters. This is an initial subset of the
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
- Valid targets are currently `TypeInfo` parameters. The source attribute's usage
  restricts placement to parameters. Native analysis validates the parameter
  identity and type; malformed and unsupported contracts fail with `DAM1001`.
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
  explicit policy where not otherwise proven. Polymorphic contract compatibility
  remains a gap. Public constructors and instance property accessors selected by
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
policies; they do not remove explicitly requested capabilities. Inferred reflection
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

Generic-parameter/return/field/property annotations, string type-name resolution,
receiver annotations, arbitrary generic target retention, richer member flags,
class/base annotation inheritance and complete interprocedural inference remain
unimplemented. General class virtual-method annotation compatibility is not
qualified by this slice. Do not treat an annotation as a blanket no-trim promise.
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
additional compilation work; no performance improvement is claimed. Policies supplied
only through explicit retention files are not analysis seeds in this follow-up.
See [native before/after and regression evidence](validation/dynamic-access-closure-20261028/README.md).
