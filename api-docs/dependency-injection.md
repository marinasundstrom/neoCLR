---
title: Dependency injection
---
# Dependency injection (development)

Contracts live in `Neo.DependencyInjection.Abstractions`; the default container
lives in `Neo.DependencyInjection`. `ServiceCollection` and
`ServiceProvider` define registration and resolution; `DefaultServiceCollection`
is the default implementation. Import `Neo.DependencyInjection.*` for typed
registration and lookup extensions. This development API follows .NET DI's
service identities, registration order, constructor selection and lifetimes,
using neoCLR `Result` and `Optional` contracts.


## Try it

Register an existing instance, build a provider, then resolve it by its service type. The caller owns supplied instances; `use` disposes the provider.

```raven
module Examples.Extensions.Introductions.DependencyInjection

import System.*
import System.Result.*
import Neo.DependencyInjection.*

class Message {
    val Text: string
    init(text: string) { Text = text }
}

func Main() -> int {
    let services: ServiceCollection = DefaultServiceCollection()
    let Ok(_) = services.AddSingleton<Message>(Message("Hello from DI")) else { return 1 }
    let Ok(provider) = services.Build() else { return 1 }
    use owned = provider
    let Ok(message) = provider.GetRequiredService<Message>() else { return 1 }
    Console.WriteLine(message.Text)
    return 0
}
```

