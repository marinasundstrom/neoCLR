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
The metadata API now also preserves public/internal ownerless function access;
Raven now preserves explicit public/internal and default internal function access.
The separate metadata API also preserves public/internal/private static method access;
Block and expression bodies now share compiler lowering on both targets.
Primitive value-producing conditionals use shared branch joins and execute only the
selected branch. Value blocks support initialized locals, assignments and calls
and internal if/loop control flow before their final primitive expression. Returns
and jumps leaving a value block remain unsupported.
Native eager Boolean and/or/xor now verify and execute with exact Boolean operands;
the independent metadata writer and Raven now produce them through the shared body
path, preserving eager left-to-right operand evaluation.
Raven emits these helpers through shared declaration plans, with compiler and runtime
checks rejecting inaccessible dependency calls.
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
projections and JIT/AOT execution remain possible directions. **Future direction
(2026-10-07):** investigate native AOT first with ARM64 as the primary architecture,
retain the interpreter, and design hot reload independently of execution mode.
An AOT-compiled web app is the proposed POC destination, starting with scalar and
Hello World experiments before a minimal HTTP endpoint.
Reloadable AOT would need compatible precompiled replacements and explicit state
and code-lifetime rules. .NET runtime, ABI and compiler layers are comparison points;
extra execution paths bring verification, portability and testing obligations.
**Development experiment:** an isolated ARM64 scalar tool now emits native objects
for wrapping integer arithmetic, direct calls, locals and branches/loops, validated
through C consumers against the interpreter. Checked arithmetic and division now
propagate arithmetic Fault statuses through a versioned experimental C boundary.
The first standalone executable milestone is now
[Hello World](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-hello/README.md),
with UTF-8 console support linked into the image. The bounded pipeline now compiles
Raven source to neoCLR metadata containing its IL, then compiles that artifact to
native ARM64 code. Standalone NEOX and neoIL inputs also remain supported. A bounded
[value/member profile](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-values/README.md)
now runs Raven constructors, fields, accessors and record-copy/branch samples natively,
including nested reference-free payloads, ordinary output parameters, Byte tags and
overloaded members. An [ordinary Raven Some/None app](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-union/README.md)
now runs as a standalone ARM64 executable using explicit closed-world code selection;
its unused generated formatting/boxing members are reported as excluded. A console-input consumer remains the next step toward HTTP Server. Execution budgets and general managed
services remain unsupported.
A local generic Result app now runs with one closed instantiation per local value type;
`let ... else` and `if let` patterns cover success and non-match branches. Plain positional
`let` deconstruction remains a pinned Raven native-emitter gap. Multiple generic instantiations and native input/lifetimes are still pending. The first
real library Result probe passed the interpreter and recorded an external-call AOT boundary. A subsequent nongeneric value-library slice now compiles separate Raven library and
application artifacts into one executable, checking original access scopes before
selection. Generic value-library specialization now also passes a Pair<int, byte>
consumer with copied values and default initialization, limited to one closed shape per
type definition. Explicit runtime-owned System/Object validation contexts now pass;
the real library Result now executes both success/error paths in a standalone ARM64
image. Original interface conformance is checked before relationships are omitted from
the private direct-call projection; interface execution remains unsupported. Native
byte-input prerequisites now include a tested nested Result/Option outcome model.
Empty static member owners now compile without object allocation; actual Console.ReadByte
now passes bounded primitive erased-value storage and reaches generic helper calls.
Primitive pack/test/unpack preserves exact types and propagates incorrect-unpack faults;
primitive static generic helpers now specialize with source identities retained in build
reports. Explicit `--compile-system` now compiles selected managed System seed helpers.
Immutable UTF-8 literals now cross value-profile locals, calls and output slots. The
actual ReadByte wrapper reaches native service admission; its input service binding
and dynamic text lifetimes remain future work. An explicit failure binding now preserves
user messages and managed traces in caller-owned native records; a shared interpreter
host view uses the same runtime message catalog and 64-frame truncation rule. A linked
renderer and standalone exit-1 failure app are validated. Interpreter CLI execution
faults use the same message/trace format and exit convention. Explicit `fault` instructions now
return UserFault status through native calls without publishing a result; message/stack
diagnostics are available through opt-in ABI v3 and String-based System.Fail now has an
explicit native binding. Native input remains unimplemented.
General AOT applications, the web demo, JIT and hot reload remain future work.
Metadata alongside native images and stable calling conventions are a future
interop exploration; the current compiler inspection report is build tooling only.
[Scalar experiment](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/aot-scalar/README.md).

