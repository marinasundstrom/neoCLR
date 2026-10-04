# Source-built union bootstrap (development)

This bounded native gate compiles the unchanged iteration contracts, Propagatable,
Option and Result into NeoCLR.Collections.dll. A second ordinary compiler invocation
imports only that artifact, then neoCLR verifies and executes the consumer. This is
not the complete dual-target library gate or the collection application gate.

Owners:

- Primitive semantic declarations: the explicit `--reference-storage-core` image.
  Its CLI bodies are never executed; it contains no source-owned union/collection types.
- Retained executable services: `union-seed.neoil`, module System. Its inventory is
  Void, Char, RuntimeTypeHandle, Object, String and Console. Native intrinsics implement
  string concatenation, type-handle display and console output.
- Source library: every declaration in `union-ownership.json`, consumed by the compiler
  as its ownership/iteration configuration. No seed copies of these declarations exist.

Object.ToString is a temporary target adapter: it asks the existing native type-handle
query for the display name directly instead of allocating the guest introspection facade.
That removes the Option/Result bootstrap cycle without a placeholder body. The runtime's
existing intrinsic boxed-primitive formatting and ordinary value overrides still dispatch.
Unlike a full .NET core library, this seed deliberately exposes only the services required
by this gate; unsupported core member references reject. Broader source-library ownership
and executable .NET adapters remain subsequent work.

From the neoCLR worktree, with an already built Raven driver and neoCLR runtime:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven -p:UseRavenCoreReference=false \
  -p:WarningLevel=0 -- --reference-storage-core /tmp/UnionCore.dll
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_unions.py \
  --compiler /absolute/path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --runtime target/release/neoclr --core /tmp/UnionCore.dll \
  --output /tmp/source-unions-fresh
