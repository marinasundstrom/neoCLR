<a id="possible-platform-directions"></a>

# Project direction

These are open areas of development and exploration. For what you can use today,
start with the [guides](../guides/). The repository's
[roadmap](https://github.com/marinasundstrom/neoCLR/blob/main/docs/platform-roadmap.md)
tracks priorities and implementation work; the ideas here are not release commitments.

<a id="http-poc"></a>

## HTTP and networking

The [HTTP library](../features/web/) supports bounded, cleartext HTTP/1.1 exchanges.
Broader streaming, TLS, persistent connections and newer HTTP protocols would support
more applications. Each needs suitable resource ownership, cancellation and error
contracts.

[HTTP design and capabilities](https://github.com/marinasundstrom/neoCLR/blob/main/docs/http-capabilities.md)

## Text

[UTF-8 strings](../features/strings/) distinguish graphemes, scalars and bytes.
Further work includes normalization, richer text positions and language-sensitive
operations. These need to fit the existing text model and make scanning and allocation
costs understandable.

[Text design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/text-model.md)

## Introspection

[Introspection](../features/introspection/) describes types and members;
[reflection](../features/reflection/) provides bounded invocation and construction.
Broader dynamic loading and native-executable reflection need explicit identity,
resolution and code-lifetime rules.

[Introspection design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/introspection-design.md)

## Function types: delegates evolved

[Function types](../features/functions/) use structural signatures for callable values.
Possible nominal specializations and a common Callable interface remain separate
proposals, with conversion and identity contracts to resolve.

[Function proposals](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/delegates-evolved.md)
· [Callable interface](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/callable-interface.md)

## Extended CLI metadata and structural types

The [current metadata format](../metadata/) connects native libraries, tools and the
runtime. Further CLI-derived representation and structural types should preserve
identity across those boundaries. Familiar file containers alone cannot provide
compatibility with ordinary CLR execution or tooling.

[Metadata design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/extended-cli-metadata.md)

## Collections

[Collection interfaces](../features/collections/) distinguish reading, replacement
and growth. Grouping, additional reductions and immutable or frozen providers are
possible extensions. A read-only view and an immutable collection need different
guarantees when another reference can mutate the data.

[Collection design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/collection-contracts.md)

<a id="time"></a>

## Time and globalization

[Time](../features/time/) and [globalization](../features/globalization/) cover civil
dates, instants, zones, calendars and culture information. Broader locale data,
zone rules and localization require both data sources and clear conversion policies.

[Time design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/date-time-design.md)
· [Globalization design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/globalization-design.md)

<a id="io"></a>

## Storage and streams

[Storage and streams](../features/files/) provide host and memory workflows.
Richer enumeration and asynchronous I/O would extend them. Pending operations need
clear buffer ownership and cleanup on cancellation.

[Stream design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/stream-design.md)

<a id="async"></a>

## Completion and execution

[Tasks](../features/tasks/) separate completion, Result errors and cancellation.
Runtime-owned suspension, continuation affinity and broader concurrent execution
remain open. These affect scheduling and resource lifetimes as well as language syntax.

[Task design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/task-model.md)

<a id="runtime"></a>

## Runtime and language integration

[Native compilation](../features/native-compilation/) is developing alongside the
interpreter, with ARM64 as the primary native target. Broader application coverage,
portability and sustained server execution remain areas of work. JIT, trimming and
hot reload need additional contracts; hot reload is not limited in principle to
interpreted execution.

[Execution design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/execution-architecture.md)

## Further module capabilities

[Modules](../features/modules/) form namespaces and own declarations. Module-private
access, re-exports and module-centered runtime discovery are possible extensions.
Independent module loading would need separate lifetime and dependency rules.

[Module assessment](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/module-system-assessment.md)

<a id="feedback"></a>

## Discuss an idea

Use [GitHub issues](https://github.com/marinasundstrom/neoCLR/issues) to describe a
use case, suggest an alternative or discuss a limitation. Detailed proposals and
implementation evidence belong in the repository, where their assumptions and
history can be reviewed together.
