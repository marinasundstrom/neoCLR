# Logging: typed state and explicit providers

Development (2026-10-28): `Neo.Logging` is an optional application-service
assembly built around the Microsoft.Extensions.Logging abstractions.

A Logger receives an event level, event identity, typed state, an optional Error
and a formatter. DefaultLoggerFactory filters before formatting and sends events
to the enabled providers. `Logger<T>` selects a category from the declared type;
string categories are available when a stable external name matters.

The default factory starts with an Information threshold and no providers.
AddConsole installs a synchronous UTF-8 console provider. Null loggers provide
reusable disabled implementations. AddLogging registers the factory and typed
loggers with Neo.DependencyInjection while preserving explicit overrides.

Providers can receive typed scope state and return disposable scope handles.
Ambient async scopes, message-template parsing, generated logging, configuration
binding and concurrent factory mutation remain future work. This foundation
requires serialized access and follows the runtime's terminal-fault behavior.

See the [logging API guide](/docs/logging.html) for signatures, ownership,
filtering, disposal and current differences from .NET.

Provider-neutral contracts and helpers live in `Neo.Logging.Abstractions`; the
default factory, console provider and DI registration live in `Neo.Logging`.
Logging and localization integration depend on `Neo.DependencyInjection.Abstractions`.
See the [integrated application sample](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/README.md).
