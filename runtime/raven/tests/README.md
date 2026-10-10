# Raven runtime-library tests

Development tooling, written in Raven and executed by neoCLR. Tests are ordinary
module-level functions; a class is not required to group them. The host tooling compiles, discovers metadata, builds adapters, launches suites
and verifies their output. Assertions,
execution and reporting live in Raven.

## Core contract v1

This is the intended stable source contract for the repository's test suites, not
a released testing package. Preserve these semantics as discovery and execution
support grow; incompatible changes require an explicit migration.

- A test has a nonempty, unique **Id**, a separate nonempty display **Name**, and
  a synchronous body `() -> Result<unit, TestFailure>`.
- `TestSuite.Add(id, name, body)` registers a test. `Skip(id, name, reason)` registers
  a skipped case with a nonempty reason. Registration errors and empty suites are
  reported before any test executes.
- `Run()` executes serially in registration order. `Ok(())` passes;
  `Error(failure)` fails that test and execution continues. A skipped case has no
  body invocation. Each run creates a fresh report; subsequent runs do not append
  to a previous report. Reports are not promised deeply immutable.
- `TestOutcome` is `Passed`, `Failed(TestFailure)` or `Skipped(string)`.
  `TestExecution` preserves Id, Name and Outcome. `TestRunReport` exposes Results,
  Passed, Failed, Skipped and Error. A nonempty Error denotes a suite configuration
  error, separate from an assertion failure. ExitCode is 0 for success (including
  all-skipped suites), 1 for test failures, and 2 for configuration errors.
- `Assert.Equal(expected, actual)` supports int and ordinal string equality.
  `Assert.True(condition, message)` checks a Boolean condition. Both return
  `Result<unit, TestFailure>`. Failures carry Message, Expected and Actual strings.
- `ConsoleReporter.Write(report)` formats results separately from execution.
  Human-readable wording is not a versioned machine protocol.

From the compiled collection suite:

```raven
[Test("ArrayList copy has independent storage")]
func ListCopy() -> Result<unit, TestFailure> {
    let source = ArrayList<int>()
    source.Add(4)
    let copy = ArrayList<int>(source)
    source[0] = 9
    _ = Assert.Equal(1, copy.Count)?
    return Assert.Equal(4, copy[0])
}
```

Propagate each assertion with `?`, or return its result. Ignoring a failure result
can falsely pass a test; the current compiler does not enforce this convention.
The explicit discard before `?` is the currently qualified native spelling.
Terminal runtime Faults abort the suite process and are not converted into assertion
failures. Isolation is currently per suite, not per test. Async tests, data cases,
fixtures and generic/collection assertions are not implemented yet.

## TestAttribute discovery (development)

`NeoClr.Testing.TestAttribute : System.Attribute` is now source-included with the
framework. Its `[AttributeUsage(AttributeTargets.Method)]` contract rejects other
targets and duplicate annotations. Use `[Test]` or `[Test("human-readable description")]`.
`var Description: string { get; set; }` exposes the supplied text; the parameterless constructor
sets an empty string. Empty descriptions select the qualified function name.
Descriptions affect reporting, never test identity or ordering.
`[Test(Description: "text")]` is also supported; named assignment overrides a
constructor description, matching normal attribute construction order.

```raven
[Test("ArrayQueue preserves FIFO")]
func QueueOrder() -> Result<unit, TestFailure> {
    let queue = ArrayQueue<int>()
    queue.Enqueue(4)
    return Assert.True(queue.Dequeue() is Some(4), "First input must be removed first")
}
```

The host tool `tools/testing/NeoCLR.TestDiscovery` dynamically discovers annotations
in a compiled native assembly through neoCLR's metadata/introspection model. It
uses exact catalog-owned attribute and result-type identities, not an attribute
name alone. It never loads the test assembly into the CLR or executes attribute
constructors/test bodies. The framework currently lives in the test assembly;
separately packaged marker libraries remain future work.

