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

## Function types: delegates evolved

Two open proposals explore function types as structural signatures while retaining
familiar callable bindings to a method and optional receiver or captured environment.
Possible nominal specializations would preserve explicit type identity. A proposed
Callable interface could accept different signatures through a common contract,
similar to the role of .NET Delegate. This adds identity, conversion and introspection
contracts that still need evaluation.

These proposals are **not implemented on main**. Its development callback syntax
continues to use the existing nominal Func/delegate runtime.

[Delegates evolved proposal](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/delegates-evolved.md)
· [Callable interface proposal](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/callable-interface.md)

## Extended CLI metadata and structural types

Feature-branch exploration considers a CLI-derived format for structural arrays,
tuples, Function types, unions/intersections and synthesized members, with later Raven
integration. Reusing ordinary declarations could retain useful tooling while native
signatures preserve neoCLR identity. New readers and explicit capability checks would
be required; container readability does not imply CLR execution compatibility.
An isolated experimental codec now exercises framing, structural signatures and
synthesized member descriptions.
A bounded PE container probe retains ordinary metadata readability, but standard
rewriting can discard the extension. A development .NET model now emits a bounded
primitive Int32/Boolean subset as PE or existing native assemblies. Native output includes functions
outside types and has a direct neoCLR load/run test. The PE reader recovers callable
declarations and resolves the emitted method-reference subset through explicit
dependencies, with documented signature limits. A first opt-in Raven consumer now
binds a small program and emits native output through the separate library, with a
successful neoCLR run. Calls can now be imported from read-only definitions without
retaining the producer builder graph. An optional Raven adapter now exposes explicit
configuration and compiler diagnostics; a cross-file call executes in either file
order. A bounded native declaration reader now feeds that .NET-based input provider
through a reference-only PE projection; execution uses the original native artifact.
The end-to-end case now compiles both a Raven library and its consuming application,
including overload selection and a library-local call. A three-assembly dependency
chain loads and runs in neoCLR, with missing and wrong-revision transitive dependencies
rejected. Native target adapters
and broader ordinary metadata coverage come next, followed by structural extensions. An initial feature-branch PE/#Neo execution profile now loads native metadata directly
in neoCLR and runs the Raven two-library case to 42. Initial Hello World examples
also run directly and through an entry-point function call. The Raven adapter now
supports Unit-returning helpers and imported static library methods, verified by
compiling a library and its consumer separately; parameterless entry points now support both Int32 and Unit, with Unit applications exiting zero.
An opt-in `rvnc neoclr` command now emits native assemblies from source files and
writer-produced native library references. Loading compiler symbols from translated
System now has a first explicit static Int32 callable import: Math.Min binds from
translated metadata and executes against that binary library. Full System symbol loading
remains pending; the command still uses host primitives for binding. Native emission now
uses Raven’s shared Compilation.Emit validation pipeline through an explicit backend;
shared linear-body lowering now feeds .NET and native method-builder adapters.
The same Int32 and Unit Hello/helper programs execute through both backends, now also
using a shared callable signature and declaration-builder contract.
Type/signature builders and broader instruction coverage remain follow-ups.
The selected-System driver still has a host/projection type collision to resolve; the
same-compilation Hello/helper case is the current codegen acceptance target.
The metadata builder also exposes typed opcode Emit overloads for its current linear
subset, consumed by Raven; broader instructions and editable bodies remain future work.
Namespaced static library types also retain their identity through compiler reimport
and native execution. Namespace-owned free functions remain a separate metadata gap. Its initial JSON payload remains supported; a bounded binary profile now avoids
JSON parsing during runtime loading. Production integration and structural runtime
support remain pending;
this is not a published platform format.

The metadata feature branch also translates the complete current assembled System
library from JSON to standalone native binary containers. Runtime verification and
Hello/generic dependency consumers pass. This is a load-format regression baseline;
direct Raven class-library source compilation and broad compiler symbol import remain
next steps. A larger versioned native profile now also runs selected Raven samples
against binary collection System and preserves unsigned floating operand bits. These
native-only containers have no .NET reference projection. The neoil assembler now
produces them directly with `--format neox`, verified against the separate .NET reader
and existing Raven sample behavior.

The 2026-10-01 metadata experiment now reuses Raven's compiler-lowered bodies for its
bounded .NET/neoCLR emission path, including implicit Int32 returns. Both bounded backends
also share per-emission callable identity resolution and source callable plans,
preserving overloads and assembly/type ownership. Public nongeneric static-type
identity/naming plans also feed separate .NET and native type builders. Partial static
classes now produce one native type across files, preserving every part’s methods. Initialized
Int32 locals and assignments now pass through the shared body path and metadata writer;
signed/negated comparisons, if/else and while loops with break/continue now run on both runtimes.
String signatures, locals and helper results now support computed Unicode console
output and separately compiled native text helpers. The independent metadata
writer also supports typed argument stores; Raven source parameters remain immutable. Nulls, text operators, general
object/field support and exception regions remain development work.
Backend instruction/type profiles now admit shared bodies selectively. Signed Int32/Int64 division, remainder, bitwise AND/OR/XOR and signed shifts now execute through both
backends, including truncation and zero/overflow fault cases. The profiles
also admit assembly functions, static methods and static types independently.
The separate metadata API also preserves public/internal/private static method access;
Raven source integration is subsequent work.
Public/internal static helpers preserve visibility in CLI and native output and in
the temporary compiler reference projection. General metadata
category contracts, codegen portability and metadata importer work remain development
tasks on the feature branches.

[Metadata proposal](https://github.com/marinasundstrom/neoCLR/blob/main/docs/proposals/metadata-format.md)

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

Development on the metadata branches also preserves Boolean parameters/results across
Raven declarations, native reference projections and imported overloads. Typed Boolean
locals, assignment, equality and short-circuit &&/|| now share the same body path. Int32/no-result
entrypoints and the existing System bootstrap limits remain unchanged.

The shared compiler body path also supports value-returning calls in statement position,
using checked stack discards while retaining call side effects and no-result Unit behavior.

Development metadata also preserves Int64 parameters/results/locals and exact constants.
Raven shares signed widening and unchecked narrowing with native assembly execution;
entrypoints remain Int32/Unit, and broader type/conversion support is still pending.

Signed unary +, - and ~ now use the shared compiler path for Int32/Int64, including
minimum-value wrapping tests against binary assemblies executed by neoCLR.

The compiler now uses one primitive value/no-result contract for signatures and locals,
with separate .NET/native type mappers. This is an internal migration boundary; general
nominal, generic and array type support remains pending in the native backend.
