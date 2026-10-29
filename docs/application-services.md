# Optional application services (development)

Author direction, 2026-10-28: application services use the `Neo` root namespace.
Each API family has its own project and assembly. Neo.Logging now adds the
[logging foundation](logging.md); Neo.Configuration adds the [configuration foundation](configuration.md).

The assembly/dependency table below describes the current abstraction split.

Both runtime and extension components use
`<runtime|extensions>/<component>/<src|test>/<project>/<namespace folders>`.
Project files sit inside their project directories. A component may group multiple
source projects and zero or more test projects; it could form its own solution,
but no solution files are required or added. For example, IO sources live under
`runtime/System.Runtime/src/System.Runtime/System/IO`. Raven test suites move to
the component whose API they exercise; the shared framework lives under
`tools/testing/NeoCLR.Testing`. The [suite catalog](../scripts/runtime-test-projects.json)
preserves command-line suite selectors. Each Neo component has an abstraction source project, an implementation source
project and its component tests.
Contracts and provider-neutral helpers live in the sibling `.Abstractions` projects;
default containers, JSON providers and console logging remain in implementations.
Culture and general formatting primitives remain in System.Globalization in
System.Runtime. System.Runtime and System.Data do not reference the Neo libraries.
The development SDK ships and references the Neo libraries through its generated
class-library props; consumers selecting references themselves can omit them.

## Comparison and tradeoffs