```

The output directory must be fresh. Evidence records every command, source/artifact hash,
stdout and exit status. Expected output is `Option.Some(40)` then `Result.Error(7)`,
with exit 42. The consumer checks copy independence, false-output initialization, unit
residuals, error residuals, generic pattern matching and boxed virtual display. Missing
library input and duplicate source ownership in a seed must reject before publication.

The seven older native consumers use their own full CoreProbe/seed baseline; the small
core here intentionally does not supply their configured typeof facade.

## ArrayList assessment

Generate the next primitive profile with `--reference-collection-storage-core` in
place of `--reference-storage-core`, then pass that image and `--collections` to the
same Python driver. This adds public CLI Func/Action declarations for Raven's existing
callback binding and the marked namespace declaration for System.Fail. These are explicit
compiler bootstrap symbols; no source collection or union declaration is copied into core.
The callback declarations remain the existing temporary CLI transport, not a new structural
Function semantic design or a claim that the separate Function experiments are integrated.

`arraylist-ownership.json` adds unchanged ArrayList and its internal iterator to the
source library. `collection-seed.neoil` includes the union seed and supplies System.Fail
through the existing terminal runtime intrinsic. The negative-capacity execution test
checks the real diagnostic and failure exit, so its reference-only core body is never
mistaken for executable behavior.

Current result: the library and a separate native-import consumer compile and execute
successfully. The consumer has no library source inputs. Alias mutation, independent copy,
iteration and Find callback checks return 42. The negative-capacity consumer also compiles
against the library alone and terminates through System.Fail. Missing-library and duplicate
seed ownership guards remain required. The previous blocked assessment is retained as
historical evidence; the current driver expects successful separate execution.

Native reader function signatures now project through metadata-only FunctionTypeInfo views,
with generic substitution and explicit dependency resolution. Raven maps the Boolean
predicate shape to its existing callable symbols, and emission authors the callback operand
from those symbols. Explicit no-result callbacks remain an unsupported Raven import category;
metadata views preserve their distinction from inhabited-unit callbacks. Wider function
semantics, .NET class-library adapters and the full collection application remain separate work.

### Separately compiled HashMap and comparers

Use the same collection primitive core and compiler/runtime paths as the ArrayList
workflow, replacing `--collections` with `--hashmap` and choosing a fresh output directory.
`hashmap-ownership.json` extends source ownership to Comparer, FunctionComparer,
EqualityComparer, FunctionEqualityComparer, Map, MutableMap and HashMap. Their sources
are unchanged. The consumer receives only the emitted native library reference.

The consumer forces every key into one bucket, grows beyond initial capacity, rejects
an existing key, replaces and inserts through Set, checks missing Option payloads,
checks Keys snapshot independence, and mutates a shared ArrayList through a retrieved
value. Both equality/hash callbacks and ordering callbacks execute through imported
contracts. Expected stdout is empty and exit status is 42. Existing capacity-failure,
missing-reference and duplicate-ownership guards also run. HashMap does not currently
expose removal. This is native execution evidence, not the full .NET library gate.

See [recorded commands, revisions and hashes](../hashmap-import-2026-10-03.json).
No metadata, compiler or runtime changes were necessary for this extension. The source
contract mirrors the supported CLR generic interfaces and callable operations; no new
NeoCLR semantics or format version is introduced. Query composition and the broad
application remain the next acceptance boundary.

### Native extension declaration/import regression

Use `--extensions` with the same paths and primitive core. This runs the cumulative
HashMap gate, then builds `extension-library.rvn` as a second native library and compiles
`extension-consumer.rvn` against both artifacts with library sources absent. Generic
receiver predicates and a method-generic selector execute (empty stdout, exit 42).
This is an isolated compiler regression, not a replacement for runtime query sources.

Raven uses its existing CLR lowering: extension receiver type parameters become method
parameters on a nongeneric static container. The container carries the standard
`System.Runtime.CompilerServices.ExtensionAttribute` marker through the same bounded
embedded-attribute profile as native unions. Native namespace discovery reads the marker
through introspection, then leaves receiver applicability and inference to Raven binding.
No format version, instruction or runtime dispatch change was needed. Constrained
extensions, static extension members and extension properties are outside this slice.

`query-ownership.json` records the cumulative *assessment* sources, including unchanged
SingleError and Operators. Compilation now stops at the object-to-generic conversion in
`OfType<U>`, publishing no library. Do not use that manifest as proof of executable query
support. See [extension evidence](../extension-import-2026-10-03.json) and
[full source assessment](../query-source-assessment-2026-10-03.json). Next add the supported
CLI-equivalent unboxing/generic-cast operation through shared emission, metadata and
runtime validation, then resume the unchanged broad application.

### Unchanged full query library executes

Use `--queries` with the same compiler/runtime/core paths and a fresh output directory.
This selects `query-ownership.json`, compiles the complete unchanged SingleError and
Operators sources cumulatively with unions and collections, and then compiles only
`query-consumer.rvn` against the emitted native artifact. OfType, Filter, Map, ToList and
Single execute with empty stdout and exit 42. Heterogeneous boxed integers/string values
check filtering and unboxing; a retrieved ArrayList checks retained object identity.
The capacity-failure, missing-library and duplicate-ownership guards also run.

The earlier OfType assessment is historical: UnboxAny authoring and shared lowering now
close it using the existing CLI/native instruction. The unchanged broad application
still rejects `Order[].Filter` during binding. The next task is canonical array iteration
contract participation in extension receiver inference/conversion, followed by native
emission and runtime verification. No sample rewrite is used to bypass that failure.

### Nominal Array<T> assessment (not an execution gate)

Author direction on 2026-10-03 keeps arrays backed by nominal Array<T>; structural array
identity changes are deferred to structural-types work. Vector storage remains distinct
from the nominal member/iteration contract, just as CLI vectors have special storage.

`array-ownership.json` extends the cumulative source set with unchanged System/Array.rvn,
selects `ArrayShapeTypeName = System.Array`1`, and explicitly selects source-owned
Propagatable for NeoCLR propagation. Its library compiles and imports. The independent
`array-consumer.rvn` compiles, but execution fails while converting the vector to the
source-owned Iterable contract: the runtime cannot select a matching implementation.
This is a linking/backing contract gap, not grounds for structuralizing arrays or
rewriting the application. The ordinary `--queries` success gate remains unchanged.

Reproduce by using the same ordinary driver arguments as the query evidence, substituting
`array-ownership.json`, compiling its source list to `NeoCLR.Collections.dll` in a fresh
directory, then compiling only `array-consumer.rvn` against that artifact and running it
with the same seed. Exact commands and artifact hashes are recorded in
[the nominal array assessment](../nominal-array-assessment-2026-10-03.json).
Next connect vector storage to the output-owned nominal Array<T> descriptor and its
source-authored iterator through explicit identity-aware compiler/runtime contracts.
Do not rely on the old translated System.ArrayEnumerable adapter's hard-coded names.

