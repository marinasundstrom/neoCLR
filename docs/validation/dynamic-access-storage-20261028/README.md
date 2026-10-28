# Field/property member requirements — 2026-10-28

The [native consumer](../../experiments/dynamic-access/storage/Main.rvn) places its
DAM contract on an auto-property. The constructor stores `typeof(Message)` into its
backing field, and Main deserializes using the property. No retention file or
annotated constructor parameter supplies roots. Native execution exits 0 and prints
`retained`; source row 6 receives flags 513. No TypeStore/field/property warnings
remain. Existing JSON descriptor-flow DAM2001 and dynamic-dispatch DAM2004 remain.

- 30 focused dynamic-access tests pass, covering instance/static/inherited fields,
  record construction, unknown writes, property getter/setter contracts, computed
  getters, field-address warnings, implicit static initialization and structural
  tuple storage alongside previous parameter/return tests.
- `--storage-attributes` passes CLI/native roundtrip, reference projection and
  introspection for instance fields, static fields and properties.
- Five runtime libraries rebuild; native rendering, API fingerprint validation and
  four snapshot tests pass. The HTTP JSON TypeInfo APIs and callback-storage field
  are annotated with the same 513 contract as JsonSerializer.
- The previous return-only backend rejects storage annotations with DAM1001.

[Commands, observations and hashes](results.json) identify the tested artifacts.
Compiler source is Raven `a2b4f47ec` plus the neoCLR metadata-writer change, recorded
as `a2b4f47ec+storage-metadata-worktree`. Bundle: `target/dynamic-access-20261028/bundle-storage`;
backend: `storage-codegen`; executable: `storage-native-fixed/app` under that target
directory. Runtime Contract settings remain unchanged. No performance claim or
website deployment is made.

The first native attempt exposed a structural-tuple lookup bug (`unknown base type`).
The fix excludes structural storage from nominal field lookup, and the focused
regression and native rerun pass. Literal-field attributes, indirect writes,
arbitrary computed backing associations, generic/receiver annotations and additional
member categories are not claimed by this qualification.

```sh
cargo test -q --lib dynamic_access::tests
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -p:WarningLevel=0 -- --storage-attributes
python3 scripts/build-api-docs.py --check
python3 scripts/test_native_api_snapshot.py
```
