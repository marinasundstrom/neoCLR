---
toc: false
---

# Guides

Start with a working application, then follow the APIs it uses.
[Install and run a sample](/try/) if this is your first visit.

<a id="cases-apis-in-context"></a>

## Build an application

| I want to… | Start here | Then explore |
| --- | --- | --- |
| Call or serve an HTTP API | [Web and HTTP](/features/web/) | [Server and JSON case](/cases/http-server/) |
| Query a PostgreSQL database | [SQL databases](/features/sql/) | [Runnable PostgreSQL case](/cases/postgresql/) |
| Read files and text | [Files and storage](/features/files/) | [Streams](/docs/streams.html) |
| Coordinate operations | [Tasks and async](/features/tasks/) | [Cancellation and timers](xref:System.Concurrency) |
| Publish native code | [Native compilation](/features/native-compilation/) | [Performance measurements](/benchmarks/) |

<a id="language-and-everyday-code"></a>

## Work with data

- [Collections and queries](/features/collections/) — lists, maps, sets and transformations.
- [Arrays](/features/arrays/) and [indexing and slicing](/docs/sequence-access.html).
- [JSON](/docs/json.html) — nodes, models and serialization.
- [Strings and UTF-8](/features/strings/) — characters and explicit encoding.
- [Configuration](/features/configuration/) — layered settings, sections and DI.
- [Localization](/features/localization/) — JSON resources, scoped lookup and fallback.
- [Dates and times](/features/time/) and [globalization](/features/globalization/).
- [Optional and Result](/features/outcomes/) — absence and expected errors.
- [Error handling](/features/error-handling/) — domain errors, propagation and optional context diagnostics.

## Understand the platform

Read [Raven for neoCLR](/raven/) for the language used in the examples.
Coming from C#? The [.NET comparison](/comparison/) explains familiar concepts
and deliberate differences.

[Reference types and value types](/features/types/) explain copying, identity,
boxing and equality. [Modules](/features/modules/) organize declarations.
[Introspection](/features/introspection/) describes them;
[reflection](/features/reflection/) can construct objects and invoke members.
[neoIL](/features/neoil/) demonstrates the runtime contracts in executable IL, including structural function types.
[Architecture](/architecture/) explains the compiler and runtime layers.

## Go deeper when you need to

Use the [API reference](/docs/) for exact signatures, errors and limits.
Specialized guides cover [TCP networking](/features/networking/),
[TLS](/docs/ssl-streams.html), [native interop](/features/native-interop/),
[garbage collection](/features/gc/) and [metadata](/metadata/).
[Proposals](/proposals/) describe ideas that are not implemented contracts.