[Investigation and tradeoffs](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-execution-investigation.md)

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

Development metadata/runtime/compiler support now preserves namespaces on ownerless
functions; the temporary CLI projection uses encoded global names. Original integer
Math declarations execute on both .NET and binary neoCLR. Full class-library source
compilation and native symbol import remain incomplete.

The development metadata API now represents root classes, primitive mutable instance fields, constructors and instance methods. API-produced construction, mutation and aliasing execute on .NET and binary neoCLR. Explicit property/accessor associations now round-trip through native metadata and CLI projections, with owned read-only property snapshots. Raven now compiles the unchanged Order declaration and executes construction/property checks on .NET and binary neoCLR. Both the metadata API and Raven now support nominal local aliasing and property mutation; ordinary nonvirtual instance calls also execute through shared codegen. Owned nominal signatures now support factories, self-return, constructor parameters and alias mutation on both runtimes. Broader collection/delegate/union emission remains subsequent work. The development Raven adapter now also accepts the neoCLR semantic profile with a validated CLI declaration core: native binary Hello World/function calls, Unit entry, arrays and owned interface dispatch run without a host core reference. This is bounded profile integration, not native metadata symbol loading or full class-library bootstrap.


Development checkpoint (2026-10-01): Raven private mutable primitive storage now
emits as a field without property/accessor rows. The selected Order consumer and its
private-storage helper verify and execute on both .NET and binary neoCLR; this is
bounded emission coverage, not a complete class-library build. Computed getters and
implemented property accessors now share the same pipeline, including private setters
and optional backing storage. Explicit root constructors also accept expression bodies,
with overload selection and argument-order checks on both runtimes.

Default root constructors and primitive field/property initializers now share compiler
initialization with .NET and execute in the binary neoCLR consumer.

Explicit mutable primitive instance fields now preserve public/internal/private access
through Raven's native collector and the existing field metadata. Initialization and alias
mutation run on both targets. Mutable owned nominal fields and private storage now also
preserve class identity, initialization and stored-object alias mutation on both targets.
Owned nominal auto-properties, computed getters and explicit accessors now also round-trip
and execute, including private setters and forward-declared constructor initializers.
Readonly instance storage now preserves flags and constructor-only direct stores, with
readonly managed field addresses outside construction. Raven private `val` storage and
stored `val` properties execute on the updated runtime; this is shallow storage protection.
Static fields, indexed/generic properties and external class imports remain future work.


The development Raven target also accepts explicit parameterless root `base()` after
checking its bound System.Object identity. It reuses existing root initialization on
both targets; side-effecting initializers execute before block or expression bodies.
User-defined base initialization and general constructor delegation remain future work.

Development metadata API: vector declarations now round-trip primitive and owned-class
arrays as CLI SZARRAY/native ArrayRef, including locals and property storage. Executable
array allocation/indexing now passes C# API-produced binary runtime checks; Raven
consumption now executes Order arrays, indexed mutation and Length on both targets.
Shared array iteration now also executes nested/labeled control flow; generic collections
and the full runtime class-library build remain incomplete.

Development metadata work now includes indexed property signatures and overloads,
preserving CLI/native accessor associations. API-produced indexed assemblies now
verify and execute on neoCLR; Raven also emits shared indexed accessor calls, including
overloads and multiple indices.

The next development milestone is generic emission. Unconstrained method/function
declarations now preserve named generic parameters, locals and array signatures;
instantiated calls and Raven consumption are still in progress.

The development metadata producer also emits unconstrained generic function/static
method instantiations and forwarding, verified from binary assemblies on neoCLR.
Raven generic source emission now exercises this path with owned static functions and
methods. Generic types, constraints and imported generic symbols remain separate work.

Development metadata producer support now includes unconstrained ordinary generic
instance methods on owned root classes, verified through binary native execution.
Generic types and virtual/constrained generic dispatch remain future work.

The development metadata API also emits typed default initialization, including generic
method parameters, with CLI/native execution tests and definite-assignment checks.