[Complete sample project](https://github.com/marinasundstrom/neoCLR/tree/main/samples/extensions/src/DependencyInjectionIntro) · [Build and run instructions](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/README.md#introductory-examples)

## Registration and activation

`ServiceDescriptor(service, source, lifetime)` is immutable. `ServiceSource`
selects `Constructor(TypeInfo)`, `Factory((ServiceProvider) -> Result<Object,
ServiceError>)` or `Instance(Object)`. A collection accepts descriptors through
`Add`, or uses `Add(service, implementation, lifetime)`, `AddFactory` and
`AddInstance`. Invalid types, assignability, lifetimes and instance identities
return `InvalidRegistration`. Instance descriptors require singleton lifetime.
Service instances are reference types with fully bound identities. Open generic
service/implementation definitions are supported for constructor registrations;
value-type services are not supported.

`AddTransient<TService, TImplementation>()`, `AddScoped<...>()` and
`AddSingleton<...>()` register implementations. Their single-type overloads
register the type as its own implementation. Factory overloads accept
`(ServiceProvider) -> Result<TService, ServiceError>`; `AddSingleton(instance)`
retains a caller-owned instance. Registration returns `Result<unit, ServiceError>`.

`Build()` returns `Result<ServiceProvider, ServiceError>`. It snapshots the
registrations, so later additions do not affect an existing provider. Constructor
plans select the public constructor with the most available dependencies. Another
applicable constructor is ambiguous when it requires a type absent from that
selection. Build does not call constructors or factories. Optional parameter
defaults, collection injection and keyed parameter annotations are not implemented. `ActivationError` distinguishes unsupported types, missing or
ambiguous constructors and invocation failures.

Factories receive the current provider. A singleton factory always receives the
root provider, even when requested from a scope. Factory results are checked for
assignability before being cached or returned. Invalid results return
`InvalidFactoryResult`; cyclic resolution returns `Cycle`. Reported failures reset
resolution state so subsequent calls can retry. Guest terminal faults retain the
runtime's terminal-failure behavior; they are not converted into ServiceError.

## Resolution

| Member | Result |
| --- | --- |
| `GetRequiredService(TypeInfo)` | `Result<Object, ServiceError>`; Missing when unregistered |
| `GetService(TypeInfo)` | `Result<Optional<Object>, ServiceError>`; None only when unregistered |
| `GetServices(TypeInfo)` | `Result<Sequence<Object>, ServiceError>`; empty when unregistered |
| Corresponding `<T>()` extension | The same behavior with typed values |

Single lookup uses the last registration. Plural lookup preserves registration
order; separate singleton registrations retain separate instances. All lookup
forms propagate activation and factory errors. `GetServices<T>` projects its
result storage without a second element-array copy. ServiceProvider itself is
implicitly available for constructor injection and single lookup, but cannot be
explicitly registered and is not included in plural lookup.

## Lifetimes and disposal

Transient registrations create a value on each resolution. Scoped registrations
cache within one scope. Singletons cache within the root provider and are shared
by all its scopes. `CreateScope()` returns an independently disposable provider;
a scope created from another scope shares the same root, not the parent's cache.
Root resolution of scoped services always returns ScopeRequired. This deliberate
always-on validation prevents captive scoped dependencies in singleton activation;
.NET exposes configurable scope validation.

Dispose each created scope. Created disposable services are released once per
provider, in reverse creation order; duplicate references within that provider
are deduplicated. Explicit instances remain caller-owned. A factory transfers
ownership of its result: avoid returning caller-owned disposable objects through
a factory or aliasing ownership across providers. Prefer instance registration
for external ownership. Root disposal invalidates live scopes but does not dispose
their owned services. Disposal is idempotent. Resolution and scope creation after
disposal return Disposed. A factory that disposes its provider cannot publish a
new result; the result is released if necessary.

Resolution is synchronous and invocation-owned, consistent with serialized guest
continuations and isolated worker heaps. Providers are not a shared-object
threading contract. Async factories/disposal are not supported. Registration
lookup is linear; activation uses one argument array and ordinary reflection.
There is no allocation-free or optimized-throughput claim.

## Native execution

Automatic construction requires constructor metadata and executable retention.
The native build helper accepts repeatable
`--reflection-constructor-type Full.Type.Name` arguments, resolved from the final
compiled application. The private schema-7 policy sets `constructors` and
`construct`. Do not reuse metadata indices from a different compilation.

The current native retention contract supports nongeneric implementation
declarations with supported scalar/reference constructor arguments. Closed generic
service interfaces, such as `Localizer<Feature>`, are ordinary service identities.
Generic implementation construction requires further native retention work;
explicit factories can construct types already supported by ordinary codegen.
Missing constructor metadata remains an explicit runtime fault, not an empty
constructor list. General reflective method invocation is a separate capability.

## Keyed services and function objects

`ServiceDescriptor.ServiceKey` is an optional Object key. The four-argument
constructor accepts `(service, key, source, lifetime)`. Null denotes the ordinary
unkeyed identity, matching .NET. Other keys use normal neoCLR Object equality;
key equality must remain stable while registered. Primitive value keys box at the
Object boundary. No AnyKey wildcard or implicit fallback is provided.

`AddKeyed`, `AddKeyedFactory` and `AddKeyedInstance` are the runtime-type
registration forms. Typed `AddKeyedTransient`, `AddKeyedScoped` and
`AddKeyedSingleton` provide implementation, self-registration and factory forms;
the singleton form also accepts an instance. Keyed factories receive
`(ServiceProvider, Object?)`, with the registered key as their second argument.

`GetRequiredKeyedService<T>(key)`, `GetKeyedService<T>(key)` and
`GetKeyedServices<T>(key)` select only that type/key pair. Runtime-type forms take
`(TypeInfo, Object?)`. Last-registration and ordered-plural rules apply within each
pair, and caches remain per registration. Unkeyed lookup excludes non-null keys.
Missing required keyed lookup returns `MissingKeyed(type, key)`; null-key missing
lookup retains Missing. Factory failures remain errors in optional lookup too.
Automatic constructor parameters currently resolve unkeyed dependencies; keyed
parameter annotations and key inheritance are not implemented. A factory can
explicitly resolve a keyed dependency.

Bound Function objects are services. Register a value of `(int) -> int` with
`AddKeyedSingleton<(int) -> int>(key, function)`, retrieve that same type, and invoke
the returned function normally. The registration does not execute the function.
Multiple methods sharing one structural signature can therefore use different
keys. Bound instance methods retain their receiver. Registering a function does
not transfer disposal ownership of its captured receiver; register an owned
receiver separately when the provider should dispose it. Function objects have no
automatic-constructor registration: use instance or factory registration.

The key model follows [.NET keyed DI](https://learn.microsoft.com/en-us/dotnet/core/extensions/dependency-injection/overview).
The deliberate differences are Result/Optional outcomes, neoCLR's own value equality
(including canonical text equality), always-on scope validation and the documented
subset of constructor/retention support.

## Conditional defaults

`ServiceCollection.TryAdd(ServiceDescriptor)` returns `Result<bool, ServiceError>`:
true when added, false when the exact service type/key already exists, or an error
when a new descriptor is invalid. Existing registrations win regardless of lifetime
or activation source. A duplicate is skipped before validating its replacement.
Factories are never invoked by registration checks.

Typed `TryAddTransient`, `TryAddScoped`, `TryAddSingleton` and their `TryAddKeyed*`
forms mirror the Add helpers, including implementation mappings, self registration,
factories and singleton instances. Runtime-type helpers include `TryAdd`,
`TryAddFactory`, `TryAddInstance` and keyed equivalents. Null keys share the unkeyed
identity; non-null keys use ordinary equality. The bool result is a deliberate
neoCLR addition to .NET's conditional-registration convention. `TryAddEnumerable`
and wildcard keys are not part of this foundation. Registration may allocate a
descriptor or factory adapter even when skipped; no allocation-free claim is made.

Registration checks use `TypeInfo.ContainsGenericParameters`, the shared recursive
identity query. Inheritance openness (`IsOpen`) does not reject an otherwise closed
service. Open generic constructor mappings are supported; open factory/instance registrations are rejected.

## Tested application example

The [DI feature page](/features/dependency-injection/) includes a complete scoped
greeting example. Its source is shared with the built-in container tests; the
[runnable project](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/dependency-injection/sample)
shows the exact native constructor-retention option.

## Open generic mappings (development)

Register `typeof(Service<>)` with `typeof(Implementation<>)` using a constructor
source and the desired lifetime. Both must be generic definitions of matching
arity. Resolution closes the implementation with the requested service arguments,
checks constraints and assignability, then uses the ordinary constructor plan.
Exact closed registrations take precedence for single lookup. Plural lookup
includes matching closed and open registrations in registration order.

Caches and cycle detection are per registration and closed service identity;
keys and lifetimes retain their existing rules. Invalid closure returns
InvalidRegistration; constructor-plan failures return Activation. Native programs
must retain the generic implementation constructors and the required closed
service shapes. Unretained construction is an explicit failure, not dynamic code
generation. Collection injection and keyed constructor annotations remain gaps.

## Namespace and assembly migration (2026-10-28)

Replace System.DependencyInjection imports with Neo.DependencyInjection and rebuild
against the new assembly. No old-name compatibility shim is provided. See the
[application-service boundary](https://github.com/marinasundstrom/neoCLR/blob/main/docs/application-services.md) for dependencies and migration.


Development assembly split (2026-10-28): contracts, descriptors, lifetime/error
types and typed registration/resolution helpers live in
Neo.DependencyInjection.Abstractions.dll, referencing only System.Runtime.
Neo.DependencyInjection.dll contains the default container and constructor activation.
Namespaces remain unchanged; rebuild consumers against the new assembly identities.
No new friend access or public implementation helper is introduced.
