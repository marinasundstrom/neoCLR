# Descriptive backtick discovery qualification — 2026-10-26

Author-directed post-release work, validated on macOS ARM64 with Raven
`codex/neoclr` `4981768ebcc346ddb78cb34fa2714d73934d6019`. The compiler was built
against this checkout's metadata project; all five native libraries were rebuilt,
including the PostgreSQL provider. The isolated development bundle retains the
Preview 18 runtime/JIT/AOT tools. No published bundle or release pin was changed.

## Implemented scope

- Source-linked internal spelling helper replaces the adapters' ASCII-only gate.
  Names are preserved in native metadata and manifests; generated references quote
  namespace components and the complete function name independently.
- Data-source lookup accepts exact local or qualified logical names and rejects
  missing/ambiguous matches. Generated strings preserve Unicode/emoji without JSON
  surrogate escapes. Inline argument identities retain their existing JSON spelling.
- Seven IP-address tests now carry their former descriptions in backtick names.
  Their test bodies are unchanged. The suite has its own entry point because the
  previously shared collections entry point references a collection-only comparer.
- Test discovery covers spaced/Unicode/emoji/dotted names, inline rows and a named
  source. Constructor/named description fixtures remain as compatibility checks.
  The benchmark discovery sample moves Sum's description into its name and covers
  inline and provider names. This is not a repository-wide rename.

## Validation

[Retained results](results.json) contain the exact outputs and selections:

- Seven IP-address tests pass in interpreter, JIT and AOT: 21 case executions.
  Reuse the successful IP portion of `target/backtick-names-20261026/tests-v3`;
  that combined run later failed on obsolete `0..<` syntax in the other fixture.
- Final discovery qualification passes the nine-case intentional-failure fixture
  and all exact-ID, Unicode filter, argument/source-row, category and invalid-input
  checks in all three modes. The intentional assertion failure must return exit 1.
  Final-artifact rediscovery reproduces the registry and manifest exactly.
- Seven benchmark selectors pass exact listing and one warmup/two checked operations
  in each mode (21 worker runs). Final-artifact rediscovery is identical.
- Missing source, invalid source return type and duplicate row are rejected.
- `scripts/build-api-docs.py --check` passes. The framework is manually documented
  outside RavenDoc's guest type selection; no public signature or snapshot changed.

The earlier discovery attempt also exposed JSON UTF-16 surrogate escapes in Raven
literals; emitting scalar text fixes it. An intermediate successful-execution run
was invalidated when the harness changed to add exact-ID coverage; only the final
unchanged-input discovery run is claimed as passing.

Reproduce with a matching unreleased development bundle:

```sh
python3 scripts/test-runtime-library.py --bundle <bundle> --output <fresh-tests> \
  --suite ip-address-values --suite discovery-contract \
  --mode interpreter --mode jit --mode native
python3 benchmarks/framework/validate_names.py --bundle <bundle> --output <fresh-benchmarks>
python3 scripts/validate-test-discovery.py --bundle <bundle> --output <fresh-rejections> \
  --case missing-source --case source-type --case duplicate-row
python3 scripts/build-api-docs.py --check
```

## Compatibility and limits

neoCLR already supports these names; this changes consumer source generation,
not runtime naming, compiler semantics, Runtime Contract or CLI bridge encoding.
Compared with .NET framework display-name properties, descriptive identifiers
avoid redundant naming but couple editorial changes to identity and sort order.
Keep optional Description overrides when independent display text is useful.

[IP-address old/new names](ip-address-renames.json) map names within
`NeoClr.RuntimeTests`; IDs still incorporate assembly/module/qualified-name lengths.
The [benchmark API notes](../../../api-docs/benchmarking.md#descriptive-identifiers-development)
record selector changes. Historical evidence is retained under its original names.

Backticks and line breaks inside names cannot be represented in Raven identifiers;
the adapters reject them without imposing a native metadata rule. Dotted namespace
interpretation and flattened-name collision rejection remain adapter limits.
Preview 18's compiler predates this syntax. Windows, broader suite migration and
performance comparisons are outside this focused qualification.