### Broad native application gate (2026-10-03)

Run `verify_source_unions.py --application` with the same required `--compiler`,
`--runtime`, `--core` and fresh `--output` arguments. This implies `--arrays` and builds
all sources selected by array-ownership.json as one native library. It then compiles a
minimal imported SingleError display regression and unchanged
`application-order-collections.rvn` with only the emitted library reference. The broad
application must match its checked-in expected text exactly and exit 0.

The retained seed's `WriteLine(Int32)` delegates to existing native Int32ToString and
WriteLine(String) services. It is an executable bootstrap adapter, not a consumer stub.
The library's source methods, unions and arrays are imported natively. Evidence includes
source/artifact hashes, ownership, compiler payloads and executed commands. This passes
the native broad gate; .NET source-library service adapters and paired execution remain
open. Earlier assessments above describe the failure state before this gate.

### Paired .NET assessment

Run `python3 verify_dotnet_sources.py --compiler /absolute/path/rvnc.dll --output /tmp/fresh`
from this directory (or use its full repository path). The driver builds the real
`dotnet-services` adapter, compiles and executes its storage/failure consumers, then builds
the same unchanged class-library sources plus System/Functions.rvn. The broad consumer
receives emitted references only. Runtimeconfig files explicitly select .NET 10; no
probe-only compiler setup or reference-only service bodies are used.

The driver always writes validation.json and returns failure when a later gate fails.
Current outcome: adapters pass, library emits, consumer rejects invalid CLR System.Void
storage through a TypeLoadException. This is a recorded blocker, not a dual-target pass.
Explicit unit storage mapping, failure-before-publication and .NET array/interface backing
remain to be completed. Native `--application` continues to pass.

#### .NET unit milestone

The driver now uses dotnet-ownership.json and asserts its Libraries catalog matches the
native array manifest. The additional Unit contract explicitly maps source System.Void
to the adapter's fieldless UnitValue; it does not replace the CLR core. Following library
compilation, reference-only union, ArrayList, HashMap and query consumers must execute with
exit 42 and expected stdout. These pass, as do the service adapter tests.

The full .NET application compiles and starts, but remains a failing assessment: its
array-query section terminates abnormally because CLR arrays have no implementation of
the selected custom iteration interface. An explicit .NET adapter conversion is next.
The driver still returns nonzero and records the partial output, rather than counting
successful compilation or a prefix as broad application success.

## Native sample expansion (2026-10-03)

The `verify_source_unions.py --arrays` and `--application` gates additionally compile,
verify and execute these consumers against the separately built native source library:

- `array-interface-count-consumer.rvn`: inherited Count access plus shared array mutation,
  exit 42.
- Unchanged `library-option.rvn` and `library-option-propagation.rvn`: imported union
  construction, matching and propagation, exact output and exit 0.
- Unchanged `library-collection-capabilities.rvn`: source Array<T>/ArrayList interface
  views, mutations, query materialization and list growth, exact output and exit 0.

The Count regression previously failed metadata stack validation because the projected
interface accessor received a vector without the required receiver reference conversion.
Portable codegen now emits that conversion when admitted by the target and Raven's
existing implicit-reference rules. No metadata/runtime change or nominal-array redesign.
The consumer commands include only the library artifact, never its Raven source files.

Broader unchanged samples still reject: `library-array-callbacks` needs native no-result
callback import; `library-list-filters` needs captured function emission;
`library-generic-collections` needs Int64.CompareTo and source/seed Date support. These
are distinct layers and are not evidence for resuming CLR source-library adapter work.
Next bounded native step: no-result callback import/emission with array callback execution,
keeping CLI void separate from inhabited unit and structural Function experiments separate.

## Native array callback milestone (2026-10-03)

