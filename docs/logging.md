# Neo.Logging foundation (development)

Neo.Logging is an optional assembly in
`extensions/Neo.Logging/src/Neo.Logging`, with Raven tests in the component's
`test/Neo.Logging.Tests` project. Its implementation references System.Runtime, Neo.Logging.Abstractions and
Neo.DependencyInjection.Abstractions. Provider-neutral contracts, typed/null adapters
and literal helpers live in the sibling Neo.Logging.Abstractions project, referencing
only System.Runtime. Configuration binding remains future work; output-folder
changes were explicitly deferred by the author.

## Baseline and decisions

The baseline is Microsoft.Extensions.Logging in .NET 10:
[ILogger](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Logging.Abstractions/src/ILogger.cs),
[LoggerFactory](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Logging/src/LoggerFactory.cs),
[Logger](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Logging/src/Logger.cs), and
[EventId](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/Microsoft.Extensions.Logging.Abstractions/src/EventId.cs).
The public boundary preserves generic `Log<TState>` and `BeginScope<TState>`.
The author explicitly chose compiler/runtime support over an object-state
provider workaround. Native dispatch therefore retains the method type arguments;
providers receive state and a formatter separately. Filtering precedes formatting.

A string-only contract would be smaller but lose structured state and force eager
formatting. An object-state contract would add boxing and erase the provider's
compile-time state type. Explicit generic state also leaves room for message
parsing and generated logging without committing to a parser now. Serilog's
[structured events](https://github.com/serilog/serilog/wiki/Writing-Log-Events) and
SLF4J's [lazy/key-value logging](https://slf4j.org/manual.html) support retaining
state separately from rendering; neither API is substituted for the .NET baseline.

Raven interfaces omit the I prefix: Logger, `Logger<T>`, LoggerProvider and
LoggerFactory. DefaultLoggerFactory is the concrete factory. It caches ordinal
category names, defaults to Information, and starts without providers. Adding a
provider updates existing loggers. Each registration is owned and disposed once
by the factory; duplicate registrations are intentionally not deduplicated.
NullLogger and `NullLogger<T>` are reusable, disabled singletons.

ConsoleLoggerProvider is an explicitly added, synchronous provider. It renders
UTF-8 text to Console.WriteLine or a supplied `(string) -> unit` sink. The sink
remains caller-owned. Optional `System.Error` diagnostics use FormatError, with
no implicit stack capture. EventId equality and hashing use only its numeric Id;
Name is descriptive. No global service or ambient logger is installed.

## Current gaps relative to .NET

- Factory and provider mutation require serialized owner-thread access. .NET's
  thread-safe factory and ambient async scope storage are not yet implemented.
- BeginScope forwards typed state and aggregates provider handles. The console
  provider has a no-op scope. There is no built-in ambient scope provider.
- Literal helpers do not parse message templates or provide generated logging.
  Structured callers use `Log<TState>` directly.
- Errors are neoCLR diagnostic Error values instead of CLR Exception values.
  Terminal provider faults follow the runtime's fail-fast behavior; the .NET
  factory's catch-and-aggregate provider exception behavior is not implemented.
- Typed categories use TypeInfo.DisplayName. Generic/nested names may differ from
  the .NET category formatting convention; stable public categories can be passed
  directly to CreateLogger(string).
- Disposing the default factory disables previously created aggregate loggers.
  Creating loggers or adding providers afterward faults. No provider disposal
  exceptions are recovered.
- Typed `Logger<T>` covariance, implicit EventId integer conversion and the CLR
  nullable-analysis notnull scope constraint remain API gaps.
- No configuration binding, per-category rules, filters by provider, reload,
  background queues or external providers are included in this foundation.

These are development gaps, not permanent replacements for the .NET contract.
See the [API guide](../api-docs/logging.md) for registration and ownership details.
