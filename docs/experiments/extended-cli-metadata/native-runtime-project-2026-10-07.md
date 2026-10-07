# Native Runtime project gate — 2026-10-07

Raven `eb5744ba4` exposes the existing source Object-root contract in projects.
NeoCLR now has a checked-in native Runtime project and matching ownership manifest.
The final compiler includes shared-line `819d3780f`; exact binary/source/input hashes
and command output are preserved in the [record](native-runtime-project-2026-10-07.json).
The compiler's command label names the preceding graph slice plus local changes;
it is not a claim that the unmodified parent revision implements this feature.

## Outcome

- The native System.Runtime project compiles 175 audited sources through `rvnc neoclr
  --project`. Data, Networking and Web source directories are excluded.
- The retained seed is finalized against the exact emitted native Runtime artifact.
- Unchanged application-order-collections compiles separately with an artifact Reference,
  without Runtime sources in the consumer. Runtime execution matches the checked-in
  expected output exactly and exits 0, including its mutation, callback and identity checks.
- C# native project contracts pass for source-root selection, explicit async-owner
  preservation, invalid boolean/executable/conflicting imported-root rejection, and the
  existing graph/import checks. The preceding slice's 67 ordinary project-system checks
  are reused; this slice changes only the native provider and its tests.

## Reproduce

```sh
python3 scripts/verify-native-runtime-project.py \
  --compiler /tmp/native-runtime-project-final-compiler1007/rvnc.dll \
  --compiler-revision eb5744ba4 \
  --core /tmp/failure1006b/Core.dll \
  --bootstrap-directory /tmp/array-runtime1007/runtime-owned \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Debug/net10.0/NeoCLR.Metadata.Translate.dll \
  --runtime target/release/neoclr \
  --output /tmp/native-runtime-project-fresh
```

The output directory must be fresh. The harness records the two MSBuild environment
properties supplying Core and compile-time seed, all commands and dependency hashes.
It builds the checked-in project; it does not regenerate a substitute source-list project.
See [project instructions](../../../runtime/raven/projects/README.md) for direct commands.

## Remaining boundaries

This is an explicit-bootstrap gate, not a bootstrap-free release. The primitive Core,
compile-time retained seed and retained JSON model remain required inputs. Finalization
binds that model to the new Runtime artifact; no application/library reference is
converted to .NET metadata. The legacy CLI bridge project remains separate.

Higher-level Data/Networking/Web projects, Platform service extraction, automatic
seed-stage orchestration, shipping artifact collection and live editor acceptance remain
open. The source-root property changes no public Runtime API or metadata encoding, so
API snapshots and website rendering are unaffected. See the roadmap for release gates.
