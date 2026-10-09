---
toc: false
---

# Guides

Choose a topic after [building your first project](../try/). Each guide explains
observable behavior, shows a tested example where useful, and records limits and
relevant differences from .NET. Use the [API reference](../docs/) to look up exact members.

Coming from .NET? Start with the [platform comparison](../comparison/) for familiar
concepts, different contracts and compatibility limits.

## Platform foundations

- [Modules](../features/modules/) — organize declarations into modules that form namespaces.
- [Architecture](../architecture/) — compiler, runtime layers, services and execution modes.
- [Metadata format](../metadata/) — assembly ownership, PE/NEOX layers and validation.

## Cases: APIs in context

Small, runnable examples show how APIs work together to solve a concrete problem.
They need only enough application context to explain the choices and results.

- [Case: Building a Http server app](/cases/http-server/) — accept a station report,
  connect a client, and return a JSON acknowledgement. Start with the
  [general HttpClient example](/features/web/) for the client API overview.

## Language and everyday code

| Guide | What you’ll learn |
| --- | --- |
| [Raven for neoCLR](../raven/) | Read the language examples and distinguish the two targets |
| [Option and Result](../features/outcomes/) | Match and propagate absence or expected errors |
| [Function types](../features/functions/) | Delegates evolved: bind methods to structural signature types (development) |
| [Arrays](../features/arrays/) | Use shared array storage, bounds and collection capabilities |
| [Collections and queries](../features/collections/) | Read, replace, grow, filter and transform collections |
| [Strings and UTF-8](../features/strings/) | Distinguish graphemes, scalars and bytes |
| [Dates and clocks](../features/time/) | Validate civil dates and obtain a clock instant |
| [Globalization](../features/globalization/) | Cultures, calendars and localized formatting |

## Operations and runtime services

| Guide | What you’ll learn |
| --- | --- |
| [Native compilation](../features/native-compilation/) | Standalone ARM64 execution, supported workloads and limits |
| [Native benchmarks](../benchmarks/) | Measurements and methodology for interpreter/native comparisons |
| [Tasks and async](../features/tasks/) | Await results, complete promises and understand isolated workers |
| [Files and Storage](../features/files/) | Resolve storage items and use byte/text streams |
| [Console and standard streams](../features/console/) | Handle input, output, end-of-input and typed errors |
| [Networking](../features/networking/) | Resolve a hostname and exchange bytes through a TCP client |
| [Web and HTTP](../features/web/) | Client/server exchanges, handlers, JSON and bounded stream uploads |
| [Introspection](../features/introspection/) | Discover types and members and understand descriptor identity |
| [Reflection](../features/reflection/) | Construct objects, invoke methods and access fields and properties |
| [Garbage collection](../features/gc/) | Understand managed lifetimes and collection controls |

## Development walkthroughs

These examples require the matching development toolchain. They are not promises
about the latest downloadable bundle.

- [Inspect compiled metadata](https://github.com/marinasundstrom/neoCLR/blob/main/docs/il-inspection.md): development `neoclr disassemble` reads native PE, NEOX and JSON artifacts without executing them; output is diagnostic, not reassemblable.
- [Storage providers](../docs/storage-experiment.html): run the same workflow on disk
  and application-owned memory storage.
- [File Transformer](../docs/file-transformer.html): combine byte streams, text and JSON.
- [Pending reads](../docs/pending-read.html): inspect the controlled delayed-operation experiment.
- [Object and value contracts](../docs/objects.html): understand identity, equality and record behavior.
- [Terminal faults](../docs/faults.html): distinguish host failures from typed application errors.

See [development setup](../try/#development) for toolchain requirements. Broader ideas
belong in [direction and proposals](../proposals/), separate from implemented guides.
