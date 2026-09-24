---
toc: false
---

# Guides

Choose a topic after [building your first project](../try/). Each guide explains
observable behavior, shows a tested example where useful, and records limits and
relevant differences from .NET. Use the [API reference](../docs/) to look up exact members.

## Language and everyday code

| Guide | What you’ll learn | Availability |
| --- | --- | --- |
| [Raven for neoCLR](../raven/) | Read the language examples and distinguish the two targets | Preview 9 |
| [Option and Result](../features/outcomes/) | Match and propagate absence or expected errors | Preview 9 |
| [Arrays](../features/arrays/) | Use shared array storage, bounds and collection capabilities | Preview 9; later development notes labeled |
| [Collections and queries](../features/collections/) | Read, replace, grow, filter and transform collections | Preview 9 |
| [Strings and UTF-8](../features/strings/) | Distinguish graphemes, scalars and bytes | Preview 9; later development notes labeled |
| [Dates and clocks](../features/time/) | Validate civil dates and obtain a clock instant | Preview 9 |

## Operations and runtime services

| Guide | What you’ll learn | Availability |
| --- | --- | --- |
| [Tasks and async](../features/tasks/) | Await results, complete promises and understand isolated workers | Preview 9 baseline; evolving development contracts |
| [Files and Storage](../features/files/) | Read bounded UTF-8 files and follow the provider model | Preview 9 file helpers; Storage and streams are development |
| [Console and standard streams](../features/console/) | Handle input, output, end-of-input and typed errors | Development after Preview 9 |
| [Introspection](../features/introspection/) | Discover types and members and understand descriptor identity | Preview 9 discovery; later equality and ownership changes are development |

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
