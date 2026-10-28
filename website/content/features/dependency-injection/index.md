# Dependency injection: explicit ownership and lifetimes

Development (2026-10-28): this API now has its own `Neo.DependencyInjection.dll` assembly. Update former `System` imports and rebuild consumers with the matching libraries.

The development `Neo.DependencyInjection` API lets you register services,
build a provider and choose transient, scoped or singleton lifetimes. Automatic
constructor injection uses public constructors; explicit factories and existing
instances use the same registration model.

`DefaultServiceCollection` implements `ServiceCollection`. Typed helpers such as
`AddSingleton<Localizer<Feature>, Strings>()` and `AddTransient<Greeting>()` keep
service relationships visible. The built-in tests use these registrations to
inject a typed localizer into a greeting service.

## A greeting command with a scoped service

This sample registers settings once and lets the container construct a scoped
`Greeter`. A scope can represent a command or request. `?` propagates registration
and resolution failures; `use` releases the scope and provider when the function
returns, including an early error return.

```raven
module Examples.DependencyInjection

import System.*
import Neo.DependencyInjection.*
import System.Result.*

public class GreetingSettings {
    val Prefix: string

    init(prefix: string) {
        Prefix = prefix
    }
}

public class Greeter {
    private val settings: GreetingSettings

    init(settings: GreetingSettings) {
        self.settings = settings
    }

    func Greet(name: string) -> string => settings.Prefix + ", " + name + "!"
}

// A scope can represent one command, request or other bounded unit of work.
public func CreateGreeting(name: string) -> Result<string, ServiceError> {
    let services: ServiceCollection = DefaultServiceCollection()
    _ = services.AddSingleton<GreetingSettings>(GreetingSettings("Hello"))?
    _ = services.AddScoped<Greeter>()?
    use root = services.Build()?
    use scope = root.CreateScope()?
    let greeter = scope.GetRequiredService<Greeter>()?
    return Ok(greeter.Greet(name))
}
```

`CreateGreeting("Ada")` returns `Ok("Hello, Ada!")`. The sample's shared source is
compiled into the built-in DI tests, alongside the larger graph and disposal cases.
The [complete runnable project](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/dependency-injection/sample)
includes its entry point and build instructions.

## Lookups distinguish absence from failure

`GetRequiredService<T>()` returns the value or an error. `GetService<T>()` returns
None only when no registration exists; a failed factory remains an error.
`GetServices<T>()` returns every registration in order. A single lookup uses the
last registration, so an application can replace defaults deliberately.

Build takes an independent snapshot and prepares constructor plans without
creating services. Factories receive the resolving provider and return `Result`.
Cycles, invalid factory results and captive scoped dependencies are checked.

## A scope owns its services

Transient means one value per resolution, scoped means one per scope, and singleton
means one per root provider. Dispose scopes explicitly. Container-created disposable
services are released in reverse creation order; registered instances remain
caller-owned. A singleton cannot capture a scoped service through the root provider.

The API follows .NET DI conventions, adapted to neoCLR's explicit error model and
serialized guest execution. It does not add shared-object threading or async
factories. Constructor argument storage is allocated only when activating a
service; typed plural lookup reuses its result storage.

## Current boundaries

Native automatic construction requires explicit constructor retention, resolved
from exact source identities. Open generic constructor mappings close the
implementation with the requested service arguments. Exact closed registrations
win for single lookup; plural lookup preserves registration order. Singleton and
scoped caches are separate for each closed type. Native execution requires the
corresponding retained closed shapes. Collection injection and keyed
constructor-parameter annotations remain future work.

Read the [API contract](../../docs/dependency-injection.md) for signatures, errors,
ownership and native build configuration. The built-in dependency-injection suite
covers registration, constructor selection, typed localization, lifetimes and
reentrant disposal.

## Register a function by key

A bound function can itself be the service. The container returns the function
object; your code decides when to call it. This excerpt follows the built-in
function-service test:

```raven
let increment: (int) -> int = Increment
let doubleValue: (int) -> int = DoubleValue
_ = services.AddKeyedSingleton<(int) -> int>("increment", increment)
_ = services.AddKeyedSingleton<(int) -> int>("double", doubleValue)
```

Resolve with `provider.GetRequiredKeyedService<(int) -> int>("double")` and call
the returned value. A bound instance method also retains its receiver. Distinct
keys let methods share one structural function type without inventing wrapper
interfaces. The function value and a factory that creates services are separate
registration forms.

Keyed transient, scoped and singleton registrations use the same ownership rules.
Keys use ordinary value equality; null selects unkeyed services. Keyed lookup does
not fall back to an unkeyed registration, and unkeyed lookup excludes keyed values.

## Supply defaults without replacing application choices

`TryAddTransient`, `TryAddScoped`, `TryAddSingleton` and their keyed variants add
only when the exact service type/key is absent. They return `Ok(true)` when added
and `Ok(false)` when skipped; invalid new registrations return an error. These
helpers never execute factories while checking registrations.

The `Increment` and `DoubleValue` functions above may be declared directly at
module level. The built-in tests register both module-level functions and a bound
instance method, then resolve and invoke them through their structural function type.

Provider contracts, descriptors and typed helpers are available independently in
`Neo.DependencyInjection.Abstractions`. The default container remains in
`Neo.DependencyInjection`; namespaces are unchanged and consumers must rebuild.
