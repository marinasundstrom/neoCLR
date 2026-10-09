# Native POC bundle

## Next release qualification (2026-10-09)

The author has requested Windows x64 and macOS ARM64 packages for the next POC.
Neither target is certified by older reports. Rebuild the matching compiler,
runtime, libraries and editor; run the extracted sample and host checks on each.
ARM64 AOT experiments do not imply Windows x64 native-code generation support.

The current retained seed adds the ordinal String replacement service after
Preview 12. Its freshly assembled SHA256 is
`0d44005b8fb48a4655fef1cbb33b5e37987d9a06a8dc8ecbbeb3b1622f0137fd`.
The release validator checks this current pin; Preview 12's historical pin below
is unchanged. The seed remains an explicit bootstrap input, and full native-only
core bootstrap remains outside this bounded release qualification.

This is the development native release path. It does not invoke the CLI translation
bridge. Raven libraries are built into native metadata first, then distributed with
the matching native-enabled SDK, VSIX and runtime. The primitive CLI core and retained
native runtime seed remain explicit bootstrap inputs; full System source ownership
is still separate work. The bundle does not claim qualification on other hosts.

## Build and stage

First use `scripts/build-native-poc-libraries.py` and the checked-in ownership profile
as described in the [native source gate](experiments/extended-cli-metadata/native-source-release-2026-10-05.md).
Use the same SDK compiler, primitive core and retained seed when staging:

```sh
python3 scripts/package-native-poc.py \
  --sdk "$SDK" --vsix "$VSIX" --libraries "$LIBRARIES" \
  --core "$CORE" --seed "$SEED" --runtime "$RUNTIME" \
  --runtime-revision "$RUNTIME_REVISION" --output "$CANDIDATE"
```

The output directory must be new. The command checks library bytes against the build
report, compiler/core/seed identity by SHA256, current selected sources against their
build hashes, and canonical ownership before staging. It records supplied runtime and
compiler revision declarations separately from file hashes; a declared revision alone
is not proof of binary origin. Runtime compilation remains a separate explicit step.
The manifest records whether the packaging checkout was dirty.

The resulting `neoclr-native-poc.tar.gz` is a local candidate with no inferred release
version. No tag, release upload or website deployment occurs. It includes tooling,
source-built libraries, bootstrap files, authored API XML, preserved notices, consumer
samples, verification tools, and relative VS Code projects/tasks. Consumer projects do
not contain the library sources. Shared XML supplies descriptions for matching IDs;
it does not imply complete native API documentation coverage.

## Extract and qualify

Extract into a new location, including a path with spaces when checking portability.
From the extracted `neoclr-native-poc` directory:

```sh
python3 tools/verify-native-bundle.py --report ../acceptance.json
```

This requires Python 3 and the .NET SDK/runtime required by the packaged compiler
(current candidate: net11 host and net10 project reference packs). No repository
checkout or global Raven installation is used by those commands. The verifier checks
all manifest file hashes before invoking the bundled compiler/runtime, compiles five
separate projects, executes collections/identity, Tasks/await and JSON mapping, then
runs actual HTTP/JSON client/server checks with exact result assertions. A changed
file fails before compilation. It writes a failure report and exits nonzero on failure;
a launch or compilation success alone is not the gate.

For IDE use, install `editor/raven-vscode.vsix`, open one individual `samples` folder,
and run the `neoCLR: Run` task. The bundled server comes from the installed extension;
project reference and task paths are relative to the extracted installation. Do not
set a development-checkout language-server override for installation acceptance.

Hash verification establishes consistency with the manifest, not a signed trust chain.
This tooling supplements the still-failing legacy source/archive snapshot audit; it
does not rewrite that audit's result or assert that legacy bootstrap generation works.
The provenance procedure below closes the retained-input reproduction gap.
Publication remains separate from local packaging and does not expand the translation bridge.

## Preview 12 bootstrap provenance

The retained seed is now pinned at `runtime/raven/native/poc-seed.neoil`. It is the
bounded primitive/service selection previously assembled by the source JSON gate,
not a replacement implementation of application or source-library APIs. Build it:

```sh
neoclr assemble runtime/raven/native/poc-seed.neoil System.neox --format neox
```

The primitive CLI reference is reproduced by the existing declaration generator:

```sh
dotnet build docs/experiments/raven-target/Probe.csproj -p:RavenRoot="$RAVEN_SOURCE"
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-comparer-storage-core Core.dll
```

On 2026-10-05, with Raven integration revision `2554322c4` and NeoCLR `ef9d53ce`,
these exactly reproduce the previously accepted SHA256 artifacts:

- Core.dll: `64aae0e5ef83fe113a38d21c74cabe1bfabbd9de3526d941ea2236b630686bbc`
- System.neox: `2175d583a36e6de17086d25adc31376d9c3aea4a0c287f81043d3649dfc5c1dc`

The generator reuses the existing bootstrap declarations; no application/native
library is translated into CLI metadata. Full native primitive bootstrap is later work.
The author selected `v0.1.0-preview.12`, macOS arm64, and integration into NeoCLR main.