Raven's shared emission path now uses that initialization contract for default(T)
and generic array clearing; binary consumers verify numeric defaults and null-reference
faults after clearing object elements on .NET and neoCLR.

The development producer now supports static generic type owners with separate type/
method parameter scopes and constructed method references. Generic instance layouts
and fields remain future work in this direct emission API.

Development metadata producer work also supports generic reference classes with typed
fields, constructors and instance calls. API-produced CLI/native binaries and Raven consumers execute the
same storage case, including nested generic class values. Generic property/indexer metadata also preserves owner scope and accessor associations,
with Raven consumers verified on both runtimes.
The ordinary native compiler driver now explicitly enables checked-storage reservation
from the selected bootstrap. Separately compiled helper consumers exercise alias mutation
and uninitialized-read faults. The full source-built class library remains incomplete;
see the [experimental metadata API](/docs/experimental-metadata.html).

Development producer metadata also supports constructed generic field references with
exact receiver/value checks; external assembly fields remain outside the current API.

Development metadata also retains nominal class bounds on type parameters through CLI
and native metadata, with invalid concrete arguments rejected. Other constraint kinds
remain under development.


Development checkpoint: the experimental Raven metadata target now preserves owned
nominal generic bounds and class/struct/new requirements through binary execution.
The selected generic consumer runs on .NET and neoCLR; this does not establish full
class-library emission. Open constrained operations, broader bounds and native symbol
loading remain future integration work. See the host metadata API reference for the
bounded producer contract.


The development metadata target now compiles the complete unchanged Language
class-library source and executes it on both .NET and neoCLR. Shared static-property
accessor calls use ordinary CLI semantics; full collection/library emission remains
in progress. Supported behavior follows .NET unless a divergence is explicitly chosen.

The experimental producer and Raven target now also preserve invariant comparer
interface declarations through native assembly loading and CLI reference projection.
Interface dispatch and full collection compilation are not covered by that checkpoint.

That declaration coverage now includes unchanged Disposable and `Iterator<T>` sources,
with inherited nongeneric interfaces and abstract property metadata. Execution through
an interface remains a separate acceptance gate.

The development producer also supports owned interface signatures and constructions.
Unchanged `Iterable<T>` loads, and nullable interface references pass through parameters,
results and array storage on both targets. Interface dispatch remains the next gate.

Owned nongeneric interface method/property dispatch now executes through two concrete
classes on .NET and native neoCLR. Generic dispatch and full class-library compilation
remain development gaps; this uses ordinary CLI callvirt and existing runtime lookup.

Development metadata now supports static primitive-vector calls between separately
emitted Raven library and application binaries. The native-profile runtime probe
checks overloads and shared array mutation. Nominal/generic dependency imports and
a native compiler symbol loader remain future work; CLI declaration projection is
still the temporary input bridge.

The development branch also supports imported unconstrained static generic methods
with primitive/vector arguments. Separately emitted Raven library and application
binaries execute generic calls directly in neoCLR. Nominal and generic declaring-type
imports remain open; this does not yet provide general collection-library imports.

The independent metadata producer now retains external class/interface signatures and
constructed generic references across CLI and native binary output. A separate library
and consumer verify/run with `Box<consumer Order>`. Raven now consumes those external
type signatures, with a separately emitted Raven library/consumer pair executing in
neoCLR. Imported member calls and value/union signatures remain open; the unchanged
collections sample now reaches its `Option<Order>` signature boundary.

Imported static generic calls now also accept consumer-owned reference types, external
constructions and caller generic parameters. Raven binaries verify and execute this
boundary in neoCLR; value/union contracts and full collection imports remain open.

Development static imports now include dependency-local nominal method signatures.
A Raven library factory returns `Box<consumer Order>` and preserves the payload alias
through native execution. Value-type unions and instance/generic-owner member imports
remain open; this is not yet full collection-library compilation.

The development metadata producer now preserves owned value-type categories in CLI
and native binaries. Defaults, primitive fields, generic forwarding and arrays execute
on both runtimes. Generic payload storage and imported union/value types remain open;
Raven's collections sample is still blocked at `Option<Order>`.

