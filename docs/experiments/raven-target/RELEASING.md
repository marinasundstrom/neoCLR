# Releasing the Raven/neoCLR experiment

This is a neoCLR experimental distribution containing a separately packaged Raven
build. It does **not** follow Raven's full product release cycle or imply a normal
Raven release. Keep compiler changes on the Raven experimental feature branch; do
not merge or tag Raven main just to ship this bundle.

## Scope and evidence

The [release walkthrough](../../runtime-raven-preview.md) defines two primary paths:
the runtime with `samples/neoil`, and Raven programs with the matching experimental
SDK/VSIX. Keep old Neo language material historical and outside the main demonstration.
Run `tools/verify_neoil.py --samples samples/neoil --system lib/System.neoil --runtime bin/neoclr`
from the extracted bundle in addition to the Raven checks below. A complete release
must carry both paths and their documentation, not just compiler integration assets.

Use [the neoCLR preview acceptance criteria](../../raven-preview-acceptance.md) for
what the demonstration must expose. All existing public runtime APIs are in scope;
full CLR/compiler feature parity is not. Maintain the supported/unsupported matrix
and list outstanding defects plainly. Packaging is not evidence of API support.

Before packaging, record the exact clean neoCLR and Raven commit IDs, Raven branch,
build tool versions, target OS/architecture, and an explicit experimental package
version. Choose a fresh version for each build; do not overwrite a previously
published artifact. The installed `0.1.12-neoclr.6` is a validated local build, not
a prescribed next release version.

Run focused compiler regression tests for changed Raven code and neoCLR tests for
changed runtime code. Run the end-to-end Raven integration checks and all release
demo programs. This targeted evidence replaces a full Raven release gate; it does
not replace testing the experiment itself. No unrelated Raven release workflows,
marketplace publication, or default-SDK changes are required.

## Package Raven separately

From the selected Raven feature-branch checkout, set `EXPERIMENT_VERSION` to the
chosen fresh version and `EXPERIMENT_RID` to the supported runtime identifier (for
example `osx-arm64`). Set `EXPERIMENT_OUTPUT` to a fresh absolute artifact directory.
The following are existing Raven packaging scripts; run them sequentially:

```sh
RAVEN_PACKAGE_OUTPUT="$EXPERIMENT_OUTPUT" scripts/package-sdk.sh "$EXPERIMENT_RID" "$EXPERIMENT_VERSION"
RAVEN_PACKAGE_OUTPUT="$EXPERIMENT_OUTPUT" scripts/package-vscode.sh "$EXPERIMENT_VERSION"
```

The output includes the versioned SDK archive and `raven-vscode.vsix`. Retain both
as distinct release assets, together with checksums and provenance. Check `git
status` after packaging because the scripts regenerate sources and stage extension
content. Review any source changes before assigning final provenance; do not label
a dirty build as an exact committed revision.

Build neoCLR from its recorded revision and bundle the matching runtime library,
metadata declarations, bridge, demo projects and instructions. Package these as
neoCLR assets rather than inserting them into an ordinary Raven release. The bridge
must use the same compiler build used for validation.

## Build the checkout-independent bundle

After committing the source changes and building the separate SDK, use a fresh output
directory. The packaging command requires clean neoCLR and Raven checkouts:

```sh
python3 docs/experiments/raven-target/package_bundle.py --raven /path/to/Raven --sdk /path/to/raven-sdk-VERSION-osx-arm64 --runtime /path/to/neoclr --output /fresh/path/to/bundle --version runtime-api-poc-BUILD
```