The array/application gates additionally execute unchanged `library-array-callbacks.rvn`
with exact output `7`, `42`, `First`, `Second` and exit 0. Its explicit no-result
callback imports retain Action-shaped symbols independently of inhabited source unit.
Nested vectors now round-trip through metadata; the runtime's registered nominal array
backing cast preserves storage identity with exact element arguments. The additional
`nested-array-callback-consumer.rvn` mutates nonempty inner arrays through callbacks and
returns 42 through the original aliases. This replaces the previously recorded callback
blocker; captured function emission in list-filters remains open. No structural Function
branch integration, rectangular arrays or array covariance is implied.

## Instance callback prerequisite (2026-10-03)

The array/application gate also compiles `instance-callback-consumer.rvn` against the
separate library and executes it with exit 42. Binding a Matcher method, then changing
its target and passing the callback to ArrayList.Find verifies receiver identity and
shared mutation. It exercises an explicit source method group, not generated closure
lowering. Unchanged library-list-filters remains blocked on captured lambdas; do not
count this consumer as that sample passing.

## Native reference capture milestone (2026-10-03)

Raven `623cbc1d8` emits private closure frames for immutable reference locals using
existing metadata instance Function bindings. Compared with CLR closures, this bounded
profile supports shared object mutation and escaped callbacks, but not shared reassigned
variables, value captures, parameters or receiver captures. These reject explicitly;
this is not a replacement for general lexical closure lowering. Ordinary Raven/.NET
emission and Runtime Contract/bootstrap configuration remain unchanged.

The array/application acceptance gate now runs unchanged library-list-filters against
its separately built native library and checks every output line. The additional
captured-reference-consumer checks escaped lifetimes, distinct factory invocations and
two callbacks sharing one object's mutations. A mutable reference binding rejects before
publication. Seven native consumers and 31 focused .NET function tests also pass.
Metadata/runtime are unchanged; the preceding 128-group metadata evidence is reused.
No public metadata API or website API snapshot changes are required for this compiler
slice. Structural Function experiments remain separate; the known stale guest API
snapshot is not regenerated by this change. Full-library completion is not claimed.

## Native query acceptance and remaining source surfaces (2026-10-04)

The array/application gate now executes unchanged library-query-basics and
library-query-names with their checked-in expected output. This covers Skip/Take,
Any/All, Count, Fold, Concat/FlatMap and chained Filter/Map through native references.
The focused query-lifetime-consumer provides an instrumented Iterable/Iterator source;
all query operators are imported from the separately compiled class library. It checks
lazy construction, early termination, exhaustion, repeat enumeration, idempotent Dispose
and no further source movement after disposal, returning 42. It is an independent
lifetime test, not a replacement for the still-blocked introspection-heavy OfType sample.

These checks exercise disposal behavior analogous to .NET enumerable operators through
neoCLR's own Iterable/Iterator contracts. No exception-unwind guarantee is claimed.
Compiler `623cbc1d8`, runtime/metadata base `c8ab1967`, primitive core and ownership
manifest are unchanged. The complete expanded application gate passes; unaffected
31-test .NET and seven-consumer evidence from the preceding slice is reused.

A seven-sample assessment records current binding failures rather than inventing emitter
fixes: maps and reference-payloads need guest introspection declarations (among other
missing library types); OfType needs MethodInfo/MemberInfo and type acquisition members;
comparers needs StringComparer and Int32.CompareTo; result needs Math.Abs. Associated
inference/pattern diagnostics can be secondary and are not independently classified as
compiler defects. Next bounded expansion should establish numeric/comparer ownership
and primitive member contracts before the larger guest introspection source group.
See ../query-acceptance-2026-10-04.json for commands, artifact hashes and diagnostics.

## Native primitive capture prerequisite (2026-10-04)

Raven `1b15715bf` admits immutable Int32, Int64, Boolean and Byte locals alongside
reference captures. Fresh frames copy these immutable values on lambda evaluation;
mutable locals, arbitrary struct values, parameters and receiver captures still reject.
The primitive-capture consumer imports FunctionEqualityComparer and HashMap from the
source-built library, checks an integer divisor policy, escaped callbacks, long/byte/bool
captures and distinct array-loop item captures (123). This is native execution evidence,
not a claim that the separately recorded .NET loop-capture issue is repaired.

The test also exposed missing Byte/Int32-to-Int64 promotion in portable binary emission.
Emission now follows the existing bound operator operand types, including mixed signed
comparisons, without changing binding or ordinary CLR emission. Relative to .NET this
restores the expected supported integer behavior, rather than adding a numeric semantic
divergence. No metadata writer/runtime change or new public API is required. Explicit
Runtime Contract, primitive core and source/seed ownership remain unchanged.

