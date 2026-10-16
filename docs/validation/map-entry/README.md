# Mutable collection helper qualification — 2026-10-16

This bounded slice implements `MutableMap<K,V>.Entry(key)` and
`MutableSet<T>.GetOrInsert(value)` as ordinary library extensions. Concrete types
retain their existing names. GetItem/list helpers are deferred. See the
[contract and research](../../collection-entry-design.md) and
[public member reference](../../../api-docs/collection-entries.md).

## Inputs and reproduction

Raven.Core main commit: `513b24eb6` (main-based development commit `aa04eb38e`).
Integrated native-line library carry: `afd5fe6e3` on `codex/neoclr`, with all 13
focused .NET 10 checks passing on that branch. Helper sources match Raven main.
The initial carry was `e7ba9f6ad` on `codex/collection-entry-native`,
based on `cb83fb9dd`. The shared native checkout had concurrent compiler edits, so
this carry was developed and validated independently rather than merging branches
or including those changes. After the author identified the incomplete integration,
the carry was applied to codex/neoclr above its checked-expression commit, preserving
that work. No compiler semantic change is part of this slice.

The native five-library bundle is
`target/collection-entry-libraries-final-20261016`, compiled with Raven `cb83fb9dd`.
The final consumer compiler is an isolated `e7ba9f6ad` build copied to
`target/collection-entry-compiler-final-20261016`. The commits differ only in the
Raven.Core library, its tests/docs and changelog; the native compiler implementation
is unchanged. Result manifests retain exact binary/library hashes.

```sh
python3 scripts/build-native-class-library.py \
  --compiler /path/to/native/rvnc.dll \
  --core target/sliceable-libraries-20261015-final/Core.dll \
  --bootstrap-directory target/tuple-info-bootstrap \
  --translator tools/metadata/NeoCLR.Metadata.Translate/bin/Release/net10.0/NeoCLR.Metadata.Translate.dll \
  --output /tmp/collection-entry-libraries --compiler-revision cb83fb9dd \
  --with-postgresql
python3 docs/validation/map-entry/raven/run.py \
  --compiler /path/to/native/rvnc.dll --bundle /tmp/collection-entry-libraries \
  --output /tmp/collection-entry-success
python3 docs/validation/map-entry/raven/run.py \
  --compiler /path/to/native/rvnc.dll --bundle /tmp/collection-entry-libraries \
  --output /tmp/collection-entry-fault --case target-fault
```

The runner requires the existing macOS ARM64 interpreter/codegen binaries, Xcode
host compiler and Rust toolchain. It links the existing UTF-8/string-comparer host
bindings, sanitizer-enabled AOT host and managed JIT host. It records commands,
exit codes, output/log hashes, compiler binaries, runtime, codegen and host inputs.
This is not Windows, .NET Framework or NanoFramework qualification.

## Coverage

The native [success consumer](raven/Main.rvn) checks interface extension resolution,
absence/presence, eager and lazy insertions, callback invocation counts, Set and
transformation chaining, retained handles after growth and replacement, nested
callbacks and write precedence, case-insensitive key policy, and canonical set
reference identity through insertion/removal/slot reuse/clear. The
[fault consumer](raven/TargetFault.rvn) checks propagation of a terminal factory
Fault in each execution mode.

Raven.Core's focused tests also exercise removal/reinsertion of map keys, original
exception identity, side effects surviving callback failure, reference payload
mutation, present null versus absence, dictionary/SortedDictionary behavior and
HashSet/SortedSet comparison policies. The compiled .NET 10 sample returns 42.

## Documentation validation

The matching reference scaffold is regenerated from CollectionDeclarations and all
new public types are selected in `api-docs/types.json`. XML comments and native
sidecars cover each member. The native documentation audit at
`target/collection-entry-docs-20261016` passes, and rendered pages show native
Function callback signatures and the correct declaring System.Runtime library.
The checked-in native snapshot includes the three new helper types and
MutableSet/HashSet.Find. Reference fingerprint validation and four native snapshot
tests pass. No new API is excluded. No standalone website build or publication was
performed for this change.

## Development findings

The first native consumer used a mutable captured integer counter; that existing
capture form is not admitted by the native compiler. The final consumer uses an
immutable reference to a mutable Counter object. Explicit Object.ReferenceEquals
checks stored object identity. No new language workaround is introduced into the
library itself. The initial AOT harness lacked string-comparer host bindings; using
the existing text host profile closes that harness configuration issue.

The old copied runner expected compiler DLL filenames absent from the active build.
Execution succeeded, but evidence finalization failed; the final runner fingerprints
actual compiler assemblies. A shared compiler rebuild also temporarily removed native
emission support. Final qualification therefore uses an isolated compiler copy.
The fresh full build hit an MSBuild worker failure; a targeted single-worker build
succeeded. These are retained as development findings, not passing result manifests.

## Recorded results

- [Native success](native-success.json): interpreter exit 42, sanitizer-enabled AOT
  exit 42 and managed JIT guest exit 42.
- [Native callback failure](native-fault.json): expected UserFault text, interpreter
  exit 1 and AOT/JIT host fault exit 2.
- Raven main focused .NET 10 suite: 13 passed (8 map, 3 set, 2 existing lookup).
  The isolated neoCLR-line carry also passes all 13 .NET 10 checks.
  The compiled Raven sample exits 42.
- Native documentation rendering passes; four snapshot tests and the matching
  reference fingerprint check pass. No full platform matrix is claimed.
