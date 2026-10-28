# Dependency injection foundation

Author direction, 2026-10-21: commit localization first, then implement dependency
injection including automatic constructor injection. Localization is committed as
`da9e6aec7`. The initial internal activation experiment is now integrated into the development
class library. See the [public contract](../../../api-docs/dependency-injection.md)
for the current API and limits.

## Constructor plans

[ConstructorPlan.rvn](../../../extensions/Neo.DependencyInjection/src/Neo.DependencyInjection/Neo/DependencyInjection/ConstructorPlan.rvn) separates availability checks from
activation. Preparation selects the most-parameter public instance constructor
whose dependency types are available, then checks competing applicable
constructors. It retains the selected constructor and parameter metadata for reuse.
Activation asks the resolver for each argument and invokes that exact constructor
through the existing public reflection contract. No additional runtime activation
service or compiler-specific encoding is introduced.

Compare the [.NET DI overview](https://learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection/)
and [default container constructor selection](https://github.com/dotnet/runtime/blob/main/src/libraries/Microsoft.Extensions.DependencyInjection/src/ServiceLookup/CallSiteFactory.cs).
The intended container follows registration, transient/scoped/singleton lifetime,
and ownership conventions. The API uses Result errors instead of exceptions,
consistent with current neoCLR APIs. Competing constructors are checked by their
parameter-type sets, matching the default .NET container.
Optional defaults and enumerable injection are not implemented. Open generic constructor mappings are being qualified in [the integration record](open-generics.md).
Keyed services and conditional TryAdd registrations are supported. Closed typed dependencies such as Localizer<Feature> are ordinary
service identities. Abstract/interface/value implementation types are rejected; open definitions require an open generic constructor mapping.
Inheritance-open classes remain valid; TypeInfo.IsOpen is not a generic-state flag.

Preparation does not instantiate dependencies. Activation reserves one argument
array and passes it directly to exact constructor invocation, avoiding an
intermediate list and reflection snapshot. Plans are cached per provider snapshot.

## Class-library integration

[Services.rvn](../../../extensions/Neo.DependencyInjection/src/Neo.DependencyInjection/Neo/DependencyInjection/Services.rvn)
provides public ServiceCollection, ServiceProvider and immutable ServiceDescriptor
contracts. Registrations accept implementation types, factories or caller-owned
instances, including module-level functions and bound instance methods. Generic
Add/TryAdd lifetime helpers share the same registration and key-equality paths.
Build snapshots descriptors and caches constructor plans without activating services.

GetRequiredService reports Missing, GetService returns None only for an absent
registration, and GetServices preserves registration order. A single lookup uses
the last registration. Factory and activation errors propagate through all three.
Typed plural results project existing storage instead of copying a second array.

Transient objects are fresh, scoped objects belong to independent scopes, and
singletons belong to the root. Root scoped resolution rejects (always-on scope
validation, unlike .NET's configurable policy). Current-provider injection is
implicit. Cycles reject without poisoning subsequent resolution. Created disposable
services are tracked once per provider and disposed in reverse creation order;
registered instances remain caller-owned. Reentrant disposal during factory
execution rejects publication and releases newly created results. Root disposal
invalidates live scopes; callers still dispose those scopes to release their objects.

This is invocation-owned infrastructure following serialized guest execution;
isolated workers cannot share provider objects. Lookup is linear and reflective
activation allocates argument storage. No allocation-free or shared-object
concurrency claim is made.

## Remaining integration gaps

- TypeInfo generic-state flags supply shared openness checks. MethodInfo flags and
  descriptive generic method discovery are implemented separately; DI constructor
  activation does not require retaining ordinary methods.
- Native automatic construction currently retains nongeneric declarations only.
- Open generic registration, enumerable constructor injection and keyed constructor
  annotations remain beyond the closed-service foundation.

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

All 13 built-in tests pass in the interpreter using native compiler `d14b7340a`,
including typed factory callbacks, conventional lookups, independent singleton
slots, current-provider injection, scope/cycle failures and reentrant disposal.
See [container test report](services-runtime-tests.json) and
[output](services-runtime-tests.log). Earlier four-test activation evidence is
retained separately. Native activation now also passes all 13 tests; see [native report](services-native-tests.json).
Exact invocation uses schema-7 constructor and execution retention; final-image
name resolution prevents stale discovery ordinals. The same 13-test prototype also passed JIT qualification before public integration.

The native compiler correction substitutes explicit generic extension arguments
inside structural-function signatures. Before the fix a block-bodied factory kept
an unsubstituted TService result; expression-bodied and .NET cases passed. All 11
focused compiler tests pass afterward. Runtime Contract, metadata and ABI remain
unchanged; see the [integration note](../../raven-cli-bridge.md).

The public integration refreshes native API documentation and the website. The
suite now participates in the default qualified run and remains selectable with
`--suite dependency-injection`. Historical
prototype evidence above is retained separately from current public API evidence.

## Public integration evidence

The public suite has 18 built-in tests, including keyed function instances, TryAdd
default preservation and a diamond graph across independent scopes. All 18 pass on each of the interpreter, native executable and JIT (54 executions).
See the [interpreter report](public-interpreter-tests.json) and
[native/JIT report](public-native-jit-tests.json). Native qualification initially exposed missing structural
Function handling in reference `unbox.any` projection; the backend now routes it
through the existing checked reference cast, retaining normal null/type behavior.
The matching native API snapshot check passes and the website build checks 5,564 pages.

Both focused backend projection regressions pass: function `unbox.any` becomes a
checked reference cast, while whole-union boxing retains its structural identity
restriction (`cargo test -p neoclr-codegen --lib boxing::structural_union_tests`).

## Tested website sample — 2026-10-21

The complete [greeting project](sample/README.md) shares its implementation with
the built-in DI suite. All 19 tests pass in interpreter/native/JIT (57 executions);
[foundation report](foundation-tests.json) also contains the earlier two-test
method-discovery run. The separately built native executable prints `Hello, Ada!`;
[build provenance](sample-build.json) and [output](sample-output.log) are retained.

The author subsequently requests registering a provider against `Localizer<>`
and resolving arbitrary closed `Localizer<HomePage>` services. This supersedes
our earlier open-generic deferral: shared generic definition/construction metadata,
closed implementation activation and cache slots per registration/closed service
are now required before DI/localization closure. The sample above qualifies the
existing closed-service path, not this new requirement.

## Open generic foundation closure — 2026-10-21

The subsequent [open-generic implementation](open-generics.md) now qualifies all
21 DI tests and 10 localization tests in interpreter, native and JIT execution.
A caller-owned singleton factory and transient open adapter resolve two unrelated
typed resource scopes through ordinary constructor injection. Native constructor
retention remains explicit; broader runtime specialization and matched-package
release qualification are separate work.