The script publishes the compiler bridge with its dependencies, generates matching
declaration metadata, flattens the target runtime library, copies the language server,
samples and validation scripts, and records revision/version/file-hash provenance.
It currently supports the tested macOS arm64 host. It does not publish or install.
The resulting README explains extraction, configuration and the dedicated project tasks.
The builder also copies neoCLR's LICENSE, notice inventory and preserved license
texts, and Raven's LICENSE/THIRD-PARTY-NOTICES.txt; the file manifest covers them.
Before publishing binary assets, review the actual bridge, server, SDK and VSIX
dependency inventories against their notices. Carrying upstream source notices alone
does not establish complete attribution for every bundled dependency. The validated
local .6 artifacts predate this packaging correction and must not be presented as
containing it; prepare a fresh candidate rather than overwriting those artifacts.

`run_project.py`, the saved-program suite and focused checks accept either `--raven`
for source development or `--bridge /path/to/Probe.dll --system /path/to/System.neoil`
for a published build. Each build attempt owns fresh output, and failed compilation
cannot execute stale output. The supplied library is copied alongside that output.

## Validate the actual packaged build

Extract/install into an isolated directory or VS Code profile; preserve the user's
normal Raven SDK selection. Follow [local installation instructions](VSCODE.md) with
the new artifact paths. Record actual paths and versions in the build manifest.

1. Check the packaged compiler and language server start with the documented .NET
   prerequisite. Record compiler version and the matching source revision.
2. Open a bundled Raven project targeting neoCLR. Verify completion and hover resolve
   the supplied runtime declarations and do not silently fall back to host .NET APIs.
3. Edit, save, build and run a demo through the neoCLR task. Verify value/reference
   behavior, Result/Option propagation and Void completion, plus the runtime-library
   demonstrations required by the current coverage inventory. Capture guest output.
4. Exercise expected failures: incompatible source and unsupported IL must fail
   without executing stale output; recoverable API failures must return their Result
   cases. File tests must use temporary directories and verify round-trip bytes.
5. Repeat from the packaged layout outside the development checkouts. Remove any
   dependence on developer-specific paths, or document and validate the explicit
   source-build prerequisite. Do not claim a standalone bundle unless this passes.

The stdio server checks, saved-project checks, match matrix and file probes in this
folder are repeatable development evidence. Run them with the packaged server and
compiler/bridge where the distribution supports it. Include `verify_application.py`
and `verify_orders.py` from the bundle: these verify application type semantics,
interface/virtual dispatch, escaping/shared captures, GC and order persistence using
the packaged bridge/runtime. Their temporary projects preserve the installed demo.
Source-checkout checks alone do
not establish that the packages work.

## Publish and repeat

Create a manifest containing both source revisions, package versions, platform and
prerequisites, asset SHA-256 hashes, validation commands/results, coverage matrix and
known limitations. Use `local-toolchain.json` as an example of provenance, not as a
manifest to overwrite with unverified results.

Update `CHANGELOG.md` under Unreleased for the build/release work and follow
[neoCLR's changelog workflow](../../changelog.md) when publishing. Keep prior release
notes unchanged. Publish the neoCLR build and matching experimental Raven SDK/VSIX
together with installation instructions and the manifest. A new tag/version/date
is chosen when publishing; this procedure does not announce one.

For the next experimental release, repeat revision selection, focused validation,
separate packaging and package-level smoke tests. Reuse the procedure and extend the
coverage evidence as the bridge grows; a full Raven release remains independent.

## Experimental tool dependency notices

The [tool notice inventory](../../../third-party/raven-tools/README.md) supplements
both projects' upstream notices. Check the actual SDK and runtime bundle:

```sh
python3 docs/experiments/raven-target/verify_tool_notices.py --tools /path/to/artifacts
```

For the VSIX, also compare its production JavaScript dependency graph when rebuilding
Raven; the .deps.json check only covers .NET packages. Preserve the five reviewed npm
package notices and refresh the inventory if the esbuild inputs change. Distribute
`third-party/raven-tools` together with Raven's LICENSE and THIRD-PARTY-NOTICES.txt
as a companion attribution archive for the separate SDK and VSIX. The runtime bundle
contains the same texts. Do not describe the separate upstream assets as containing
notices that are supplied only in the companion archive.
