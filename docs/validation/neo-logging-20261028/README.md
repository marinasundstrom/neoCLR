# Neo.Logging and generic interface qualification — 2026-10-28

Neo.Logging is a separate optional component and assembly with typed state/formatter
contracts, typed scopes, factory/provider abstractions, a default category factory,
console/null implementations and DI registration. The author explicitly selected
generic interface support over an object-state workaround. Output-folder changes
were deferred. See [design and gaps](../../logging.md).

## Toolchain and evidence

- Raven `codex/neoclr` commit **9724897ce** adds native capability admission and
  generic virtual call emission; Raven main was not merged or modified. The
  [native source probe](raven-generic-interface.json) verifies generic method
  forwarding through a constructed/inherited interface in both source orders.
- The [library build](library-build.json.gz) records source and producer hashes
  for Runtime, Data, Networking, Web, PostgreSql, DI, Localization and Logging.
  The bundle is a local qualification harness, not a published SDK release.
- [Five focused runtime tests](generic-interface-tests.txt) cover generic dispatch,
  independent instantiations, generic owner/inherited relationships, malformed
  arity and unsupported static/virtual/constrained categories. Existing nominal,
  generic method, variance and explicit-interface suites add 41 passing tests.
- [Native specialization](specialization-tests.txt): 28 pass, one pre-existing
  ignored test. Both value-state dispatch and different method arguments with
  identical erased signatures are covered. Private dispatch suffixes preserve
  original reflection/display identities.
- [Metadata writer test](metadata-test.txt) executes a generic interface call on
  .NET and checks the generic abstract declaration. The ordinary Raven host-CLI
  generic implicit implementation probe separately exposed a TypeLoadException;
  that general-main candidate remains open and is not counted as a passing target.
- Eleven logging tests pass in interpreter and native execution. The tests cover
  thresholds and lazy formatting, structured state/error forwarding, ordinal
  category reuse and late providers, scopes, duplicate-registration ownership,
  disposal, UTF-8 console output and open-generic DI activation. Initial
  [two-mode evidence](logging-tests.json.gz) and the subsequent
  [native dispatch-name review run](logging-native-review-tests.json.gz) retain
  exact development tool hashes; [final pinned-bundle evidence](qualified-logging-tests.json.gz) confirms all 22 executions pass.
- API reference and website cover all public logging types and their members.
  Layout (3), native snapshot (4), interface names (2), website script (26) checks
  pass. The website validates 6,329 pages; existing unrelated missing-description
  notices remain. Publication was not performed.

## Reproduction

Build with scripts/build-native-class-library.py using compiler commit 9724897ce,
--with-postgresql and a fresh output. Prepare the matching local harness with
scripts/prepare-jit-bundle.py. Run scripts/test-runtime-library.py --suite logging
--mode interpreter --mode native with that bundle and the matching runtime/codegen.
The tests are in extensions/Neo.Logging/test/Neo.Logging.Tests.

Run cargo test --test generic_interfaces and the nominal_interfaces,
generic_methods, interface_variance and explicit_interfaces suites. Run cargo test
-p neoclr-codegen --lib specialization::tests. Run the metadata test project with
--generic-interfaces and Raven's NeoClrMetadataProbe with --generic-interface-runtime.
Use the native library audit, update-native-api-snapshot.py, build-api-docs.py --check
and build-website.py for the matching documentation snapshots.

No JIT or platform matrix is claimed. Configuration, concurrent mutation, ambient
async scopes, template parsing and broader generic virtual-method categories remain
outside this foundation. There are no performance claims.