The expanded broad gate, seven native consumers and 37 focused C#/.NET tests pass.
Both mutable reference and mutable scalar captures reject without publishing output.
The full unchanged comparer sample remains blocked by missing StringComparer service
bindings and primitive CompareTo contracts; this focused case is not a replacement for
that sample. The promotion fix is recorded in Raven as a deferred general candidate.

## Source-built comparer and explicit primitive bootstrap (2026-10-04)

`comparer-ownership.json` extends the source library with unchanged
`System/StringComparer.rvn`; applications import the emitted artifact, not that source.
The new `--reference-comparer-storage-core` exporter mode supplies declaration-only
Int32.CompareTo, String.CompareOrdinalIgnoreCase and RuntimeServices.StringHashOrdinalIgnoreCase
on the primitive storage core. The executable `comparer-seed.neoil` supplies overflow-free
integer comparison and the case-folded hash adapter. Shared union seed adapters now
expose String equality/comparison operators and Object hashing/reference identity through
existing runtime services. No placeholder CLI body is executed. Full primitive source
compilation remains open, so these members remain bootstrap-owned and explicitly scoped.

Generate the core with the existing Probe build against the matching Raven compiler:

```sh
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-comparer-storage-core /tmp/ComparerCore.dll
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_unions.py \
  --comparers --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /tmp/ComparerCore.dll --output /tmp/fresh-comparer-gate
```

Raven `7b3928239` emits the bound static operator call and explicitly admitted Object
hash slot. Metadata import maps only selected explicit core String/Int32 members to
intrinsic receiver storage; it does not import String as a nominal layout. String uses
its reference value and Int32 a managed address. Missing/nonvirtual Object hash slots,
wrong results, virtual primitive members and incorrect receiver modes reject. Native
and CLI encodings remain unchanged. The emitter still uses compiler symbols and host
artifact contracts; native library references have no fallback to CLI projection.

The focused string-comparer-consumer verifies UTF-8 scalar order (intentionally different
from .NET UTF-16 ordinal order), Unicode simple-folded equality/hashes, matching Object
hashes, map replacement and extreme signed comparisons. It passes along with the broad
application and seven native consumers. 128 metadata groups, dedicated native/static
String binding checks and 34 focused C#/.NET tests pass. Guest API snapshot validation
still reports the known stale snapshot; the host API manual reference is updated.

The unchanged full library-comparers sample now reaches an unsupported integer-range
BoundForStatement. It was not edited or replaced as an acceptance claim. Range lowering
is the next bounded task; general primitive source ownership and the larger guest
introspection library are separate subsequent work.

## Unchanged comparer sample execution (2026-10-04)

Raven `1fb1bbd45` removes the signed integer-range blocker. The `--comparers` gate now
compiles and executes unchanged library-comparers against the separate source-built
library, requiring exactly `Comparer contract passed` and exit 0. The original sample
was not rewritten. The focused range-consumer returns 42 after checking evaluation
order/once-only bounds, inclusive/exclusive ascending and descending loops, zero-step
and direction-mismatch empty loops, nested labeled continue, break, loop captures and
Int64 ranges. Ordinary .NET range emission remains the established independent path.

A subsequent comparer parameter-receiver gap required argument-address emission.
The metadata IILGenerator now offers LoadArgumentAddress(index) and raw Ldarga; CLI
and native writers share exact-type/slot validation. Static and instance parameters,
generic method scopes and mutation execute on both runtimes. Receiver slots, already
byref parameters, invalid indices (including unreachable code) and mismatched stores
reject. Runtime ldarga already existed, so no VM or format change was necessary.
The final Object.ReferenceEquals call required support for CLI ELEMENT_TYPE_OBJECT;
that signature now retains the output's explicitly supplied core identity, including
Object array elements. No implicit dependency resolution or nominal layout is inferred.

The comparer core, retained seed and ownership manifest are unchanged from the preceding
slice. CLI remains the primitive bootstrap only; source library/application references
remain native. 129 metadata groups, 55 focused .NET tests, seven native consumers,
native argument-address/binding checks and the expanded application gate pass. API
manual docs include the new host member; the separate guest API snapshot remains stale.
Signed range increment retains existing add semantics, with no new overflow policy;
unsigned/fractional ranges and broader captures remain outside this native profile.

