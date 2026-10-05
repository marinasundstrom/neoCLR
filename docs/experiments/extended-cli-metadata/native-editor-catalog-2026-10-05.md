# Native editor prerequisite: shared reference snapshots

Date: 2026-10-05. Raven implementation: `42b744686` on `codex/metadata-consumer`;
neoCLR evidence base: `42d7a4b2` on `codex/extended-cli-metadata`. The editor release gate remains open. Inspection showed that the
MSBuild workspace loads references as ordinary CLI metadata and the language server
does not yet select the native adapter. The native driver separately owned bootstrap,
seed and native-reference loading. Repeating that setup inside LSP would risk divergent
assembly snapshots and dependency choices.

Raven now exposes `NeoClrReferenceCatalog` in its native adapter and the driver consumes
it. The catalog reads the core once, derives identity from those bytes, and shares the
same semantic reference in the retained-seed emission binding. Each selected native
reference is read directly; source ownership checks and exact artifact identity remain
explicit. Like ordinary .NET metadata references, old compilations keep stable snapshots;
replacement requires a new catalog/compilation. There is no automatic file watching,
dependency search, execution or CLI fallback.

## Validation

C# probe:

```sh
dotnet run --project tools/NeoClrMetadataProbe -p:WarningLevel=0 \
  -p:NeoClrMetadataProject=<metadata-project> -- \
  --reference-catalog <primitive-core> <System-seed> <output-directory>
```

It authors a library, imports it, replaces it at the same path and checks old/new
semantic snapshots. It emits a consumer against the replacement without library sources.
That consumer executes on neoCLR with exit 42 and empty stdout/stderr. Missing semantic
dependencies produce diagnostics; missing files, duplicate/conflicting identities,
malformed images, invalid seeds and duplicate seed ownership are rejected.

The ordinary native driver still compiles and runs unchanged inheritance and collection
samples with exact output. A malformed native reference exits 1 without publishing an
assembly. The ordinary .NET driver compiles and runs the unchanged inheritance source,
producing `7\n42\n` with exit 0. [Execution records](native-editor-catalog-2026-10-05.json)
include input hashes and the tested worktree base revisions.

## Next bounded work

1. Define native project properties and evaluated dependency selection, keeping the
   shared workspace target-neutral and the NeoCLR adapter optional.
2. Connect project loading and rebuilding to this catalog and the existing native
   backend. Reuse the driver runtime-contract/ownership configuration.
3. Exercise the real stdio LSP for diagnostics, completion, hover and navigation into
   source-absent libraries. Rebuild semantic state after reference replacement and
   report dependency failures as project diagnostics.
4. Connect VS Code build/run to the same evaluated configuration; validate a .NET
   editor control and publish matching setup/download instructions at release.

No editor-facing configuration or website capability claim is added in this slice.
The host API is documented in Raven's compiler docs and linked from the experimental
metadata reference; the guest API snapshot is unchanged.
