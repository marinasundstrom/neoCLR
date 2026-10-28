# Optional application services (development)

Author direction, 2026-10-28: application services use the `Neo` root namespace.
Each API family has its own project and assembly. Neo.Logging now adds the
[logging foundation](logging.md); Neo.Configuration remains a planned follow-up.

| Namespace and assembly | Project | Dependencies |
| --- | --- | --- |
| `Neo.DependencyInjection` | `extensions/Neo.DependencyInjection/src/Neo.DependencyInjection/Neo.DependencyInjection.rvnproj` | System.Runtime |
| `Neo.Logging` | `extensions/Neo.Logging/src/Neo.Logging/Neo.Logging.rvnproj` | System.Runtime, Neo.DependencyInjection |
| `Neo.Localization` | `extensions/Neo.Localization/src/Neo.Localization/Neo.Localization.rvnproj` | System.Runtime, System.Data, Neo.DependencyInjection |

Both runtime and extension components use
`<runtime|extensions>/<component>/<src|test>/<project>/<namespace folders>`.
Project files sit inside their project directories. A component may group multiple
source projects and zero or more test projects; it could form its own solution,
but no solution files are required or added. For example, IO sources live under
`runtime/System.Runtime/src/System.Runtime/System/IO`. Raven test suites move to
the component whose API they exercise; the shared framework lives under
`tools/testing/NeoCLR.Testing`. The [suite catalog](../scripts/runtime-test-projects.json)
preserves command-line suite selectors. This slice has one source and one test
project per Neo component.
DI and localization sources live under `extensions/Neo.DependencyInjection/src/Neo.DependencyInjection/Neo/DependencyInjection`
and `extensions/Neo.Localization/src/Neo.Localization/Neo/Localization`. Localization includes lookup contracts,
formatting extensions, JSON providers, diagnostics and DI registration in one
assembly. Culture and general formatting primitives remain in System.Globalization
in System.Runtime. System.Runtime and System.Data do not reference the Neo libraries.
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
Unlike a finer abstractions/provider split, this slice keeps each family together:
Neo.Localization therefore depends on Data and DI even for a contracts-only user.
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

No runtime intrinsic, Runtime Contract configuration or compiler bridge encoding
changes are needed; these are ordinary namespace and assembly identities. The native
reference is authoritative for these APIs; the older CLI scaffold still does not
project them. See the [DI reference](../api-docs/dependency-injection.md) and
[localization reference](../api-docs/localization.md).

## Validation

Focused evidence for this slice is recorded in
[the namespace and assembly migration qualification](validation/neo-services-20261028/README.md).
