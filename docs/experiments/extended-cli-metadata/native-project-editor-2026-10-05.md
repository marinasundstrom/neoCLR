# Native project/editor semantic gate — 2026-10-05

This slice connects native metadata to evaluated Raven projects and the existing
language-server semantic pipeline. It does not complete VS Code build/run qualification.

## Reproduce

Use Raven's `codex/metadata-consumer` integration line and the matching NeoCLR
`codex/extended-cli-metadata` metadata project. The machine-readable companion records
revisions and artifact hashes. Set these shell variables to absolute paths:

- `RAVEN`: Raven checkout.
- `METADATA`: `tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj`.
- `CORE`: the explicit CLI primitive bootstrap.
- `SEED`: the retained native System seed.
- `OUT`: a fresh output directory.

```sh
dotnet run --project "$RAVEN/tools/NeoClrMetadataProbe" \
  -p:NeoClrMetadataProject="$METADATA" -- \
  --native-project "$CORE" "$SEED" "$OUT"
dotnet build "$RAVEN/src/Raven.LanguageServer/Raven.LanguageServer.csproj" \
  -f net10.0 -p:UseRavenCoreReference=false -p:NeoClrMetadataProject="$METADATA"
python3 scripts/check-native-editor.py \
  "$RAVEN/src/Raven.LanguageServer/bin/Debug/net10.0/Raven.LanguageServer.dll" \
  "$OUT/project" "$OUT/lsp-evidence.json"
```

The C# fixture authors a native library through the metadata builder, then evaluates
an SDK project containing only a consumer source and an artifact HintPath. No library
source is included in the consumer. It checks that there is one native reference plus
one primitive bootstrap, no generated host source, and no binding error. It rejects an
absent adapter, absent library, wrong target/core selection and unsupported project
references. Failed loads publish no partial workspace project. Catalog checks also
cover snapshot replacement, native emission, bad artifacts and identity conflicts.

The stdio check requests hover on the imported `Updated` method, completes its owner
and then edits the document to call a nonexistent `Missing` member. It requires a
semantic error notification for that member. The actual protocol transcript and SHA256
hashes are recorded by the script; this proves native semantic queries, not reference
file-watcher invalidation. The fixture disables TypeOf runtime mappings because that
service is outside this small library's contract.

## Scope and controls

The focused `ProjectFileCompilationOptionTests` checks cover the established .NET and
CLI-bridge project option paths. No runtime, instruction, metadata encoding or guest API
changes are needed in this slice. Prior native sample execution evidence is reused;
this editor fixture itself does not execute the program.

Remaining: native reference refresh without CLI substitution; imported declaration
navigation; source-built System ownership/async setup through project configuration;
shared project build/run and actual VS Code acceptance. Missing-project presentation
in LSP also needs a dedicated acceptance test beyond the host's transactional failure
checks. A larger source-library editor gate remains necessary before release.

Validated Raven revision: `d1f68ea126cff3ea5f289217004ed5078b763026`. NeoCLR metadata/runtime base:
`8f41114bbf0320e7748660329847eef825a4fd15`. Native project probe and stdio editor check passed;
twelve focused project tests passed after the final evaluator change.
[Machine-readable evidence](native-project-editor-2026-10-05.json).
