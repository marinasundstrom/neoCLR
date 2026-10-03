# Source-built union bootstrap (development)

This bounded native gate compiles the unchanged iteration contracts, Propagatable,
Option and Result into NeoCLR.Collections.dll. A second ordinary compiler invocation
imports only that artifact, then neoCLR verifies and executes the consumer. This is
not the complete dual-target library gate or the collection application gate.

Owners:

- Primitive semantic declarations: the explicit `--reference-storage-core` image.
  Its CLI bodies are never executed; it contains no source-owned union/collection types.
- Retained executable services: `union-seed.neoil`, module System. Its inventory is
  Void, Char, RuntimeTypeHandle, Object, String and Console. Native intrinsics implement
  string concatenation, type-handle display and console output.
- Source library: every declaration in `union-ownership.json`, consumed by the compiler
  as its ownership/iteration configuration. No seed copies of these declarations exist.

Object.ToString is a temporary target adapter: it asks the existing native type-handle
query for the display name directly instead of allocating the guest introspection facade.
That removes the Option/Result bootstrap cycle without a placeholder body. The runtime's
existing intrinsic boxed-primitive formatting and ordinary value overrides still dispatch.
Unlike a full .NET core library, this seed deliberately exposes only the services required
by this gate; unsupported core member references reject. Broader source-library ownership
and executable .NET adapters remain subsequent work.

From the neoCLR worktree, with an already built Raven driver and neoCLR runtime:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven -p:UseRavenCoreReference=false \
  -p:WarningLevel=0 -- --reference-storage-core /tmp/UnionCore.dll
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_unions.py \
  --compiler /absolute/path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --runtime target/release/neoclr --core /tmp/UnionCore.dll \
  --output /tmp/source-unions-fresh
```

The output directory must be fresh. Evidence records every command, source/artifact hash,
stdout and exit status. Expected output is `Option.Some(40)` then `Result.Error(7)`,
with exit 42. The consumer checks copy independence, false-output initialization, unit
residuals, error residuals, generic pattern matching and boxed virtual display. Missing
library input and duplicate source ownership in a seed must reject before publication.

The seven older native consumers use their own full CoreProbe/seed baseline; the small
core here intentionally does not supply their configured typeof facade.
