# Open generic localization integration — 2026-10-21

Status: development foundation qualified; not yet a published release.

The author requires registering a provider once, then resolving
`Localizer<HomePage>` through an open `Localizer<>` registration. An already-created
closed localizer instance does not meet this requirement.

## .NET comparison

Microsoft.Extensions.Localization registers a singleton IStringLocalizerFactory and
an open transient mapping from IStringLocalizer<> to StringLocalizer<>. DI closes
the implementation with the requested arguments. StringLocalizer<T>'s constructor
receives the nongeneric factory and calls Create(typeof(T)). It delegates lookup
to that localizer. The container does not understand resource files or locales.

Primary sources consulted 2026-10-21:

- [Localization registration](https://github.com/dotnet/aspnetcore/blob/main/src/Localization/Localization/src/LocalizationServiceCollectionExtensions.cs)
- [Typed adapter](https://github.com/dotnet/aspnetcore/blob/main/src/Localization/Abstractions/src/StringLocalizerOfT.cs)

## Shared implementation

- Preserve source declaration identity when matching generic definitions; never
  compare display names or metadata tokens without their declaring scope.
- Close the implementation using the requested service's generic arguments and
  validate constraints and assignability before invoking constructors.
- Prefer an exact closed registration over an open registration for single lookup;
  plural lookup includes all applicable registrations in registration order.
- Cache and detect cycles by registration slot plus closed service identity, so
  singleton/scoped Localizer<HomePage> and Localizer<OtherPage> cannot collide.
- Keep resource selection in a replaceable nongeneric provider and a typed adapter.
  Keep the existing selected-culture, fallback and recoverable loading contracts.
- Native execution needs explicit retention of closed generic constructors and
  their executable dependencies. Descriptive method retention alone does not grant
  this capability. Unretained construction must fail explicitly.

Validation must include two unrelated resource scopes, constructor injection,
closed-registration precedence, keyed/plural resolution, lifetime independence,
constraints/rejection and interpreter/native/JIT execution. No broader open generic
method invocation is implied by this work.

## Implementation

The native emitter now distinguishes a generic declaration token (`typeof(Foo<>)`)
from a constructed signature (`typeof(Foo<T>)`). Both use existing `ldtoken`;
the declaration token contains no caller-scoped generic parameters. The runtime
keeps declaration metadata descriptive: an open declaration is not executable
object storage. CLI emission uses its normal generic-definition TypeRef/TypeDef.

`TypeInfo.GetGenericTypeDefinition` preserves scoped identity.
`MakeGenericType(Sequence<TypeInfo>)` returns `Result<TypeInfo, GenericTypeError>`;
closed arguments only are supported. Native construction requires retained
specializations and reports `NotRetained` when absent. This is a bounded native
capability, not a replacement for .NET generic semantics. Dynamic code generation
and partially open construction remain gaps.

`LocalizerFactory.Create(TypeInfo)` selects resources; `DefaultLocalizer<T>` calls
it with `typeof(T)`. `AddLocalization(factory)` uses TryAdd for the caller-owned
singleton factory and transient open adapter. JSON factory construction performs
no I/O; its first lookup loads and caches a catalogue per resource type, using a
fixed culture. Invalid catalogues fault on lookup; recoverable loading remains
available through `JsonLocalizerProvider.Load`. The cache assumes serialized guest
execution and is not a concurrent shared service contract.

Two unrelated native compiler cases surfaced during qualification: an empty
collection converted directly to Sequence is not admitted by portable lowering,
and a later local with the same name as a completed loop binding can trigger
RAV0174. These are unresolved compiler gaps, not intended language restrictions.

The native DI fixture reached 1,026 intermediate types and 6,176 functions after
specialization. The original 1,024 raw-load-set bound remains; the intermediate
selection inventory permits up to twice the executable type limit, allowing
source declarations and their generated shapes to coexist before pruning.
Executable TYPES remains 1,024 and FUNCTIONS remains 2,048; native GC/layout
budgets and the host ABI are unchanged. Diagnostics include actual counts.

## Library proof cases

DI's `DefaultServiceProvider.Close` is the production `MakeGenericType` consumer:
it closes the implementation from the service arguments before constructor planning.
Built-in tests cover two closed types, scopes, singletons, exact overrides, keys,
plural resolution, disposal and repeated cycle failures. Typed localization exercises
this mechanism through two unrelated resource scopes and one factory registration.

For `MakeArrayType`, built-in `type-relations` tests construct a vector identity
from its element type, create real storage through `ArrayReflection.Create`, and
check integer, reference and jagged identities. `json-streams` deserializes JSON
using a dynamically constructed vector type. The existing JSON mapper takes the
complete target type, so rebuilding that type internally would add redundant work.
No SQL-mapper migration is warranted: its current supported target is a row object,
not a dynamically selected collection shape.

The JSON proof exposed a native inventory gap: executable array casts retained a
storage shape but neither type construction nor reflective array creation admitted
it without a redundant type token. Both binders now share array-shape discovery
from explicit type tokens and executable casts/tests. This closes an implementation
gap without introducing arbitrary runtime specialization.

## Qualification

With Raven `cc64f787d`, the built-in type-relations (6), DI (21) and localization
(10) suites pass in interpreter, native and JIT execution: 111 test executions.
The DI/localization checks include provider-once resolution, independent closed
lifetimes, exact overrides, keys/plural order, cycle recovery and disposal.
Native source lint, API snapshot checks and the 5,622-page website build pass.
This is development qualification, not a matched release package or a new
performance claim. Array construction adds a focused JSON proof passing in all three modes: 114
built-in executions total. See [validation evidence](open-generic-validation.json).
The unrelated full JSON suite was stopped; no full-suite pass is claimed.
The final shared array inventory fix is qualified by the focused JSON proof;
unaffected DI/localization evidence is reused.