Discovery accepts accessible parameterless, nongeneric **module functions** returning
`Result<unit, TestFailure>`. Attributed class methods, unsupported signatures,
repeated markers, duplicate IDs and empty discovery are configuration errors.
Overloads/data rows/async/fixtures remain unsupported. The current generated source
adapter requires ordinary ASCII Raven identifiers. Function IDs use length-prefixed
assembly name, logical module name and qualified function name plus `()`; they do
not include metadata row numbers, descriptions or assembly versions. This keeps IDs
stable across recompilation; they are scoped to the selected test assembly, not a
global package/version identity. Tests sort by ordinal qualified function name.

`scripts/discover-runtime-tests.py` first compiles with an empty registry, then
reads the artifact and writes `TestRegistry.rvn` and `tests.json`. The final compile
includes the generated `RegisterDiscoveredTests(suite)` function. Typed method
references explicitly retain the test bodies for AOT; there is no general reflective
invocation ABI or blanket metadata root. Discovery occurs **on the host before final
compilation**, not inside a running native executable. Guest module scanning and
late-loaded test assemblies remain gaps, not new runtime semantics. A source change
requires rediscovery and recompilation; the validation harness always performs both.

The checked collection suite uses this path with no handwritten registration list.
The contract suite checks constructor/named descriptions, fallback names, ordering, ignoring unmarked
functions/another marker, no discovery-time test execution, and continuation after
failure alongside a manually registered case. Both run through the same Raven TestSuite contract in interpreter and AOT.
`TestSuite.Add` and `Skip` remain supported: callers can register cases manually,
call `RegisterDiscoveredTests`, or combine them. Duplicate IDs still fail registration.

Future grouping (author direction, 2026-10-10): evaluate repeatable category/trait
attributes, separately from descriptions and stable IDs. Group/filter at discovery
or reporting rather than requiring classes. Compare .NET testing category/trait
conventions before selecting the public attribute contract. No grouping attribute is implemented yet; runner filtering is described below; manual registration remains an option. Groups
should enable selective execution, not merely presentation. Keep selection separate
from discovery so a future compiler source generator can supply equivalent cases
and group metadata without changing the runner contract.

