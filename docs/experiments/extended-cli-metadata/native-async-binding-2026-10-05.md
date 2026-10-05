# Native async declaration binding — 2026-10-05

Raven `77549bd30` closes the shared binding blocker from the [POC sample inventory](poc-sample-inventory-2026-10-05.md).
All five selected async/HTTP samples now pass binding against the unchanged, independently
emitted Numbers/Http libraries. In particular the HTTP client's propagation/conversion
errors disappear when its Task result identity is correctly recognized. All five still
fail the explicit native async state-machine emission guard and publish no assembly.
This is not an async execution claim or an increase in completed executable samples.

## Contract and boundaries

The driver accepts `--async-library Numbers` alongside the existing explicit core, native
references, seed and ownership manifest. Compiler API clients use
`MetadataImportOptions.WithAsyncAssemblyName(string?)` and can read `AsyncAssemblyName`.
The immutable selection affects Task<T> and AsyncTaskMethodBuilder<T> only in the selected
registered native assembly. Missing or malformed providers cannot fall back to bootstrap
CLI declarations. Artifact identity/conflict checks remain with the dependency catalog.

Compared with the existing .NET/CLI importer, the native importer was missing these
special classifications. The fix supplies them behind explicit NeoCLR ownership rather
than recognizing arbitrary names in unrelated assemblies. Raven continues to own async
return inference, awaiter pattern checks and result propagation. Native introspection
supplies metadata facts; emission does not reopen the importer. The .NET default is
unchanged. There is no new metadata format, instruction, Task implementation or runtime
suspension behavior. This selection does not yet validate every async builder protocol
member: the public generic reference-class shape is checked, with normal semantic
binding validating used awaiter operations.

## Validation

- Pre-change focused async baseline: 32 tests.
- Final .NET/option regressions: 33 tests, including existing pending/completed heap async
  behavior and immutable native provider selection.
- C# native probe: selected/unselected symbols, generic GetResult substitution, return
  and await binding, malformed interface/value-type declarations, missing/bootstrap
  providers, .NET denial, and no output publication at the native emission boundary.
- Five unchanged POC samples: binding succeeds, native emission rejects explicitly.
  [Commands, source/dependency hashes and diagnostics](native-async-binding-2026-10-05.json).

```sh
# After building Raven with the existing NeoClrMetadataProject property:
dotnet tools/NeoClrMetadataProbe/bin/Debug/net10.0/NeoClrMetadataProbe.dll \
  --native-async-symbols /path/to/IntrospectionCoreParams.dll /path/to/Numbers.dll
# In neoCLR:
python3 scripts/check-native-poc-samples.py \
  --compiler /path/to/rvnc.dll --core /path/to/IntrospectionCoreParams.dll \
  --seed /path/to/System.neox --ownership /path/to/ownership.json \
  --reference /path/to/Numbers.dll --reference /path/to/Http.dll \
  --async-library Numbers --case library-async --case library-async-default-queue \
  --case library-async-cancellation --case http-json-server --case http-json-client \
  --output /tmp/fresh-native-async-inventory
```

The inventory command reports blocked cases normally; it is not a passing acceptance
gate. Use the exact previously recorded bootstrap/library inputs or reproduce their
source HTTP gate. No independently useful .NET fix needs a main backport from this slice.

## Next bounded implementation

1. Resolve the synthesized state machine's required interfaces and builder protocol from
   the same native async provider, including exact parameter identities.
2. Collect synthesized owners, fields and callable bodies in the portable declaration
   pipeline and native adapter. Existing CallableSignature and syntax guards reject
   async methods; do not simply remove those guards before the generated graph exists.
3. Exercise an awaitless completed method, then a pending await retaining local state,
   then entry-point completion and cancellation; resume the five unchanged samples.

Reuse the existing heap AsyncLowerer. Keep exceptions limited to the selected target's
existing capabilities. Do not emulate await with blocking queue pumping or introduce
runtime suspension/green threads. Broader System/root replacement is not a prerequisite
for these explicit retained-seed POC cases.
