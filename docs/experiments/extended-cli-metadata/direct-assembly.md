# Direct neoil native assembly output

Feature-branch development, 2026-09-30, based on runtime `dab19089` with this producer
slice. `assemble --format neox` emits standalone schema 3 directly from the resolved
native Module. The Rust writer and independent .NET reader are separate implementations
of the same encoding. The binary writer/reader use no serialized JSON module
intermediate. The existing neoil parser still handles inline JSON metadata directives.
JSON baselines below are used only for independent test comparisons.

## Validation

- 21 focused Rust tests: CLI options/no-overwrite/no-output-on-invalid-body, generic
  dependency assembly/execution, codec limits and existing input/output paths.
- Four independent C# reader comparisons preserve every native metadata value: the
  full matching Raven System and OptionPositional, FloatingMath and ValueCopy apps.
- All three applications run with the directly assembled binary System and match their
  established output. The System image is 5,542,303 bytes and retains 417 types and
  4,090 functions. Binary assembly itself verifies the load set before output.

[Artifact hashes and results](direct-assembly.json) bind the tested producer and reader.
The source samples and baseline come from the prior schema-3 consumer run; the Raven
compiler/bridge mappings are unchanged. These tests do not establish native Raven
source emission, a CLI projection or native compiler symbol loading.

## Reproduce

Build the runtime and C# checks, then use the saved inputs from the
[schema-3 experiment](raven-sample-translation.md#follow-up-schema-3-closes-the-observed-transport-gaps):

```sh
cargo build --release --bin neoclr
dotnet build tools/metadata/NeoCLR.Metadata.Experimental.Tests
python3 docs/experiments/raven-target/collection_library.py target/System.neoil
python3 docs/experiments/raven-target/verify_direct_assembly.py \
  --runtime target/release/neoclr \
  --metadata-tests tools/metadata/NeoCLR.Metadata.Experimental.Tests/bin/Debug/net10.0/NeoCLR.Metadata.Experimental.Tests.dll \
  --system-source target/System.neoil \
  --baseline target/extended-cli-metadata/raven-library-profile3 \
  --output target/extended-cli-metadata/direct-native-assembler
```

Use fresh output paths. The baseline directory must include the three selected samples,
their JSON artifacts and System.json from the matching collection profile. The recorded
macOS release build uses the same command-local SDK 26.2 override documented for earlier
runs; no machine configuration changes are required by the format.

Normal use needs only neoil source and the runtime assembler:

```sh
neoclr assemble examples/hello.neoil hello.neox --format neox
neoclr run hello.neox
```

JSON remains the default and explicit `--format json` is supported. Native emission
is a supported-subset, whole-module assembler with verification and fixed budgets,
not complete CLI/ILAsm parity or a production stable-format guarantee.
