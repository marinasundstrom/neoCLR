# Raven runtime-library tests

Development tooling, written in Raven and executed by neoCLR. Tests are ordinary
module-level functions; a class is not required to group them. The Python script
only builds and launches suite processes and verifies their output. Assertions,
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

## TestAttribute and introspection: next layer

Author-selected direction: discover attributed test functions using introspection,
then adapt them to the same case, invocation and report contracts. Explicit
registration remains useful and supported. Classes may eventually provide fixtures,
but must not become mandatory containers for module-level tests.

The proposed marker is `NeoClr.Testing.TestAttribute`. It is **not implemented in
this slice**. Discover it by attribute type identity through custom attribute data,
without executing attribute constructors or test bodies. Introspection discovers
and validates declarations; reflective invocation or a generated typed adapter
provides execution. These responsibilities remain separate.

The first discovery slice should accept parameterless, non-generic module functions
with the existing Result return type. Derive IDs from assembly/module/function
identity, never metadata row numbers or display text; document overload identity
before accepting overloads. Invalid attributed signatures and duplicate IDs must
produce configuration errors, not silently disappear. Establish deterministic
ordering instead of relying on metadata enumeration order.

Existing `MethodInfo` and `GetCustomAttributesData` describe the public introspection
surface. Before claiming native discovery, qualify compiler emission for attributed
module functions, selected-module enumeration, attribute identity retention through
trimming, callable-body retention and invocation of the Result return value.
AOT requires an explicit discovery/rooting policy. Test metadata should not root
unrelated application code. A generated registry is a possible AOT adapter, not a
substitute for silently ignoring missing metadata. Record any bridge changes in
both compiler and integration documentation.

Async invocation, data rows and optional fixtures should be additional adapters;
they must preserve synchronous tests, IDs and pass/fail/skip semantics. Attribute
options, fixture lifetime and async signatures remain future design decisions.

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

Explicit registration works with current typed calls and AOT reachability. Attribute
discovery removes repetitive registration but requires retained metadata and safe
invocation. The execution contract lets us add that capability without rewriting
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

The seven collection tests cover copies, FIFO/LIFO, comparer equality, query
traversal and iterator disposal. The runner-contract suite deliberately produces
one failure and exits 1: the host harness requires exactly that outcome, the
subsequent pass, skip, structured expected/actual values, and rejection of empty
suites and duplicate IDs. Both suites run natively and with the same assemblies
in the interpreter. A successful harness ends with `Runtime library tests: PASS`.
[Initial macOS ARM64 evidence](validation.json) records source and tool hashes.
Windows framework qualification remains pending until its action passes.

This source-included test helper is not part of System.Runtime or its public API
reference assembly. This document covers its complete initial contract. Packaging
it as a reusable library will require public XML/API reference documentation.
