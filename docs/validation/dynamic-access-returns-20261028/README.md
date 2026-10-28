# TypeInfo return contract qualification — 2026-10-28

The author requested remaining annotation targets and all member categories. This
record qualifies the return layer only; fields/properties, generic parameters,
receivers and broader flags remain in progress.

## Result

- 23 focused Rust dynamic-access tests pass, including known factory returns,
  unknown producers, insufficient guarantees and interface/override mismatches.
- The isolated metadata return probe passes CLI execution/reflection, native
  roundtrip, CLI reference projection, immutable introspection and rejection of
  missing/zero/duplicate/wrong-table return identities.
- Raven's native attribute probe passes source emission/reimport, method/parameter
  separation, typed/named arguments and usage validation, now including returns.
- All five native libraries build with the updated compiler. Native API rendering,
  reference/source fingerprint checks and four snapshot tests pass.
- The [native factory consumer](../../experiments/dynamic-access/returns/Main.rvn)
  builds and exits 0 with `retained`. Its known return infers flags 513 for source
  row 6 (Message); no explicit policy is supplied. Existing JSON descriptor-flow
  DAM2001 and dynamic-dispatch DAM2004 warnings remain. No return/consumer-flow
  warning is present.
- The previous backend rejects the same artifact's new `return_token` field;
  matching compiler/metadata/backend versions are required. This is a compatibility
  control, not a before/after performance comparison.

[Results, commands and artifact hashes](results.json) retain the observations.
Compiler source is Raven codex/neoclr at commit `a2b4f47ec` (built before committing, recorded as
`1a8266166+return-annotations-worktree` in the bundle);
the native bundle is `target/dynamic-access-20261028/bundle-return`, backend is
`return-codegen`, and the executable is `returns-native/app` under that directory.
Runtime support comes from the restored Preview 18 bundle; no runtime execution
engine change is required. No performance claim or website deployment is made.

## Reproduction

```sh
cargo test -q --lib dynamic_access::tests
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -p:WarningLevel=0 -- --return-attributes
python3 scripts/build-api-docs.py --check
python3 scripts/test_native_api_snapshot.py
```

Raven probe (from Raven):

```sh
dotnet run --project tools/NeoClrMetadataProbe -p:WarningLevel=0 -p:NeoClrMetadataProject=/Users/robert/Projects/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj -- --native-attributes /Users/robert/Projects/neoclr/api-docs/reference/NeoCLR.CoreProbe.dll
```

The pre-change aggregate `--member-attributes` probe fails while loading its own
large test assembly (`too many field definitions`). It is not counted as passing;
the isolated return probe avoids that unrelated fixture-size limit. The first
compiler-driver build encountered the existing Raven.Core bootstrap cycle; building
with `UseRavenCoreReference=false` succeeds. No compiler/backend behavior was
changed to work around either test setup issue.