## Native integer sample bootstrap (2026-10-04)

With compiler `1fb1bbd45`, unchanged `library-integers.rvn` now compiles and executes
through the ordinary native driver. The comparer-storage primitive core explicitly
adds `Int32.Equals(Int32)` and a nonvirtual `Int32.ToString()` declaration. The retained
seed supplies readonly byref receivers: equality uses `ceq`, formatting calls the existing
Int32ToString runtime service. The sample checks local and parameter receivers, equality,
comparison and formatting at both signed extrema. The expanded `--comparers` acceptance
gate requires exact stdout and exit 0, alongside the separate native library/application.

This is explicit primitive bootstrap ownership, not compilation of `System/Int32.rvn`.
As with existing bootstrap primitive methods, ToString is a concrete direct member;
it does not establish .NET's virtual Object-slot override or boxed dispatch semantics.
Declaration-only CLI placeholder bodies never execute. The eventual native primitive
source contract must preserve those distinctions explicitly. Application/library references
remain native, and metadata/runtime encoding and both compiler backends are unchanged.
No new .NET regression run is needed for this bootstrap-only slice; the preceding
55 focused tests remain the compiler baseline. The full dual-target library gate stays open.

## Native value-result receivers (2026-10-04)

Raven `9d06edf80`'s portable body planner now gives supported value-returning property/indexer getters
and ordinary calls a temporary local address for instance calls. It evaluates the
receiver once, before arguments, and never writes the copy back. Existing local,
parameter and field receivers retain their storage addresses; parenthesized receivers
preserve that distinction. Existing managed-reference/local-address capabilities govern
admission. Byref results and other unsupported expressions are not guessed into copies.
No importer objects or target-specific builders enter the shared plan. The established
.NET body emitter is unchanged; C# controls verify getter-copy versus field mutation.

NeoCLR's explicit primitive bootstrap now includes Int64.CompareTo with exact-width
readonly byref receiver validation. CLI uses the ordinary Int64 member reference and
native output uses the existing primitive owner form; no format or VM changes. This is
not source-built Int64 and does not add arbitrary primitive virtual dispatch. Hosts must
regenerate the comparer core and matching retained System seed together. Native library
and application references still use direct metadata import.

The new value-result-consumer executes against the separately compiled native collection
library: ArrayList<long> copy/indexer behavior, getter/call evaluation order, signed
extreme comparisons, and mutable-struct copy versus stored-field mutation all pass.
Expanded application acceptance, seven native consumers, 129 metadata groups, dedicated
native binding execution/rejection checks, three new C#/.NET checks and 26 existing
range/function checks pass. The unchanged full library-generic-collections sample now
rejects only because Date is absent, before output publication. Date's source depends on
larger globalization contracts; no stub or modified sample substitutes for it.

This portable planner extension is a deferred general candidate for independent
main-based validation when another backend consumes it. The current .NET emitter
already implements temporary receiver behavior; no .NET repair is claimed.

## Source-built Duration foundation on both targets (2026-10-04)

The native acceptance tool now provides `--calendar-foundation`, which extends the
comparer ownership manifest with unchanged `ComparableTo<T>`, `EquatableTo<T>` and
`Duration` sources. These declarations belong to the source-built library; the retained
seed and primitive core are unchanged. A native consumer imports only the emitted
library and exercises ArrayList<Duration> storage, default values, copying, iteration,
equality and signed-extreme comparison. The existing broad native application still runs.

The same value-contract consumer also compiles and executes on ordinary .NET against
an independently emitted library containing those three unchanged sources. The .NET
control uses the net10 targeting pack and installed Microsoft.NETCore.App with embedded
compiler shims, no reference-only runtime service stubs. Both consumers return 42 with
empty stdout. This validates a bounded common source subset, not full collection-library
parity. No compiler, metadata, runtime or guest API implementation changed in this slice.

