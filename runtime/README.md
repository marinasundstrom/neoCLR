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
`tools/testing/NeoCLR.Testing`. `runtime/raven` retains compiler-service templates,
legacy bridge fixtures and historical validation receipts, not active API projects.

## Obsolete class-library mirrors

`System/**/*.neoil` and the `System.neoil` bootstrap manifest are **legacy
class-library mirrors**, not a second maintained implementation of the library.
Do not add new library types or methods there or copy new Raven APIs into them.
Some mirrors differ from today's Raven semantics; they are not API documentation.
They remain checked in because the embedded bootstrap and older tests still
consume them. Archive them once those consumers have been migrated explicitly.
Moving them now without moving their consumers would break existing test paths.

The older `raven/generated` bridge artifacts are also not authored source. Do not
hand-edit them to implement new behavior. Historical fixture refreshes must use
their recorded producer and retain provenance.

## neoIL remains useful

The neoIL language and low-level assembler tests are not deprecated. Examples,
interpreter/verifier fixtures and deliberately isolated runtime contract probes
can continue to use it. `neoCLR/Runtime/*.neoil` provides older bootstrap service
bindings; their use by fixtures is distinct from authoring the source library's
service declarations in `System.Runtime/src/System.Runtime/neoCLR/Runtime/*.rvn`.

New library work—including Slice and Memory—starts in Raven source and its
explicit compiler/runtime metadata contracts. Runtime support remains in Rust;
that does not make hand-authored neoIL a library implementation path.

## Archival gate

Before moving legacy mirrors to an archive, inventory references from build.rs,
manifest expansion, test include_str calls, examples and historical bridge tools.
Migrate active consumers to native Raven artifacts or purpose-specific test
fixtures, retain a reproducible historical bundle where needed, and verify those
consumers. Archiving mirrors is separate from deleting neoIL support or tests.

Development Character migration (2026-10-21): the retained neoIL fixtures now use
`System.Character` for canonical runtime identity. Their old CLI generation JSON
receipts remain historical records, not fresh build evidence. Production libraries
are rebuilt from the Raven projects; the native bundle records current source hashes.
