---
title: Logging
---
# Logging (development)

`Neo.Logging.dll` supplies optional logging abstractions and a synchronous default
implementation. Import `Neo.Logging.*`; the assembly references System.Runtime
and Neo.DependencyInjection. It preserves the generic state boundary of
Microsoft.Extensions.Logging, following Raven's convention of interfaces without
an I prefix.

| Type | Contract |
| --- | --- |
| `LogLevel` | Trace=0, Debug=1, Information=2, Warning=3, Error=4, Critical=5, None=6. None never enables logging. |
| `EventId` | Value with Id and optional Name. Constructors accept id or id/name. Equality and hashing use only Id; ToString uses Name when non-null, otherwise the numeric Id. |
| `Logger` | `Log<TState>(level, eventId, state, error, formatter)`, `IsEnabled(level)`, `BeginScope<TState>(state)`. |
| `Logger<T>` | Logger with a declared category type. |
| `LoggerProvider` | CreateLogger(categoryName) and Dispose. |
| `LoggerFactory` | CreateLogger(categoryName), AddProvider(provider), Dispose. |
| `DefaultLoggerFactory` | Empty provider set and Information threshold by default; overload accepts a minimum LogLevel, including None. Invalid thresholds fault. |
| `DefaultLogger<T>` | Adapter constructed with a LoggerFactory; uses typeof(T).DisplayName as its category. |
| `ConsoleLoggerProvider` | Default Console.WriteLine sink, or supplied `(string) -> unit` sink. Synchronous output; disposing disables its loggers and leaves the sink open. |
| `NullLogger`, `NullLogger<T>` | Instance singleton; always disabled, never evaluates formatters, returns a reusable no-op scope. |
| `NullLoggerFactory` | Instance singleton; returns NullLogger, ignores providers without taking ownership, and permits no-op disposal. |

`Log<TState>` takes `(TState, Error?) -> string` as its formatter and an optional
System.Error. The default factory filters before invoking providers; enabled
providers decide when to render state. Each enabled console write produces
`level: category[id] message`, followed by an optional formatted diagnostic on a
new line. No stack trace is captured implicitly. Category and message retain
UTF-8 text. Event names do not change numeric console event identifiers.

The factory caches category loggers using ordinal, case-sensitive names. Added
providers immediately participate in existing categories. Every AddProvider
registration transfers disposal ownership; registering twice produces two
outputs and two disposal calls. Factory disposal is idempotent and disables
existing aggregate loggers. CreateLogger and AddProvider after disposal fault.

BeginScope forwards state to every provider, including providers currently
disabled for a severity. Null handles mean unsupported scopes. The aggregate
handle disposes every returned handle once, in registration order. The default
console provider does not render scopes. There is no ambient async scope flow.

`LoggerExtensions` provides literal Log overloads and LogTrace, LogDebug,
LogInformation, LogWarning, LogError and LogCritical. Braces are literal text;
message-template parsing is not implemented. LoggerFactoryExtensions adds
`CreateLogger<T>()` and `AddConsole()`.

## Dependency injection

`ServiceCollection.AddLogging()` registers a container-owned singleton
DefaultLoggerFactory and singleton open-generic `Logger<> -> DefaultLogger<>`.
`AddLogging(factory)` registers a caller-owned factory instead. Both return
`Result<unit, ServiceError>`, preserve existing registrations, and can be repeated.
Resolve LoggerFactory to add providers during startup. Explicit typed logger
registrations take precedence over the open-generic fallback.

Native reflection activation needs constructor retention for consumers,
`Neo.Logging.DefaultLoggerFactory`, and ``Neo.Logging.DefaultLogger`1``.
The project builder accepts these exact names via --reflection-constructor-type.

The foundation requires serialized access. Configuration integration, concurrent
mutation, ambient scopes, generated logging, provider exception aggregation and
message templates remain gaps. Typed generic category spelling can differ from
.NET; pass an explicit string category for a stable external name. See the
[design and comparison](https://github.com/marinasundstrom/neoCLR/blob/main/docs/logging.md)
and [feature overview](/features/logging/).

The initial typed logger contract is invariant; .NET typed-category covariance
and EventId implicit integer conversion are not yet provided. Scope generic
parameters do not encode the CLR nullable-analysis `notnull` constraint.
