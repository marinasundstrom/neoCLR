# Dependency injection foundation

Author direction, 2026-10-21: commit localization first, then implement dependency
injection including automatic constructor injection. Localization is committed as
`da9e6aec7`. This increment starts with internal activation infrastructure and
built-in contract tests; it does not publish ServiceCollection/ServiceProvider yet.

## Constructor plans

[ConstructorPlan.rvn](ConstructorPlan.rvn) separates availability checks from
activation. Preparation selects the most-parameter public instance constructor
whose dependency types are available, then checks competing applicable
constructors. It retains the selected constructor and parameter metadata for reuse.
Activation asks the resolver for each argument and invokes that exact constructor
through the existing public reflection contract. No additional runtime activation
service or compiler-specific encoding is introduced.

Compare the [.NET DI overview](https://learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection/)
and [default container constructor selection](https://github.com/dotnet/runtime/blob/main/src/libraries/Microsoft.Extensions.DependencyInjection/src/ServiceLookup/CallSiteFactory.cs).
The intended container follows registration, transient/scoped/singleton lifetime,
and ownership conventions. This prototype uses Result errors instead of exceptions,
consistent with current neoCLR APIs. Competing constructors are checked by their
parameter-type sets, matching the default .NET container.
Optional defaults, keyed services, enumerable injection and open registrations are
not implemented. Closed typed dependencies such as Localizer<Feature> are ordinary
service identities. Abstract/interface/open/value implementation types are rejected.

Preparation does not instantiate dependencies. Activation allocates its result and
argument storage; currently the Sequence reflection overload also snapshots its
arguments. This is a correctness foundation, not an allocation improvement claim.
A subsequent container should cache plans and use indexed dependency slots; it
must not share mutable argument arrays across concurrent resolutions.

## Next integration gates

- ServiceCollection and ServiceProvider interfaces and default implementations.
- Immutable registration snapshot, explicit factories/instances and automatic
  constructor registrations, with checked implementation assignability.
- Cycle detection, scoped/singleton caches and captive-scope rejection.
- Ownership: caller-owned instances stay caller-owned; container-created disposable
  services are disposed once in reverse creation order; use-after-disposal rejects.
- Concurrency and native/AOT retention validation before advertising server use.
- API reference and website documentation when the public contracts are added.

## Compiler gap fixed during integration

The direct callback `type => Some<Object>(strings)` exposed missing traversal of
construction payloads in Raven's shared bound-tree walker. This was a general
capture-analysis defect, not a native-only constraint. Raven main `69dde95e3`
visits ordinary constructor arguments, initializer entries and union-case payloads;
its deliberately carried native counterpart is `5f6e7a29c`. The direct callback
is restored in the DI fixture, with no resolver-method workaround.

The five existing capture tests passed before the change; three new union-payload
cases reproduced missing captures. After the fix, 77 focused capture/union tests
pass, and all five new construction regressions (including ordinary constructors
and object initializers) pass. Tests check semantic capture identity and emitted
.NET execution; see [focused regressions](raven-regressions.log) and
[construction regressions](raven-construction-tests.log). No Runtime Contract,
metadata encoding or closure ABI changes.

## Validation

All four built-in tests pass in the interpreter with native compiler `5f6e7a29c`: selection, retained activation with
Localizer<Feature>, ambiguous/missing/private/interface rejection, and missing or
wrongly typed resolved arguments. Native/JIT activation is not yet qualified.
See [test report](runtime-tests.json) and [output](runtime-tests.log). No public
API changed in this internal experiment, so no API snapshot refresh or website
build is required for this increment.

Run explicitly with `scripts/test-runtime-library.py --suite dependency-injection
--mode interpreter` and a matching bundle/output directory. This internal suite
is opt-in until native constructor retention has its own qualification.
