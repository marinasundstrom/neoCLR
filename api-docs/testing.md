# Raven testing helpers

<a id="raven-testing-helpers-development"></a>

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

Prefer a descriptive function name when it supplies the test's whole description:

```raven
[Test]
func `IPv4 parsing and canonical formatting round trip`() -> Result<unit, TestFailure> {
    _ = CheckAddress("0.0.0.0", "0.0.0.0", false)?
    _ = CheckAddress("127.0.0.1", "127.0.0.1", false)?
    _ = CheckAddress("255.255.255.255", "255.255.255.255", false)?
    return Assert.True(true, "Valid address corpus completed")
}
```

See the executable [IP address tests](https://github.com/marinasundstrom/neoCLR/blob/main/runtime/raven/tests/ip-address-values/Tests.rvn)
for the helper and complete corpus. Optional descriptions remain useful as display
overrides. Moving description text into the function name changes its ID,
name-based filters and ordinal registration order. Backticks themselves are absent
from metadata, selectors and reports.

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

Supported tests are public/internal, nongeneric module functions
returning exactly `System.Result<System.Void, NeoClr.Testing.TestFailure>` from the
selected runtime/test catalogs. Other attributed declarations fail rather than
being ignored. Development discovery accepts Unicode and spaced function names,
including Raven backtick identifiers. Generated references quote namespace
components and the complete function name separately, so a period in a function
name is preserved. This requires the unreleased backtick-capable Raven compiler;
Preview 18's compiler predates that syntax. neoCLR metadata already carries these
names; this is a source-adapter change. Class fixtures and async tests remain gaps.
Parameterized tests require data attributes.

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
Parallel scheduling, class fixtures and future source generators remain proposals.


<a id="argumentsattribute-and-methoddatasourceattribute-development"></a>

## ArgumentsAttribute and MethodDataSourceAttribute

Testing follows [TUnit's attribute names](https://tunit.dev/docs/comparison/attributes/).
Both attributes target methods, allow multiple annotations and accompany `[Test]`.
They belong to `NeoClr.Testing`; they have no benchmark framework dependency.

`ArgumentsAttribute` constructor overloads are `(int)`, `(string)`, `(bool)`,
`(int, int)`, `(int, int, int)`, `(string, string)`, and `(string, int)`.
Each annotation supplies one complete row, whose values must exactly match the
function parameters. Null and general attribute arrays are unsupported native
metadata gaps. There are no data attribute properties or named arguments yet.

`MethodDataSourceAttribute(methodName: string)` names a public/internal parameterless,
nongeneric module function returning `T[]` for a test accepting one `T`. Unqualified
names resolve in the test's module; qualified names resolve in the same assembly.
Generated registration calls the provider once and captures each element. The
provider must return 1–10,000 rows in deterministic order. Sources execute at suite
registration, not during metadata inspection. Empty/oversized sources fault; invalid
provider signatures, duplicate sources/inline rows, and inline arity/type mismatches
fail discovery. An input object can contain multiple fields; sequences, tuple
spreading, async sources and source arguments are not yet supported.

Each row is a separately reported/filterable test. Inline selectors append `(values)`;
source selectors append `/Qualified.Provider[index]`. Indices are zero-based and
change meaning if source order changes. Source entries in the metadata inventory
have `dynamic: true`; they describe registration templates rather than enumerated
runtime cases. See the [tested data examples](https://github.com/marinasundstrom/neoCLR/blob/main/runtime/raven/tests/discovery-contract/Tests.rvn).


<a id="categoryattribute-development"></a>

## CategoryAttribute

`[Category("name")]` tags a `[Test]` module function and all its argument rows.
`CategoryAttribute : Attribute` has `init(name: string)` and read-only `Name: string`.
It targets methods and allows repeated annotations. Discovery rejects empty category
names; repeated identical names are deduplicated, and categories are sorted ordinally
in the metadata inventory. Categories affect selection, not case identity or order.
Class/assembly category inheritance is not supported yet.

`TestSuite.Add(id, name, body, categories: string[])` registers manual categories,
copying the array contents. Empty category names produce a registration error.
The existing three-argument overload registers an untagged test.
`RunCategory(category: string) -> TestRunReport` selects exact, case-sensitive category
matches. The CLI spelling is `--category <name>`; choose one of category, substring
`--filter`, or exact `--id`. Empty input and unmatched selection return configuration
exit code 2. Execution remains serial; categories are not concurrency constraints.

See the [testing project guide](https://github.com/marinasundstrom/neoCLR/blob/main/tools/testing/README.md)
for build/discovery commands, runner selection and project ownership.

### Escaped names and source adapters (development)

Data-source attributes take logical metadata names without backticks, either a
local function name or its qualified name. Ambiguous matches are rejected.
Generated adapters preserve Unicode text, including supplementary characters,
in both source references and string literals. Inline-row ID suffixes retain
canonical JSON spelling independently of Raven literal spelling.

Raven identifiers cannot contain a backtick or line break; discovery rejects names
it cannot represent in generated Raven source. These are source limitations, not
neoCLR metadata restrictions. Namespace metadata is still interpreted as dotted
qualification. Flattened qualified-name collisions are rejected as duplicate IDs;
this change does not introduce a new identity format.
