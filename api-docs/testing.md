# Raven testing helpers (development)

`NeoClr.Testing` is a source-included repository test framework, not a released
System.Runtime assembly or an installed test SDK. Include the framework Raven
sources from `runtime/raven/tests/framework`. These types are outside the guest
CoreProbe/RavenDoc selection; this manual reference covers that separate surface.

## TestAttribute

```raven
[AttributeUsage(AttributeTargets.Method)]
public class TestAttribute : Attribute {
    init()
    init(description: string)
    var Description: string { get; set; }
}
```

Marks a test method. The parameterless constructor sets Description to `""`; the
string constructor stores its argument. Named Description assignment is supported
and takes precedence over constructor data. Discovery reads metadata without
executing the constructor or setter. Empty text selects the qualified function
name for display. Changing the description does not change the test ID or order.

```raven
[Test("ArrayQueue preserves FIFO")]
func QueueOrder() -> Result<unit, TestFailure> {
    let queue = ArrayQueue<int>()
    queue.Enqueue(4)
    return Assert.True(queue.Dequeue() is Some(4), "First input must be removed first")
}
```

The marker uses .NET Method targeting, AllowMultiple=false and Inherited=true
usage defaults. The current discovery adapter inspects declarations only; it does
not implement inherited fixture discovery. `[Test(Description: "text")]` is the
named form, analogous to NUnit's Description property.

## Discovery and registration

The host `NeoCLR.TestDiscovery` tool accepts a native test assembly, output registry
path, output JSON manifest path and explicit dependency files. It resolves exact
metadata identities through neoCLR introspection. It does not load executable code
or probe for dependencies. Unsupported signatures, missing dependencies, duplicate
IDs and an empty discovered set fail with exit code 2. Failure diagnostics identify
the declaration when its signature is available.

Supported tests are public/internal, parameterless, nongeneric module functions
returning exactly `System.Result<System.Void, NeoClr.Testing.TestFailure>` from the
selected runtime/test catalogs. Other attributed declarations fail rather than
being ignored. Plain ASCII declaration identifiers are required by this first
source adapter; class fixtures, async tests and data rows are future work.

`scripts/discover-runtime-tests.py` compiles an inspection artifact and generates
`RegisterDiscoveredTests(suite: TestSuite)` plus `tests.json`. Include its output via
`NeoClrTestRegistry` in the test project, then compile the executable. The generated
typed function references retain precisely the selected test bodies for AOT.
Discovery is a host build step; guest module scanning/late loading remain gaps.
This is not a compiler source-generator API.

## Runner contract

These source-included helpers are currently assembly-internal:

- `TestSuite.Add(id: string, name: string, body: () -> Result<unit, TestFailure>)`
  registers an executable case. `Skip(id, name, reason)` registers a skipped case.
  Nonempty unique IDs/names and nonempty skip reasons are required. Manual and
  discovered registrations may be combined; duplicate IDs fail configuration.
- `TestSuite.Run() -> TestRunReport` runs serially in registration order, continues
  after Result failures, and returns an error for invalid/empty registration before
  running bodies. Runtime Faults remain terminal; they are not assertion failures.
- `TestRunReport` has Results, Passed, Failed, Skipped, Error and ExitCode. ExitCode
  is 0 for successful runs, 1 for test failures, and 2 for configuration errors.
- `TestOutcome` has Passed, Failed(TestFailure) and Skipped(string) cases.
  `TestExecution` carries Id, Name and Outcome.
- `Assert.Equal` supports int and string; `Assert.True` accepts a Boolean and a
  message. Both return `Result<unit, TestFailure>`. Propagate or return failures.
  TestFailure has Message, Expected and Actual string properties.
- `ConsoleReporter.Write(report)` writes human-readable results. Its text is not a
  versioned machine protocol.

Test IDs use length-prefixed assembly name, logical module name and qualified
function name followed by `()`. IDs are scoped to the selected assembly; descriptions,
metadata tokens and assembly versions are excluded. Discovery sorts ordinally by
qualified function name; manual additions retain their chosen registration order.

See the [complete repository contract and validation](https://github.com/marinasundstrom/neoCLR/blob/main/runtime/raven/tests/README.md).
Grouping/category selection and future source generators are proposals, not APIs.
