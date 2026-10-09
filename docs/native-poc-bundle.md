# Native POC bundle

## Development project-to-executable workflow (2026-10-09)

From a neoCLR source checkout on macOS ARM64, build the current AOT tool and use
an ordinary executable `.rvnproj` importing the selected split bundle's
`lib/NeoCLR.ClassLibrary.props`:

```sh
cargo build --locked --manifest-path tools/aot-poc/Cargo.toml
python3 scripts/build-native-project.py \
  --project /path/to/App.rvnproj \
  --bundle /path/to/neoclr-native-poc \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output /path/to/new-native-output
/path/to/new-native-output/app
```

The output directory must be new. The command uses the bundle's compiler to
evaluate/build the project, compiles the resulting neoCLR metadata/IL, then links
the console host and selected runtime adapters. Apple `xcrun` selects Clang and
the SDK explicitly. The compiler still needs its documented .NET SDKs; the
resulting executable needs only macOS libSystem. This source-checkout helper is
development work, not a new capability in the published Preview 13 bundle.

The explicit `macos-arm64-console-v1` profile supports synchronous console entry,
selected UTF-8 text operations and Int32 formatting, using the existing private
ABI v4, native root reporting and a bounded 1 MiB nonmoving managed heap. It
renders guest faults, checks that guest root frames have unwound and collects at
quiescent shutdown. It supplies no guest command-line arguments, task pump, socket
services, file services, guarded recursion or general native reflection. Unsupported
reachable services/instructions fail through the backend's existing diagnostics.
This is a bounded deployment policy, not a permanent platform restriction or a
stable public hosting ABI.

`build.json` records commands, diagnostics, selected compiler/backend/library and
adapter hashes, SDK/compiler identification, emitted artifact hashes and dynamic
dependencies. Library catalog hashes are checked before compilation. The project
must reference the same bundle; arbitrary extra project-reference assemblies are
not yet added to the AOT dependency catalog. The helper currently consumes Raven's
`Native build output:` line (last output after reference builds); a structured
compiler output contract is future work. Recorded hashes establish input consistency,
not a signed provenance chain or a complete reproducible MSBuild input inventory.

An unsuccessful build retains its report and intermediate files but publishes no
`app`. Existing output directories are rejected without replacing their contents.
The command does not run the application. The focused acceptance harness does:

```sh
python3 scripts/verify-native-project-build.py \
  --bundle /path/to/neoclr-native-poc \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output /path/to/new-qualification-directory
```

It checks projects/output paths with spaces, UTF-8 and interpolation, a guest
division fault against the interpreter, executable-only deployment with an empty
environment, unsupported-recursion rejection and preservation of existing output.
The [2026-10-09 acceptance record](experiments/native-project-build-validation.json)
passes those cases and rejects a failed rebuild despite its previously emitted DLL.
Packaging this workflow with matching backend/adapters is the next slice; Windows
native compilation and HTTP project publication remain separate qualification.
This reuses the [.NET deployment comparison](native-execution-investigation.md):
project-level native build ergonomics are familiar, while neoCLR's currently
bounded runtime-service profile and explicit bootstrap dependencies remain costs.

## Preview 13 qualification (2026-10-09)

[Preview 13](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.13) is published
for Windows x64 and macOS ARM64. Public bundle links return HTTP 200; all five
uploaded asset digests match the qualified local artifacts.

The author has requested Windows x64 and macOS ARM64 packages for the next POC.
Both matching compiler/runtime/library/editor bundles pass extracted consumers
and 26 installed-editor checks per host. Canonical source validation and focused
Windows/macOS host checks pass. [Exact evidence](preview-13-validation.json) records
package hashes and the separate runtime, compiler, library and source-test revisions.
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