Producer value types also support generic payload storage and initialized local-address
field mutation. CLR and native tests retain value copies and reference payload aliases;
imported value types and Raven union lowering remain development work.

The development metadata API supports direct assembly/type/field/function declarations,
static and instance methods, and root-class constructors, with builders sharing those
declarations. Manual CLR/native execution covers function calls, object creation,
readonly-field initialization and struct storage. Direct nongeneric interface contracts also dispatch through existing implementation
helpers. Interface relationship definitions now support direct inherited/implemented
edges. Method definitions now own instruction/local/label storage; helpers operate on
that same body. Property/accessor declarations also share this graph. Definitions are Cecil-like;
generation builders are intended to follow Reflection.Emit-style convenience patterns.
Arbitrary instruction editing and loaded assembly modification remain pending; this is not a
published general-purpose assembly editor.

Direct generic declarations now share parameter names and constraint storage with builders;
constrained calls execute on CLR and neoCLR. The intended architecture separates definitions,
encoded metadata and PE packaging, with readers reversing those boundaries. Full separation
and editable loaded definitions remain development work.

Imported value-type signatures now preserve their category through the metadata API and
native loading. A Raven consumer of a separately produced native value library executes
successfully. The collections sample advances to a body-lowering invocation gap; full
union/member support remains development work. New imported-value images require the
updated runtime.

Development integration now executes imported constructed interface dispatch and final
class calls through Raven-produced native consumers (42). The metadata library also
supports closed generic interface implementations and matching CLI projection. The
unchanged collections application next stops at propagation-expression lowering; this
is not yet complete application support.

The following shared-lowering checkpoint resolves a concrete-case construction failure
that masked the propagation path. The unchanged collections probe now reaches native
out-local admission. Managed-reference and byref-call support remain development work;
the full application still has not executed.


The next development slice exposes standard typed `ldobj`/`stobj` through the metadata
API for owned local addresses. C# producer tests execute generic copies and local
updates on CLR and neoCLR (42), with definite-assignment rejection tests. Byref
parameter/call support and the full collections application remain incomplete.


Writable managed-reference parameters now survive metadata emission, generic substitution,
imports and CLI projection. Separate test assemblies mutate a caller local on CLR and
neoCLR (42). Out assignment guarantees and Raven propagation admission remain subsequent
work; this metadata-only slice does not complete the collections application.


Explicit output contracts now preserve CLI Param Out flags and native out_parameters
through metadata imports and projection. Test producers prove assignment on normal
return, including forwarded outputs, and separate library/consumer execution returns
42 on both runtimes. Raven admission and imported value receivers remain integration work.


Raven's shared emission path now consumes those ref/out contracts. Five native controls
pass, including source forwarding/mutation, with 64 focused C# tests passing. The
unchanged collections sample advances to imported value-receiver TryGetOutput admission;
full application and runtime-library compilation remain incomplete.


Metadata value-instance calls now preserve managed receiver addresses and caller mutation.
Separate generic value/out library consumers execute on CLR and neoCLR (42); constructors
and constrained interface dispatch remain outside this development slice. Raven now consumes these contracts through an explicit value-receiver capability: a
separate native-library consumer verifies and executes mutation and generic out calls
(42). The unchanged collections sample advances to a lowered throw guard; terminal
failure support, full application execution and runtime-library compilation remain open.

Development naming correction: terminal guest failure is requested with
`System.Fail(message)`; the host receives a `Fault`. The former `System.Fault` call
spelling requires migration and a matching compiler/reference/runtime bundle. This does
not add guest exception handling or process-abort behavior.

Development metadata emission now includes literal terminal failure. Native execution
uses the existing fault instruction; ordinary CLI metadata can express the corresponding
throwing path. Raven distinguishes its generated propagation guards from source throws
and preserves .NET behavior. The unchanged collections sample now reaches imported
carrier construction; complete native propagation/application execution remains pending.

Development metadata and Raven emission now support imported value constructors,
including generic owners, with explicit initialization checks. Separate-library consumers
execute successfully (42). The larger collections sample still needs nested type identity
support for its Option.None carrier; complete class-library execution remains open.

Development metadata APIs now preserve nested nongeneric class/value declarations
through CLI and native PE, with constructor execution checked on both runtimes.
The following checkpoint records the integrated compiler consumer.

