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
fixtures, filtering and generic/collection assertions are not implemented yet.

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
conventions before selecting the public attribute contract. No grouping attribute
or filtering API is implemented yet; manual registration remains an option. Groups
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

Next: build ownerless MethodInfo/ParameterInfo and attribute snapshots without a
synthetic declaring type, expose guest GetFunctions, then qualify explicit native
retention and callable registration. Current host-generated registrations remain the
working discovery path throughout that work.
