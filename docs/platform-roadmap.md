# neoCLR platform roadmap

## Active priority: full-System capability batches (2026-10-04)

The author directed a strategic assessment of blockers that unlock the most library
compilation, replacing API-by-API progression. Follow the
[full-System strategy](experiments/extended-cli-metadata/system-compilation-strategy.md)
and [fresh audit](experiments/extended-cli-metadata/system-compilation-audit-2026-10-04.json).
Earlier chronological next-step notes below are historical where they conflict.

Current checkpoint: the Number numeric-family story is complete, including all ten
source implementations and separate generic-library/consumer execution. The cumulative
70-source numeric library compiles in the fresh [post-Number audit](experiments/extended-cli-metadata/system-post-number-audit-2026-10-04.json).
Earlier Number/Self/numeric prerequisite notes below are historical.

The source-order binding defect is now fixed: provisional conversions no longer enter
Raven's cache during declaration binding. Empty-first and reversed 70-source builds
pass; the 76-source combination with Tasks/Concurrency also compiles. Artifact-only
numeric, generic Number, broad application and task consumers execute. Raven 459856a71
and independently validated main f0c3b75a0 each pass 139 focused .NET tests.
[Execution and regression evidence](experiments/extended-cli-metadata/source-order-conversions-2026-10-04.json).

The common text-service prerequisite now executes through native libraries: Unicode
scalar classification, grapheme/vector operations, Unicode casing and UTF-8 slicing.
The expanded seed preserves the numeric gate. Explicit interface properties and pattern
if-expressions pass native emission. The text prerequisite evidence below predates the
completed canonical text ownership gate described next. [Text evidence](experiments/extended-cli-metadata/text-services-native-2026-10-04.json).

See [explicit-property integration](experiments/extended-cli-metadata/explicit-properties-2026-10-04.md).

Source-owned String now executes instance operations and collection-interface dispatch
through a separate native consumer, with an explicit retained seed and ownership manifest.
See [the String integration](experiments/extended-cli-metadata/source-string-2026-10-04.md).
Source-owned Char and String now execute together, including literal/pattern/equality
lowering, interfaces and the real Sequence<char> String constructor. Separate native
consumers pass the existing grapheme, comparison, slice and construction samples.
See [text gate and evidence](experiments/extended-cli-metadata/source-text-2026-10-04.md).
Imported nullability and wider source groups remain open.