Nested imports, generic values and structural Function callbacks now execute through
Raven's direct metadata backend. The unchanged collections application compiles to
native PE/#Neo, verifies and runs against explicitly bound translated System, with map
lookup, union propagation, queries, shared object identity and iteration checked against
expected output. The experimental metadata API retains CLI reference identity while
binding selected signatures to their native implementation.

This is development work on the metadata/compiler feature branches. CLI snapshots still
provide symbols and the existing translation still supplies the runtime library; native
semantic importing and complete class-library source emission remain open. The host
metadata API and its restrictions are documented in the [API reference](/docs/).

The next source-emission checkpoint compiles the unchanged collection interface sources
through `Sequence<T>`, including constructed interface bases and its indexer. An inherited
property/indexer consumer runs on CLR and neoCLR with both source orders; the real
neoCLR target profile also passes, including generic provider and iterator classes.
The unchanged ArrayList source now compiles to native PE and executes growth, independent
copies, iteration, callback searches and Option results. Expected invalid-capacity and
index faults use System.Fail through explicit namespace-function binding. The authoring
seed and translated System dependencies are still required. This bounded checkpoint
does not yet compile the entire System library with the native backend.

Unchanged callback comparer classes also compile and execute through native Function
fields and interface dispatch. Noncapturing callbacks are covered; closure environments
and complete library compilation remain outside this checkpoint.

HashMap and its source dependencies now compile together and execute native PE, covering
collisions, growth, updates, independent key snapshots and Option lookups through map
interfaces. Translated System still supplies remaining library dependencies.

Source-built collection checks also cover reference payloads and shared object identity.
The full application with source-built queries remains blocked at the source/seed
iteration contract boundary; its existing translated-library execution is separate.

Direct native metadata reading has begun: the experimental host library now materializes
primitive namespace functions into shared definitions without a CLI projection. Broader
metadata coverage remains development work.

Raven now binds a bounded native function library directly into its semantic model,
including overloads and exact dependency identities. Its primitive core still comes
from an explicit CLI bootstrap. Native call emission now runs across assembly boundaries,
including a Raven-produced library read directly and consumed by another Raven program.
Both tested consumers return 42; nominal/generic native importing remains open.

The direct-reader development profile now also includes fieldless nongeneric static
classes and primitive static methods. Instance types and richer native signatures are
still pending; the general writer/runtime support is broader than this reader profile.

Fieldless instance classes, constructors and primitive instance methods now also pass
the direct native import/emission/runtime path. Fields, richer signatures and reference
comparison lowering remain gaps; the CLI primitive core bootstrap is still required.

Primitive instance fields now load directly into metadata definitions and Raven symbols.
A Raven-built stateful class executes through imported constructors and methods (42).
Direct imported field emission and richer field signatures remain development gaps.

Direct public primitive field loads and stores now execute across native assembly
boundaries, including writes through a local alias. The metadata library also emits
ordinary CLI field MemberRefs. Richer signatures and field-owner categories remain open.

Development checkpoint (2026-10-02): direct native metadata reading now includes local
class parameter/result signatures. A Raven-produced native library and consumer execute
factory, identity-call and constructor-argument paths in neoCLR (42). This remains a
bounded feature-branch profile with an explicit CLI primitive core; full native System
loading is pending.

Development checkpoint (2026-10-02): class-valued fields now participate in direct native
metadata loading and Raven emission. A native library consumer replaces and mutates a
stored object while preserving the original (42). Existing nominal CLR behavior and
PE/#Neo encoding are retained; cross-dependency signature loading remains pending.

Development checkpoint (2026-10-02): direct native signatures now resolve classes in
explicit dependencies. Raven-produced payload/holder libraries and their consumer run
as three native assemblies (42). Exact identities are preserved without a CLI projection;
full native System importing and broader signature categories remain pending.

Development checkpoint (2026-10-02): direct native importing now includes one-dimensional
primitive and class array signatures. Raven's three-assembly consumer preserves array
aliases and replaces elements across library boundaries (42). Broader metadata categories
and full native System loading remain pending; the wire format is unchanged.

