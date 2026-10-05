# Native POC bundle

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
Publication and clean-source provenance for the remaining bootstrap inputs remain
explicit release decisions/work, rather than reasons to expand the translation bridge.
