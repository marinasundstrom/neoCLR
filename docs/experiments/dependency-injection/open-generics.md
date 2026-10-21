# Open generic localization integration — 2026-10-21

Status: implementation work, not a shipped contract.

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

## Intended shared implementation

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
