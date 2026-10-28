# Project retention qualification — 2026-10-28

The [project file contract](../../native-retention.md) is qualified on macOS ARM64
using the restored Preview 18 bundle under
`target/selection-box-scan-20261028/release/neoclr-native-poc` and its matching
`bin/neoclr-codegen`. Compiler/library changes are not part of this slice.

Focused Python checks pass: six project-retention tests, six existing sample-runner
tests and three scalar-interop builder tests (15 total). They cover absent/default
and explicit relative files, malformed/duplicate/unknown declarations, conflicting
options, unsupported profiles, final-image ordinal changes, mixed modes, dependency
generic definitions, and missing/ambiguous names.

Run with `python3 -m unittest discover -s scripts -p test_project_retention.py`,
then the same command with `test_native_sample_arguments.py` and
`test_native_interop_build.py`.

Three builds of the checked-in DI sample pass with the current project builder:

1. Automatic `App.retention.json`: native execution prints `Hello, Ada!`, exit 0.
2. Explicit file with Greeter constructors + methods and GreetingSettings
   properties: native execution prints `Hello, Ada!`, exit 0. Backend accepts the
   combined schema-8 policy; this does not independently qualify every reflected
   property operation or method inspection operation.
3. Explicit Greeter method-inspection-only file: compilation succeeds but execution
   exits 1 at `TypeConstructors` with an unretained metadata fault. This verifies
   method inspection does not silently grant constructor retention. The existing
   diagnostic says “property metadata”; that wording is not changed here.

[Results](results.json) retain policies, input/executable hashes, exit codes and
output. Build commands use `scripts/build-native-project.py --project
 docs/experiments/dependency-injection/sample/App.rvnproj --bundle <bundle> --aot
 <bundle>/bin/neoclr-codegen --output <new-output>`, adding `--retention-file
 <absolute-policy-path>` for the explicit cases. Generated full build logs and
native artifacts remain under `target/project-retention-20261028`.

No backend/runtime semantics changed. Windows execution, installed build-kit
repackaging and automatic JIT policy discovery were not tested. No managed API or
API snapshot changed; no website build was needed for these documentation edits.