.NET places application DI in
[Microsoft.Extensions.DependencyInjection](https://learn.microsoft.com/en-us/dotnet/api/microsoft.extensions.dependencyinjection)
and separates [culture primitives](https://learn.microsoft.com/en-us/dotnet/core/extensions/globalization-and-localization)
from application resource services. neoCLR uses the author-selected `Neo` prefix
instead of Microsoft's vendor prefix. Optionality alone is not a universal rule
for System namespaces; the boundary here is application-service policy.

A namespace-only change would distinguish the API names but retain core assembly
coupling. Separate assemblies make that dependency explicit and allow consumers
to omit these services. The cost is additional artifacts and reference management.
The second slice separates abstractions from implementations, so contracts-only
localization consumers no longer depend on Data or DI.
No size, allocation or performance improvement is claimed.

## Internal JSON reader exception

System.Data grants InternalsVisibleTo to Neo.Localization so JsonLocalizerProvider
can call the existing internal StringMapReader. It preserves the direct flat-map
parser's 65536-byte/4096-entry limits, strict UTF-8/escape validation and duplicate
key errors without materializing a JSON DOM. The existing general JSON API has a
smaller default bound and different document semantics; substituting it is not a
behavior-preserving assembly move. Parser helpers remain internal rather than
becoming public solely to avoid the grant. This is production implementation
coupling, not a public extension contract. Revisit the grant when a suitable public
bounded JSON reading API supports the same contract and migrate this consumer first.

## Migration

Replace `System.DependencyInjection.*` and `System.Localization.*` imports with
`Neo.DependencyInjection.*` and `Neo.Localization.*`. Reference the new assemblies
and rebuild consumers against a matching compiler/library bundle. Old qualified
type identities have no aliases or forwarding shims. Update reflection and native
retention names, particularly ``Neo.Localization.DefaultLocalizer`1`` (generic arity
one). API URLs move from `/docs/api/System/` to `/docs/api/Neo/` for these families.
Historical validation artifacts retain their original identities and source paths.

The namespace move uses ordinary assembly identities. The abstraction split also
requires the native compiler callback-nullability fix described in the
[bridge notes](raven-cli-bridge.md); Runtime Contract configuration is unchanged. The native
reference is authoritative for these APIs; the older CLI scaffold still does not
project them. See the [DI reference](../api-docs/dependency-injection.md) and
[localization reference](../api-docs/localization.md).

## Validation

Focused evidence for this slice is recorded in
[the namespace and assembly migration qualification](validation/neo-services-20261028/README.md).

## Abstraction assemblies (second slice, 2026-10-28)

Each component now contains a sibling source project named
`Neo.<Component>.Abstractions`; namespaces remain `Neo.DependencyInjection`,
`Neo.Logging` and `Neo.Localization`. Consumers must rebuild against the new
assembly identities. No old-assembly type forwarders are supplied.

| Assembly | Contents | Dependencies |
| --- | --- | --- |
| Neo.DependencyInjection.Abstractions | Collection/provider contracts, descriptors, lifetimes/errors, typed registration/resolution helpers | System.Runtime |
| Neo.DependencyInjection | Default container and activation policy | System.Runtime, Neo.DependencyInjection.Abstractions |
| Neo.Configuration.Abstractions | Hierarchical settings, builder/source/provider contracts and helpers | System.Runtime |
| Neo.Configuration | Default builder/root/sections, memory provider and DI integration | System.Runtime, Neo.Configuration.Abstractions, Neo.DependencyInjection.Abstractions |
| Neo.Configuration.Json | Optional appsettings.json source/provider | System.Runtime, System.Data, Neo.Configuration.Abstractions, Neo.Configuration |
| Neo.Logging.Abstractions | Logger/factory/provider contracts, EventId/levels, typed adapter, literal helpers and null implementations | System.Runtime |
| Neo.Logging | Default factory, console provider and DI integration | System.Runtime, Neo.Logging.Abstractions, Neo.DependencyInjection.Abstractions |
| Neo.Localization.Abstractions | Localizer/factory contracts, typed adapter, formatting and missing-key decorators | System.Runtime |
| Neo.Localization | JSON providers/factory, loading errors and DI integration | System.Runtime, System.Data, Neo.Localization.Abstractions, Neo.DependencyInjection.Abstractions |

For example, the logging abstraction project is
`extensions/Neo.Logging/src/Neo.Logging.Abstractions/Neo.Logging.Abstractions.rvnproj`,
with sources below `Neo/Logging`. No new friend grants or public implementation
helpers are needed: descriptor matching stays private to the DI implementation.
The existing System.Data -> Neo.Localization friend grant remains solely for JSON
parsing; the localization abstraction assembly does not reference Data.

This follows the .NET split between
[DI abstractions](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.DependencyInjection.Abstractions/src/Microsoft.Extensions.DependencyInjection.Abstractions.csproj),
[logging abstractions](https://github.com/dotnet/runtime/tree/v10.0.0/src/libraries/Microsoft.Extensions.Logging.Abstractions)
and localization abstractions. Abstractions include small provider-neutral
adapters and helpers, not just interfaces. The benefit is a verifiable dependency
boundary; the cost is three additional assembly artifacts and rebuilding consumers.
The current SDK still references all bundled assemblies for convenience. Explicit
project references can select only the abstractions. No performance claim is made.

The [integrated sample](../samples/extensions/README.md) composes DI, console logging
and JSON localization; its separate compile-only consumer project references only
System.Runtime and the abstraction assemblies. AddConsole remains available through
an import of Neo.Logging, but its declaring extension type is now
ConsoleLoggingExtensions; LoggerFactoryExtensions is provider-neutral.

Native linking accepts the full split bundle; the former eight-dependency cap
is removed. Existing total type/method inventory bounds remain in place.

Native receiver discovery and the private verification projection now agree on
reachable boxing. A stack-only value constructor does not create an interface
receiver, and unreachable boxing does not retain executable interface views after
the corresponding methods are trimmed. Original conformance metadata and reflection
relationships remain intact; actual boxed values retain dispatch. This fixes
composition of file-backed localization and logging's Error formatting without
changing the .NET-style value boxing contract.

## Configuration foundation (2026-10-29)

Neo.Configuration.Abstractions references only System.Runtime. Neo.Configuration
references System.Runtime, its abstractions and Neo.DependencyInjection.Abstractions.
It adds ordered providers, hierarchical sections, an in-memory source and DI
registration. The integrated service consumes configuration through its abstraction.
See the [configuration scope and .NET comparison](configuration.md).