A compile inventory of unchanged Date/calendar/globalization dependencies reaches
binding errors for missing string indexing, RuntimeServices.SystemCultureName and
RuntimeServices.UnixTimeToLocal. The latter produces cascading invalid-index diagnostics.
No output is published. These are the first observed blockers, not an exhaustive list of
emission/runtime gaps. Date is not replaced with a stub or a source-edited approximation.
Next work must give those primitive/service contracts explicit owners and executable
adapters while preserving the shipped grapheme-based Char/indexing contract; existing .NET behavior is the control.

Reproduce with the existing comparer-storage core and current compiler/runtime:

```sh
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_unions.py \
  --calendar-foundation --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /tmp/LongComparerCore-1004.dll --output /tmp/fresh-duration-gate
```

This mode adds three sources (25 total native library sources) and runs the normal
.NET driver for the independent three-source foundation. Reports contain all commands,
source/artifact hashes, ownership and compiler/runtime revisions. The calendar inventory
and executed gate are recorded in [the evidence](../duration-foundation-2026-10-04.json).

## Native calendar service and grapheme-length bootstrap (2026-10-04)

The comparer-storage declaration core and retained seed now expose the existing
RuntimeServices.SystemCultureName() and UnixTimeToLocal(long) contracts. The latter
returns the runtime's legacy Int32 value array; the seed adapter copies its eight fields
into a fresh nominal array reference, following the existing translated-library adapter.
That explicit conversion costs one allocation and eight element copies per call; no
performance improvement is claimed. The primitive core contains metadata-only declarations,
while execution calls the real runtime services. Rebuild the core and seed together.

The base seed also implements String.get_Length through StringGraphemeCount. Native
String.Length counts extended grapheme clusters, unlike .NET's UTF-16 code-unit length.
The preceding Duration integration note's reference to preserving scalar indexing was
incorrect: the shipped Char/indexing contract is grapheme-based. Scalar traversal is a
separate API. This slice does not yet add String's indexer or native Char signature support.

The calendar-foundation consumer checks combining/ZWJ grapheme length, host culture
service invocation without assuming a locale, eight date/time fields, positive and
negative fractional Unix ticks, fresh independent array storage and the unchanged
out-of-range fault. Expanded native application acceptance and paired .NET/NeoCLR Duration
consumption pass. An old seed fails member validation before output publication.
The unchanged Date/calendar/globalization inventory now has only two string-indexing
binding errors; the missing services and cascading array-index errors are resolved.
Later emission/runtime gaps remain unassessed while binding fails. No compiler, metadata
or runtime implementation changed, and ordinary .NET emission is untouched.

The calendar-foundation command above still applies, but regenerate the declaration
core with `--reference-comparer-storage-core` before running this slice. An older core
has no calendar service declarations; an older seed lacks their implementation (and
String.get_Length). The source-ownership manifest stays unchanged. The conversion is
owned by the retained seed until the native service array ABI is intentionally migrated.
See [the shipped text contract](../../../design/text-abstraction.md) and
[execution/inventory evidence](../calendar-services-2026-10-04.json).

## Direct native grapheme character imports (2026-10-04)

Raven `2bd39805c`'s native emitter now admits the configured core Char in imported native callable
signatures and resolves it through the explicit host artifact binding. This keeps
character signatures in the symbol-to-emitter path; no importer object is reused.
Metadata decodes/encodes canonical CLI CHAR while native output uses the existing
grapheme Char representation, including the intrinsic method owner. The primitive
core/seed expose String's indexer and Char.ToString; regenerate the comparer core.

A separately compiled CharacterContracts library returns and accepts char. Its separate
consumer indexes combining and emoji ZWJ graphemes and preserves their full text through
native import, calls and ToString. The broad native gate, paired Duration controls,
seven native consumers and 130 C# metadata groups pass; a dedicated C# native execution
check covers reimport, projection and invalid character aliases/receiver contracts.
CLI C# controls preserve UTF-16 code units, including surrogates. The ordinary .NET
emitter and runtime are unchanged; CLI declaration projections cannot carry native
multi-scalar grapheme values as executable CLR char values.

The unchanged Date dependency inventory now passes binding and reaches unsupported
BoundPatternAssignmentExpression emission (discard assignments after propagation).
A separate character-array receiver test explicitly rejects before publication: the
portable emitter still lacks array-element addresses. The successful character consumer
does not claim that capability. Both gaps remain visible; next is the Date discard path.