Development checkpoint (2026-10-02, metadata feature branch): native non-indexed
properties now read into Raven symbols and execute across separately compiled native
assemblies, including class/array setters and static getters (42). This extends the
existing CLI-shaped property/accessor model without a new encoding. Indexers and full
native core loading remain pending; CLI core/translated System bootstrap inputs remain.
[Evidence](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/native-properties-2026-10-02.json)

Development checkpoint (2026-10-02): direct native loading now also supports indexed
property signatures. Raven binds overloads and executes cross-library indexed reads
and writes (42). The existing CLI-shaped property/accessor format is unchanged;
setter-only source access and full native core loading remain pending.

Development checkpoint (2026-10-02): setter-only native indexer assignments now compile
and execute too (42), sharing the property parameter contract with .NET. Reads require
a getter; the metadata format is unchanged. Full native core loading remains pending.

Development checkpoint (2026-10-02): direct native loading includes nongeneric
interfaces and local inheritance/implementations. Raven compiles a consumer of two
implementations and neoCLR executes inherited method/property dispatch (42). This keeps
the existing CLI-shaped contract and callvirt behavior; generic interfaces and full
native core loading remain pending.

Development checkpoint (2026-10-02): interface-valued fields and arrays now have a
three-assembly Raven-to-neoCLR test. Imported signatures preserve identity, alias writes
and dispatch after replacement (42), using the existing compiler and runtime paths.
The primitive CLI bootstrap and broader generic/value import work remain.

Development checkpoint (2026-10-02): a shared Raven fix now diagnoses incompatible
expression-bodied returns before emission for .NET and native dependencies. The 106
focused compiler tests and all six native runtime consumers pass. Native generic import
and the full class-library bootstrap remain under development.

The development metadata reader now preserves unconstrained static generic methods
and namespace functions. C# consumers execute imported generic calls on CLR and neoCLR;
Raven now imports this profile and executes generic forwarding, overloads and array
aliases; all seven native consumers pass (42). Generic owners and constraints remain
pending. This is not full generic import.

Development checkpoint (2026-10-02): direct native generic root classes now preserve
owner parameters. Raven constructs and uses `Box<int>`/`Box<Item>` through shared generic
substitution; all seven native consumers return 42. Constructed signature types,
constraints and full native class-library bootstrap remain under development.

Development checkpoint (2026-10-02): local closed generic signatures such as `Box<int>`
now read directly into immutable metadata definitions and Raven symbols. Native
factory/identity calls execute successfully; open/external constructions and constraints
remain future import work.

Development checkpoint (2026-10-02): scoped local constructions such as `Box<T>` and their
vectors now retain method/owner parameter identity through direct native import. Raven
inferred calls execute successfully. External generic constructions and constraints
remain pending.

Development checkpoint (2026-10-02): external generic constructions now preserve exact
assembly scope through native metadata and Raven. A three-assembly generic consumer
executes successfully. Constraint import and full native core/bootstrap remain pending.


Development checkpoint (2026-10-02, metadata feature branch): Raven now authors native
namespace-function references from compiler symbols for primitive, method-generic and
vector signatures, without reading imported method definitions on that path. Seven
native consumers execute (42); 107 metadata C# groups pass. Nominal signatures and
type-owned members still use the earlier reader-backed route. The independent library
instruction-generator API remains planned. No native format change is required.


The same development path now authors public native root-class identities, including
unconstrained generic types, directly from Raven symbols. Other type profiles and member
references still use reader-backed imports; this is a bounded architecture migration,
not a new metadata format or complete reader/emitter independence.


Native namespace-function signatures now also reconstruct root-class constructions
and vectors from Raven symbols, including cross-dependency `Box<T>` forwarding. Member
references and richer type profiles remain on the earlier reader-backed route.


Public nonvirtual root-class methods and constructors now also reconstruct from Raven
symbols, including generic-owner parameters. Generic construction and mutation execute
on CLR and neoCLR; interface/virtual profiles and fields still require further boundary
work. The assembly format is unchanged.


Native root-class field references now use symbol-owned storage signatures and explicit
layout ordinals. .NET continues to use named field references. Richer layouts and
interface dispatch remain further boundary work; the native instruction set is unchanged.


Nongeneric interface relationships and abstract dispatch references now also reconstruct
from Raven symbols. Existing inheritance/dispatch and interface-storage consumers pass;
generic interfaces and broader class inheritance remain pending.


