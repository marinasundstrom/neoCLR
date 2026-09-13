# Try the generic-array Raven/neoCLR build

For the newer .12 tools, use [the collection installation](raven-collections-local-build.md).
The .11 record below remains specific to that earlier build.

The locally installed SDK and VS Code extension are **0.1.12-neoclr.11**. This is
an experimental local build, not a published neoCLR or Raven release. It includes
generic managed arrays, invariant mutable-array diagnostics, the separated native
allocation API, and the existing runtime-library, union and query demonstrations.

## Open and run

Run this prepared launcher:

```sh
/Users/robert/.neoclr/experiments/generic-arrays-20260913/open-demo.sh
```

It opens the new demo with its isolated VS Code profile and extension. Open
`Main.rvn`, then select **Terminal → Run Task → neoCLR: Run saved project**.
Save edits before running. The ordinary Raven Run/Debug toolbar is not the neoCLR
execution pipeline. The configured task compiles, imports, verifies and runs the
saved project with its bundled runtime library.

The sample demonstrates `Array<int>` and `int[]` sharing the same array, including
mutation through aliases, indexed iteration, Iterable parameters, direct GetIterator,
query extensions and reflection. Nested `Array<Array<int>>` and `int[][]` are also
interchangeable. Expected output:

```text
42
2
42
8
50
42
1
42
1
System.Int32
1
9
```

Completion on either array spelling includes Length, GetIterator, Where, Select and
ToList. Other samples are in `../tools/samples`, including `application-orders.rvn`
for the Result/Option, file and application workflow. Keep your edits before copying
a different sample into Main.rvn.

## Installation and validation

- SDK: `/Users/robert/.raven/sdk/0.1.12-neoclr.11`
- Demo: `/Users/robert/.neoclr/experiments/generic-arrays-20260913/demo`
- VS Code profile: `/Users/robert/.neoclr/vscode/generic-arrays-20260913`
- Artifacts: `/Users/robert/.neoclr/builds/generic-arrays-toolchain-20260913`

The workspace explicitly selects the SDK and installed VSIX language server. The
normal SDK selection and older demos/profiles remain available; hashes of 1,074
existing Raven source files were unchanged by installation.

All seven packaged suites passed: 59 saved-project cases, 28 query checks,
application behavior, order workflow, native memory, direct NeoIL and 52 editor
checks. All 712 pristine payload hashes matched the bundle and archive. The installed
SDK reports .11, VS Code lists the .11 extension, the installed VSIX server passes
52 editor checks, and the exact configured Run task produces the output above.
These editor checks exercise the LSP protocol; they do not claim a full debugger.

[Build evidence](experiments/raven-target/generic-arrays-toolchain.json) records
revisions, artifact hashes, validation commands and installation paths. The tools
require .NET 11; validation used SDK 11.0.100-rc.1.26425.128 on macOS arm64. The
companion dependency notices are retained beside the artifacts.

## Boundaries

Array allocation still uses ordinary Raven array expressions; there is no new
`Array<T>(length)` constructor. The configured metadata shape supplies interface
contracts, while arbitrary class members are not projected. Mutable arrays remain
invariant; this does not add read-only array variance or select a broader collection
hierarchy. Native-pointer conversion gaps remain documented in the
[native-memory API](raven-native-buffer-api.md). See the
[generic array contract](generic-managed-arrays.md) for the runtime/compiler split.
