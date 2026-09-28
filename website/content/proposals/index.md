# Possible platform directions

Feature pages describe current behavior. These ideas are possible extensions,
not release commitments. The [platform roadmap](https://github.com/marinasundstrom/neoCLR/blob/main/docs/platform-roadmap.md)
selects work; its theme trackers own status and evidence. The current HTTP POC has a
finite feature scope and has passed its macOS arm64 package checks. Release preparation
is separate from possible feature extensions.

<a id="http-poc"></a>

## HTTP and networking

The [current HTTP client and server](../features/web/) exchange bounded HTTP/1.1
messages and typed JSON. Known-length uploads read source streams in small chunks;
responses and server request bodies remain buffered. Possible extensions include
async bodies, response streaming, TLS, persistent connections and richer message events.
HTTP/2 and HTTP/3 need protocol-specific providers, multiplexing, flow control and
independent stream lifetimes. .NET's handler boundary is a useful comparison, while
neoCLR must define typed errors and ownership for its own runtime.

[Client/server capabilities and modern HTTP direction](https://github.com/marinasundstrom/neoCLR/blob/main/docs/http-capabilities.md)

## Text

[String](../features/strings/) uses immutable UTF-8 and grapheme-based Char values,
with explicit scalar and byte access. Normalization, casing, comparers and text
positions are possible extensions. Unlike .NET's UTF-16 code-unit indexing,
grapheme indexing suits user-perceived characters but requires scanning and different
memory assumptions. Encoding-specific types and culture policies need separate contracts.

[Text design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/text-model.md)

## Introspection

[Discovery and bounded reflection](../features/introspection/) support the current
JSON mapper. Broader invocation, dynamic loading, offline metadata and emit remain
open. Compared with .NET reflection, separating descriptions from execution could
help offline tools, but requires explicit resolution, identity and lifetime rules.

[Introspection design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/introspection-design.md)

### Function types and nominal type information

Selected direction, not implemented: Function types will describe callable shapes,
and Function objects will hold the method and any bound environment for invocation.
They will replace delegates. `NominalTypeInfo` will carry declaration names and
namespaces, while `TypeInfo.IsNominalType` distinguishes nominal types. This begins
a wider nominal/structural split; it does not yet change tuple or union identity.
Named function types remain a possible later addition, with identity rules open.
Unlike .NET's nominal delegate types, structural shapes can share identity across
different methods. The cost is migration of compiler metadata, callback APIs and
introspection consumers.

[Function type design and migration plan](https://github.com/marinasundstrom/neoCLR/blob/main/docs/function-types.md)

## Collections

[Sequence, MutableSequence and List](../features/collections/) distinguish reading,
replacement and growth. Ordering, grouping, seedless reduction and immutable or
frozen providers are possible extensions. Like .NET read-only views, a Sequence can
observe mutations through another alias; stronger guarantees need explicit contracts.

[Collection design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/collection-contracts.md)

<a id="time"></a>

## Time and globalization

[Dates, times, instants, durations and clocks](../features/time/) supply the foundation.
[Provisional globalization APIs](../features/globalization/) now add Gregorian/Hebrew calendars, cultures and system discovery. [Named zones and a DateTime union](../features/time/) are now provisional development APIs. Broader zone rules, locale data, scheduling and independent unified localization remain future work. .NET DateOnly,
TimeOnly, TimeProvider and CultureInfo are comparison points; additional concepts
also bring data dependencies and conversion rules.

[Time design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/date-time-design.md)
· [Globalization](https://github.com/marinasundstrom/neoCLR/blob/main/docs/globalization-design.md)

<a id="io"></a>

## Storage and streams

[Provider-bound storage and synchronous streams](../features/files/) support host
and memory workflows. A file catalog could guide richer metadata and enumeration;
async I/O needs explicit pending-buffer ownership and cancellation. Directional
interfaces express capabilities more narrowly than .NET Stream, at the cost of
additional contracts and adapters.

[Filesystem design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/filesystem-design.md)
· [Stream design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/stream-design.md)

<a id="async"></a>

## Completion and execution

[Tasks, promises and cancellation tokens](../features/tasks/) separate completion,
expected Result errors and cooperative cancellation. Generated state machines and
an invocation scheduler provide execution today. Runtime-owned suspension,
continuation affinity, broader cleanup and general concurrent submission remain
future work. Unlike .NET's exception-based Task failures, explicit outcomes require
different compiler and library contracts. No public Scheduler API is selected.

[Task design](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/task-model.md)

<a id="runtime"></a>

## Runtime and language integration

The interpreter runs a bounded imported CLI subset. Native interoperation, language
projections and JIT/AOT execution remain possible directions. .NET runtime, ABI and
compiler layers are comparison points; extra execution paths bring verification,
portability and testing obligations. No replacement backend is selected.

[Execution direction](https://github.com/marinasundstrom/neoCLR/blob/main/docs/execution-architecture.md)

<a id="feedback"></a>

## Discuss an idea

Concrete application scenarios, alternative designs and counterexamples help evaluate
proposals. [Discuss on GitHub](https://github.com/marinasundstrom/neoCLR/issues)
or browse the [original proposals](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/README.md).