Next expand the verified text-service foundation into stream, storage and JSON sources, ordinary cross-assembly class inheritance,
and the metadata-handle/introspection cluster. Expand remaining service families and
callback/storage/generic support from demonstrated failures. See the strategy's
[current reassessment](experiments/extended-cli-metadata/system-compilation-strategy.md#current-reassessment-after-number-2026-10-04)
for bounded gates and ordering.

The full 166-source attempt still stops in binding; its 141 distinct missing runtime
service members measure the selected bootstrap's coverage, not missing runtime
implementations. The audit adds compilation evidence only. Preserve existing native
execution gates and ordinary .NET controls; full rebuilt-library dual-target parity
remains open. No backend rewrite, format fork, speculative async prerequisite or
performance claim follows from this assessment.


Fixed-offset milestone (2026-10-04): unchanged TimeOffset joins the 48-source --offsets
gate. Separate native consumers execute signed/fractional tick round trips, offset
limits and civil boundaries; expanded native acceptance and paired Duration pass.
Raven bc30fb0f2 admits imported value-type static properties via explicit capability;
17 focused C# controls pass. Next bounded work: explicit time-zone bootstrap services
(database version, lookup, system zone, offset and local mapping). Source inventory
rejects those missing services before output; cascading errors remain unassessed.
Full .NET calendar/offset parity, array-element addresses and legacy snapshot refresh
remain open. No metadata/runtime/bootstrap change in this slice.
[Evidence](experiments/extended-cli-metadata/offset-source-2026-10-04.json).


Clock milestone (2026-10-04): the 47-source --clock gate builds source-owned Instant,
Clock and SystemClock, then executes separate clock/overflow/interface consumers.
Expanded native application/calendar/globalization and paired Duration pass; two runtime
clock tests pass. Explicit UnixTimeTicks core/seed binding is required. Next bounded
library work: fixed offsets, then time-zone service contracts. Full .NET clock/calendar
parity and array-element receiver addresses remain open. Legacy translated snapshot
refresh fails in its union-reference build ('None' not in scope); keep that maintenance
blocker explicit without changing unverified hashes.
[Evidence](experiments/extended-cli-metadata/clock-source-2026-10-04.json).


Globalization milestone (2026-10-04): the unchanged larger sample now executes through
native library imports with explicit --instructions 1000000. Default limits are unchanged.
Expanded --calendar acceptance, Hebrew formatting, broad native application, paired
Duration and six CLI tests pass. Next bounded source-library work: Clock/SystemClock
dependency ownership; then Instant/TimeOffset/time-zone contracts. Their absence in the
current 43-source subset is confirmed by compiler inventory. Full .NET calendar parity
and array-element receiver addresses remain open. No compiler/metadata change.
[Evidence](experiments/extended-cli-metadata/globalization-budget-2026-10-04.json).


Date/calendar milestone (2026-10-04): terminal runtime-call control flow now closes
let-else failure paths (Raven 5c60425db). The --calendar gate builds 43 unchanged sources;
separate Date formatting/arithmetic, generic collections and broad native application
consumers execute. Paired Duration and 60 focused .NET checks pass. Unchanged Hebrew
date formatting also matches expected output. The larger globalization consumer compiles
but reaches the CLI instruction budget; next bounded work is explicit execution-budget
configuration for that native multi-assembly case. Full .NET calendar coverage and
array-element receiver addresses remain open. No metadata/runtime implementation change.
[Evidence](experiments/extended-cli-metadata/calendar-source-2026-10-04.json).


Nested propagation milestone (2026-10-04): eager binary local initializers now lower
in evaluation order (Raven cb0f48bd7; main backport 7db0f3dfe). Native reference-only
Result checks, broad application acceptance and paired Duration pass; focused .NET
tests pass (12 integration, 11 main). Next bounded task: Date.ToString's metadata
verifier rejection, local loaded before store on some path. Date compilation publishes
no artifact and execution remains open. This slice does not generalize arbitrary
expression propagation or change metadata/runtime contracts.
[Evidence](experiments/extended-cli-metadata/nested-propagation-2026-10-04.json).


Discard propagation milestone (2026-10-04): Raven a99152da0 normalizes discarded
propagation in shared lowering and emits portable discard statements. The independently
compiled native Result consumer checks once-only evaluation and early failure returns;
expanded native application acceptance and paired Duration pass. Eighteen focused
.NET checks pass; the general lowering fix is independently isolated at 8e0f88eda
with six tests, then backported to main as 22539952c with the same six checks passing.
Unchanged Date/calendar sources move to an unlowered nested propagation
expression; a minimal `Read()? + 1` reproduces rejection before publication.
Next bounded task: normalize nested expression propagation while preserving evaluation
order and early returns. Array-element receiver addresses and full Date execution remain
open. No metadata/runtime/bootstrap change.
[Evidence](experiments/extended-cli-metadata/discard-propagation-2026-10-04.json).


Grapheme character milestone (2026-10-04): explicit Char signature import, canonical
CLI encoding and native intrinsic owners allow separate-library character calls and
String indexing to execute. Raven `2bd39805c` preserves symbol-only emission for core
Char. Broad native acceptance, paired Duration, seven consumers and 130 C# groups pass.
Unchanged Date sources now pass binding; next bounded task is discarded propagated
values in the portable emitter. Array-element receiver addressing is a separate recorded
gap with a failure-before-publication test. No VM/format change.
[Evidence](experiments/extended-cli-metadata/characters-2026-10-04.json).

Calendar bootstrap services (2026-10-04): explicit core/seed bindings now execute host
culture lookup, local-time conversion and grapheme-count String.Length. The existing
value-array-to-reference-array adapter preserves nominal array ownership; Unicode,
component/fraction/storage and range-fault checks pass alongside native broad application
and paired Duration acceptance. No compiler/runtime implementation change. The unchanged
Date dependency inventory now reports only string-indexing binding errors. Next bounded
work remains the grapheme Char signature/indexer contract across import, emission and
runtime linking; the earlier scalar-indexing wording was incorrect, not a new direction.
[Evidence](experiments/extended-cli-metadata/calendar-services-2026-10-04.json).

Duration foundation (2026-10-04): unchanged Duration, ComparableTo and EquatableTo
compile into separately consumed libraries on both targets. The same value-contract
consumer returns 42 on .NET and NeoCLR; native ArrayList<Duration> and broad application
acceptance also pass. No compiler/metadata/runtime implementation change. The Date
source inventory now identifies binding prerequisites: string indexing, SystemCultureName
and UnixTimeToLocal. Next bounded work is the explicit native string-index/character
contract needed by culture selection/formatting, then the two executable service adapters.
Do not infer later emission/runtime completeness from this binding inventory.
[Evidence](experiments/extended-cli-metadata/duration-foundation-2026-10-04.json).

Value-result receiver milestone (2026-10-04): Raven `9d06edf80` preserves getter/call
copy semantics while native bootstrap imports admit exact Int64 receiver storage.
A separately compiled ArrayList<long> consumer verifies copy/indexer behavior, evaluation
order, signed extrema and mutable struct receiver semantics. Broad native acceptance,
seven consumers, 129 metadata groups and 29 focused C#/.NET checks pass. The unchanged
full generic-collections sample now rejects only for absent Date. Next bounded work is
inventorying Date/globalization source dependencies before selecting a coherent library
expansion; do not substitute a Date stub. Source primitive ownership remains open.
[Evidence](experiments/extended-cli-metadata/value-receivers-2026-10-04.json).

Integer bootstrap acceptance (2026-10-04): unchanged library-integers now executes
with exact output using explicit Int32.Equals/ToString seed contracts. No compiler,
metadata or runtime code change; the expanded native broad gate passes. Source-built
Int32 and boxed virtual dispatch remain open. The next unchanged generic-collections
sample rejects at binding for absent Int64.CompareTo and Date; these are missing
selected-library contracts, not demonstrated .NET regressions.
[Evidence](experiments/extended-cli-metadata/integers-2026-10-04.json).

Full comparer sample execution (2026-10-04): Raven `1fb1bbd45` lowers bound signed
Int32/Int64 ranges behind the explicit native capability and uses argument addresses for
value-parameter receivers. Metadata now exposes LoadArgumentAddress/Ldarga and decodes
CLI Object signatures through the explicit core identity. Unchanged library-comparers
compiles and runs with exact output against the separately built native source library.
129 metadata groups, native address/binding execution checks, 55 focused .NET tests,
seven native consumers and the expanded broad gate pass. No runtime/format change.
Next bounded work is reassessing the remaining unchanged numeric/collection samples
against the expanded library; broader guest introspection and full primitive source
ownership remain open. [Evidence](experiments/extended-cli-metadata/ranges-comparers-2026-10-04.json).

Native source comparer milestone (2026-10-04): unchanged StringComparer.rvn now
compiles into the independently consumed native class library. The explicit comparer
bootstrap supplies primitive String/Int32 members and runtime services; metadata imports
validate primitive receiver modes and the Object.GetHashCode virtual slot. Raven
`7b3928239` admits that bounded slot and bound static binary operators. The focused
consumer verifies ordinal/folded equality and hashing, Unicode scalar ordering, map
replacement and extreme Int32.CompareTo. Expanded broad native gate, seven consumers,
128 metadata groups, dedicated native binding checks and 34 C#/.NET tests pass.
Next bounded blocker: integer-range loop lowering in unchanged library-comparers.
No runtime/format change or full comparer-sample completion is claimed.
[Evidence](experiments/extended-cli-metadata/source-comparers-2026-10-04.json).

Native primitive capture prerequisite (2026-10-04): Raven `1b15715bf` admits immutable
Int32/Int64/Boolean/Byte local captures through the existing closure frames. A separately
compiled FunctionEqualityComparer/HashMap consumer executes with a captured divisor,
escaped callbacks and distinct per-iteration native values. Mixed-width arithmetic now
honors the bound operator's promoted operand types. Broad native acceptance, seven
consumers and 37 focused .NET/C# checks pass. No metadata/runtime or bootstrap change.
Next: explicit StringComparer runtime-service and primitive CompareTo ownership for the
unchanged comparer sample; arbitrary structs/mutable/parameter/receiver captures remain
outside this profile. [Evidence](experiments/extended-cli-metadata/primitive-captures-2026-10-04.json).

Native query acceptance (2026-10-04): unchanged library-query-basics and
library-query-names compile and run with checked-in expected output against the
separately compiled native source library. An instrumented iterator consumer confirms
lazy construction, early/exhaustion disposal, repeated enumeration and idempotent
explicit disposal through imported Filter/Map/Take implementations. The expanded broad
application gate passes without compiler/runtime changes. A seven-sample assessment
records remaining library-surface gaps: numeric/comparer APIs and guest introspection
are absent from this selected library. Next bounded expansion is numeric/comparer source
ownership and primitive contracts; guest introspection remains a separate, larger step.
These are binding diagnostics, not evidence of new .NET regressions or full-library
completion. [Evidence](experiments/extended-cli-metadata/query-acceptance-2026-10-04.json).

Native reference captures (2026-10-03): Raven `623cbc1d8` lowers lambdas capturing
immutable reference locals into fresh private frames using existing instance callback
bindings. Unchanged library-list-filters now compiles, verifies and executes with exact
output against the separately built native class library. Escaped callbacks, independent
factory instances and shared object mutation pass; mutable bindings reject before output.
The broad application, seven native consumers and 31 focused .NET tests pass. No runtime,
metadata format or bootstrap configuration change. Mutable/value/parameter/receiver
captures remain unsupported; general lexical shared storage and full-library coverage
remain open. [Evidence](experiments/extended-cli-metadata/reference-captures-2026-10-03.json).

Receiver-bound callbacks (2026-10-03): Raven `a53b6412a` and the metadata writer now
support owned nongeneric nonvirtual reference-instance callback targets. The runtime's
existing instance binding preserves shared mutable receiver identity, confirmed by CLR
and NeoCLR execution and a source method-group consumer of separately built ArrayList.Find.
All 128 metadata groups, 31 focused .NET tests, seven native consumers and the expanded
broad source-library gate pass. This is a captured-lambda prerequisite, not completion:
next work remains closure-frame lowering for unchanged library-list-filters, including
shared mutable captures and correct lexical lifetimes. No runtime/format-version change.
[Evidence](experiments/extended-cli-metadata/instance-callbacks-2026-10-03.json).

Native array callbacks (2026-10-03): Raven `35464aabf` preserves imported no-result
callback contracts separately from inhabited source unit and emits receivers through the
configured nominal array backing. The metadata library admits nested vector signatures
using existing CLI SZARRAY/native ArrayRef encodings; the runtime accepts exact nominal
backing casts without copying storage. Unchanged library-array-callbacks and a nonempty
nested-array mutation consumer now pass alongside the broad source-library application.
Validation: 127 C# metadata groups, 27 focused .NET tests, 19 runtime tests and seven native
consumers pass. Next bounded native gap: captured callbacks in library-list-filters.
[Evidence](experiments/extended-cli-metadata/native-array-callbacks-2026-10-03.json).

Native end-to-end sample expansion (2026-10-03): Raven `3367f3200` fixes vector
receivers for projected inherited interface accessors, exposed by Array<int>.Count in
unchanged collection-capabilities. Existing reference conversion uses semantic symbols
and target capability checks; nominal Array<T>, metadata and runtime remain unchanged.
The source-library driver gate now includes a Count/mutation regression and three
unchanged samples (Option, propagation, collection capabilities), all verified/executed
with exact output/status alongside the broad order-collections application. Seven native
consumers and 13 focused .NET tests pass. Next bounded native task: no-result callback
import/emission for unchanged library-array-callbacks. Captured functions and missing
numeric/date source coverage remain separate gaps; full-library completion is not claimed.
[Evidence](experiments/extended-cli-metadata/native-array-receiver-2026-10-03.json).

Native catalog validation (2026-10-03): Raven `42503ec23` converts conflicts between
native snapshots and the explicit primitive bootstrap into RAVT003, including failures
while constructing the introspection catalog. A C# regression reproduces the previous
uncaught exception and now verifies both reference orders and unchanged output bytes/
position on failed emission. Seven native consumers pass. No Runtime Contract, format,
runtime or ordinary .NET behavior changes; prior broad-application evidence is reused.
[Evidence](experiments/extended-cli-metadata/native-catalog-validation-2026-10-03.json).

Native type materialization cleanup (2026-10-03): Raven `777499170` constructs ordinary
type and union/case/companion symbols from canonical introspection views and removes
obsolete raw-signature mapping helpers. The separately compiled source-union library
and unchanged application-order-collections pass, as do all seven native consumers.
Runtime Contract/bootstrap configuration, metadata format and ordinary .NET paths are
unchanged. The native gate is evidence for this cleanup, not full dual-target completion.
[Evidence](experiments/extended-cli-metadata/native-type-materialization-2026-10-03.json).

Native type facts (2026-10-03): Raven `003b9a38d` consumes declaration and generic
parameter facts through the retained type view. The host facade now supplies declared
parameter Name with explicit CLI limitations. All 127 metadata groups and seven native
consumers pass; ordinary .NET is unchanged. Remaining definition use is concentrated in
root declaration materialization and union transport, requiring separate review.
[Evidence](experiments/extended-cli-metadata/native-type-facade-2026-10-03.json).

Native callable cleanup (2026-10-03): Raven `015e6f66d` constructs method, constructor
and module-function symbols from introspection views, preserving declaration order and
canonical member identity. All seven native consumers pass. Remaining definition use
is in type/union construction and explicit legacy bindings, not callable-symbol creation.
[Evidence](experiments/extended-cli-metadata/native-callable-facade-2026-10-03.json).

Native importer cleanup (2026-10-03): Raven `12b545bc1` consumes field/property
introspection views directly and reuses canonical accessor symbols. Seven native consumers
pass before/after. The remaining legacy references have explicit seed/probe callers;
none were removed speculatively. Continue reviewing remaining declaration construction
against existing facade facts, preserving ordinary .NET behavior.
[Evidence](experiments/extended-cli-metadata/native-member-facade-2026-10-03.json).

Array-lowering boundary (2026-10-03): Raven `19a3e84c0` confines vector expansion to
portable body planning; ordinary .NET retains its established loop emitter. The 86
focused checks, native broad application and native labeled-loop case pass. Loop capture
now matches main's known failure rather than acquiring a different integration result;
correct lexical closure lifetimes remain independent work. Next target cleanup: inventory
native-versus-legacy reference consumers before removing any bridge branch.
[Evidence](experiments/extended-cli-metadata/portable-array-boundary-2026-10-03.json).

Compiler simplification implementation (2026-10-03): Raven `c2a66d82a` removes the
second production .NET body emitter while retaining the native planner and metadata
library. All 94 selected checks pass before/after, and the native broad application
executes against its separate library. Constraint expectation reconciliation is committed
separately; closure lifetime/shared array lowering remains the next unresolved compiler
audit. Further importer and legacy-bridge cleanup is still pending, not completed.
[Evidence](experiments/extended-cli-metadata/dotnet-emitter-simplification-2026-10-03.json).

Target-boundary audit (2026-10-03): retain the NeoCLR metadata library and native
symbol/backend boundaries. Raven audit commit `10f5f0089` records 71 further passing
.NET checks on both lines and one shared constraint round-trip failure. Reconcile that
baseline discrepancy and evaluate simplifying the second production .NET body path
before expanding adapters. These are bounded review recommendations; no backend has
been removed or replaced. See the [conversation record](development-timeline.md).

Direction reassessment (2026-10-03, author-directed): pause new adapter/compiler work
and re-establish ordinary Raven/.NET behavior against main before selecting another
implementation slice. The proposed CLR array adapter below is suspended. Current main
and integration both pass 36 focused array/iteration checks and an ordinary .NET
array/IList mutation/LINQ executable. The failing bootstrap configuration asserts custom
NeoCLR interfaces for CLR vectors; it is not evidence that ordinary .NET arrays regressed.
Keep native acceptance, ordinary .NET regression checks and cross-runtime source-library
compatibility evidence distinct. This reassessment does not claim the earlier paired
source-library gate complete or permanently remove it from scope.
[Audit](experiments/extended-cli-metadata/dotnet-main-audit-2026-10-03.json).

Paired .NET unit milestone (2026-10-03): explicit source-void/unit mapping removes invalid
CLR generic storage. Separate union, ArrayList, HashMap and query consumers run successfully
on .NET; 22 unit-contract tests pass on both Raven branches. Native broad execution remains
passing. The .NET broad consumer now compiles and executes its expected prefix before
custom array-interface execution terminates abnormally. Next bounded task: an explicit
.NET array adapter conversion, with unsupported configurations rejected before emission.
[Evidence](experiments/extended-cli-metadata/dotnet-unit-execution-2026-10-03.json).

Paired .NET assessment (2026-10-03): executable storage/failure adapters pass. The shared
safe override-return strengthening fix is independently validated on Raven's main-based
fix branch. Native broad execution remains green. The unchanged .NET source library emits,
but consumer compilation fails loading Propagatable instantiated with CLR System.Void.
Next bounded task: explicit source-unit/CLR-value projection and rejection of invalid void
storage before publishing an assembly. Then prove .NET array/interface adaptation and the
unchanged broad consumer. [Evidence](experiments/extended-cli-metadata/dotnet-source-assessment-2026-10-03.json).

Broad native gate update (2026-10-03): unchanged application-order-collections compiles
against the separately emitted source library and executes with exact checked-in output
and exit 0, including collection mutation, callback/query composition, union propagation,
array iteration and shared object identity. Reproduce with the bootstrap acceptance script
`--application`; [evidence](experiments/extended-cli-metadata/broad-native-execution-2026-10-03.json).
Next bounded gate: executable .NET source-library adapters and the paired unchanged sample;
full-runtime compilation and broader native categories remain subsequent work.

Nominal-array execution update (2026-10-03): the separate native consumer now mutates
vector storage through MutableSequence<int>, observes the same storage through its array
reference, and executes Filter/ToList through source Array<T>/ArrayIterator<T> (42).
[Evidence](experiments/extended-cli-metadata/nominal-array-execution-2026-10-03.json).
Next: resume unchanged application-order-collections with this ownership manifest and
close its runtime-service binding gaps. Full dual-target library/application execution
remains open.

Author clarification (2026-10-03): retain nominal Array<T> backing and defer structural
array decisions. Source Array/iterator and separate consumer compilation succeed; the
next bounded task is identity-aware vector-to-nominal interface backing/runtime dispatch.
[Assessment](experiments/extended-cli-metadata/nominal-array-assessment-2026-10-03.json).

Query gate update (2026-10-03): unchanged complete query sources now compile and import
into an executing native consumer (OfType/Filter/Map/ToList/Single, boxed values and shared
reference identity). [Evidence](experiments/extended-cli-metadata/query-import-2026-10-03.json).
The unchanged broad application next fails at array extension lookup (Order[].Filter);
configured iteration identity/conversion is the next bounded task. The full dual-target gate remains open.

Query integration update (2026-10-03): native extension declaration/import now executes
a separate generic extension library and consumer. The unchanged full query library
advances to OfType object-to-generic conversion, which still rejects before publication.
[Assessment](experiments/extended-cli-metadata/query-source-assessment-2026-10-03.json).
The broad application and full dual-target source-library gate remain open.

Source-library gate update (2026-10-03): unchanged HashMap and comparer sources now
compile into a separately imported native library and execute collisions, growth,
replacement, callback dispatch and shared object mutation. No backend change was needed.
[Evidence](experiments/extended-cli-metadata/hashmap-import-2026-10-03.json).
Query composition, the broad application and executable .NET class-library adapters remain open.

Author priority clarification (2026-10-03): finish Raven targeting neoCLR and executable
end-to-end compilation. CLI tables, signatures and instructions remain the baseline;
new semantics and broader format redesign are subsequent experiments, not prerequisites
for the current gate. Make only metadata changes required by demonstrated integration
failures. The current PE/#Neo execution transport remains transitional; retaining it for
this gate does not establish ordinary CLI metadata or IL as authoritative at runtime.
See [scope clarification](design/dual-target-compilation.md#integration-priority-clarification-2026-10-03).

Union continuation (2026-10-03): bounded type custom-attribute authoring, native/CLI
round trips and catalog-based introspection now provide the metadata foundation.
Raven union publication and imported semantic reconstruction remain pending; see the
[foundation scope](design/dual-target-compilation.md#union-attribute-foundation-2026-10-03).

Author-approved implementation gate (2026-10-03): [dual-target driver and source-library plan](design/dual-target-compilation.md).
Paired ordinary-driver Hello and separate generic library/consumer cases now execute on
both targets; native references load directly with explicit --core-reference. Next the broad
collections sample, retaining only the explicit primitive bootstrap/runtime seed.

Author clarification (2026-10-02): importer and emitter must communicate through Raven
symbols, without emission reusing loader objects. Raven compiler contracts and the
metadata library body-generator API are independent boundaries. The first bounded
slice now emits primitive/method-generic/vector namespace-function references from
symbols and artifact values (seven consumers execute; 107 metadata C# groups pass).
Public top-level root-class identities, including unconstrained generics, now also
author directly from symbols; other type profiles retain their existing conversion
metadata path. Namespace-function signatures now also carry root-class constructions and vectors
through symbol-only reference authoring. Public nonvirtual root-class methods and constructors now also reconstruct from
symbols, including owner generic parameters. Public nongeneric root-class fields now also author from symbol storage and explicit
layout ordinals. Nongeneric interface identities, direct relationships and abstract dispatch now also
author from symbols. The library now exposes its own IILGenerator, consumed only behind Raven’s NeoCLR
adapter. The audit identifies translated CLI bindings, unsupported profiles, host setup
and lazy symbol materialization as remaining reader dependencies. The generator now also owns body-authoring internals; builder methods forward while
writer-side validation remains unchanged. Static native containers now also reconstruct as declaration owners from symbols,
with value signatures still excluding static classes. The audit found native concrete implementations do not carry CLI-projection final/virtual
flags. Native callable reader fallback is now removed; incomplete contracts diagnose.
Native type/field fallbacks and the emitter resolver are now removed too; all seven
consumers execute. Native host bindings now use the registered compiler reference's
captured identity/digest without a separately supplied reader definition. Legacy CLI
bindings remain explicit. Reader-backed semantic materialization remains supported.
The generic consumer assessment exposed closed generic field storage rejection. That
bounded gap is now fixed: a separate BoxStorage library holds Box<int> and Box<int>[];
all seven consumers execute (42), and 108 metadata groups plus CLR/both-container
field execution pass. Fields on generic declaring owners and generic interface imports now also pass:
reader relationships retain owner arguments, emitted references are symbol-authored,
and generic inherited dispatch executes for primitive and nominal payloads. All seven
Raven consumers and 108 metadata groups pass, with .NET and both-container execution.
See [generic field/interface evidence](experiments/extended-cli-metadata/generic-fields-interfaces-2026-10-02.json).
Author follow-up proposes a pure metadata resolution/view layer to unify local and
external inspection. The initial C# MetadataLoadContext and assembly/module/nominal facade are implemented;
Raven uses them instead of its private dependency resolver. 109 metadata test groups pass;
[all seven consumers execute](experiments/extended-cli-metadata/introspection-context-2026-10-02.json).
Constructed type and field views now project signatures for Raven; canonical view-to-symbol
mapping preserves array identity. Method/parameter views now also project open method scopes; Raven no longer performs
recursive generic-signature projection. Declared property/direct interface views now also
substitute owner arguments for Raven (2026-10-03); [109 C# groups and seven runtime consumers pass](experiments/extended-cli-metadata/introspection-properties-interfaces-2026-10-03.json). Bounded transitive interface traversal now also lives in the facade. Generic method inspection now also supports canonical constructions on open/closed owners.
Constructor/member classification and accessibility metadata now live in the facade;
paired driver execution passes. External generic interface implementation/inheritance now
works across separately compiled contracts, implementation and consumer assemblies,
including diamond traversal and dispatch on both targets. See [driver evidence](experiments/extended-cli-metadata/dual-driver-external-2026-10-03.json).
A checked-in ownership manifest and primitive-only bootstrap now support separately
compiled iteration/collection contracts on both targets. Next extend ownership to the
retained seed and ArrayList dependencies; also implement the author-requested native Self
signature path through the metadata APIs. Ordinary native driver opt-in now enables the
existing checked-storage intrinsic, with separately compiled generic helper consumers
executing 42 and rejecting uninitialized reads. The unchanged Option/Propagatable native
interface out-parameter blocker is now resolved: metadata views preserve modes and
three-assembly ref/out dispatch executes on both targets. Unchanged Propagatable emits;
Option now reaches source-union/declaration emission rejection. The metadata foundation
now admits inline generic value payloads and direct native value snapshots; tag/payload
copy execution and imported constructors return 42 on CLR and neoCLR. Raven imports
these as structs. Ordinary top-level source structs now compile and execute on both targets with generic
inline payloads, default constructors and independent copies. Native emission selects an
explicit value declaration capability. Native nested class/value snapshots and Raven symbol ownership now also preserve
same-named case scopes; direct nested constructor imports execute on both runtimes.
See [nested case evidence](experiments/extended-cli-metadata/nested-cases-2026-10-03.json).
Ordinary nested source class/value declaration emission now also executes on both
targets, with scoped payload identity and copy checks; see
[nested source evidence](experiments/extended-cli-metadata/source-nested-2026-10-03.json).
Byte discriminator storage, conversions and signatures now execute on both targets;
see [byte evidence](experiments/extended-cli-metadata/byte-discriminator-2026-10-03.json).
Generated union declaration discovery now retains complete case/member contracts and
physical generic case owners. Native preflight reaches synthesized ToString overrides;
unchanged Option now passes value-type interface admission and reaches the same override
blocker. Metadata-generated constrained interface dispatch already executes with mutation
and independent copies on both runtimes; compiler declarations/concrete calls also pass.
See [value-interface evidence](experiments/extended-cli-metadata/value-interfaces-2026-10-03.json). Generated case constructors and payload getters now also pass shared body planning,
including generic payloads; see [core-body evidence](experiments/extended-cli-metadata/union-core-bodies-2026-10-03.json).
Bounded value ToString override declarations now preserve and execute the Object slot on CLR;
native writing now validates the explicit retained System slot, retains override names/flags,
and executes imported direct calls plus boxed ordinary/generic Object dispatch. See
[native override evidence](experiments/extended-cli-metadata/native-value-overrides-2026-10-03.json). See
[override declaration evidence](experiments/extended-cli-metadata/value-override-authoring-2026-10-03.json).
Raven now admits the bounded override and executes local ordinary/generic value calls through
explicit --runtime-seed binding. Source unions and unchanged Option reach generated ToString
conversion lowering; [driver evidence](experiments/extended-cli-metadata/raven-value-overrides-2026-10-03.json)
records the existing CLR return-nullability mismatch separately.
Union/case metadata, local synthesized display and separately compiled plain/generic
union matching now execute on both targets. Native imports retain parameter names and
physical case scopes; external value operands are authored from symbols. The unchanged Option/Result and iteration sources now build into a separate native
library and execute in a reference-only consumer with the bounded executable seed.
See [source union evidence](experiments/extended-cli-metadata/source-unions-2026-10-03.json).
The .NET adapter and broader collection/application gates remain open. See
[separate union evidence](experiments/extended-cli-metadata/union-separate-execution-2026-10-03.json). See
[union declaration evidence](experiments/extended-cli-metadata/union-declarations-2026-10-03.json). See
[union payload evidence](experiments/extended-cli-metadata/union-payloads-2026-10-03.json) and
[source value driver evidence](experiments/extended-cli-metadata/source-values-2026-10-03.json). ArrayList now has explicit Fail/callback bootstrap bindings and passes source-included
execution with alias mutation, independent copies, iteration and Find. Separate library
import and execution now also pass after function-signature materialization and metadata-only
callback views. See [separate ArrayList evidence](experiments/extended-cli-metadata/arraylist-import-2026-10-03.json). See the dual-target tracker for evidence. The first Self contract-only slice now preserves Self
through native readers/writers and scoped introspection; Raven and typed dispatch authoring
remain pending. Configured unit generic union payloads now bind and execute (42); ordinary CLI void
remains rejected without the unit contract. The broad source-library/application gate remains open.
See [closed generic field evidence](experiments/extended-cli-metadata/closed-generic-fields-2026-10-02.json).
See [native host binding evidence](experiments/extended-cli-metadata/native-host-bindings-2026-10-02.json).
See [native reference boundary evidence](experiments/extended-cli-metadata/no-native-reference-fallback-2026-10-02.json). See [callable boundary evidence](experiments/extended-cli-metadata/no-native-callable-fallback-2026-10-02.json). See [static-owner evidence](experiments/extended-cli-metadata/symbol-only-static-owners-2026-10-02.json).
See [body-engine evidence](experiments/extended-cli-metadata/body-engine-2026-10-02.json). See [generator evidence](experiments/extended-cli-metadata/body-generator-2026-10-02.json). See [interface evidence](experiments/extended-cli-metadata/symbol-only-interfaces-2026-10-02.json). See [field evidence](experiments/extended-cli-metadata/symbol-only-fields-2026-10-02.json). See [member evidence](experiments/extended-cli-metadata/symbol-only-members-2026-10-02.json).
See [nominal call evidence](experiments/extended-cli-metadata/symbol-only-nominal-calls-2026-10-02.json). See [type-reference evidence](experiments/extended-cli-metadata/symbol-only-types-2026-10-02.json). See [validation and remaining scope](experiments/extended-cli-metadata/symbol-only-functions-2026-10-02.json). See [direction and current gaps](design/extended-cli-metadata.md#independent-compiler-and-library-generation-boundaries-2026-10-02).

Author-directed priority (2026-10-02): proceed with direct native metadata import into
Raven, developing the reader/writer library toward its existing builders → definitions
→ metadata → PE architecture. Primitive namespace functions now materialize directly into the existing definitions
(103 metadata groups pass). Raven now binds these native functions directly with
exact dependency identity and compilation-owned symbols; 67 .NET regressions pass.
Native calls now import and execute across that boundary, including a Raven-produced
library read directly and consumed by Raven (42). Fieldless nongeneric static classes
and primitive static methods now also load directly into Raven; Boolean/Int32 overload
calls execute in neoCLR (42). Fieldless instance classes now also support direct
constructor/member import, allocation, local aliases and calls (42). Primitive fields
now load into definitions/symbols, and a stateful class consumer returns 42 through its
constructor and instance method. Direct public primitive imported field loads/stores
now execute too, including alias writes and direct constructor/field-read consumers (42).
Local nominal class parameter/result signatures now also load directly, including
factories, namespace/static/instance identity functions and nominal constructor arguments;
the Raven consumer executes (42). Local nominal fields now also load and emit directly; a Raven consumer replaces an
object field and mutates the replacement without changing the original (42).
Cross-dependency class signatures now resolve through explicit exact-identity metadata
resolvers; Raven-produced payload/holder libraries and their consumer execute (42).
All five native consumers pass. Primitive and nominal class arrays now also load directly
in method/constructor/field signatures, including external element types; cross-library
array aliases and element replacement execute (42). Non-indexed properties now also load with canonical accessors and execute across native
libraries (42), including static getters and nominal/vector setters. Private setter and
read-only writes diagnose. Indexed properties now also load directly, including overloads; cross-library indexed
reads/writes execute (42). Nongeneric interfaces and local inheritance/implementations now also load; inherited
method/property dispatch executes across native assemblies (42). All six consumers pass.
Interface-valued fields, constructor parameters and arrays now also work across native
libraries, preserving aliases and dispatch after replacement (42). The early-return diagnostic gap is now independently reproduced and fixed on .NET
(106 focused tests); the native negative case diagnoses before emission and all six
consumers still execute (42). The metadata reader/import API now preserves static generic methods/functions,
including parameter vectors; CLR and native execution pass (42, both containers).
Raven now imports those generic symbols through shared inference and constructed-method
substitution; all seven native consumers execute (42), including generic forwarding and
array aliases. Unconstrained native generic root classes now also import with owner-scoped
parameters and shared constructed-type/member substitution. Box<int>/Box<Item> construction,
methods, properties and vector aliases execute (42); all seven consumers and 105 metadata
groups pass. Local closed generic signatures now also retain immutable definition/argument
identity; Raven consumes Box<int> factory/identity calls (42). Scoped local constructions
Box<T> and their vectors now also import and execute with preserved method/owner parameter
identity; all seven consumers still return 42. External constructions now also resolve
through exact dependencies; a three-assembly generic consumer executes (42), with 106 C#
metadata groups passing. Qualified namespace-function lookup now includes native ownerless
methods; inferred/explicit calls execute, invalid explicit arguments diagnose and .NET
controls pass. See [qualified lookup evidence](experiments/extended-cli-metadata/native-qualified-functions-2026-10-02.json).
Constraints remain pending before full System loading. See [external generic evidence](experiments/extended-cli-metadata/native-external-generics-2026-10-02.json). See [open signature evidence](experiments/extended-cli-metadata/native-open-signatures-2026-10-02.json). See [closed signature evidence](experiments/extended-cli-metadata/native-closed-signatures-2026-10-02.json). See [generic owner evidence](experiments/extended-cli-metadata/native-generic-owners-2026-10-02.json).
See [generic import evidence](experiments/extended-cli-metadata/native-generic-symbols-2026-10-02.json). See [return diagnostic evidence](experiments/extended-cli-metadata/native-return-diagnostics-2026-10-02.json). See [storage evidence](experiments/extended-cli-metadata/native-interface-storage-2026-10-02.json).
See [interface evidence](experiments/extended-cli-metadata/native-interfaces-2026-10-02.json). Setter-only indexed assignments now also compile and execute (42);
source reads still require a getter. See [setter-only evidence](experiments/extended-cli-metadata/native-writeonly-indexers-2026-10-02.json). See [indexer evidence](experiments/extended-cli-metadata/native-indexers-2026-10-02.json). See [property evidence](experiments/extended-cli-metadata/native-properties-2026-10-02.json). See [array evidence](experiments/extended-cli-metadata/native-array-signatures-2026-10-02.json). See [external signature evidence](experiments/extended-cli-metadata/native-external-signatures-2026-10-02.json). See [nominal field evidence](experiments/extended-cli-metadata/native-nominal-fields-2026-10-02.json) and [nominal signature evidence](experiments/extended-cli-metadata/native-nominal-signatures-2026-10-02.json).
Native reading/import now precede further source-library/bootstrap expansion. CLI projections remain existing
controls, not the new integration route. See the [implementation alignment](design/extended-cli-metadata.md#direct-native-semantic-import-implementation-alignment-2026-10-02).

Development checkpoint (2026-10-02): the metadata feature branch integrates the
structural Function runtime and direct Raven Function emission. The unchanged collections
application now compiles to native PE/#Neo, verifies and executes with exact expected
output against the explicitly bound translated System library. All 95 metadata contract
groups pass. Unchanged ArrayList plus its source interfaces now compile to native PE
and execute growth, copy independence, iteration, callback searches and Option results,
with expected negative-capacity/index faults. Explicit namespace-function dependency
binding and configured array-shape length support complete this bounded source slice.
Translated System remains required; full library source emission and native semantic
symbol loading remain open. Unchanged callback comparer implementations now also execute
through Function fields and interface dispatch (42). HashMap and its source dependencies
now also compile and execute collision/growth,
update, key-snapshot, inherited-interface and Option lookup checks (42). Reference-payload consumers now also verify shared object identity and replacement
independence (42). The broad application with source collections exposes a mixed
source/seed iteration identity boundary, still present when query operators are included
as source. Next establish a coherent source bootstrap core/iteration contract before
extending query emission; closure environments and remaining System units remain open.
Author-directed integration stays on the feature branches;
no main-branch feature merge or roadmap milestone completion is implied.
See [bundle-hashed evidence](experiments/extended-cli-metadata/collections-end-to-end-2026-10-02.json)
and [integration contracts](raven-cli-bridge.md#unchanged-collections-end-to-end-acceptance-2026-10-02-development).

**Updated 2026-10-02.** This is the authoritative default for work priorities,
milestone sequencing and scope. Explicit author directions take precedence.

## Current work

**Imported constructor checkpoint (2026-10-02):** value constructors and imported
class/value constructors preserve CLI/native identity and construction safety. Raven
constructs imported generic values and executes against a separate native library (42).
All 84 metadata groups, 29 focused compiler tests and five native controls pass. The
unchanged collections sample still rejects Option<Order>(None) because None is a nested
CLI type. Next implement nested definitions/references and exact declaring-type identity
in the metadata/import contracts, then return to native System dependencies.

**Propagation guard checkpoint (2026-10-02):** the metadata API emits literal terminal
failure using the existing native fault instruction, with C# success/failure execution
on CLR and neoCLR. Raven explicitly marks generated invalid-carrier guards; its .NET
adapter retains null-throw behavior, while native emission terminates with a diagnostic.
All 83 metadata checks and 14 focused Raven tests pass. The unchanged collections sample
now passes the guard and reaches imported carrier construction from None; CLI control
still emits 7168 bytes. Next implement imported value/union carrier constructors and
continue into native System dependencies; no full application execution is claimed.
[Sample evidence](experiments/extended-cli-metadata/collections-after-terminal-guard-2026-10-02.json).

**Author-directed naming correction (2026-10-02):** the terminal namespace action is
`System.Fail(message)`; `Fault` remains the result. Update the matching compiler,
reference and library bundle together. This correction precedes, and does not replace,
the next propagation terminal-failure emission slice. See [the contract](system-fault.md).

**Raven value-receiver checkpoint (2026-10-02):** imported value methods now consume
managed local/ref receiver addresses through explicit shared emission capabilities.
A separate native-library Raven consumer verifies and executes mutation and generic out
calls (42). The unchanged collections sample advances to a lowered throw guard; CLI
control still emits 7168 bytes. Next define terminal failure handling for compiler-generated
invalid propagation carriers before following remaining union/System dependencies.
[Native control](experiments/extended-cli-metadata/raven-value-receiver-validation-2026-10-02.json),
[collections](experiments/extended-cli-metadata/collections-after-value-receivers-2026-10-02.json).

**Value-receiver metadata checkpoint (2026-10-02):** owned/authored and imported value
instance methods now preserve initialized managed receivers through CLI/native emission
and reference projection. Separate generic value/out consumers execute on both runtimes
(42); value constructors and constrained dispatch remain rejected. Next connect Raven's
imported value-receiver admission and address emission to this validated contract, then
rerun the unchanged collections sample.

**Raven ref/out checkpoint (2026-10-02):** shared .NET/neoCLR emission now preserves
ref/out signatures, local addresses, indirect access and uninitialized output locals.
Five native controls pass, including source forwarding/mutation (42); 64 focused C#
tests pass. The unchanged collections sample now rejects imported value-receiver
`TryGetOutput(out Order)` invocation admission, with CLI control still 7168 bytes.
Next implement exact imported value-receiver/member contracts, then follow the same
sample through propagation failure/Unit and native System identity requirements.
[Controls](experiments/extended-cli-metadata/raven-ref-out-validation-2026-10-02.json),
[collections](experiments/extended-cli-metadata/collections-after-raven-ref-out-2026-10-02.json).

**Sample reassessment (2026-10-02):** four small native execution controls pass; the
31-case inventory has four library emit/verify successes, eighteen emission rejections
and nine binding failures. All twelve selected applications emit their CLI controls but
none reaches native execution. Next connect ref/out locals and calls through Raven's
portable emission contract, then follow the unchanged collections sample into imported
value receivers and exact System dependencies. Whole-library compilation requires an
implementation seed; its 700 binding diagnostics are not 700 independent codegen gaps.
[Assessment and evidence](experiments/extended-cli-metadata/readiness-assessment-2026-10-02.md).

**Out-call checkpoint (2026-10-02):** explicit output contracts now map CLI Param Out
flags to existing native out_parameters metadata. Generic forwarding, imported calls
and native CLI projection preserve them; producer validation requires assignment on
every normal return. Metadata library/consumer binaries execute on CLR and neoCLR (42).
Raven admission and imported value receivers remain the next integration work; the
unchanged collections application has not advanced yet.
[Contract and validation](design/extended-cli-metadata.md#output-parameter-contracts-2026-10-02).

**Byref-call checkpoint (2026-10-02):** writable managed-reference parameters now
survive CLI/native emission, generic substitution, imports and CLI projection. An
initialized local can be passed and mutated through a separate assembly; metadata C#
checks execute on CLR and native verify/run returns 42. Out assignment contracts and
imported value receivers remain the next propagation gaps; Raven admission and the
unchanged collections sample are not yet advanced by this metadata-only slice.
[Contract and checks](design/extended-cli-metadata.md#writable-ref-parameters-2026-10-02).

**Managed-local checkpoint (2026-10-02):** metadata builders now emit typed ldobj/stobj
for owned local addresses, including generic copies, with definite-assignment checks.
The same C# producer executes on CLR and loads/verifies/runs as a native binary (42).
This is the first bounded part of propagation support; byref parameter signatures,
out-call contracts and imported value receivers remain open. The unchanged collections
sample is still blocked at native out-local admission. See
[the contract and validation](design/extended-cli-metadata.md#typed-local-object-operations-2026-10-02).

**Shared-lowering checkpoint (2026-10-02):** fix the concrete-case construction
exception masked by semantic-model fallback. The apparent propagation rejection is
now resolved at shared lowering; unchanged collections native emission reaches
synthesized uninitialized out-local admission. The CLI control still emits 7168 bytes.
The general Raven fix is isolated as `dcc77ef5f` on the main-based compiler-fixes branch,
with 25/25 focused tests passing on both compiler lines. Next build explicit managed
local-address/byref-signature/value-receiver support for propagation, retaining CLR/CIL
semantics and capability checks. Union/protocol execution and native System identity
mapping are not complete. [Evidence](experiments/extended-cli-metadata/collections-after-shared-union-lowering.json).

**Imported-member checkpoint (2026-10-02):** imported values retain their signature
category, and constructed interface/final virtual class calls now use explicit imported
member contracts. Separate metadata and Raven consumers execute on neoCLR (42); the
metadata case also executes on CLR. The unchanged collections sample now passes
TryAdd/Add admission and stops at `BoundPropagateExpression` (`?` in PendingOrder).
Next investigate propagation/union lowering, retaining the original sample and explicit
dependency contracts. This is not proof that the full collections application or native
System identity mapping is complete.
[Dispatch evidence](experiments/extended-cli-metadata/imported-interface-validation.json),
[unchanged sample](experiments/extended-cli-metadata/collections-after-interface-dispatch.json).

**Active author-directed priority (2026-10-01):** the author considers the metadata API
architecture good enough for now. Resume the unchanged Raven/native end-to-end case;
defer broad API migration, naming cleanup and encoding/PE extraction unless a concrete
integration blocker requires them. This supersedes the earlier instruction to prioritize
full definition unification. Preserve the recorded architecture as direction, not a gate.

Historical validation against metadata `e13c8634` and Raven `80edf8fbe` confirms the unchanged
collections sample binds and emits its CLI control (7168 bytes), but native emission
still rejects PendingOrder's Option<Order> signature. The immediate work is preserving
imported value categories through admission, signatures, native dependency loading and
projection, then advancing the original sample to its next observed blocker.
[Fresh evidence](experiments/extended-cli-metadata/collections-after-definition-migration.json).
Do not widen reference-type admission to make value unions pass. Remaining instance/
generic-owner imports, union operations and translated-System identities must be
validated incrementally; their completion is not claimed by the current probe.

**Layering (author clarification, 2026-10-01):** builders → definitions → metadata → PE,
with readers reversing the encoding/container boundaries. Current combined encoding/PE
packaging and immutable reader snapshots still need separation/materialization. This is
an architectural target, not a new binary format. Direct generic types now share names
and constraint storage with builders; constrained calls execute on both runtimes (42).

**Builder direction (author clarification, 2026-10-01):** retain Cecil-like definitions,
with generation builders inspired by Reflection.Emit, without replacement/drop-in claims.
Both layers share one graph. Property declarations/accessor associations now participate
in that graph; existing Add-style builder APIs remain available.

**Definition migration checkpoint (2026-10-01):** authored assembly/type/field construction
now shares declaration objects with builder facades and retains existing CLI/native
encoding. Manual struct execution returns 42 on both runtimes; Raven cross-assembly
probes remain green. Canonical method declarations and authored entry/function views now follow; direct
full instruction editing, remaining generic migration and reader materialization
remain open. Method definitions now own instruction/local/label storage, with existing
helpers using that same body and cached read-only local/label views. Interface relationships
now have append-only authored definitions; inherited dispatch executes on both runtimes.
Direct nongeneric interface and abstract-method declarations now dispatch on CLR/neoCLR
(42), while relationship registration still uses builder helpers.
Direct root-class constructors and instance methods now execute readonly initialization
and object calls on CLR and neoCLR (42).
Direct static type methods now attach through TypeDefinition.Methods and execute through
the manual assembly-function call chain.
Direct assembly-function construction and helper-call execution are now covered as well. The author reaffirmed CLI/CIL as the baseline to extend or modify, and continued
authorization for end-to-end work. [Contract](../api-docs/experimental-metadata.md#authored-definitions-first-migration-slice).

**Author-directed architecture correction (2026-10-01):** definitions must be the
canonical editable assembly graph, usable directly; builders must be optional facades
that mutate the same definitions, and writers must consume definitions. The current
immutable-snapshot/separate-builder graph does not meet this requirement. After finishing
the in-progress value payload/address slice (633a4a4a), prioritize this refactor before
more builder-only capability expansion. The first acceptance case is direct definition
construction of the author's struct/field example plus preserved builder compatibility.
[Plan and current gap](../api-docs/experimental-metadata.md#definition-first-model-author-direction-2026-10-01)
and [conversation](development-timeline.md#2026-10-01-definitions-are-the-editable-metadata-model).
The author subsequently endorsed Mono.Cecil alignment where it matters; follow the
[sourced alignment map](../api-docs/experimental-metadata.md#cecil-alignment-reference-reviewed-2026-10-01).
This changes the immediate implementation sequence, not the Raven/native end-to-end goal.

**Value-type category prerequisite (2026-10-01):** the metadata producer now emits
owned nongeneric/generic value definitions with primitive fields and preserves CLI
VALUETYPE categories through native binary projection. Default values, field reads,
generic forwarding and arrays verify/run on CLR and neoCLR (42). Raven admission and
the collections Option<Order> gate remain unchanged. Generic payload storage and addressed local mutation now pass (633a4a4a); explicit
imported value-type/category mapping and union emission remain open. The newer
definition-model direction above takes priority; do not represent value unions as classes.
[Evidence](experiments/extended-cli-metadata/value-type-validation.json).

**Nominal method imports (2026-10-01):** separately emitted Raven binaries now execute
static factory/reader calls with Box<consumer Order>, payload alias mutation and nominal
overload matching (42). Dependency-local reference signatures are imported through the
metadata API; value-type unions and imported instance/generic-owner methods remain open.
The unchanged collections sample still stops at Option<Order>; preserve its value-type
category rather than widening reference admission. [Probe](experiments/extended-cli-metadata/nominal-method-validation.json)
and [current gate](experiments/extended-cli-metadata/collections-after-nominal-methods.json).

**Imported generic arguments (2026-10-01):** Raven/native execution now covers
consumer-owned nominal arguments, imported constructions and caller method/owner
parameter forwarding through external unconstrained static generic methods (42).
This closes an instantiation gap; the unchanged collections gate remains the
Option<Order> signature and downstream member/native-identity requirements.
[Evidence](experiments/extended-cli-metadata/generic-library-validation.json).

**Raven external-signature checkpoint (2026-10-01):** the target adapter now consumes
registered external reference types and constructions. Separate Raven-produced binaries
verify/run (42), including nullable locals, interface arrays and generic forwarding.
The unchanged collections gate advances past Register admission to PendingOrder's
Option<Order> return signature. Imported value/union contracts, constructors/member calls
and translated-System native identity mapping remain open; direct sample execution is
not complete. [Probe](experiments/extended-cli-metadata/external-signature-validation.json)
and [current gate](experiments/extended-cli-metadata/collections-after-external-signatures.json).

**External signature prerequisite (2026-10-01):** the independent metadata API now
emits external reference classes/interfaces and generic constructions, preserving CLI
TypeRef/TypeSpec shape and native dependency scope. An API-produced Box<consumer Order>
case loads/verifies/executes as binary assemblies (42), including generic forwarding.
This does not yet advance the unchanged collections sample: next connect these types
to imported member signatures and translated-System native identity mapping in Raven.
[Evidence](experiments/extended-cli-metadata/imported-type-validation.json).

**Primary acceptance reaffirmed (2026-10-01):** use unchanged existing samples to
prove Raven → neoCLR assembly → runtime loading/execution. API work and focused probes
support this gate. After imported generic-method support, order-collections still binds
and emits the CLI control but direct emission writes no image: Register's imported
MutableMap<int, Order>/ArrayList<Order> signatures remain unsupported. Next address
external nominal/generic type and member identities against the translated System
library, then advance through this sample's constructor/callback/union requirements.
[Fresh evidence](experiments/extended-cli-metadata/collections-after-generic-imports.json).

**Author reaffirmation (2026-10-01):** .NET metadata is the baseline, with explicit
neoCLR extensions such as assembly-level functions. The Cecil-like API is the primary
abstraction for inspecting, modifying and creating assemblies. Preserve familiar
metadata shape/behavior; current immutable snapshots, editable producer graphs and
bounded imports are incremental coverage, not the final loaded read–edit–write model.
[Conversation](development-timeline.md#2026-10-01-net-metadata-baseline-and-a-complete-cecil-like-lifecycle).

**Imported generic-method checkpoint (2026-10-01):** separate Raven library and
application binaries now execute unconstrained static generic methods with concrete
primitive/vector arguments (42), including arity overloads and alias mutation. This
completes a bounded method-instantiation prerequisite; imported nominal identities,
generic declaring types and constrained dependencies remain open before collection
imports. [Evidence](experiments/extended-cli-metadata/generic-library-validation.json).

**Imported vector checkpoint (2026-10-01):** direct Raven emission now supports
static primitive-vector calls across separately emitted library/application assemblies.
The neoCLR-profile binary pair verifies and returns 42, including overload selection
and shared array mutation. This establishes a bounded external signature path;
nominal/generic collection imports remain the next gap, and native symbol loading
remains deferred. [Evidence](experiments/extended-cli-metadata/vector-library-validation.json).

**Collections binding checkpoint (2026-10-01):** the imported carrier-constructor
bug is fixed in Raven binding and integrated into local main (`46491585e`). The
unchanged order-collections sample now imports, verifies and runs with exact output;
assembled binary App and System also verify/run. This is legacy-bridge/assembler
control evidence, not direct Raven metadata emission or a fresh System source build.
Direct emission now reaches the imported generic Register signature boundary instead
of the old profile gate. Keep imported collection contracts/member references as the
next backend acceptance step. [Evidence](experiments/extended-cli-metadata/order-collections-binding-validation.json).

**Native profile checkpoint (2026-10-01):** after independently proven general Raven
fixes reached local main and a bounded .NET refactor-parity audit, direct emission now
accepts the real neoCLR semantic profile with a validated CLI declaration core.
Hello World via another function, Unit entry, array iteration and owned interface
dispatch verify and run as native binaries without host core references. Core-name
and version mismatches reject before output. This completes the bounded profile gate,
not the implementation seed or native metadata importer. Next prepare implementation
bootstrap dependencies and generic collection/imported-member contracts; keep the
known imported-carrier and loop-capture issues explicit when expanding samples.
[Evidence and scope](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md#native-profile-gate-and-refactor-parity-follow-up).

**Post-dispatch assessment (2026-10-01):** the author requested a broader library/app
readiness check. Owned nongeneric interface dispatch now executes through the direct
binary pipeline. All twelve selected existing samples bind and emit ordinary CLI with
the real target snapshot, but direct emission still rejects the neoCLR target profile.
Three unchanged applications run through the current legacy bridge; order-collections
exposes invalid Option constructor selection in emitted CLI before import. The next
proposed integration milestone is the native target profile and implementation seed,
then generic collection contracts/imported members and broad union/callback emission.
The whole-library attempt with an application-only snapshot is not a supported bootstrap.
See the [assessment, evidence and acceptance gates](experiments/extended-cli-metadata/readiness-assessment-2026-10-01.md).

**Interface declaration checkpoint (2026-10-01):** unchanged Comparer<T> and
EqualityComparer<T> now emit ordinary CLI/native interface contracts and load/verify
in both file orders. This does not yet prove interface dispatch. Disposable and Iterator<T> now join that evidence with abstract properties and
nongeneric inherited contracts. Iterable<T> and interface-valued reference/default
flow now execute too. Owned nongeneric implementations and interface method/property dispatch now execute
through two classes and reference arrays in both orders (42). The author requests
reassessment against existing library/application samples next; generic dispatch and
inheritance remain deferred. [Evidence](experiments/extended-cli-metadata/interface-library-runtime-validation.json).

**Whole-source checkpoint (2026-10-01):** complete, unchanged
System.Globalization.Language now executes on .NET and neoCLR in both file orders
(und/sv/he; result 42), using shared static-property accessor calls. No new opcode or
runtime format is needed. Comparer<T>/EqualityComparer<T> bind but await interface
emission; ArrayList needs native library dependencies. [Current evidence](experiments/extended-cli-metadata/whole-library-runtime-validation.json).
The author reaffirmed .NET behavior/instruction semantics as the baseline; unsupported
features are coverage limits unless a divergence is explicitly chosen.

**Constraint checkpoint (2026-10-01):** owned nominal bounds and class/struct/new
requirements now pass Raven → independent metadata API → native binary loading and
execution alongside .NET. This is a bounded integration, not complete class-library
emission. Per the author's request, pause expansion to assess the state and select
subsequent work from concrete consumer gaps. [Assessment and evidence](experiments/extended-cli-metadata/state-assessment-2026-10-01.md).

**Latest author-directed acceptance (2026-10-01):** compile real Raven class-library
parts before broader metadata loading. The original integer Math Min/Max/Sign
functions now emit with native namespace identity and execute 11 boundary cases on
.NET and binary neoCLR in both source orders. This is selected-source coverage under
the host-core bootstrap, not a full System build. Whole Math/UnicodeScalar/GC still
require native dependencies. Next use the [order-collections application](experiments/raven-target/samples/application-order-collections.rvn)
to drive constructor/property, object, generic and delegate contracts incrementally.
The unchanged Order declaration now emits with canonical properties/backing fields,
accessors and its explicit constructor. A separate Main constructs it and checks
Boolean/Int32 boundaries; both source orders verify/run on .NET and binary neoCLR,
returning 42. The independent API supplies fields, constructors and property associations;
Raven reuses compiler-synthesized accessor bodies through the shared instruction plan.
Both the metadata API and Raven now support owned nominal locals; mutation through
an alias is observed through the original object on both runtimes. Ordinary instance
calls also execute, including private helpers and ordered argument side effects.
Private mutable primitive storage now emits fields without property/accessor rows,
with qualified reads and mutation validated on both runtimes. Computed getters and
implemented block/arrow accessors now execute too, including private setters and
`field` backing storage with preserved property metadata. Explicit root constructors
now accept expression bodies with overload/argument-order execution coverage. Default
constructors and primitive field/property initializers now share the compiler initialization
plan and execute on both targets. Owned nominal parameters/results now flow through
shared backend type mapping; factories, self-return, constructor arguments, overloads
and alias mutation verify/run on .NET and binary neoCLR in both file orders. The current
nominal-signature/default-constructor/primitive-initialization slices are complete. Explicit
mutable primitive fields also preserve visibility, initialization and alias mutation
through binary execution. Mutable owned nominal fields and private storage now preserve
class identity, initializers and alias mutation in both source orders on both runtimes.
Owned nominal properties now preserve auto/computed/explicit accessor associations,
private setters and backing storage through the same pipeline. A shared binder fix
also refreshes provisional storage initializers without replacing field identity, now
including private stored properties. Explicit root `base()` uses the existing initialization
contract after semantic validation; side-effecting initializers and both body forms execute
on both runtimes. Readonly instance storage now preserves CLI/native flags and rejects
ordinary direct/managed-address writes in verification and execution; Raven consumes it
for private `val` storage and stored `val` properties. The connected vector milestone
is now implemented: metadata declarations now preserve CLI SZARRAY/native ArrayRef for
primitive and owned-class arrays. API-produced binaries now allocate, index, mutate
and measure arrays on neoCLR, including aliasing and bounds faults. Raven now emits
array literals, storage, calls, indexing and Length on both targets; Order-array alias
mutation and nested/labeled iteration return 42 in both source orders. This bounded
array emission milestone is complete: [evidence](experiments/extended-cli-metadata/array-runtime-validation.json).
The next connected milestone is indexed properties: metadata producer/projection
support now preserves index parameters and overloaded accessor associations.
API-produced binaries verify and execute indexed accessors on neoCLR (42); shared
Raven now emits indexed accessor calls and associations; overloads, multiple indices
and read-only getters execute on both targets in both source orders (42). A concrete
Order collection now also proves nominal/array indices, object aliases, evaluation
order and indexed bounds faults. The author directs generics as the next connected
milestone. Generic function/static-method declarations now preserve named parameters,
CLI GenericParam/MVAR and native method parameter identities; generic locals and arrays
round-trip. Instantiated calls and Raven consumption follow. Broader field/constructor shapes and
generic/delegate/union contracts remain subsequent work. The full consumer still has 49 binding errors
from native collection/LINQ/union dependencies under the host-only bootstrap.
[Evidence](raven-cli-bridge.md#namespaced-functions-and-real-math-source--2026-10-01).

**Compatibility baseline reaffirmed (2026-10-01):** preserve ordinary .NET/CLI metadata
and instruction encodings, with explicit neoCLR extensions. The current native #Neo
execution payload plus CLI reference projection is a working bridge, not yet full
extended-CLI executable compatibility. Keep that gap visible while adding ordinary
compiler coverage; do not treat this bridge as a permanent replacement format.
[Design boundary](design/extended-cli-metadata.md#compatibility-baseline-reaffirmed--2026-10-01).

**Author-directed metadata exploration (2026-09-30).** Begin extended CLI metadata
on `codex/extended-cli-metadata`, based on main, with later Raven integration. The
author confirms the Cecil-style metadata API remains an independent project, consumed
by Raven's neoCLR target through compiler-owned loader/emitter adapters; integration
does not move the library into the compiler. [Project ownership](design/extended-cli-metadata.md#independent-metadata-project-raven-target-consumer-2026-09-30).
**Latest sequencing clarification:** establish a working metadata format and its APIs,
then integrate the refactored compiler, then add improvements such as structural types.
The direct native producer/load test below is the bounded acceptance baseline;
ordinary compiler-required metadata coverage is next.

**Architecture follow-up (2026-10-01):** after reviewing Reflection dependencies, the
author directed a staged codegen refactor. The first slice now feeds both bounded
backends compiler-lowered bodies, including implicit value returns. The first callable reference table now shares symbol identity resolution with typed
backend handles. Shared source callable plans separate native source validation from
builder creation. Shared public/internal nongeneric static-type plans now drive backend type
builders, including partial declarations coalesced by symbol identity with every part
validated. Primitive signatures/locals now share logical value/no-result types and
backend-owned mappers; broader type/field references and full declaration traversal remain next,
and the first paired body capability now supports initialized Int32 locals/assignments.
Comparisons (including negated forms), if/else and lowered loops with break/continue now execute on both runtimes;
Int32/Boolean/no-result signatures now flow through both backends, native imports and
reference projections, with Boolean local initialization/assignment, equality and short-circuit &&/|| and discarded primitive call results. Int64 signatures/locals and signed Int32↔Int64
conversions and signed unary +/−/~ now share that path; broader conversions and general type/field references remain next;
String signatures, locals, literals and imported text helpers now share the path,
with computed console output on both runtimes. The independent metadata API also supports typed argument stores with caller
isolation; ordinary Raven source parameters remain immutable. Backend-owned instruction/built-in-type capability profiles now admit shared bodies
before native builder allocation, with restricted-profile tests validating
selective admission. Signed Int32/Int64 division, remainder, bitwise AND/OR/XOR and signed shifts now execute on both backends. Logical assembly-function/static-method/static-type categories now share those
profiles. Public/internal static type visibility is also preserved across both targets.
The independent metadata API now preserves public/internal/private static method
visibility, with direct binary runtime enforcement. Raven now emits these methods
through shared visibility admission and preserves access in compiler references.
Block and expression bodies now share compiler lowering and execute on both targets.
Eager Boolean AND/OR/XOR now share that path with exact Boolean metadata/runtime
operands; mixed Boolean/integer inputs remain invalid.
Primitive value-producing if/else now shares typed branch joins on both backends;
value blocks also permit initialized locals, assignments, calls and internal if/loop
control flow before the result, with returns/outgoing jumps rejected during planning.
The metadata API now supports public/internal ownerless functions with native access
enforcement; Raven now preserves explicit public/internal and default internal access.
General object/field and broader metadata-category contracts and
metadata loading remain future slices. See [integration scope and validation](raven-cli-bridge.md#compiler-lowered-native-bodies--2026-10-01).

**Shared compiler pipeline:** Raven's explicit native backend now participates in
`Compilation.Emit`; rvnc and API wrappers share compiler setup and target validation.
The author directs reuse of common .NET/neoCLR lowering with backend abstractions for
builder differences. The shared linear-body model now feeds .NET/native method-builder adapters and
executes the same Int32 and Unit Hello/helper compilations on both runtimes, including
assembly functions, explicit/implicit returns and an empty Unit entry. Supported callable
signatures and declaration-builder contracts are now shared too; concrete adapters preserve
CLI type-method and native assembly-function ownership. Type/signature builders, generics and
general member bodies remain subsequent boundaries. The author asks to avoid large workarounds
and prioritize the Hello/helper end-to-end case. Shared metadata loading is deferred;
the optional System driver has exposed a host/projection type collision and explicitly
rejects that call. Its direct API case still passes; driver import is not yet reliable. [Integration boundary](raven-cli-bridge.md#shared-emission-pipeline--2026-09-30).

**Latest acceptance:** Raven's opt-in emitter now writes binary PE/#Neo assemblies;
neoCLR decodes their native metadata directly. Hello World, an entry-point call to
Greet, both source-file orders and the two-library chain all verify/run successfully.
Schema-1 JSON containers remain readable. The separate Cecil-style API still supplies
CLI reference projections to Raven's existing .NET semantic provider. The author
identifies class-library compilation and Raven symbol loading as the next consumer,
with existing JSON-to-assembly translation as a bootstrap. The complete current System
JSON now translates to a standalone binary native envelope, verifies all 641 IL functions
and runs both Hello cases and a generic dependency chain. This has no CLI reference
projection. Next: compile the runtime class library from Raven source and compare
against these translated artifacts; broad symbol import, a native symbol provider
and production registration remain unimplemented.
[Binary profile](design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30)
and [measured phases](experiments/extended-cli-metadata/binary-loading.md).

**Author-directed consumer exploration:** Exercise translation of the existing Raven
CLI bridge's test/sample output before expanding direct compiler emission. A proper
neoil assembler producing native assemblies is a future producer. The Raven collection
library profile exceeds the current binary transport bounds; retain that explicit gap
while isolating application translations with matching JSON System. Floating-point
operand tests additionally expose the need for full UInt64 bit-pattern encoding.
**Follow-up implemented:** standalone schema 3 adds that encoding and bounded larger
library capacity; FloatingMath, OptionPositional and ValueCopy now run against binary
Raven System. The neoil binary producer and direct native compiler/symbol import remain
next work. [Profile and validation](design/extended-cli-metadata.md#library-execution-profile-3--2026-09-30).
[Experiment and reproducible checks](experiments/extended-cli-metadata/raven-sample-translation.md).

**Direct assembler checkpoint:** `assemble --format neox` now emits schema-3 native
assemblies directly from neoil, with verification before output and JSON kept as the
default. The full Raven collection System and three sample applications pass independent
.NET metadata comparisons and native execution. Direct Raven class-library source emission
and native compiler symbol import remain next. [Producer evidence](experiments/extended-cli-metadata/direct-assembly.md).

**Author-requested performance evidence:** A release comparison finds the full Raven
System assembly 63% smaller and about 20% lower decode time than current pretty JSON,
but no meaningful complete-startup/run improvement (about 3.45 seconds). Linking and
admission dominate; direct-typed JSON is a faster diagnostic alternative to both current
decoders. The author explicitly accepts smaller files as an improvement and defers
runtime optimization to the future. Retain this baseline and continue metadata/compiler
integration; the benchmark does not reprioritize runtime optimization.
[Method and limits](experiments/extended-cli-metadata/json-vs-assembly-benchmark.md).

**Raven continuation:** The author selects continued Raven work after the size benchmark.
The native compiler adapter now emits Unit-returning helpers and static library methods,
reimports their CLI no-result projections and runs a separate application against the
binary library. This removes the artificial Int32 return from side-effecting helpers;
entry points remain Int32. Namespaced static classes now preserve identity through
compiler reimport and native calls, including same-name types in different namespaces
and both source orders. Wider signatures, native symbols and actual runtime class-library
source compilation remain next. [Compiler contract](raven-cli-bridge.md#unit-returning-native-helpers-and-library-methods--2026-09-30).

**Native compiler command:** An opt-in `rvnc neoclr` command now compiles source files
and native writer-produced PE references directly to PE/#Neo, including assembly-owned
functions. The author confirms symbol loading from translated System/System.Runtime is
also required. Prioritize that loader acceptance alongside native emission; host primitive
binding is only a bootstrap, not evidence of translated System symbol loading. Broader
native declaration reading/projection or a native symbol provider is still required.
[Command and loader boundary](raven-cli-bridge.md#opt-in-native-compiler-command--2026-09-30).

**First translated System symbol slice:** An explicit static Int32 callable view now
adapts selected native declarations to Raven's existing semantic importer. Math.Min binds
to that view and executes against the translated System through both API and rvnc paths.
This is not full core import: primitive binding remains hosted, and generic/instance/field/
property contracts remain next. The author supports adapting the existing importer first,
then a separate provider as needed, and removing Reflection.Emit from .NET in the future.
A common instruction/operand model and later ILProcessor-like editing are recorded future
metadata API directions. [Scope and architecture](raven-cli-bridge.md#translated-system-callable-import--2026-09-30).

**Native entry contract:** Raven now emits parameterless Unit Main as a no-result
native entry, including global/static ownership and helper calls, and neoCLR exits zero.
The metadata reader/writer reuse format 5's existing no-result encoding. This removes
an artificial compiler restriction while keeping metadata support and target integration
as the main objective. Full System import and richer callable/body coverage remain next.
[Entry contract and evidence](raven-cli-bridge.md#unit-entry-points--2026-09-30).

**Compiler-facing opcode surface:** The independent metadata API now exposes bounded
Emit overloads for typed opcode/operand construction, and Raven's native emitter uses
them. Convenience helpers share the same path. This advances target code generation;
it does not claim full opcode coverage or ILProcessor-style editing. Broader metadata,
System symbol import and native source coverage remain the objective.
[Contract](raven-cli-bridge.md#opcode-based-metadata-emission--2026-09-30).

**Earlier metadata checkpoints (historical progression):** The PE reader now recovers
owned global/type callable declarations and the writer's static Int32 signature subset;
21 C# contract groups cover ownership, unsupported signatures, decoding limits and
explicit MemberRef resolution for the emitted nominal static method subset. General
signatures, broader member contracts and body import remain readiness gaps. The author
now selects **Refactor Raven for platform targets** (activity
`01a0f154-2448-7df3-8536-c837097b46c2`) as the integration consumer: attempt a bounded
compiler import/emission/runtime case once this subset suffices, then drive remaining
metadata support from that case rather than waiting for exhaustive coverage. Stage 1
now passes: Raven binds a small source program and an API-produced PE dependency;
an opt-in adapter consumes public operations and emits native bytes through the separate
library, which neoCLR verifies and executes to 42 ([evidence](experiments/extended-cli-metadata/raven-compiler-validation.json)).
The frontend still uses the .NET metadata provider; native provider/production emitter
composition are next. The case exposed shared operation and parameter-import fixes,
validated by 87 focused Raven tests and integrated into local Raven main. See the
[inspected compiler boundaries and next case](design/extended-cli-metadata.md#refactored-compiler-as-the-next-end-to-end-consumer-2026-09-30). Structural types remain a future
design requirement (arrays, tuples, Function types, unions/intersections and synthesized
members), not a prerequisite for that first compiler integration. The
[design and staged acceptance plan](design/extended-cli-metadata.md) is exploratory;
an isolated codec/inspector now validates experimental framing and structural
signatures, host-catalog nominal resolution and synthesized-member references with
30 focused tests. Differently numbered fixture references resolve to equal structural
type/member keys. A bounded PE32 #Neo probe preserves conventional metadata for
.NET/Cecil inspection; Cecil rewriting strips the extension. An experimental marker/digest and explicit expected-input profile now reject
stripped/changed metadata in 11 recognition cases. The first .NET reader/writer library now covers envelope framing, with four shared
fixtures and 49 cross-reader rejection cases. Its structural signature codec now passes
14 cross-reader vectors and 103 rejection cases. .NET reference tables and explicit-catalog
structural identity pass 95 shared vectors (69 rejections). .NET synthesized-member
contracts now pass 49 shared vectors (30 rejections). A typed .NET reference-profile
Read/Create/Write facade passes 36 shared vectors (29 rejections). Bounded .NET PE32
artifact recognition/extraction passes 50 shared cases (44 rejections). Following the
author’s Cecil suggestion, the [provisional API direction](design/extended-cli-metadata.md#cecil-inspired-object-model-direction-2026-09-30)
places assembly/module/reference/definition objects above these codecs. An initial
read-only model now reads real assembly/module/TypeDef declarations, with generic/nested
Unicode and snapshot-scoped reference consumer checks. AssemblyRef identity and explicit
host resolution now pass 11 standalone C# contract tests. The author selects the
Cecil-like model as the primary compiler abstraction for metadata/PE manipulation,
with adaptations as needed. Physical TypeRef resolution and a controlled static-Int32
PE writer now support an end-to-end producer baseline. API-produced application and
library PEs execute in neoCLR through the existing CLI bridge and matching native
library, returning 42. Broader IL/signature coverage, general rewriting, Raven codegen
integration and direct native #Neo loading remain pending. The author also requires
functions outside types: the model now exposes assembly-owned functions and emits
native format-5 assemblies directly, with a cross-assembly function test returning 42
in neoCLR without the CLI bridge ([evidence](experiments/extended-cli-metadata/native-validation.json)). Production NEOX loading and structural runtime support remain unimplemented. This scopes the requested exploration
without promoting all proposals or merging the structural runtime experiment.
The author additionally requires eventual reader/writer support on both .NET and
neoCLR. The author further clarifies that the .NET API should support later Raven compiler
integration, with a potential Raven port beneath the pending Metadata Introspection API;
[readiness criteria and open choices](design/extended-cli-metadata.md#net-api-direction-and-potential-raven-port-2026-09-30)
keep this distinct from completed integration. The [cross-platform library plan](design/extended-cli-metadata.md#reader-and-writer-support-on-net-and-neoclr)
separates shared format/conformance contracts, .NET tooling, native support and an
actual neoCLR guest library. Concrete consumers are Raven’s symbol loader/code
generation and neoCLR assembly loading into Introspection/assembly emission; these
full library consumers remain planned; the .NET framing foundation is implemented.

**Immediate focus — Raven neoCLR target support (author-selected 2026-09-30).**
After the bounded main backport below, put the structural Function experiment on
hold and work in Raven to improve neoCLR target support. The Function runtime
branch remains isolated; its proposals are open, not completed on main. Specific
acceptance now includes integrating target-gated Self in Raven and neoCLR while
keeping structural types on feature branches in both repositories. This author
direction supersedes the earlier restriction on Raven main integration; a native
metadata layer and replacement backend remain future work. General compiler fixes
still require independent validation.
The earlier Web API direction below remains recorded for later resumption.

**Author-directed library backport (2026-09-30).** Bring Raven callback function
syntax and lazy OfType filtering to main independently of the structural Function
runtime branch. This bounded library change keeps the existing callable and
introspection models. It is complete; the subsequent author direction above selects
the next focus. See the
[query contract and evidence](raven-query-api.md#runtime-type-filtering--2026-09-30-backport).

**Author-directed release — Preview 11 (2026-09-27).**
The current bounded surface is qualified for release with macOS arm64 Raven tools
and a Windows x64 native-runtime ZIP. See [release notes](preview-11-release-notes.md)
and [qualification evidence](preview-11-validation.json). Generic async, shared Task.Run,
text/number foundations, nested JSON and routing samples are included. Windows
Raven SDK/bridge qualification remains separate. The full Linux suite passes; a
validator-only exit-code correction has independent archive-smoke evidence.
This closes release preparation without adding optional Web API capabilities.

**Preceding direction — minimal Web API (author-selected 2026-09-27).** Focus on
serving a useful Web API, nested JSON serialization/deserialization, and a
route parser used within an existing HttpServer handler, including named and typed
parameters. This later author direction defers the earlier separate WebApplication
project and Minimal API infrastructure.
A rudimentary SQL interface with a SQLite provider is an optional follow-on based
on the existing proposal, not a prerequisite or approval of its complete surface.
The [HTTP tracker](http-capabilities.md#active-direction--minimal-web-api) owns
application acceptance; the [bounded plan](web-api-plan.md) proposes the sequence
and records design choices still to validate. The first nested typed JSON slice is
implemented in development with [consumer and HTTP evidence](experiments/json-object-mapping/nested-validation.json).
JSON now matches the 1,024-byte HTTP body budget, with [boundary and case evidence](experiments/json-object-mapping/payload-validation.json).
Typed arrays now support collection payloads; see [collection evidence](experiments/json-object-mapping/collection-validation.json).
The author's route/union refinement selects the next bounded routing slice: direct
RoutePattern/RouteMatch parsing and optional application union dispatch are now
implemented in development; see [the case](experiments/http-routing/README.md).
A [generated attribute-driven mapper experiment](experiments/route-union-mapper/README.md)
validates schemas and reuses compiled patterns. The author subsequently selects
runtime attribute reflection with cached startup mapping. Member/parameter
[attribute data](attribute-introspection.md) is now implemented in development;
the [runtime mapper case](experiments/runtime-route-mapper/README.md) now validates route schemas at startup and retains patterns, conversion bindings and case/carrier constructors for request handling. Source generation remains a future alternative. The mapper ships as tested Preview 11
sample source; it is not an installed SDK mapper API.
Enum, Uuid and Option JSON mapping remain requested and pending.
This explicitly supersedes the previous general useful-library priority; M2–M6
remain candidates. Preview 10's completed POC stays closed.

### Completed POC and preceding library direction

The bounded HTTP POC is [complete on macOS arm64](http-capabilities.md#finish-this-poc),
including independent peers, known-length uploads and the extracted-package check.
Feature scope is frozen; do not automatically start another HTTP feature.

**The POC is released and done for now:**
[Preview 10](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.10),
published 2026-09-27. Option, Result
and TaskOutcome use standard Raven unions, and the native reflection boundary uses
the matching Option layout. See [release evidence](preview-10-validation.json).

The [toolchain/release tracker](tracking/toolchain-release.md) owns residual compiler
observations and delivery maintenance. Hosted CI passed all six split jobs at
`1404454e` (slowest 8.88 minutes); later migration corrections passed the focused
checks requested by the author and final extracted-package checks. The released POC feature
scope remains frozen; the new Web API direction is a separate increment. Thematic consolidation is complete.

**Preceding direction: useful library APIs**, selected by the author after release.
The author-selected comparer slice is implemented in development: equality/hash and
ordering policies, callback adapters, StringComparer.Ordinal and a HashMap policy
constructor. See the [library/data sequence and evidence](tracking/library-data.md#active-direction--useful-api-gaps).
The author-selected explicit String comparison modes and matching ignore-case policy
are implemented in development. The [String/System.Text foundation review](design/text-abstraction.md#systemtext-foundation-review--2026-09-27)
is complete: keep the UTF-8/grapheme model, but establish scalar, position, ownership
and conversion-progress contracts before dependent API expansion. The author clarifies
that the immediate System.Text scope is encoding/decoding foundations and possibly
a small builder, with Swift as the closer model for text API shape.
The [library tracker](tracking/library-data.md#string-design-review-before-further-expansion)
owns the boundary experiment, [paired text API sketch](design/text-abstraction.md#consumer-api-sketch-identical-behavior-two-vocabularies)
and the shared Encoding/Decoder integration in both stream adapters. Development
constructors accept encoding selection with UTF-8 defaults and strict ASCII, backed
by focused consumer checks. The [bounded report construction evaluation](design/text-abstraction.md#bounded-report-construction-evaluation--2026-09-27)
passes its contracts but does not justify promoting the managed builder: ordinary
concatenation is faster for the tested report sizes. Keep public builder promotion
deferred; do not start a builder optimization project. The [Encoder acceptance/drain evaluation](design/text-abstraction.md#encoder-progress-and-writer-evaluation--2026-09-27)
led to the [public Encoder integration](design/text-abstraction.md#public-encoder-integration-development--2026-09-27):
independent factories, bounded progress and explicit StreamWriter.Finish are implemented
in development with matching API artifacts and focused consumers. The bounded
UTF-8/strict-ASCII encoding foundation is complete. Broader codecs, general writer
completion and further text capabilities need separately scoped consumer work. General
scalar/range APIs and the broader portfolio are not prerequisites. Experimental
types and proposed names are not adopted System APIs.
The subsequent author-selected [casing and Int64 report slices](design/text-casing-integer.md)
are implemented in development: Unicode 17 full default String casing, typed strict
Int64 parsing, decimal formatting and bounds properties. Focused consumer/native/
metadata checks and matching API artifacts cover this bounded work. No new broader
milestone is selected by completing these slices.
The author next selected [Number and concrete primitive parsing](design/numeric-contracts.md).
The bounded development slice is implemented with focused consumer/native/metadata
evidence; all numeric parsers use NumberParseError. A parsing interface is on hold. The author also
identified [general interface capabilities](tracking/runtime-language.md#interfaces-as-a-platform-capability)
as important direction: static members, default bodies and member access control.
Number is a first consumer, not a permanent numeric-only interface model.
Native metadata integration now separately compiles and executes the unchanged
Single/Double Number implementations through explicit scalar ownership; see the
[checkpoint](experiments/extended-cli-metadata/system-compilation-strategy.md#source-floating-primitive-ownership-2026-10-04).
The subsequent cumulative source-library rebuild executes all ten numeric types with
seed ownership removed. The metadata generator now executes concrete static constrained interface calls on
both targets. Owned nongeneric method interface bounds now round-trip and import into
Raven symbols; external nongeneric method bounds now also retain scoped identity and
execute in a separate-contract metadata fixture. Open method-parameter constrained calls
now execute for owned interface targets, including native Self substitution. External
static and constructed instance call targets now execute through Raven. The ordinary
driver rebuilds all ten numeric source types, compiles a separate generic algorithms
library, and runs a source-free consumer of every Number member and generic forwarding
with exit 42. See the completed generic Number gate in the system compilation strategy.
The next bounded interface slice adds application defaults and public/private static
helpers through Raven, with [focused evidence](experiments/interface-helpers/README.md).
The [interface limitations table](tracking/runtime-language.md#interface-limitations--development-checkpoint-2026-09-27)
separates native support from Raven import gaps. A subsequent bounded
[explicit class implementation slice](experiments/explicit-interface-implementations/README.md)
adds ordinary methods on application classes/interfaces. Accessors, generic/value-type
import, private instance helpers and broader static/default/accessibility cases remain open.
The author also selected [Result/Task Main integration](experiments/entry-results/README.md),
with direct async Main adoption in relevant samples after focused validation.
This is a bounded startup integration, not a new broader milestone.
The author next selected [Task.Run with shared captured objects](tracking/runtime-language.md#author-selected-taskrun-work--2026-09-27)
as the canonical work-submission API, with runtime-selected execution. Development
now has native shared captures, typed/completion-only overloads and task unwrapping.
The [original Task.Run compiler integration failures](experiments/task-run/compiler-gaps/README.md)
are corrected: short-name lookup and block-lambda returns use independently tested
Raven fixes integrated into both branches. Direct completion-only await
now passes with the target compiler unit-result fix. Mutable-local
sharing in ordinary async methods is corrected by the independently tested Raven
closure fix. Generic-method capture metadata is repaired in Raven.
[Generic async application import](experiments/task-run/README.md#generic-async-application-import)
now handles the constructed state machine, shared closure and generic holder, with
forced suspension, identity and cancellation checks. Closed ordinary static helpers
and their generated generic types follow normal Raven metadata.
Async methods inside generic classes have a separately reproduced Raven arity bug.
Thread's future public role is open.
M2–M6 remain candidate applications; no complete File Catalog
or additional HTTP feature is required by the comparer slice.

The author next selected [constructor discovery and Reflection execution](reflection-members.md):
argument-based activation, invocation and field access, with Result failures and a
separate website feature page. This is bounded library/runtime work, not selection of
a new application milestone.

The author selected a bounded `System.Runtime.GC` API after Reflection: execution-local
object counters, explicit full collection and lifetime use. See the
[GC contract](runtime-gc.md); generation/byte accounting and collector tuning remain
outside this one-off author-directed slice.

The author selected [value tuple support](tuples.md) on 2026-09-28, using
`System.Tuple` as the neoCLR name corresponding to .NET `System.ValueTuple`.
This is a bounded author-directed addition; it does not replace the Web API direction.
The [runtime tracker](tracking/runtime-language.md#value-tuples--2026-09-28) owns status.

## Theme trackers

Each work item has one status owner. Use that tracker for its status, remaining
scope, next evidence and linked design. Cross-theme dependencies link to their
owner; they do not create a second checklist.

| Theme | Status owner | Scope |
| --- | --- | --- |
| HTTP and networking | [Client/server capability tracker](http-capabilities.md) | HTTP POC finish line, client/server matrix, transport limits and future protocol work |
| Runtime and language | [Runtime/language tracker](tracking/runtime-language.md) | Values, references, GC, async execution and deferred type-system/runtime experiments |
| Library and data | [Library/data tracker](tracking/library-data.md) | Text, collections, files/streams, time, introspection, JSON and general resource contracts |
| Toolchain and delivery | [Toolchain/release tracker](tracking/toolchain-release.md) | Raven integration, diagnostic debt, packaging, CI, API documentation and publication gates |

These are maintenance ownership boundaries, not an instruction to develop four
features in parallel. HTTP owns the application package acceptance decision;
tooling owns packaging procedure and general release checks.

## Milestones

The author now selects the Web API increment after M1. M2–M6 remain candidate
products; their earlier ordering is not approved API scope.

| Milestone | Sample product | Status / scope owner |
| --- | --- | --- |
| M1 — Communicate | Hello Service + Hello Client, now the typed HTTP/JSON exchange | Released in Preview 10; done for now |
| Web API increment | Nested/collection JSON + typed route-parser case | Active direction; [HTTP tracker](http-capabilities.md#active-direction--minimal-web-api); JSON and direct routing slices implemented; generated union mapper experiment validated |
| M2 — Work with data | File Catalog | Candidate; library/data tracker |
| M3 — Handle waiting and failure | Download Queue | Candidate; HTTP, library and runtime contracts must be selected together |
| M4 — Human time and presentation | Activity Report | Candidate; library/data tracker |
| M5 — Explain programs | Assembly Explorer | Candidate; library/data tracker, with tooling integration |
| M6 — Across hosts | Portable Sample Pack | Candidate; toolchain/release tracker |

The detailed earlier product sketches and comparisons are preserved in the
[dated roadmap](history/planning-20260927/platform-roadmap.md#milestones-at-a-glance).
An implemented supporting API does not by itself complete a milestone.

## Working rules

- Select a bounded task from the active milestone and keep its tracker and evidence
  current. A one-off author request does not silently reprioritize all future work.
- Prioritize features over optional optimization. Investigate runtime cost when it
  blocks a selected feature, materially impairs supported use, or fails a release
  criterion. Preserve defects and measurements without making indefinite profiling
  a prerequisite for unrelated work.
- At most one small companion task may accompany a larger feature. It must finish
  independently without introducing a new language, ABI, culture or lifetime contract.
- Run only validation needed for the change, including performance tests when relevant.
  Avoid routine website builds for unrelated work; keep content and API snapshots
  current. Run the full suite only when the change or uncertainty requires it. This reflects the author’s 2026-09-27 validation correction.
- Reuse the relevant .NET/CLR comparisons in linked design notes; deepen research
  when a contract changes. Follow [design research](design-research.md). Proposals
  are exploratory inputs, not specifications or implementation commitments.
- General Raven fixes must be independently validated on a main-based feature
  branch before neoCLR integration. Keep target policies isolated; follow AGENTS.md.
- Distinguish implemented source, tested development artifacts, packaged evidence
  and released behavior. Public APIs require matching reference documentation.
  Runtime release and website publication remain separate operations.

## Maintaining tracking

Put current status and next actions in the owning theme tracker. Put contracts,
alternatives and primary-source comparisons in design notes, commands/results in
experiment records, implemented changes in the changelog, and significant direction
exchanges in the development timeline. Do not append a new “current priority” section
to an older roadmap. Update the existing status row and preserve prior decisions as
history when they matter. Add a new theme only when existing ownership cannot fit it.

The 2026-09-26 issue inventory has been assigned across the theme trackers. It is a
dated inventory, not a fresh GitHub status check. Original issue details and all nine
Raven issue assessments remain in the [archived triage](history/planning-20260927/issue-fix-roadmap.md).
No issues were closed or implementations revalidated by this documentation change.

## Historical entry points

Older links are retained below and lead to dated records. Their former “current” or
“next” headings do not override the current work section above.

<a id="current-checkpoint--finish-this-http-poc-2026-09-26"></a>

- [Current checkpoint — finish this HTTP POC (2026-09-26)](history/planning-20260927/platform-roadmap.md#current-checkpoint--finish-this-http-poc-2026-09-26)

<a id="current-priority--next-release-features-2026-09-26"></a>

- [Current priority — next-release features (2026-09-26)](history/planning-20260927/platform-roadmap.md#current-priority--next-release-features-2026-09-26)

<a id="implemented-checkpoint--http-json-clients-2026-09-26"></a>

- [Implemented checkpoint — HTTP JSON clients (2026-09-26)](history/planning-20260927/platform-roadmap.md#implemented-checkpoint--http-json-clients-2026-09-26)

<a id="issue-driven-priorities--2026-09-26"></a>

- [Issue-driven priorities — 2026-09-26](history/planning-20260927/platform-roadmap.md#issue-driven-priorities--2026-09-26)

<a id="active-direction--sockets-for-a-web-application-2026-09-24"></a>

- [Active direction — sockets for a web application, 2026-09-24](history/planning-20260927/platform-roadmap.md#active-direction--sockets-for-a-web-application-2026-09-24)

<a id="http-client-pipeline-checkpoint--2026-09-24"></a>

- [HTTP client pipeline checkpoint — 2026-09-24](history/planning-20260927/platform-roadmap.md#http-client-pipeline-checkpoint--2026-09-24)

<a id="networking-and-web-release-target--discussion-2026-09-25"></a>

- [Networking and web release target — discussion, 2026-09-25](history/planning-20260927/platform-roadmap.md#networking-and-web-release-target--discussion-2026-09-25)

<a id="scheduling-checkpoint-before-the-public-socket-bridge--2026-09-24"></a>

- [Scheduling checkpoint before the public socket bridge — 2026-09-24](history/planning-20260927/platform-roadmap.md#scheduling-checkpoint-before-the-public-socket-bridge--2026-09-24)

<a id="author-directed-interface-naming--2026-09-25"></a>

- [Author-directed interface naming — 2026-09-25](history/planning-20260927/platform-roadmap.md#author-directed-interface-naming--2026-09-25)

<a id="authority-and-use"></a>

- [Authority and use](history/planning-20260927/platform-roadmap.md#authority-and-use)

<a id="how-the-plans-fit-together"></a>

- [How the plans fit together](history/planning-20260927/platform-roadmap.md#how-the-plans-fit-together)

<a id="milestones-at-a-glance"></a>

- [Milestones at a glance](history/planning-20260927/platform-roadmap.md#milestones-at-a-glance)

<a id="release-checkpoint--async-and-tasks"></a>

- [Release checkpoint — Async and Tasks](history/planning-20260927/platform-roadmap.md#release-checkpoint--async-and-tasks)

<a id="next-release-validation-efficiency"></a>

- [Next-release validation efficiency](history/planning-20260927/platform-roadmap.md#next-release-validation-efficiency)

<a id="post-release-concurrency-direction--2026-09-23"></a>

- [Post-release concurrency direction — 2026-09-23](history/planning-20260927/platform-roadmap.md#post-release-concurrency-direction--2026-09-23)

<a id="future-networking-and-web-namespaces--consideration-2026-09-23"></a>

- [Future networking and web namespaces — consideration, 2026-09-23](history/planning-20260927/platform-roadmap.md#future-networking-and-web-namespaces--consideration-2026-09-23)

<a id="minimal-http-application-dependencies--consideration-2026-09-24"></a>

- [Minimal HTTP application dependencies — consideration, 2026-09-24](history/planning-20260927/platform-roadmap.md#minimal-http-application-dependencies--consideration-2026-09-24)

<a id="progressive-delivery-before-networking--revised-2026-09-23"></a>

- [Progressive delivery before networking — revised 2026-09-23](history/planning-20260927/platform-roadmap.md#progressive-delivery-before-networking--revised-2026-09-23)

<a id="m1--communicate-hello-service-and-hello-client"></a>

- [M1 — Communicate: Hello Service and Hello Client](history/planning-20260927/platform-roadmap.md#m1--communicate-hello-service-and-hello-client)

<a id="m2--work-with-data-file-catalog"></a>

- [M2 — Work with data: File Catalog](history/planning-20260927/platform-roadmap.md#m2--work-with-data-file-catalog)

<a id="m3--handle-real-waiting-and-failure-download-queue"></a>

- [M3 — Handle real waiting and failure: Download Queue](history/planning-20260927/platform-roadmap.md#m3--handle-real-waiting-and-failure-download-queue)

<a id="m4--human-time-and-presentation-activity-report"></a>

- [M4 — Human time and presentation: Activity Report](history/planning-20260927/platform-roadmap.md#m4--human-time-and-presentation-activity-report)

<a id="m5--explain-programs-assembly-explorer"></a>

- [M5 — Explain programs: Assembly Explorer](history/planning-20260927/platform-roadmap.md#m5--explain-programs-assembly-explorer)

<a id="m6--across-hosts-portable-sample-pack"></a>

- [M6 — Across hosts: Portable Sample Pack](history/planning-20260927/platform-roadmap.md#m6--across-hosts-portable-sample-pack)

<a id="research-products-alongside-the-milestones"></a>

- [Research products alongside the milestones](history/planning-20260927/platform-roadmap.md#research-products-alongside-the-milestones)

<a id="active-progress--2026-09-23"></a>

- [Active progress — 2026-09-23](history/planning-20260927/platform-roadmap.md#active-progress--2026-09-23)

<a id="working-rules-and-immediate-next-step"></a>

- [Working rules and immediate next step](history/planning-20260927/platform-roadmap.md#working-rules-and-immediate-next-step)

<a id="later-exploration-generic-math-interfaces--2026-09-24"></a>

- [Later exploration: generic math interfaces — 2026-09-24](history/planning-20260927/platform-roadmap.md#later-exploration-generic-math-interfaces--2026-09-24)

<a id="http-status-names-and-pattern-contracts--discussion-2026-09-25"></a>

- [HTTP status names and pattern contracts — discussion, 2026-09-25](history/planning-20260927/platform-roadmap.md#http-status-names-and-pattern-contracts--discussion-2026-09-25)

## Author-directed calendar slice — 2026-09-27

The author requested useful date rendering with Gregorian/Hebrew calendars, Swedish and Hebrew cultures, invariant presentation and system discovery. The [provisional implementation and evidence](calendar-globalization.md) complete this bounded library slice without permanently reprioritizing later milestones. Unified localization stays separate, with shared interfaces and interchangeable providers/sources still to design.

Author follow-up, 2026-09-27: the next slice is the Time API and time-zone handling. DateTime and globalization have separate feature pages. This records sequencing, not implemented zone support.

### Time and named zones — implemented development slice, 2026-09-27

The selected [Time/zone slice](time-zones.md) adds TimeOffset, ZonedDateTime, named IANA
rules and explicit Unique/Ambiguous/Skipped mapping. DateTime is a nominal
parenthesized union of LocalDateTime and ZonedDateTime following the author's
clarification. Time wraps, civil addition carries the date, and elapsed Instant
addition checks overflow. The bounded 1900–2099 named-zone range and pinned 2025b
database are provisional. Broader rules/providers, parsing and scheduling remain open.

### Feature branch organization (2026-09-30)

At the author's request, structural Function work continues on
`codex/structural-types` with this Self-integrated main as its base. The former
`feature/function-types` name is retired after validation and synchronization.
The branch organization does not enable structural types on main.

Read-only callable import checkpoint (2026-09-30): Raven emission now consumes a
metadata definition snapshot without the producer builder graph. C# and Raven native
consumers return 42; [integration scope](raven-cli-bridge.md#read-only-call-imports--2026-09-30)
still excludes native symbol loading and production target registration.

Compiler adapter checkpoint (2026-09-30): the Raven consumer now calls an optional
compiler-owned emitter API with explicit contracts and diagnostics. Native verification
still returns 42; [integration scope](raven-cli-bridge.md#compiler-owned-native-adapter-checkpoint--2026-09-30)
retains the .NET input bootstrap and leaves native loading/target composition pending.

Multi-file adapter checkpoint (2026-09-30): cross-file function calls now execute
in both source-tree orders, with correct later-file diagnostics and no partial output
on validation failure. The native provider and production composition are still pending.

Native-input checkpoint (2026-09-30): a bounded native declaration reader now feeds
a temporary reference-only PE into Raven's existing semantic provider. The original
native dependency runs with all three emitted application variants to 42. See
[scope and replacement plan](raven-cli-bridge.md#native-dependency-input-through-a-reference-projection--2026-09-30);
a native symbol provider and production registration remain pending.

Raven-to-Raven dependency checkpoint (2026-09-30): the end-to-end producer is now a
Raven library, with a separately compiled Raven application consuming its native
metadata through the temporary reference projection. Overload/helper calls execute
to 42 and dependency failure cases are checked. Native symbol loading and wider
source/visibility support remain staged follow-up work.

Transitive runtime checkpoint (2026-09-30): three Raven-compiled native assemblies
load/verify/execute as a dependency chain to 42, in either supplied module order.
The native reader exposes direct exact references; missing and wrong-revision
transitive dependencies fail runtime verification. Runtime acceptance remains required;
this checkpoint is superseded by the direct container gate below; a native compiler symbol provider remains open.

Direct runtime metadata gate (2026-09-30): the same API-produced PE/#Neo library
files now feed Raven reference binding and neoCLR runtime loading. The two-library
chain and single/multi-file applications verify/run to 42. The runtime rejects missing
or altered recognition data and unsupported required execution schemas. This initial
profile carries format-5 JSON inside section 256/schema 1; it does not yet remove text
parsing or claim faster loading. Next evaluate binary native encoding and separate
load/link/verify timings, while staging the native compiler symbol provider. See the
[implementation and limits](design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30).

Author-selected first acceptance programs (2026-09-30): Hello World directly in
the entry point, then through an entry-point call to another function. Both now
compile to PE/#Neo, load/verify/run in neoCLR, print exactly one line and exit zero.
The bounded Console string-literal bridge uses an explicit compiler reference
contract; general string signatures and no-result source entry points remain staged.

Binary-loading checkpoint (2026-09-30): execution schema 2 now carries bounded CBOR
and decodes directly into the runtime module model without a JSON intermediate.
Schema 1 remains readable; the experimental Raven PE emitter selects schema 2.
Hello World/function-call and the two-library cases pass. Native indexed tables, a
compiler-native symbol provider and broader signatures remain staged work. Compare
[measured loading phases](experiments/extended-cli-metadata/binary-loading.md) before
inferring performance gains; metadata loading and execution are separate costs.

Class-library consumer direction (author clarification, 2026-09-30): compile the
neoCLR runtime class library and load its symbols into Raven. Reuse present .NET-like
metadata through an explicit temporary projection while allowing native contracts
to diverge. The author proposes translating existing JSON into neoCLR assemblies as
a bootstrap; next test a bounded real class-library slice through that translation
and Raven reference loading. General translation is not implemented by the current
static-Int32 writer. Preserve unsupported information by rejecting it, then grow
coverage from the actual library instead of encoding permanent .NET restrictions.

Generic producer follow-through (2026-10-01, author-directed): unconstrained owned
function/static-method instantiations now execute through ordinary CLI MethodSpec and
native generic call arguments. C# producer tests cover forwarding, typed vectors and
nominal identity; API binary verify/run returns 42. Raven source integration now passes
the Order generic consumer in both source orders and on both targets. Generic types,
constraints, imported generic symbols and generic instance methods remain deferred.

Expanded generic acceptance now covers inferred/explicit calls, recursive forwarding,
multiple type parameters and overloads, generic array creation/iteration, conditional
values and object identity. Both source orders return 42 on .NET/native; five binding-valid
unsupported forms reject with source diagnostics and no output. [Recorded evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
includes the tested runtime and consumer hashes. This is a bounded milestone; native
generic types, constraints, instance methods and imported generic symbols are still open.

Next generic receiver slice (2026-10-01): ordinary owned class instance methods now
preserve generic call signatures and receiver identity through API-produced CLI/native
binaries (42). Raven adapter integration and broader receiver consumers follow; generic
types/constraints and native symbol loading remain distinct next boundaries.

Expanded receiver acceptance: no-result generic copy/reverse methods, recursive
instance calls, receiver/argument evaluation order and separate receiver state execute
on both targets (42). [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the consumer and runtime hashes. Runtime checks also preserve generic receiver
and argument roots during forced collection; API validation rejects wrong/missing
receivers and generic constructor projections.

Generic-storage follow-through (2026-10-01): typed default producer support now passes
binary CLI/native execution. Local address/initobj/default helpers preserve method
parameter scope and definite assignment. Raven default(T) and generic clearing are
the next bounded integration, before generic owner/constraint and import work.

Typed default integration now passes Raven-to-.NET/neoCLR execution, including generic
numeric clearing and null-reference faults after clearing object vectors. The producer
API also preserves branch offsets around initialization and rejects invalid local-address
provenance. This completes the bounded receiver/default milestone; generic owners,
constraints, imported generics and full class-library compilation remain open.

Static generic owner producer slice (2026-10-01): independent declaring-type and method
parameter scopes now emit ordinary CLI VAR/MVAR, constructed TypeSpec/MemberRef and
MethodSpec calls. Native open/constructed owner records already support this behavior;
C# producer binaries verify/run 42 without a runtime change. Reference projection
retains type/method arities. This follows the existing [generic metadata contract](generic-metadata.md)
rather than flattening owner parameters into method parameters, which would lose type
identity. Generic instance type layouts and fields remain the next larger boundary;
Raven source integration follows this API slice.

Raven static generic owner integration now verifies/runs 42 on both targets and in
both source orders. Explicit capabilities preserve independent owner/method arguments;
the .NET resolver also fixes source method calls that previously left their declaring
type open. Tested matching producer: `0da5a3b0`, receiver runtime `6a7a0dd2` or later
on `codex/extended-cli-metadata`; Raven `codex/metadata-consumer`. No Runtime Contract
configuration change. Generic owner arrays/defaults and alias mutation pass; generic
instance layouts/fields, constraints, imported owners and full class-library compilation
remain open. See [binary evidence](experiments/extended-cli-metadata/generic-runtime-validation.json).

Static generic owner milestone complete: multiple/reordered owner arguments and
method-to-owner forwarding execute across targets; native reader scope/name checks
reject malformed declarations. Next bounded emission work remains generic instance
layouts and fields before constraints/imports/full class-library acceptance.

Generic instance class producer slice (2026-10-01): AddGenericClass and constructed
class signatures now support instance constructors, owner-typed fields and calls on
exact constructed receivers. CLI uses standard GENERICINST/VAR/member references;
native uses the existing generic record/reference contract, preserving field layout
and type identity. API-produced binaries verify/run 42 without runtime changes.
Raven integration follows. External constructed fields, generic properties, constraints
and full class-library emission remain open. This reuses the [generic metadata design](generic-metadata.md)
rather than creating a second generic storage representation.

Generic instance storage integration now passes: Raven shares generic class value,
constructor and field-body planning; native emission binds owner and method arguments
independently. Primitive, Order and nested Box<Box<int>> consumers verify/run 42 on
.NET/neoCLR in both source orders. Matching producer: `62bf5931` or later; runtime
`6a7a0dd2` or later on `codex/extended-cli-metadata`, Raven `codex/metadata-consumer`.
No Runtime Contract configuration change. Generic properties and external constructed
field references are explicit remaining bridge limits, followed by constraints/imports
and broader class-library acceptance. [Evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
records the consumer and runtime hashes; this does not establish full-library emission.

Generic property producer slice (2026-10-01): generic static/instance property values
and index parameters now preserve VAR scope and accessor associations through standard
CLI Property/MethodSemantics and existing native Constructed owners. Reader validation
requires exact canonical open owner identity/arguments. API reflection execution and
native binary verification/run (42) pass; no native schema or Runtime Contract change.
Raven accessor integration follows; external constructed fields and constraints/imports
remain separate work. This extends the existing [generic metadata contract](generic-metadata.md).

Generic property/indexer integration is now verified through Raven: setter/getter calls,
Order alias mutation and independent generic key/value parameters run 42 on .NET and
neoCLR in both source orders. Matching producer/reader: `dbe03b1a` or later on
`codex/extended-cli-metadata`; Raven `codex/metadata-consumer`; receiver runtime
`6a7a0dd2` or later. No Runtime Contract change. Fifteen focused C# tests pass;
[recorded evidence](experiments/extended-cli-metadata/generic-runtime-validation.json)
includes consumer/runtime hashes. External constructed fields, constraints and generic
imports remain open, along with broader class-library acceptance.

Constructed-field producer slice: typed field references now bind generic owners and
validate exact receiver/value types before emitting standard CLI Field MemberRefs or
existing native field operations. No schema/Runtime Contract change. Raven integration
follows; the author directs generic type constraints immediately after this slice.
Constraints must preserve CLR/native semantics rather than translating unlike flags.

Author-directed type-constraint slice (2026-10-01): owned nongeneric class bounds now
use CLI GenericParamConstraint and existing native TypeBound, preserving meaning rather
than mapping CLR flags to native notvoid/notreference. Producer/reference projection and
binary runtime verification/execution pass; invalid concrete native arguments reject.
Raven integration follows. The author requests the other constraint categories next;
class/struct/new/nullability and interface/dependent bounds require distinct contracts.
This reuses the existing [generic contract research](generic-metadata.md).

Typed boxing continuation (2026-10-03): the metadata IL generator now exposes Box and
raw Emit(Box, signature), producing standard CLI and existing native boxing instructions.
Raven's target-owned capability lowers value/generic-to-object conversions without
changing binding; .NET retains its existing backend. The normal native command executes a generic box
smoke case with the explicit core/retained-System seed. C# API tests separately observe
value dispatch, primitive display and reference identity; the smoke case alone does not
prove these semantics. Union preflight now stops at generated value-payload addressing;
Object virtual calls/formatting and union metadata preservation remain open. Runtime
Contract settings and format versions are unchanged. No implicit reference fallback.

An independent .NET arrow-method bug exposed by this regression work is fixed by
consuming the existing bound return block, retaining implicit generic boxing. Ten tests
pass both on the integration line (Raven 697a093d7) and the main-based fixes branch
(c96305e50); main is unchanged. No new language/Runtime Contract rule is introduced.

See [typed-boxing evidence](experiments/extended-cli-metadata/typed-boxing-2026-10-03.json)
for commands, source/artifact hashes and cross-repository validation (Raven 558462967).

Owned field-address continuation (2026-10-03): metadata IILGenerator now supports
LoadFieldAddress/Emit(Ldflda) for owned mutable definitions and constructed field references.
Raven admits a separate FieldAddress capability carrying only IFieldSymbol; its adapter
resolves owned field handles. Getter/method receivers nested in value fields use the
actual storage; no spill-copy workaround is introduced. Imported/readonly field addresses
remain explicit limits. Ordinary .NET stays on its existing backend. API and ordinary
compiler-command cases execute nested generic mutation through object aliases (42) on both
targets; 120 C# metadata groups and ten focused Raven tests pass. No runtime/format or
Runtime Contract configuration change. Unchanged Option and the source-union controls
now reject at `<RavenFormatUnionValue>` BoundBinaryExpression (null comparison), with no
native output. Null/reference operations, remaining formatting and union/case metadata
preservation are next; full union execution is not claimed.

[Field-address evidence](experiments/extended-cli-metadata/owned-field-addresses-2026-10-03.json)
records commands, artifact/source hashes and Raven c951e33c0.

Reference-test continuation (2026-10-03): the metadata IL generator now supports IsNull,
IsInstance and String checked casts. CLI uses standard ldnull/ceq, isinst and castclass;
native uses existing ref.isnull/isinst/castclass, with no runtime or format change. Raven
has explicit null-test/discard-type-test capabilities; non-user-defined null comparisons
reuse bound operator facts. Overloaded equality is not replaced. .NET retains its general
backend and Runtime Contract settings are unchanged. API and normal driver executions
return 42 on both targets; 121 metadata groups and focused capability tests pass. Source
union preflight now stops at Object.ToString dispatch inside the formatting helper, with
no native output. Wider patterns, core virtual dispatch, remaining formatting and native
union/case metadata remain open.

[Reference-test evidence](experiments/extended-cli-metadata/reference-tests-2026-10-03.json)
records commands/hashes and Raven 5bb8f4dfc.

Development update (2026-10-03): core Object.ToString dispatch now executes for boxed
values through the metadata API and ordinary Raven commands. Union preflight next
requires synthesized null literal emission; native union execution is still pending.

Typed null continuation (2026-10-03): Raven ad3d8a71f emits contextual reference nulls
through the existing metadata LoadDefault operation. Return/local/argument driver cases
execute on both targets; 11 focused tests pass. Plain/generic unions reach the native
union/case metadata gate. Unchanged Option next rejects its TryGetOutput case pattern.
No metadata/runtime change or union execution is claimed. See
[null evidence](experiments/extended-cli-metadata/typed-null-2026-10-03.json).

Option body continuation (2026-10-03): Raven a99cd3c3e reuses existing case-pattern
lowering in conditional branches and Boolean values, and emits RuntimeUnitContract's
inhabited value as a nominal default. Native configured unit out/value arguments
execute (42); .NET configured ValueTuple storage also executes. Existing CLI/native
instructions and runtime mappings are unchanged. Unchanged Option and its dependencies
now pass source-body preflight and reach the union/case metadata publication guard.
Next implement general custom-attribute authoring/reading and introspection, retaining
Raven's .NET UnionAttribute, case-name/ordinal and generic companion contracts through
the existing runtime custom_attributes representation. No union-only wire format or
unmarked-struct fallback is planned. Encoding and native union execution remain unproved.
[Case/unit evidence](experiments/extended-cli-metadata/union-case-unit-2026-10-03.json).


### 2026-10-04 capability-batch progress

The [System strategy](experiments/extended-cli-metadata/system-compilation-strategy.md)
unit subgate now executes unchanged MemoryStream from a separate native library,
including Result<unit, E>, shared interface identity and byte mutation. Generic-unit
storage/calls and 38 focused .NET controls pass. Erased Value/service ABI work remains
next. A cumulative single PE also exposes the existing 1 MiB envelope bound; separate
libraries remain supported. This is not full-System completion.
