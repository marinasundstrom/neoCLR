# Raven CLI bridge

## Self integration on nominal main (2026-09-30)

Self is integrated independently of `codex/structural-types`. Raven projects must
select `RavenTargetPlatform=NeoCLR` and configure the fieldless
`System.Runtime.CompilerServices.Self` marker in `NeoCLR.CoreProbe`. The shared
props do this. Raven's .NET target rejects that configuration with RAVT003.

The importer replaces the marker with native Self and admits bounded Number and
Clonable generic dispatch, including borrowed receivers. See
[self contracts](self-types.md) and [the consumer](experiments/native-self/README.md)
for inheritance and current importer restrictions. A real neoCLR metadata layer
must replace marker recognition with native contract identity/signatures; it must
preserve conformance ownership, substitution and unsupported-use diagnostics.
Compiler binding/diagnostics belong to Raven; CLI translation is owned by
`docs/experiments/raven-target` and native validation/dispatch by `src/self_types.rs`.

The nominal delegate ABI still uses Func with an inhabited Void result. This is
required even without structural Functions; Raven retains that bridge encoding.
Structural identity, Function introspection and structural assignability remain on
feature branches in both repositories. Do not use a Function-branch bundle as
acceptance evidence for this integration. General core, unit, tuple, iteration,
propagation and typeof encodings remain described in Raven's
`docs/compiler/neoclr-cli-bridge.md`; this CLI bridge is not a native metadata loader.

## Native metadata exploration (2026-09-30)

The [extended CLI metadata design](design/extended-cli-metadata.md) starts on
`codex/extended-cli-metadata` from main. It includes structural type identity,
synthesized members, explicit capabilities and a staged Raven importer/emitter path.
The [standalone codec and bounded PE probe](experiments/extended-cli-metadata/README.md)
now transport #Neo for inspection. .NET and Cecil read conventional metadata unchanged,
but Cecil rewriting strips the stream. Do not route native metadata through an ordinary
Cecil rewrite without preservation/remapping support. Current marker/carrier behavior
and Runtime Contract configuration remain as described above. No native runtime-loaded
artifact or Raven compiler bundle is validated by this experiment.

The [reader/writer architecture](design/extended-cli-metadata.md#reader-and-writer-support-on-net-and-neoclr)
plans a .NET-hosted metadata library for Raven and corresponding support on neoCLR,
including a guest-accessible library. Format codecs, semantic resolution and Raven
symbol/emission adapters remain separate; the current bridge is not that library.

The [provisional recognition contract](experiments/extended-cli-metadata/README.md#marked-artifact-recognition-2026-09-30)
requires an explicit expected-extended input profile, a metadata-root marker and a
matching stream digest. Future Raven native-metadata loading must reject failures
without falling back to carrier/ordinary CLI interpretation; ordinary .NET targeting
remains separate. This is a Python probe contract, not an implemented Raven loader.

## Independent metadata consumer, stage 1 (2026-09-30)

Raven's `codex/metadata-consumer` at `7e18edb66` adds an opt-in
`tools/NeoClrMetadataProbe` consumer of the independent metadata library. The frontend
uses the existing .NET provider and default runtime contract to bind primitive Int32
source and an API-produced PE library. A compiler-side public-operations adapter emits
the application as native format-5 JSON through the metadata API. The dependency is
also emitted natively by that API. neoCLR loads/verifies both and returns 42.
The application does not go through the existing CLI import bridge.

This is a staged bootstrap, not a new Runtime Contract option or a completed native
ICompilationEmitter/ISemanticDataLoader. The PE dependency and host-core identity are
temporary inputs to the .NET frontend. Native top-level functions have no artificial
user type. Only required Int32 values, returns, primitive unchecked/unlifted arithmetic,
local calls and the explicit static dependency are supported. Other constructs and
unresolved methods are rejected; attributes, defaults, debug data and structural
contracts are not silently advertised as preserved. Parameter names/source mappings
are not yet emitted by the native subset.

Ownership remains Raven symbols/operations/adapters -> separate metadata model/format
library -> native runtime loader/verifier/VM. The next stages replace the .NET metadata
bootstrap with native symbol loading, integrate target diagnostics/configuration, and
expand ordinary format support using this executable consumer. Shared operations and
missing-Param-row fixes are general Raven corrections, already fast-forwarded to local
main as `1ea0ca263` and `d7040e21d`; the consumer remains experimental.

Validation: the .NET 10 consumer runs the native output to 42 and rejects unsupported
division plus an unresolved method. Raven's .NET 11 operations/default-parameter filter
passes 87 tests; two invocation and one missing-parameter-row regression failed before
the fixes. Compiler builds pass for .NET 10/11. See
[hash evidence](experiments/extended-cli-metadata/raven-compiler-validation.json) and
[reproduction](experiments/extended-cli-metadata/README.md#raven-compiler-consumer-stage-1).
No full native target, structural execution, NanoFramework or release is claimed.