The development metadata library now exposes an independent IILGenerator for body
authoring. Raven uses it through the NeoCLR adapter; shared compiler interfaces remain
separate. Legacy builder instruction methods still work through the same body engine.
Loaded-body editing and instruction insertion remain future work.


The metadata library generator now owns body-authoring logic internally too. Legacy
builder instruction methods forward to it, and both paths share definition-owned storage.
Writer validation and encodings remain unchanged.


Native static-container calls, including generic methods, now reconstruct from Raven
symbols too. Static classes remain declaration owners rather than valid value types.
Translated CLI bindings and remaining virtual/profile cases are unchanged.


Native callable, type and field emission now has no reader-definition fallback: supported
references use symbols and explicit identity/layout facts, while unsupported contracts
diagnose. Seven consumers still execute. Host input setup and lazy symbol loading remain
reader-backed, and translated CLI compatibility is a separate path.


Development checkpoint (2026-10-02): Raven's native emission host can now bind a
registered native compiler reference without separately supplying its metadata reader
definition. Seven native consumers compile and execute. Lazy symbol loading and the
explicit primitive-core/translated-System bootstrap remain; this does not change the
published metadata format or promise complete class-library support.


Development checkpoint (2026-10-02): native cross-assembly fields can now hold closed
generic class values and arrays of them. Raven's generic consumer tests replacement
and aliasing, and metadata tests execute the same storage on .NET and neoCLR. Fields
on generic declaring types remain a separate development gap.


Development checkpoint (2026-10-02): Raven now consumes fields on generic native classes
and dispatches through imported generic interfaces, including inherited contracts and
class-valued arguments. C# tests execute equivalent metadata on .NET and neoCLR. Creating
new implementations of external interfaces remains a separate development capability.


Development checkpoint (2026-10-02): the C# metadata prototype now has an Introspection-
shaped facade and explicit metadata load context. Raven uses it for local/external type
resolution without runtime loading. Constructed and member views remain development work;
this is not a new guest Introspection release.

Development checkpoint (2026-10-02): the C# metadata facade now projects constructed
types and declared fields, including generic substitution. Raven consumes those views;
method/parameter projection and the guest implementation remain future work.

Development checkpoint (2026-10-02): the C# facade now exposes metadata-only method and
parameter views, preserving separate generic scopes. Raven delegates signature projection
to those views. No runtime invocation or guest Introspection API change is introduced.


Development checkpoint (2026-10-03): C# metadata views now project declared properties,
indexed signatures, canonical accessors and directly declared generic interface edges.
Raven consumes those views. Inherited-interface traversal and language policy remain in
the compiler; no guest Introspection API or runtime encoding changes in this slice.

Development checkpoint (2026-10-03): bounded inherited-interface queries now live in the
C# facade; Raven consumes the closure. Constructed arguments remain distinct and diamond
duplicates collapse. No guest/runtime API change.

Development checkpoint (2026-10-03): generic method/function signature inspection now
supports canonical constructed views with independent owner/method scopes. These host
views do not invoke code; Raven retains inference and compiler symbol construction.

Development checkpoint (2026-10-03): native metadata views now expose constructor and
accessibility facts consumed by Raven. Paired driver execution works for Hello World and
a separate generic library on both targets; the broad source-library gate is still open.

Development checkpoint (2026-10-03): external generic interface contracts now cross separately compiled Raven libraries and execute on both targets, including diamond dispatch. The source-built collections application gate remains open.

Development checkpoint (2026-10-03): source-built iteration/collection interfaces now support separately compiled consumers on both targets using explicit bootstrap ownership. The retained-seed partition, ArrayList and broad application remain in development.


Development checkpoint (2026-10-04): the native compiler path now imports and emits
all eight fixed-width integer signatures with standard CLI signedness rules. Separate
integer libraries and consumers run on .NET and NeoCLR, including high-bit unsigned
arithmetic, shifts and conversions. Canonical source primitive ownership and generic
Number dispatch remain unfinished; this is not full class-library numeric support.

The development metadata API also supports explicit numeric runtime declarations,
including scalar managed receivers and metadata-only inspection. It rejects record
storage on those declarations. This is infrastructure for source numeric ownership;
Raven's full Number/class-library gate remains unfinished.
