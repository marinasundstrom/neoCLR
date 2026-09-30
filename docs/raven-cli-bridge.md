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
