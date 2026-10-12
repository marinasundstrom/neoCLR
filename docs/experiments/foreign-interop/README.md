# Source foreign import validation (2026-10-12)

This development slice compiles the actual Raven interop attribute source into a
System.Runtime reference, compiles extern consumers with the neoCLR backend, writes
binary native metadata, rereads it and runs it against a Rust native fixture.
It uses the Raven `codex/source-object-metadata-resolution` line, separately from
Raven main. See [validation.json](validation.json) for base revisions, input hashes,
commands and ten successful cases, including the expected missing-export fault.
The report identifies working-tree implementations rather than claiming its base
commits contain the new implementation.

Build the matching runtime and compiler probe first:

```sh
cargo build --locked --bin neoclr
dotnet build /path/to/Raven/tools/NeoClrMetadataProbe/NeoClrMetadataProbe.csproj -p:NeoClrMetadataProject=/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj -m:1
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/path/to/Raven -m:1
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll --reference-core target/foreign-api/NeoCLR.CoreProbe.dll
python3 docs/experiments/foreign-interop/validate.py --raven /path/to/Raven --core target/foreign-api/NeoCLR.CoreProbe.dll --output target/foreign-validation --report docs/experiments/foreign-interop/validation.json
```

On this host the command-line-tools SDK 27 stubs were incompatible with the selected
linker. The successful harness run set `SDKROOT` to Xcode's MacOSX26.2.sdk; no global
toolchain settings changed. The host was macOS ARM64. The core above is an explicit
reference-only bootstrap, not an installed native class-library bundle.

Additional focused validation: all 11 Rust pinvoke tests, metadata author/readback
and rejection checks (`NeoCLR.Metadata.Experimental.Tests --foreign-imports`),
and seven existing Raven extern parser/semantic/.NET emission tests passed.
The compiler probe includes rejected declarations alongside the ten emitted cases.
API reference source/fingerprint validation passes. No website build was required.

Qualified behavior includes scalar calls, default export names, static class imports,
UTF-8/UTF-16 and mixed per-slot encodings, Bool8/Bool32 parameters and results,
System ABI on this host, and deferred missing-export failure. Rust tests additionally
cover empty strings, embedded NULs and incompatible descriptor rejection.
This does not qualify Win32 stdcall, other hosts, JIT/AOT imports, installed bundles,
returned-string ownership, buffer lengths/layouts, callbacks or custom marshallers.
