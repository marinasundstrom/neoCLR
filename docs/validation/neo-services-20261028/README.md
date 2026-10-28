# Neo services and component layout qualification — 2026-10-28

## Implemented boundary

`Neo.DependencyInjection` and `Neo.Localization` are separate projects and native
assemblies. Source/test projects follow
`<runtime|extensions>/<component>/<src|test>/<project>/<namespace folders>`.
Raven suites move under their owning runtime or extension component; shared test
framework source is in tools/testing/NeoCLR.Testing. Stable suite selectors resolve
through scripts/runtime-test-projects.json. No solution files were added.

The metadata check verifies that Runtime/Data no longer own either service family,
that the new assemblies own their Neo types, and that old System service identities
are absent. Runtime projects do not reference extension projects. Localization
retains its bounded flat JSON parser through the documented System.Data friend
exception; no parser, culture, DI lifetime or error semantics change is intended.
See [the design and migration](../../application-services.md).

## Evidence

- Seven libraries build with Raven `91c1edca8`: Runtime, Data, Networking, Web,
  PostgreSql, Neo.DependencyInjection and Neo.Localization. The working-tree build
  records source paths, hashes and compiler/runtime binary hashes in
  [compressed build evidence](library-build.json.gz).
- All 21 DI tests and 10 localization tests pass in interpreter and native modes
  (62 executions), including the website DI sample, open generic registration,
  typed localization, reflection retention and the cross-assembly parser call.
  [Service report](service-tests.json.gz).
- Representative relocated suites pass in the interpreter: collection construction
  10, JSON DOM 21, HTTP methods 9 and IP addresses 7 (47 total).
  [Component report](component-tests.json.gz).
- Three layout tests verify source ownership, dependency direction and test-project
  source/reference resolution. Four snapshot tests, 26 website tests and two
  interface-naming tests pass. Python syntax was checked for 97 affected scripts.
- Both API snapshots are refreshed from matching producers. Native source links,
  Neo namespace routes and declaring-assembly labels pass. The complete website
  validates 6,185 pages, with 131 previously documented legacy pages retained.
  No website deployment was performed.
- [Native metadata boundaries](assembly-boundaries.json) and the
  [qualification summary](qualification.json) retain artifact identities.

## Reproduction

Build libraries with scripts/build-native-class-library.py using the matching
compiler/runtime, --with-postgresql and a fresh --output. Prepare a matching local
bundle with scripts/prepare-jit-bundle.py using those libraries and compiler.
The qualification bundle is a local test harness, not a published SDK package.
Run scripts/test-runtime-library.py with --suite dependency-injection --suite
localization --mode interpreter --mode native; separately run collection-construction,
json-dom, http-method-values and ip-address-values with --mode interpreter.
Run the test_class_library_layout.py, test_native_api_snapshot.py,
test_build_website.py and test_raven_interface_names.py unittest modules, then
scripts/build-api-docs.py --check and scripts/build-website.py.

The initial shared collection Main exposed an unrelated comparer dependency; moved
suites now use the shared generic framework entry point or their own entry point.
An exploratory DI JIT run executed all 21 cases with exit code zero, but the strict
runner rejected emitted DAM2002/DAM2004 retention warnings. This slice claims
interpreter/native qualification, not a clean JIT gate, a full test matrix or
performance improvement. Historical evidence paths remain unchanged.
