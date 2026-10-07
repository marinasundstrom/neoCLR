# Native project graph gate — 2026-10-07

Development evidence; this is not full bootstrap or live VS Code acceptance.

Raven commit `3892b113a` implements this slice. The tested compiler binaries were built
from that source before committing; their hashes are retained in the record.

Raven's native project command now evaluates and builds ProjectReference dependencies
before the consumer. Its workspace loader reads prebuilt native artifacts rather than
loading dependency sources or substituting CLI projections. The shared project provider
owns output placement and identity validation; importer and emitter remain separate.

## Reproduce

```sh
python3 scripts/verify-native-project-graph.py \
  --compiler /tmp/native-graph-final-compiler1007/rvnc.dll \
  --compiler-revision 0f85f53b8+project-graph \
  --core /tmp/failure1006b/Core.dll \
  --runtime-library-directory /tmp/array-runtime1007/runtime-owned \
  --runtime target/release/neoclr \
  --output /tmp/native-project-graph-fresh
```

The harness requires a fresh output directory and records commands, exit status,
stdout/stderr and artifact/input hashes in evidence.json. The attached
[record](native-project-graph-2026-10-07.json) retains the actual validation run.
Primitive Core, retained seed and source-built Runtime are explicit inputs.

## Observed

- Base, Left, Right and App build in dependency order. Left and Right share Base;
  it is built once. Only Base explicitly references the source-built Runtime;
  transitive artifacts supply the complete native catalog to consumers.
- Left constructs a generic Box, Right mutates it and App observes the mutation
  through its original alias. Runtime stdout is `Native project graph passed`, exit 0.
- A cycle rejects before building, preserving all prior output bytes.
- A broken dependency fails binding and preserves all prior outputs in this case.
- C# workspace checks pass for prebuilt imports, missing projects/outputs, wrong assembly names,
  duplicate explicit references, cycles, incompatible formats and watched native artifacts.

All 67 ordinary MsBuildProjectSystemServiceTests pass (0 failures) with built dependencies
and `--no-restore -p:BuildProjectReferences=false`. The initial test build was stopped
during unrelated macro-library compilation; this focused run completed.

Unlike ordinary .NET project loading, native workspace loading requires prebuilt
artifacts. It never builds during editor load. The CLI host orchestrates compilation;
metadata remains the import boundary. Outputs use bin/neoclr/AssemblyName.dll. The
configuration is explicit per project; package/framework resolution is unsupported.
Publication is per assembly, not an all-project transaction: later failures may leave
already successful dependency builds. Incremental builds and configuration-specific
output directories are future work.

Next: convert the audited Runtime/Data/Networking/Web source groups to checked-in
native projects, define Platform service ownership, collect shipping artifacts, then
qualify live editor workflows. Website/API surfaces are unchanged by this tooling slice.
