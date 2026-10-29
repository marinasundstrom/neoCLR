# Runtime source ownership

The current class library is written in Raven. Components follow:

```text
<runtime|extensions>/<component>/<src|test>/<project>/<namespace folders>
```

For example, `runtime/System.Runtime/src/System.Runtime/System.Runtime.rvnproj`
builds sources under `System/` and `neoCLR/`. IO APIs live in
`runtime/System.Runtime/src/System.Runtime/System/IO`. Core primitives and runtime
service declarations follow their own namespaces in that source project.

Optional application components live under sibling `extensions/Neo.*` and reference
the runtime; runtime projects do not reference extensions. A component groups one
or more source projects and zero or more test projects, without requiring a solution
file. See the [build instructions](class-library-projects.md).

Raven suites live under the component they exercise: core suites in System.Runtime,
JSON suites in System.Data, HTTP suites in System.Web, networking suites in
System.Networking, and DI/localization suites in their Neo components. The
[suite catalog](../scripts/runtime-test-projects.json) maps stable command-line
suite names to project files. Shared test-framework source lives under
`tools/testing/NeoCLR.Testing`.

## Supporting implementation and fixtures

- `native/` and `codegen/` contain the native runtime and Rust backend.
- `compiler-services/` contains RuntimeAwait and RuntimeGenerator templates used by
  current native builds and SDK packaging.
- `legacy/bootstrap/System.neoil`, `legacy/bootstrap/System/` and
  `legacy/bootstrap/neoCLR/` contain the historical embedded bootstrap manifest,
  class-library mirrors and service declarations.
- `legacy/raven/` contains historical CLI bridge fixtures, generated neoIL and
  validation receipts. The loose carrier snapshots in `legacy/` preserve the
  bootstrap ABI.

All historical library material is contained under `legacy/`. It is retained
because Cargo's embedded System module, low-level Rust tests and older bridge
preparation tools still consume it. It is not a second maintained class library;
new APIs belong in the Raven component projects. Do not hand-edit generated bridge
fragments to implement new behavior.

The 2026-10-29 relocation updates build, test and packaging consumers while
preserving expanded bootstrap/profile text. Historical receipts and published
records retain their original paths and hashes; they do not certify a fresh build.
The old `runtime/System.neoil`, `runtime/System/`, `runtime/neoCLR/` and
`runtime/raven/` locations have no compatibility aliases.

neoIL itself remains supported for assembler/verifier tests and isolated contract
probes. The native Raven source-core seed under System.Runtime is also intentional.
Removing the legacy fixtures entirely requires migrating their remaining consumers;
containment does not claim that migration has happened. See
[legacy ownership](legacy/README.md).
