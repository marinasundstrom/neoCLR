---
toc: false
---

# Guides

Choose a topic after [building your first project](../try/). Each guide explains
observable behavior, shows a tested example where useful, and records limits and
relevant differences from .NET. Use the [API reference](../docs/) to look up exact members.

## Language and everyday code

| Guide | What you’ll learn |
| --- | --- |
| [Raven for neoCLR](../raven/) | Read the language examples and distinguish the two targets |
| [Option and Result](../features/outcomes/) | Match and propagate absence or expected errors |
| [Arrays](../features/arrays/) | Use shared array storage, bounds and collection capabilities |
| [Collections and queries](../features/collections/) | Read, replace, grow, filter and transform collections |
| [Strings and UTF-8](../features/strings/) | Distinguish graphemes, scalars and bytes |
| [Dates and clocks](../features/time/) | Validate civil dates and obtain a clock instant |

## Operations and runtime services

| Guide | What you’ll learn |
| --- | --- |
| [Tasks and async](../features/tasks/) | Await results, complete promises and understand isolated workers |
| [Files and Storage](../features/files/) | Resolve storage items and use byte/text streams |
| [Console and standard streams](../features/console/) | Handle input, output, end-of-input and typed errors |
| [Networking](../features/networking/) | Resolve a hostname and exchange bytes through a TCP client |
| [Web and HTTP](../features/web/) | Client/server exchanges, handlers, JSON and bounded stream uploads |
| [Introspection](../features/introspection/) | Discover types and members and understand descriptor identity |
| [Reflection](../features/reflection/) | Construct objects, invoke methods and access fields and properties |

## Development walkthroughs

These examples require the matching development toolchain. They are not promises
about the latest downloadable bundle.

- [Storage providers](../docs/storage-experiment.html): run the same workflow on disk
  and application-owned memory storage.
- [File Transformer](../docs/file-transformer.html): combine byte streams, text and JSON.
- [Pending reads](../docs/pending-read.html): inspect the controlled delayed-operation experiment.
- [Object and value contracts](../docs/objects.html): understand identity, equality and record behavior.
- [Terminal faults](../docs/faults.html): distinguish host failures from typed application errors.

See [development setup](../try/#development) for toolchain requirements. Broader ideas
belong in [direction and proposals](../proposals/), separate from implemented guides.

[Globalization](../features/globalization/) covers culture, language, system discovery and Gregorian/Hebrew rendering (provisional development APIs).