Description research (2026-10-10): [NUnit TestAttribute](https://docs.nunit.org/api/NUnit.Framework.TestAttribute.html)
exposes a writable Description property; the named-property form follows that
structure. The string constructor is a Raven convenience. Unlike NUnit's separate
name/description UI, this console runner currently uses the description as its
single display label. Empty nonnullable text replaces null as the fallback signal.
This simplifies the small runner, at the cost of conflating those display fields;
future richer reports should preserve both. Grouping candidates include NUnit's
[repeatable CategoryAttribute](https://docs.nunit.org/api/NUnit.Framework.CategoryAttribute.html)
and [property bags](https://docs.nunit.org/articles/nunit/writing-tests/attributes/property.html);
the contract is deliberately still open.

## .NET comparison and tradeoffs

Primary sources reviewed 2026-10-10:
[Microsoft.Testing.Platform test framework architecture](https://learn.microsoft.com/en-us/dotnet/core/testing/microsoft-testing-platform-architecture-test-framework)
separates discovery, execution and reporting, with stable test-node identity and
display names. We adopt those boundaries without implementing its host protocol.
[NUnit assertions](https://docs.nunit.org/articles/nunit/writing-tests/assertions/assertions.html)
normally stop a test through an assertion exception;
[multiple-assert scopes](https://docs.nunit.org/articles/nunit/writing-tests/assertions/multiple-asserts.html)
are a separate facility. neoCLR currently uses explicit Result propagation to stop
a test recoverably without runtime exception unwinding or ambient assertion state.
The cost is additional syntax, ignored-result risk and no automatic exception stack
trace. This is a platform accommodation, not a claim of better ergonomics or speed.

Explicit registration works with current typed calls and AOT reachability. Host attribute
discovery removes repetitive registration; generated typed adapters supply AOT
reachability and invocation without runtime assembly loading. The execution contract lets us add that capability without rewriting
existing tests. There is no claim of Microsoft.Testing.Platform compatibility.

## Running and evidence

Use an existing matching native development bundle and built AOT/interpreter tools:

```sh
python3 scripts/test-runtime-library.py \
  --bundle target/library-scopes-final/bundle \
  --output target/runtime-library-tests
```

The output directory must be new. On macOS, select the installed SDK with
`export SDKROOT="$(xcrun --sdk macosx --show-sdk-path)"` before building. Windows
requires the MSVC x64 tools environment. The Windows collections action runs this
script after creating its matching development bundle.

The seven discovered collection tests cover copies, FIFO/LIFO, comparer equality, query
traversal and iterator disposal. The runner-contract suite deliberately produces
one failure and exits 1: the host harness requires exactly that outcome, the
subsequent pass, skip, structured expected/actual values, and rejection of empty
suites and duplicate IDs. All suites run natively and with the same assemblies
in the interpreter. A successful harness ends with `Runtime library tests: PASS`.
[Initial macOS ARM64 evidence](validation.json) records source and tool hashes.
The initial Windows action [38039487454](https://github.com/marinasundstrom/neoCLR/actions/runs/38039487454)
succeeds at 9db8eb4c; artifact hashes have not yet been independently checked.

This source-included test helper is not part of System.Runtime or its public API
reference assembly. This document covers its complete initial contract. Packaging
it as a reusable library will require public XML/API reference documentation.


[Discovery milestone evidence](discovery-validation.json) records the matching
compiler, source/tool hashes, all three suites in interpreter/macOS ARM64 AOT, five
rejected discovery shapes, and discovered IDs/descriptions. Both final artifacts
rediscover exactly the same registration source and manifest as their inspection
build. The deliberate failing suites are successful contract checks only when their
expected output and exit code 1 match. The Windows action now runs this same gate;
qualification of this revision is pending.


## Guest discovery foundation (2026-10-10)

The runtime now has a shared metadata selector for exact assembly/module free-function
definitions, exposed to the native backend as an unstable definition-key helper.
It preserves metadata order, private functions and generic definitions so the test
framework can diagnose unsupported tests rather than silently skipping them. It
excludes type-owned methods, child namespaces and instantiated generic bodies.
Selection never invokes bodies and does not retain them for AOT.

Four focused assembly-info checks pass with native metadata enabled, including
same-namespace assembly isolation, empty modules, unknown scopes, missing/duplicate
identities, ambiguous legacy scopes and rejection of legacy catalogs by the native
bridge. Existing host GetFunctions is the behavioral baseline. The .NET comparison
remains Module.GetMethods for global callables, with neoCLR's intentional namespace
scope in place of a physical CLI module; signature inspection stays separate from
execution. This is not a new public guest API or in-process test registration.

Ownerless MethodInfo/ParameterInfo and method-level attribute recipes now exist in
the unstable backend bridge. They preserve names, signatures, module ownership and
fixed/named descriptions without invoking bodies or constructors. Public DeclaringType
is absent; a structural signature is only the private parameter identity key already
used by bound-function descriptors. Open generic signature materialization remains
an explicit fault, while its attributes remain inspectable. Two additional recipe
checks pass alongside the two module-selection checks; this is not a guest API or
native executable discovery qualification.

Next: connect these recipes to guest GetFunctions and ownerless attribute queries,
then qualify explicit native retention and callable registration. Current host-generated registrations remain the
working discovery path throughout that work.


Guest enumeration now passes the [interpreted consumer](../../../docs/experiments/guest-functions/README.md)
with the framework's real TestAttribute. ModuleInfo.GetFunctions exposes ownerless
method/parameter metadata and method-level attribute data, including descriptions.
This does not yet register callable test bodies or enable native runtime discovery;
host-generated typed registration remains the supported execution path.

Author priority (2026-10-10): runner filtering comes before grouping attributes after
this discovery slice. Keep stable IDs and display names separate, preserve manual
registration, and make selection available through the runner. Grouping remains a
later extension, not a prerequisite for selecting tests.


## Selecting tests through the runner

Development (2026-10-10): collection and discovery suite entry points now call
`TestRunner.Run(suite, arguments)` from `Main(arguments: string[])`, which parses the selection,
writes the report and returns its exit code. The contract probe also accepts these
arguments; its no-argument run retains its internal framework checks.

```sh
./app --filter ArrayQueue
./app --id manual
neoclr run app.dll --system System.runtime.neox --module System.Runtime.dll --object-root System.Runtime.dll -- --filter ArrayQueue
```

Supply the appropriate library paths/dependencies when invoking the interpreter.
The filter is passed after `--` so it reaches the guest runner.

- No arguments runs all registered cases, in registration order.
- `--filter <text>` selects a literal, case-sensitive ordinal substring of either
  the stable ID or the display name. No wildcard or expression syntax is interpreted.
- `--id <id>` selects an exact, case-sensitive ordinal ID. IDs can be taken from the
  discovery `tests.json` manifest or supplied by manual registration. Description
  changes do not affect this selection.
- Exactly one selector is accepted. Unknown flags, missing/empty values, additional
  arguments and zero matches produce a configuration error and exit code 2.
- Filtering occurs after registration validation and before any selected body runs.
  Excluded tests do not run and are not reported as skipped. Selected skip records
  remain skipped; selected failures still return 1 and permit later selected tests
  to execute. Selected passing/all-skipped runs return 0. Duplicate registration
  errors cannot be hidden by filtering.

Programmatic APIs are `TestSuite.Run()` (all), `Run(filter: string)` (substring; empty
means all), and `RunId(id: string)` (exact; empty is a configuration error). Each
returns a fresh TestRunReport and does not mutate registrations. Console parsing
and reporting stay in TestRunner/ConsoleReporter. `TestRunner.Run(suite, arguments)`
accepts guest arguments excluding the executable name. Native and interpreted
entry points supply the array directly. Both host-discovered typed adapters
and manually registered cases use this same execution path; runtime reflection is
not required for filtering. Grouping attributes and listing are not added.

.NET comparison, reviewed 2026-10-10: [VSTest filtering](https://learn.microsoft.com/en-us/dotnet/core/testing/selective-unit-tests)
provides property/operator expressions, case-insensitive matching and Boolean
composition; a bare expression selects by fully qualified name. This first neoCLR
runner is explicitly a smaller development contract, not a compatible parser.
It matches IDs and display names because those are the framework's current stable
selection fields. It uses the runtime's existing ordinal substring operation rather
than embedding a second Unicode matching implementation in the framework. The cost
is case-sensitive name searches and no expression/category support. Richer filtering
and consistent case-insensitive substring support remain gaps to evaluate, not
claims of improvement over .NET. Exact IDs provide reproducible selection now.

The runtime test harness checks selectors in both interpreter and native execution,
including discovered and manual cases, selected failures/skips, registration order,
case sensitivity, no matches and malformed command lines. It preserves the original
no-argument suite expectations alongside these checks. Use the host harness option
`--suite runner-contract` for a focused runner check (`--suite` is repeatable);
omitting it runs all suites and the fixed discovery-signature rejection checks.
Focused suite runs still discover and verify their actual test registrations. A Raven assertion case also
checks argument count, empty strings, spaces, quotes, backslashes and non-ASCII text.


Author development direction (2026-10-10): add focused Raven framework tests alongside
new runtime-library features and fixes. Use module-level TestAttribute functions for
observable behavior; manual registration remains useful for runner contracts and
adapters. Keep lower-level compiler/backend tests where guest code cannot observe the
contract. Extend/migrate coverage as areas are developed, not by replacing unrelated
working test infrastructure all at once.

[Filtering and entry-argument qualification](filtering-validation.json) records 32
native/interpreted executions and five discovery-signature rejection cases. Windows
execution is covered by the existing action and remains pending for this revision.

## Porting existing checks

Start with observable library behavior in existing standalone consumers. Split
large Boolean/exit-code programs into focused module functions marked `[Test]`,
use descriptive assertions, and propagate failures with `?`. Preserve ordering,
ownership and mutation scenarios from the original rather than merely copying a
final success message. New library behavior belongs here by default.

The first port takes `CheckMaps` and `CheckMapConstruction` from
[the standalone collection consumer](../../../docs/experiments/native-collections/Main.rvn)
into [five map tests](collections/MapMaterialization.rvn): pair snapshots and
iteration/deconstruction, independent copy construction, ToMap queries/selectors,
collisions/empty input, and reference values/comparer behavior. The old executable
remains a broader integration smoke test pending cross-platform equivalence; it
should not gain duplicate behavior tests. Duplicate-key terminal failures and
compiler rejection cases remain separate process/compiler tests.

[First migration qualification](map-migration-validation.json): all 12 collection
cases pass natively on macOS ARM64 and in the interpreter, and `--filter Map`
selects exactly the five ported tests. Windows execution is pending its action.

The next port splits queue/stack entry checks and `CheckSnapshots` into six tests in
[QueueStack.rvn](collections/QueueStack.rvn): wrapped queue growth, stack growth,
reference identity and clear/reuse for each, and FIFO/LIFO captured iterators. Small
explicit capacities force growth; assertions do not require a particular growth
factor. Captured iterators use `use` with explicit advances, preserving the original
snapshot timing rather than opening a fresh traversal after mutation.

[Queue/stack migration qualification](queue-stack-migration-validation.json): all 18
collection tests and the existing ArrayQueue/Map selectors pass native macOS ARM64
and interpreted execution. Windows qualification remains pending.

## Collection migration batch

The remaining observable checks from `native-collections/Main.rvn` are migrated as
one batch: 3 set tests, 6 construction tests, 5 additional loop cleanup tests and
14 additional query cleanup tests. The existing Any, predicate Any and break tests
remain in the original suite. This makes 46 collection tests across three projects:

- `collections`: 18 ordering, copy, map and snapshot tests.
- `collection-construction`: 9 constructor, comparer, collision and set snapshot tests.
- `collection-iteration`: 19 query and loop ownership tests.

Run all three with repeated `--suite` arguments, or omit `--suite` to run them with
the runner/discovery contracts. The default Windows action includes all projects.
The shared observable iterator records acquisition, advances, Current reads and
disposals. Query assertions preserve short-circuit and advance-only behavior;
labeled continue and return verify cleanup at their actual control-flow boundaries.

A single 46-test executable exceeded the current native metadata encoder envelope
(`NEOMETA003`). Splitting projects is a temporary packaging workaround, not a maximum
test count contract; raising or removing the metadata limit needs separate validation.
No .NET/CLR collection semantics change in this migration. Existing comparer,
snapshot, Option/Result and UTF-8 contracts remain the subjects of the tests.

[Collection batch qualification](collection-batch-validation.json): all 46 tests pass native macOS ARM64
and interpreted execution; the existing Map/ArrayQueue filters also pass. Windows
qualification remains pending.

## JSON DOM migration batch

`--suite json-dom` runs 12 attributed module tests ported from the public
`json-dom/DomContracts` and `native-json` document checks. They cover array bounds,
object order, explicit null versus missing fields, duplicate insertion atomicity,
escaped duplicate names, scalar properties, fractional number conversion, malformed
input, nested Unicode documents, all root kinds, canonical round trips and cycles.
The default gate includes this suite in native and interpreted execution.

The tests use the public System.Data.Json implementation. Reflection-based object
mapping, stream ownership/partial I/O, quota boundaries and terminal faults remain
in their existing consumers and are not claimed as migrated by this batch. String
union payloads are extracted and compared through Assert.Equal: native emission
currently rejects string-literal BoundConstantPattern (NEOMETA001). This is a compiler
gap, not a restriction on JSON field names or on the testing contract.

[JSON batch qualification](json-dom-batch-validation.json): all 12 tests pass native
macOS ARM64 and interpreted execution. Windows qualification remains pending.

## JSON stream migration batch

`--suite json-streams` runs 12 tests ported from the public JSON DOM/stream consumer.
They cover one-byte reads/writes splitting UTF-8 scalars, borrowed stream ownership,
memory-stream round trips, read/encoding/syntax failures, partial write errors,
zero progress, cycle preflight, and input/output quotas. Exact-boundary cases include
multibyte strings, escaped strings and long number tokens. Error assertions inspect
union causes rather than relying on formatted error text.

The stream interfaces expose Close rather than Disposable, so the fixtures retain
explicit Close calls. Boundary strings are built by doubling chunks to avoid making
these contract tests depend on thousands of temporary concatenations. No serializer
or I/O semantics change: existing bounded UTF-8 and ownership contracts remain the
subjects of the tests. Stream contracts and quota behavior deliberately differ from
unbounded .NET JSON convenience calls; see the existing [DOM design](../../../docs/json-dom-design.md).

The default harness includes this suite. MemoryStream's independent seek/range/closed
state tests, the acknowledgement sample, reflection mapping and broader corpus checks
remain in their existing consumers. These tests do not replace those validations.

The stream quota cases should use the release interpreter (the harness default,
`target/release/neoclr`). A debug-interpreter attempt passed the native phase but
exceeded the unchanged 180-second interpreter timeout. Keep byte-at-a-time boundary
coverage; do not infer a release performance result from that debug timeout.

[JSON stream batch qualification](json-streams-batch-validation.json): all 12 tests
pass native macOS ARM64 and release-interpreted execution. Windows remains pending.

Batch `memory-stream`: MemoryStream seek, range, quota, overwrite and closed-state behavior; 5 tests pass native macOS ARM64 and
release-interpreted execution ([evidence](memory-stream-validation.json)).
Source: [docs/experiments/json-dom/Main.rvn](../../../docs/experiments/json-dom/Main.rvn). Windows remains pending.

Batch `string-construction`: String character-sequence copying, graphemes and named arguments; 5 tests pass native macOS ARM64 and release-interpreted execution ([evidence](string-construction-validation.json)).
Source: [docs/experiments/string-sequence/Main.rvn](../../../docs/experiments/string-sequence/Main.rvn). Windows remains pending.

The String suite now requires both native and interpreted execution: Char receiver
projection and empty Object initialization in native String factories are fixed.
Remaining `nativeAdmissionGaps` are unresolved work, not native test passes.

Batch `unicode-casing`: Unicode casing expansions, contextual sigma and text preservation; 6 tests pass native macOS ARM64 and release-interpreted execution ([evidence](unicode-casing-validation.json)).
Source: [docs/experiments/casing-integer/Main.rvn](../../../docs/experiments/casing-integer/Main.rvn). Windows remains pending.

Batch `int64-parsing`: Int64 boundary parsing, lexical errors, overflow and formatting; 5 tests pass native macOS ARM64 and release-interpreted execution ([evidence](int64-parsing-validation.json)).
Source: [docs/experiments/casing-integer/Main.rvn](../../../docs/experiments/casing-integer/Main.rvn). Windows remains pending.

The missing native `neoCLR.Runtime.ParseInt64` binding is now implemented. The
suite requires both modes to pass; its earlier admission exception is removed.
Focused backend checks additionally compare lexical/error/null behavior with the
interpreter. Windows requalification remains pending.

Batch `primitive-parsing`: primitive numeric and Boolean parsing values, format errors and overflow; 12 tests pass native macOS ARM64 and release-interpreted execution ([evidence](primitive-parsing-validation.json)).
Source: [docs/experiments/numeric-contracts/Main.rvn](../../../docs/experiments/numeric-contracts/Main.rvn). Windows remains pending.

The primitive parser suite now requires native execution. Single/Double generic
values and primitive erased payloads are supported, and exact parser bindings share
the interpreter kernel. Scalar/tag, NaN, negative-zero and conversion checks pass.
Generic numeric operator and JSON conversion checks remain in the original consumer.

Batch `path-values`: Path equality, hashing, display and colliding map behavior; 5 tests pass native macOS ARM64 and release-interpreted execution ([evidence](path-values-validation.json)).
Source: [docs/experiments/path-object/Main.rvn](../../../docs/experiments/path-object/Main.rvn). Windows remains pending.

The Path suite preserves Object/interface dispatch assertions. All five tests now
pass both modes after implementing native Object slots and default source-name
display. No native-admission exception remains in the gate.

Batch `http-header-values`: HTTP header repetition, lookup validation, snapshots and request metadata; 5 tests pass native macOS ARM64 and release-interpreted execution ([evidence](http-header-values-validation.json)).
Source: [docs/experiments/http-headers/Main.rvn](../../../docs/experiments/http-headers/Main.rvn). Windows remains pending.

Batch `http-route-matching`: HTTP route parsing, captures, decoding, numeric conversion and quotas; 8 tests pass native macOS ARM64 and release-interpreted execution ([evidence](http-route-matching-validation.json)).
Source: [docs/experiments/http-routing/Main.rvn](../../../docs/experiments/http-routing/Main.rvn). Windows remains pending.

The route suite ports RoutePattern/RouteMatch behavior; application-specific helper
and route-to-union mapping checks remain in the original routing consumer.

Batch `ip-address-values`: public IPAddress parsing corpus, canonical formatting, equality and allocation churn; 7 tests pass native macOS ARM64 and release-interpreted execution ([evidence](ip-address-values-validation.json)).
Source: [docs/experiments/ip-address-hierarchy/Public.rvn](../../../docs/experiments/ip-address-hierarchy/Public.rvn). Windows remains pending.

The address suite uses the public parser and its complete 13-valid/29-invalid literal
corpus. It preserves hash/equality round trips and allocation churn, but does not
claim a new forced-GC or socket test. Native inherited Object-slot admission and
integer shifts now support its hashing path. Socket operations and the historical
private-byte-array representation probe remain separate.

The literal-pattern workaround is removed from MemoryStream, and JSON DOM now
uses a nested String constant pattern. Both suites pass with Raven
`b2f3ba0f8b8e92f0c516155f563b332b0a6fad54`; see their refreshed validation reports.
The compiler's six focused pattern checks include full-width mismatches, null and
dynamically constructed strings. Libraries retain the previous compatible metadata
snapshot; the bundle manifest records the compiler replacement and all file hashes.

Batch `uri-resolution`: URI resolution, grammar errors, ordinal identity and byte
quotas; 11 tests pass native macOS ARM64 and release-interpreted execution
([evidence](uri-resolution-validation.json)). Source:
[URI probe](../../../docs/experiments/uri/verify.py). The suite covers all 46 resolution
pairs and 14 invalid inputs, typed/text Resolve parity, Object/interface equality,
hash agreement, collection retention and the 4096-byte boundary. The original probe
retains its forced-GC and independent .NET comparison roles.

The ten author-requested migration batches add 69 library tests, bringing the library
suite to 139 tests across 15 projects, plus discovery/runner contract checks. Every
migrated suite has passing local native/interpreter evidence; no expected native
admission failures remain. The gate requires both modes. Windows x64 object emission
passes for the final Path/IPAddress/URI suites; this does not qualify Windows linking
or execution. The Windows action runs the expanded suite with a 90-minute job budget.
